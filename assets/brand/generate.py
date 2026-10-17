#!/usr/bin/env python3
"""Rebuild brand derivatives with ImageMagick; the canonical PNG is never changed."""

import json
from pathlib import Path
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parent
MASTER = ROOT / "logo.png"
CREAM = "#fff4e6"
PNG_OPTIONS = ["-strip", "-define", "png:compression-level=9",
               "-define", "png:compression-filter=0",
               "-define", "png:compression-strategy=1"]


def convert(source, destination, *operations):
    destination.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["convert", str(source), *operations,
                    *PNG_OPTIONS, str(destination)], check=True)


def icon(size, destination, opaque=False):
    operations = ["-filter", "Lanczos", "-resize", f"{size}x{size}"]
    if opaque:
        operations += ["-background", CREAM, "-alpha", "remove", "-alpha", "off"]
    convert(ROOT / "icons/icon.png", destination, *operations)


def main():
    if not MASTER.is_file():
        raise SystemExit("canonical assets/brand/logo.png is required")
    if shutil.which("convert") is None:
        raise SystemExit("ImageMagick convert is required")
    dimensions = subprocess.check_output(
        ["identify", "-format", "%wx%h", str(MASTER)], text=True
    )
    if dimensions != "1254x1254":
        raise SystemExit("the approved crop requires the original 1254 by 1254 master")
    convert(MASTER, ROOT / "logo.webp", "-resize", "1024x1024", "-quality", "90",
            "-define", "webp:alpha-quality=100")
    convert(MASTER, ROOT / "repository/social-preview.jpg", "-resize", "600x600",
            "-background", CREAM, "-alpha", "remove", "-alpha", "off",
            "-gravity", "center", "-extent", "1280x640",
            "-sampling-factor", "4:4:4", "-quality", "90")
    convert(MASTER, ROOT / "icons/icon.png", "-crop", "680x680+320+100", "+repage",
            "-filter", "Lanczos", "-resize", "1024x1024")

    for size in (16, 24, 32, 48, 64, 128, 256, 512, 1024):
        icon(size, ROOT / f"icons/desktop/icon-{size}.png")
    subprocess.run(["convert", *[str(ROOT / f"icons/desktop/icon-{size}.png")
                                for size in (16, 24, 32, 48, 64, 128, 256)],
                    str(ROOT / "icons/desktop/icon.ico")], check=True)

    iconset = ROOT / "icons/macos/SecretSorbet.iconset"
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            icon(size * scale, iconset / f"icon_{size}x{size}{suffix}.png")
    # ICNS PNG element types have fixed decoded sizes: 128, 256, 512 and 1024.
    # This packages existing ImageMagick PNGs; Python performs no pixel work.
    elements = []
    for kind, filename in ((b"ic07", "icon_128x128.png"),
                           (b"ic08", "icon_256x256.png"),
                           (b"ic09", "icon_512x512.png"),
                           (b"ic10", "icon_512x512@2x.png")):
        payload = (iconset / filename).read_bytes()
        elements.append(kind + struct.pack(">I", len(payload) + 8) + payload)
    payload = b"".join(elements)
    (ROOT / "icons/macos/SecretSorbet.icns").write_bytes(
        b"icns" + struct.pack(">I", len(payload) + 8) + payload
    )

    catalog = ROOT / "icons/ios/AppIcon.appiconset"
    entries = []
    specifications = [
        ("iphone", 20, (2, 3)), ("iphone", 29, (2, 3)),
        ("iphone", 40, (2, 3)), ("iphone", 60, (2, 3)),
        ("ipad", 20, (1, 2)), ("ipad", 29, (1, 2)),
        ("ipad", 40, (1, 2)), ("ipad", 76, (1, 2)),
        ("ipad", 83.5, (2,)), ("ios-marketing", 1024, (1,)),
    ]
    for idiom, points, scales in specifications:
        for scale in scales:
            size = int(points * scale)
            filename = f"icon-{size}.png"
            icon(size, catalog / filename, opaque=True)
            entries.append({"idiom": idiom, "size": f"{points}x{points}",
                            "scale": f"{scale}x", "filename": filename})
    (catalog / "Contents.json").write_text(
        json.dumps({"images": entries, "info": {"author": "xcode", "version": 1}},
                   indent=2) + "\n", encoding="utf-8"
    )
    for density, size in (("mdpi", 48), ("hdpi", 72), ("xhdpi", 96),
                          ("xxhdpi", 144), ("xxxhdpi", 192)):
        icon(size, ROOT / f"icons/android/mipmap-{density}/ic_launcher.png", opaque=True)
    icon(512, ROOT / "icons/android/store-icon-512.png", opaque=True)

    for size in (16, 32, 48):
        icon(size, ROOT / f"icons/web/favicon-{size}.png")
    subprocess.run(["convert", *[str(ROOT / f"icons/web/favicon-{size}.png")
                                for size in (16, 32, 48)],
                    str(ROOT / "icons/web/favicon.ico")], check=True)
    icon(180, ROOT / "icons/web/apple-touch-icon.png", opaque=True)
    for size in (192, 512):
        icon(size, ROOT / f"icons/web/site-icon-{size}.png")
    for size in (16, 32, 48, 128):
        icon(size, ROOT / f"icons/browser-extension/icon-{size}.png")


if __name__ == "__main__":
    main()
