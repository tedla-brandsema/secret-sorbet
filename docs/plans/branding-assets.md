# Branding asset preparation

The user supplied a temporary root PNG and authorized ImageMagick optimization,
app icons, favicons, and removal of the root copy after delivery. The
[roadmap](../ROADMAP.md) owns progress and its checkbox legend.

## B01 — Preserve the artwork and prepare platform assets

Depends on the supplied 1254×1254 RGBA PNG. This is one coherent asset slice;
there is no Rust behavior change or app integration.

- Keep a losslessly optimized, full-resolution canonical PNG under
  `assets/brand/`, preserving all decoded RGBA pixels. Add a smaller WebP for
  web use and an opaque 1280×640 JPEG repository social preview.
- Use the 680×680 crop at x=320, y=100 for the icon mark: the hat, sunglasses,
  sorbet and upper bowl remain readable without shrinking the wordmark.
  Preserve the full supplied badge separately. The crop is intentionally
  enlarged for 1024px app sources; it adds no source detail.
- Generate PNG sizes and multiresolution ICO files for desktops, iOS and Apple
  touch icons with opaque cream backgrounds, Android legacy launcher densities,
  browser-extension icons, and generic website favicons/touch icons. Modern
  adaptive/layered icons and app wiring remain future integration work. macOS
  iconset/ICNS assets preserve the crop's alpha.
- Store the asset pack centrally under `assets/brand/`; placeholder app
  directories do not become implemented applications through these assets.
- Include executable ImageMagick reproduction tooling and asset usage/format
  documentation. Pixel transformations use ImageMagick; optional binary
  container assembly may use Python's standard library.

## Ownership and delivery

The asset worker owns only `assets/brand/**`. The governor owns the plan,
roadmap, contributor index, changelog, final checks, root-file removal and Git.
A different read-only reviewer checks output formats, reproduction and claims.
Workers do not commit, push, remove the supplied root file, or touch other paths.

## Acceptance

- Verify the optimized master against the original using decoded RGBA bytes,
  dimensions, color space and alpha. Check file sizes and visual quality.
- Decode each PNG/WebP/ICO; check expected dimensions and ICO frame tables.
  Validate any ICNS headers/chunks and embedded PNGs. A macOS-native app or
  Xcode validation is not claimed from Linux container checks.
- Check Apple catalog JSON filenames/scales/dimensions and opacity, Android
  density sizes, and favicon/extension sizes. Inspect a visual contact sheet.
- Run regeneration after relocating the source; compare output inventory and
  hashes with the same tool versions. No new runtime tests are required for
  assets alone. Run shell/Python syntax checks, links and diff checks.
- Remove only the authorized temporary root PNG after the canonical copy and
  outputs are verified. Keep all other local/generated state.
- Independently review the complete slice, commit it, then record its actual
  hash in a separate roadmap reconciliation and push to `origin/main`.
