#!/usr/bin/env python3
"""Check raster dimensions, platform opacity, containers and the lossless master."""

import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parent
MASTER_RGBA_SHA256 = "30682b189760a84641d296b9c85fd74c32d88edffb19c35feed3f78442ab1499"


def pixels(path, *operations):
    return subprocess.check_output(["convert", str(path), *operations, "-depth", "8", "rgba:-"])


def check_raster(path, width, height=None, opaque=False):
    details = subprocess.check_output(
        ["identify", "-format", "%w %h %[opaque] %[channels]", str(path)], text=True
    ).split()
    assert details[:2] == [str(width), str(height or width)], (path, details)
    if opaque:
        assert details[2] == "true" and "a" not in details[3], (path, details)


def check_ico(path, sizes):
    content = path.read_bytes()
    assert struct.unpack_from("<HHH", content) == (0, 1, len(sizes))
    for index, size in enumerate(sizes):
        width, height, _, reserved, _, depth, length, offset = struct.unpack_from(
            "<BBBBHHII", content, 6 + 16 * index
        )
        assert (width or 256, height or 256) == (size, size)
        assert reserved == 0 and depth == 32
        assert offset >= 6 + 16 * len(sizes) and offset + length <= len(content)
    frames = subprocess.check_output(
        ["identify", "-format", "%wx%h\n", str(path)], text=True
    ).splitlines()
    assert frames == [f"{size}x{size}" for size in sizes], frames


def main():
    check_raster(ROOT / "logo.png", 1254)
    assert hashlib.sha256(pixels(ROOT / "logo.png")).hexdigest() == MASTER_RGBA_SHA256
    check_raster(ROOT / "logo.webp", 1024)
    check_raster(ROOT / "repository/social-preview.jpg", 1280, 640, opaque=True)
    check_raster(ROOT / "icons/icon.png", 1024)
    assert pixels(ROOT / "icons/icon.png") == pixels(
        ROOT / "logo.png", "-crop", "680x680+320+100", "+repage",
        "-filter", "Lanczos", "-resize", "1024x1024"
    )
    for size in (16, 24, 32, 48, 64, 128, 256, 512, 1024):
        check_raster(ROOT / f"icons/desktop/icon-{size}.png", size)
    check_ico(ROOT / "icons/desktop/icon.ico", (16, 24, 32, 48, 64, 128, 256))
    iconset = ROOT / "icons/macos/SecretSorbet.iconset"
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            check_raster(iconset / f"icon_{size}x{size}{suffix}.png", size * scale)
    content = (ROOT / "icons/macos/SecretSorbet.icns").read_bytes()
    assert content[:4] == b"icns" and struct.unpack_from(">I", content, 4)[0] == len(content)
    offset = 8
    for kind, filename in ((b"ic07", "icon_128x128.png"),
                           (b"ic08", "icon_256x256.png"),
                           (b"ic09", "icon_512x512.png"),
                           (b"ic10", "icon_512x512@2x.png")):
        assert content[offset:offset + 4] == kind
        length = struct.unpack_from(">I", content, offset + 4)[0]
        assert content[offset + 8:offset + length] == (iconset / filename).read_bytes()
        offset += length
    assert offset == len(content)
    catalog = ROOT / "icons/ios/AppIcon.appiconset"
    entries = json.loads((catalog / "Contents.json").read_text())["images"]
    assert len(entries) == 18
    assert len({(entry["idiom"], entry["size"], entry["scale"]) for entry in entries}) == 18
    for entry in entries:
        size = int(float(entry["size"].split("x")[0]) * int(entry["scale"][0]))
        check_raster(catalog / entry["filename"], size, opaque=True)
    for density, size in (("mdpi", 48), ("hdpi", 72), ("xhdpi", 96),
                          ("xxhdpi", 144), ("xxxhdpi", 192)):
        check_raster(ROOT / f"icons/android/mipmap-{density}/ic_launcher.png", size, opaque=True)
    check_raster(ROOT / "icons/android/store-icon-512.png", 512, opaque=True)
    check_ico(ROOT / "icons/web/favicon.ico", (16, 32, 48))
    for size in (16, 32, 48):
        check_raster(ROOT / f"icons/web/favicon-{size}.png", size)
    check_raster(ROOT / "icons/web/apple-touch-icon.png", 180, opaque=True)
    for size in (192, 512):
        check_raster(ROOT / f"icons/web/site-icon-{size}.png", size)
    for size in (16, 32, 48, 128):
        check_raster(ROOT / f"icons/browser-extension/icon-{size}.png", size)
    print("Brand asset checks passed: master pixels, crop, sizes, opacity, ICO and ICNS containers.")


if __name__ == "__main__":
    main()
