# Secret Sorbet brand assets

`logo.png` is the canonical 1254 × 1254 badge, including its wordmark and original
transparency. It preserves every decoded 8-bit RGBA pixel of the supplied
`Secret Sorbet Spy Scoop.png`. The root upload is not needed to rebuild assets.
The master is 1,176,296 bytes, down from 1,261,603 bytes through lossless PNG
compression and removal of ancillary metadata. Its decoded RGBA SHA256 is
`30682b189760a84641d296b9c85fd74c32d88edffb19c35feed3f78442ab1499`.

Use the full badge where its lettering remains readable. `logo.webp` is a lossy
1024 × 1024 web derivative at quality 90 with full alpha quality. Do not use it
as the source for future editing. `repository/social-preview.jpg` is an opaque
1280 × 640 repository sharing image: the full badge is resized to 600 × 600 and
centered on cream `#fff4e6`. JPEG quality 90 with 4:4:4 chroma sampling keeps it
smaller than the equivalent PNG.

## Application and browser icons

The icon artwork crops a 680 × 680 square at source offset `+320+100`, retaining
the hat, sunglasses, sorbet and part of the bowl. It omits the wordmark so the
mascot stays recognizable at small sizes. `icons/icon.png` is the 1024 × 1024
crop master. All icon derivatives use this image, with Lanczos resizing.
The crop enlarges the original raster; it is not vector artwork or added detail.

| Directory | Assets and intended use |
| --- | --- |
| `icons/desktop/` | PNGs at 16, 24, 32, 48, 64, 128, 256, 512 and 1024 pixels; Windows ICO with 16/24/32/48/64/128/256 frames. |
| `icons/macos/` | Standard `.iconset` with 16/32/128/256/512 point images at 1× and 2×; `.icns` container with PNG elements at 128/256/512/1024 pixels. |
| `icons/ios/AppIcon.appiconset/` | Opaque PNGs and `Contents.json` for legacy iPhone/iPad icon slots, including 83.5 points at 2× and the 1024 pixel App Store image. |
| `icons/android/` | Opaque legacy launcher icons in `mipmap-mdpi` through `mipmap-xxxhdpi` (48/72/96/144/192 pixels), plus a 512 pixel store image. |
| `icons/web/` | ICO with 16/32/48 frames; corresponding PNGs; opaque 180 pixel Apple touch icon; 192/512 pixel site icons. |
| `icons/browser-extension/` | PNGs at 16/32/48/128 pixels for extension icon declarations. |

The iOS, Android and Apple touch images flatten transparency onto cream
`#fff4e6`; other PNG derivatives preserve the crop's alpha. Platform masking
should be applied by the consuming application or operating system. These are
central assets; this directory does not configure an application, browser
manifest, Android adaptive icon, PWA, or storefront upload. Review them in the
actual platform build when those applications are implemented.

## Rebuilding and checking

Requirements: Python 3 and ImageMagick with PNG, WebP, JPEG and ICO support.
Generation was checked with ImageMagick 6.9.12-98 Q16 and Python 3.12. Run from
the repository root:

```sh
python3 assets/brand/generate.py
python3 assets/brand/verify.py
```

The generator reads only `logo.png` and writes derivatives under this directory;
it never rewrites the master. ImageMagick performs all cropping, resizing,
compositing, format conversion and optimization. Python only orchestrates those
commands, writes the iOS catalog, and packages existing PNG bytes into ICNS
chunks (`ic07`, `ic08`, `ic09`, `ic10`). Different ImageMagick/codec versions can
change compressed bytes or resampling details; the master pixel identity,
dimensions, crop definition, platform opacity and container sizes are checked.
The verifier compares the crop with the installed ImageMagick's resize result.

`verify.py` checks the master RGBA hash, exact crop, every generated size, opaque
platform outputs, iOS catalog references, decoded ICO frames and ICNS chunk
contents. This Linux environment does not provide Apple's ICNS renderer; the
ICNS is structurally checked against its PNG iconset. On macOS, use the
operating system's encoder if required by packaging tools:

```sh
iconutil -c icns assets/brand/icons/macos/SecretSorbet.iconset \
  -o /tmp/SecretSorbet.icns
```

To reproduce the initial lossless import while the original upload is available:

```sh
convert 'Secret Sorbet Spy Scoop.png' -strip \
  -define png:compression-level=9 -define png:compression-filter=0 \
  -define png:compression-strategy=1 assets/brand/logo.png
```

Compare decoded RGBA bytes before removing an original. PNG file hashes differ
because compression and ancillary metadata changed; the pixel hash above pins
the supplied image. No tracing or SVG conversion was performed.
