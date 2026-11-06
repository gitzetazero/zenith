# Brand tools

Everything in `branding/` (SVG, PNG, ICO) is regenerated with two Node scripts (Node 24 or newer). `node_modules` is not committed: install the dependencies in any temporary directory.

```sh
mkdir /tmp/zenith-brand-deps && cd /tmp/zenith-brand-deps
npm init -y
npm i @resvg/resvg-js opentype.js @fontsource/inter     # Windows, Linux and macOS, nothing installed on the system

cd <repo root>
export BRAND_DEPS=/tmp/zenith-brand-deps                # PowerShell: $env:BRAND_DEPS = "..."
node branding/tools/build-svg.mjs                # writes the SVGs of branding/
node branding/tools/render.mjs                   # writes branding/png/ and branding/icons/
```

- `build-svg.mjs` generates the SVGs deterministically. The mark (a Z and a four-point star) and the letters of `ZENITH` and `LAUNCHER` are hand-built geometry (no font). Only the banner taglines are text: they are converted to outlines (paths) with opentype.js from Inter (SIL OFL 1.1), so the SVGs depend on no installed font.
- `render.mjs` rasterises with resvg without loading system fonts and builds the `.ico` files (an ICO container with PNG layers of 16, 24, 32, 48, 64, 128 and 256 px) with its own code. The 16 and 24 px layers use the hand-tuned `logo-tile-small.svg`.
- The PNGs carry no text metadata or local paths.
- After changing the mark, look at the PNGs: the mark at 16, 32, 64 and 256 px on dark and light, the lockups, both banners and every ICO layer.
