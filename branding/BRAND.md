# Zenith brand

Zenith is not affiliated with Mojang or Microsoft. Minecraft is a trademark of Mojang Synergies AB.
Zenith is a fork of [Horizon](https://github.com/horizon-rust/horizon) and its launcher is a fork of XMCL.
The brand is original: it does not imitate logos, typefaces or assets of Minecraft, Horizon or XMCL.

## The idea

*Zenith* is the highest point, the spot straight above your head. The mark is a wide **Z whose upper corner is lit by a zenith star**: a four-point amber star that sits just above the end of the top bar and never touches it. Two simple shapes, so it reads at 16 px. It replaces the first mark (an arc and a disc), which looked like a generic user avatar and is retired.

The lettering (`ZENITH`, `LAUNCHER`) is drawn by hand as filled outlines (cap height 76, stems 15, bars 14) with wide geometric letterforms that match the wide Z. It is our own drawing, not a font.

## Tagline

Main: **Minecraft in Rust, at its highest point.** Alternatives:

- Minecraft in Rust, no ceiling.
- Take Minecraft in Rust higher.

Banners say "a fork of Horizon" (game) and "a fork of XMCL" (launcher).

## Files

| File | Use |
|---|---|
| `logo-mark.svg` | The mark on a transparent background, for **dark** backgrounds: mist `#F4F1FF` Z and amber `#FFB938` star. |
| `logo-mark-light.svg` | The mark for **light** backgrounds: deep indigo `#231B8F` Z and violet `#5B4BFF` star (amber is too weak on white). |
| `logo-mark-mono-black.svg`, `logo-mark-mono-white.svg` | One ink: Z and star in the same colour. |
| `logo-tile.svg` | App icon: rounded square with the indigo-to-violet gradient, mist Z and amber star. Used for `icons/` and the ICO layers of 32 px and above. |
| `logo-tile-small.svg` | The same tile hand-tuned for 16 and 24 px (heavier Z, fuller star). |
| `logo-wordmark.svg`, `logo-wordmark-launcher.svg` | Mark + `ZENITH` / `ZENITH LAUNCHER`. Mark and text switch to the dark colourway with `prefers-color-scheme: dark`. |
| `banner.svg`, `banner-launcher.svg` | 1280x640 for the README and social networks. |
| `png/` | Exports: `logo-tile-{16..1024}.png`, `logo-mark-{dark,light}-{16,32,64,256}.png`, `logo-mark-256.png` (transparent), `banner.png`, `banner-launcher.png`, `social-preview.png`. |
| `icons/` | `zenith.ico` and `zenith-launcher.ico` (layers 16 to 256 px; the launcher uses the same icon, because a badge is not legible at 16 px), `icon-512.png`, `icon-1024.png` (Electron/electron-builder). |
| `palette.json`, `palette.css` | Colours and CSS variables (dark and light theme) for the launcher theme. |
| `tools/` | Scripts that regenerate everything (see `tools/README.md`). |

## Palette

Brand:

| Name | Hex | Use |
|---|---|---|
| Primary | `#5B4BFF` | Violet: tile gradient end, star on light backgrounds, brand buttons |
| Primary deep | `#231B8F` | Indigo: tile gradient start, Z on light backgrounds |
| Accent (star) | `#FFB938` | The star on dark backgrounds, highlights (`#FF9A3C` is the warm variant) |
| Night | `#0E1030` | Dark background |
| Ink | `#14163A` | Text on light backgrounds |
| Mist | `#F4F1FF` | The Z on dark backgrounds, soft light background |

Interface tokens:

| Token | Dark | Light |
|---|---|---|
| surface | `#0E1030` | `#FAF9FF` |
| surface-variant | `#1A1D45` | `#ECEAFB` |
| primary | `#8B7FFF` | `#4A3AE0` |
| on-primary | `#0E1030` | `#FFFFFF` |
| accent | `#FFB938` | `#B86E00` |
| success | `#3DD68C` | `#12805C` |
| warning | `#FFB938` | `#9A5B00` |
| error | `#FF6B6B` | `#C62F3E` |
| outline | `#3A3E78` | `#C4C1E6` |
| text-primary | `#F2F1FF` | `#14163A` |
| text-secondary | `#A9A9D6` | `#55577F` |

## Minimum size and clear space

- Mark: at least 16 px (favicon, tray). At 16 and 24 px use the tile (`logo-tile-small.svg`) or the bare mark, never the lockup with text.
- Lockup with text: capital letters at least 14 px tall.
- Clear space: at least a quarter of the mark's width on every side.
- The star never touches the Z (at least 14 design units apart). Do not enlarge or move it.

## Do and don't

Do:
- Use the SVG or PNG files as they are; scale proportionally.
- Use `logo-mark.svg` on dark backgrounds and `logo-mark-light.svg` on light ones.
- Use the mono variants when there is only one ink (stamps, engraving, black print).

Don't:
- Do not stretch, rotate, crop or recolour the mark, and do not add shadows, glows or outlines.
- Do not combine it with block, creeper or other Minecraft, Mojang or Microsoft imagery.
- Do not use Horizon's or XMCL's logo instead of Zenith's, or in a way that looks like an official product of theirs.
- Do not suggest that Zenith is an official product of Mojang or Microsoft.

## Typeface

Recommended, not included: **Inter** (SIL Open Font License 1.1, <https://rsms.me/inter/>) for interface and docs, weights 500 and 600 for headings. If it is not available, the stack `system-ui, "Segoe UI", Roboto, sans-serif` is fine. The banner taglines are Inter 600/500 converted to outlines, so the SVGs do not need the font installed; Inter is distributed with its OFL licence (Copyright 2016 The Inter Project Authors).

## Upload the social preview on GitHub

The GitHub API has no endpoint for this: it is a manual step. In the repository: **Settings > Social preview > Edit > Upload an image** and choose `branding/png/social-preview.png` (1280x640).

## Regenerate

See `tools/README.md`.
