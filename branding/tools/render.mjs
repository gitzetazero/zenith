// Rasterises the SVGs in branding/ into png/ and icons/. Pure output of the SVGs: no fonts needed.
// Usage: BRAND_DEPS=<dir with node_modules> node branding/tools/render.mjs
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const deps = process.env.BRAND_DEPS;
if (!deps) throw new Error('Set BRAND_DEPS to a directory where `npm i @resvg/resvg-js` was run');
const { Resvg } = createRequire(path.join(path.resolve(deps), 'x.js'))('@resvg/resvg-js');
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
for (const d of ['png', 'icons']) mkdirSync(path.join(root, d), { recursive: true });

const png = (svg, width, background) =>
  new Resvg(readFileSync(path.join(root, svg), 'utf8'),
    { fitTo: { mode: 'width', value: width }, font: { loadSystemFonts: false }, ...(background ? { background } : {}) }).render().asPng();
const put = (rel, buf) => writeFileSync(path.join(root, rel), buf);
// Hand-tuned small tile for 16-24 px (heavier Z, fuller star), the normal tile above.
const tileFor = s => (s <= 24 ? 'logo-tile-small.svg' : 'logo-tile.svg');

for (const s of [16, 24, 32, 48, 64, 128, 256, 512, 1024]) put(`png/logo-tile-${s}.png`, png(tileFor(s), s));
// the bare mark on its two backgrounds (looked at by eye when the mark changes)
for (const s of [16, 32, 64, 256]) {
  put(`png/logo-mark-dark-${s}.png`, png('logo-mark.svg', s, '#0E1030'));
  put(`png/logo-mark-light-${s}.png`, png('logo-mark-light.svg', s, '#FFFFFF'));
}
put('png/logo-mark-256.png', png('logo-mark.svg', 256)); // transparent, for dark backgrounds
put('png/banner.png', png('banner.svg', 1280));
put('png/banner-launcher.png', png('banner-launcher.svg', 1280));
put('png/social-preview.png', png('banner.svg', 1280));
put('icons/icon-1024.png', png('logo-tile.svg', 1024));
put('icons/icon-512.png', png('logo-tile.svg', 512));

// ICO container with PNG-compressed layers (Vista+ format, 256 stored as 0).
function ico(sizes) {
  const imgs = sizes.map(s => png(tileFor(s), s));
  const head = Buffer.alloc(6 + 16 * sizes.length);
  head.writeUInt16LE(1, 2); head.writeUInt16LE(sizes.length, 4);
  let off = head.length;
  sizes.forEach((s, i) => {
    const e = 6 + 16 * i;
    head[e] = s === 256 ? 0 : s; head[e + 1] = s === 256 ? 0 : s;
    head.writeUInt16LE(1, e + 4); head.writeUInt16LE(32, e + 6);
    head.writeUInt32LE(imgs[i].length, e + 8); head.writeUInt32LE(off, e + 12);
    off += imgs[i].length;
  });
  return Buffer.concat([head, ...imgs]);
}
const SIZES = [16, 24, 32, 48, 64, 128, 256];
put('icons/zenith.ico', ico(SIZES));
put('icons/zenith-launcher.ico', ico(SIZES)); // same icon: a badge is not legible at 16 px
console.log('rendered');
