// Generates every SVG in branding/ (deterministic: same input -> same bytes).
// The mark (a Z lit by a four-point "zenith" star) and the letters of the wordmark are hand-built geometry (no font).
// Only the banner taglines are text; they are converted to path outlines with opentype.js from Inter (SIL OFL 1.1).
// Usage: BRAND_DEPS=<dir with node_modules> node branding/tools/build-svg.mjs
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const deps = process.env.BRAND_DEPS;
if (!deps) throw new Error('Set BRAND_DEPS to a directory where `npm i opentype.js @fontsource/inter` was run');
const req = createRequire(path.join(path.resolve(deps), 'x.js'));
const opentype = req('opentype.js');
const inter = w => {
  const b = readFileSync(req.resolve(`@fontsource/inter/files/inter-latin-${w}-normal.woff`));
  return opentype.parse(b.buffer.slice(b.byteOffset, b.byteOffset + b.byteLength));
};

const out = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const P = {
  ink: '#14163A', night: '#0E1030', mist: '#F4F1FF', white: '#F2F1FF', mute: '#A9A9D6',
  amber: '#FFB938', violet: '#5B4BFF', deep: '#231B8F',
};
const r = n => Math.round(n * 100) / 100;
const poly = pts => 'M' + pts.map(p => `${r(p[0])} ${r(p[1])}`).join('L') + 'Z';

// ---- the mark ----
// A wide Z (cap 128, width 150, bars 34) and a four-point star above its upper-right corner, never touching it
// (>= 14 units apart). `small` is the hand-tuned 16/24 px version: heavier Z, fuller star.
const STAR_C = 0.22; // control-point pull: the lower, the fuller the star
function star(cx, cy, rx, ry, c = STAR_C) {
  const q = (x, y) => `${r(x)} ${r(y)}`;
  return `M${q(cx, cy - ry)}Q${q(cx + c * rx, cy - c * ry)} ${q(cx + rx, cy)}Q${q(cx + c * rx, cy + c * ry)} ${q(cx, cy + ry)}` +
    `Q${q(cx - c * rx, cy + c * ry)} ${q(cx - rx, cy)}Q${q(cx - c * rx, cy - c * ry)} ${q(cx, cy - ry)}Z`;
}
function markGeometry(small = false) {
  const W = small ? 140 : 150, H = small ? 124 : 128, t = small ? 42 : 34, d = small ? 74 : 64;
  const z = poly([[0, 0], [W, 0], [W, t], [d, H - t], [W, H - t], [W, H], [0, H], [0, H - t], [W - d, t], [0, t]]);
  const s = small ? { cx: W + 36, cy: -22, rx: 48, ry: 62, c: 0.3 } : { cx: W + 28, cy: -14, rx: 40, ry: 54, c: STAR_C };
  const box = { x0: 0, x1: s.cx + s.rx, y0: s.cy - s.ry, y1: H }; // bounding box of the whole mark
  return { z, star: star(s.cx, s.cy, s.rx, s.ry, s.c), box };
}
// The mark centred in a square of `size` units, filling `fill` of it.
function markIn(size, fill, fg, ac, small = false) {
  const g = markGeometry(small);
  const w = g.box.x1 - g.box.x0, h = g.box.y1 - g.box.y0;
  const k = (size * fill) / Math.max(w, h);
  const tx = (size - w * k) / 2 - g.box.x0 * k, ty = (size - h * k) / 2 - g.box.y0 * k;
  return `<g transform="translate(${r(tx)} ${r(ty)}) scale(${r(k * 1000) / 1000})"><path d="${g.z}" fill="${fg}"/><path d="${g.star}" fill="${ac}"/></g>`;
}
const tileDefs = `<defs><linearGradient id="tile" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${P.deep}"/><stop offset="1" stop-color="${P.violet}"/></linearGradient></defs>`;
const tile = small => `${tileDefs}<rect width="256" height="256" rx="${small ? 52 : 58}" fill="url(#tile)"/>` +
  markIn(256, small ? 0.84 : 0.70, P.mist, P.amber, small);

function svg(w, h, inner, title, extra = '') {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-label="${title}"><title>${title}</title>${extra}${inner}</svg>\n`;
}

// ---- the letters: filled outlines, cap 76, stems 15, bars 14, built wide to match the Z ----
const H = 76;
const Gl = {
  Z: [64, [poly([[0, 0], [64, 0], [64, 14], [19, H - 14], [64, H - 14], [64, H], [0, H], [0, H - 14], [45, 14], [0, 14]])]],
  E: [50, [poly([[0, 0], [50, 0], [50, 14], [15, 14], [15, 31], [44, 31], [44, 45], [15, 45], [15, 62], [50, 62], [50, H], [0, H]])]],
  N: [62, [poly([[0, 0], [16, 0], [47, 48], [47, 0], [62, 0], [62, H], [46, H], [15, 28], [15, H], [0, H]])]],
  I: [15, [poly([[0, 0], [15, 0], [15, H], [0, H]])]],
  T: [60, [poly([[0, 0], [60, 0], [60, 14], [37.5, 14], [37.5, H], [22.5, H], [22.5, 14], [0, 14]])]],
  H: [62, [poly([[0, 0], [15, 0], [15, 31], [47, 31], [47, 0], [62, 0], [62, H], [47, H], [47, 45], [15, 45], [15, H], [0, H]])]],
  L: [48, [poly([[0, 0], [15, 0], [15, 62], [48, 62], [48, H], [0, H]])]],
  A: [66, [poly([[0, H], [26, 0], [40, 0], [66, H], [51, H], [45.5, 60], [20.5, 60], [15, H]]) + poly([[33, 23.4], [40.7, 46], [25.3, 46]])]],
  U: [62, ['M0 0H15V58Q15 62 19 62H43Q47 62 47 58V0H62V60Q62 76 46 76H16Q0 76 0 60Z']],
  C: [58, ['M58 26H43Q41 14 31 14Q15 14 15 38Q15 62 31 62Q41 62 43 50H58Q56 76 31 76Q0 76 0 38Q0 0 31 0Q56 0 58 26Z']],
  R: [60, ['M0 0H34Q58 0 58 22Q58 40 40 44L60 76H43L26 46H15V76H0Z' + 'M15 14V32H33Q44 32 44 23Q44 14 33 14Z']],
};
const GAP = 12, SPACE = 34;
// `cls`/`fill` colour the letters; x,y is the top-left of the cap box; cap is the cap height in output units
function word(text, { x = 0, y = 0, cap = 76, fill = null, cls = 't', track = 0 }) {
  const k = cap / H;
  let cx = 0, body = '';
  for (const ch of text) {
    if (ch === ' ') { cx += SPACE; continue; }
    const [w, ds] = Gl[ch];
    body += `<path transform="translate(${r(cx)} 0)" d="${ds.join('')}" fill-rule="evenodd"/>`;
    cx += w + GAP + track;
  }
  cx -= GAP + track;
  const paint = fill ? `fill="${fill}"` : `class="${cls}"`;
  return { svg: `<g transform="translate(${r(x)} ${r(y)}) scale(${r(k * 1000) / 1000})" ${paint}>${body}</g>`, width: cx * k };
}

// Own layout (no GSUB/kerning): opentype.js 1.3 chokes on Inter's ccmp lookups.
function textPath(font, text, x, y, size, fill) {
  const k = size / font.unitsPerEm;
  let d = '', cx = x;
  for (const ch of text) {
    const g = font.charToGlyph(ch);
    d += g.getPath(cx, y, size).toPathData(2);
    cx += g.advanceWidth * k;
  }
  return { svg: `<path fill="${fill}" d="${d}"/>`, width: cx - x };
}

const css = `<style>.t{fill:${P.ink}}.dk{display:none}@media (prefers-color-scheme:dark){.t{fill:${P.white}}.lt{display:none}.dk{display:inline}}</style>`;
const files = {};
const markOnly = (fg, ac) => svg(256, 256, markIn(256, 0.86, fg, ac), 'Zenith');
files['logo-mark.svg'] = markOnly(P.mist, P.amber); // for dark backgrounds
files['logo-mark-light.svg'] = markOnly(P.deep, P.violet); // for light backgrounds
files['logo-mark-mono-black.svg'] = markOnly('#000', '#000');
files['logo-mark-mono-white.svg'] = markOnly('#fff', '#fff');
files['logo-tile.svg'] = svg(256, 256, tile(false), 'Zenith');
files['logo-tile-small.svg'] = svg(256, 256, tile(true), 'Zenith');

// lockups: mark 128 + ZENITH, cap 76 scaled to 56 (the mark switches colour with the colour scheme)
const lockMark = `<g class="lt">${markIn(128, 0.9, P.deep, P.violet)}</g><g class="dk">${markIn(128, 0.9, P.mist, P.amber)}</g>`;
{
  const z = word('ZENITH', { x: 164, y: 36, cap: 56 });
  files['logo-wordmark.svg'] = svg(164 + Math.ceil(z.width) + 8, 128, lockMark + z.svg, 'Zenith', css);
  const l = word('ZENITH LAUNCHER', { x: 164, y: 46, cap: 36, track: 2 });
  files['logo-wordmark-launcher.svg'] = svg(164 + Math.ceil(l.width) + 8, 128, lockMark + l.svg, 'Zenith Launcher', css);
}

// banners
const f6 = inter(600), f5 = inter(500);
function banner(launcher) {
  let b = `<defs><radialGradient id="glow" cx="0.5" cy="1" r="0.85"><stop offset="0" stop-color="${P.violet}" stop-opacity=".5"/><stop offset="1" stop-color="${P.violet}" stop-opacity="0"/></radialGradient>
<radialGradient id="halo" cx="0.5" cy="0.5" r="0.5"><stop offset="0" stop-color="${P.amber}" stop-opacity=".28"/><stop offset="1" stop-color="${P.amber}" stop-opacity="0"/></radialGradient></defs>`;
  b += `<rect width="1280" height="640" fill="${P.night}"/><rect width="1280" height="640" fill="url(#glow)"/>`;
  // a faint zenith star and its glow, top right
  b += `<circle cx="1040" cy="150" r="260" fill="url(#halo)"/><path d="${star(1040, 150, 70, 130, 0.12)}" fill="${P.amber}" fill-opacity=".16"/>`;
  b += `<g transform="translate(0 46)">`;
  b += `<g transform="translate(96 150)">${markIn(220, 1, P.mist, P.amber)}</g>`;
  const x = 372;
  if (!launcher) {
    b += word('ZENITH', { x, y: 150, cap: 96, fill: P.white, track: 4 }).svg;
    b += textPath(f6, 'Minecraft in Rust, at its highest point.', x, 322, 36, P.white).svg;
    b += textPath(f5, 'a fork of Horizon', x, 372, 24, P.mute).svg;
  } else {
    b += word('ZENITH', { x, y: 132, cap: 84, fill: P.white, track: 4 }).svg;
    b += word('LAUNCHER', { x, y: 250, cap: 44, fill: P.amber, track: 12 }).svg;
    b += textPath(f6, 'The launcher for Zenith', x, 342, 32, P.white).svg;
    b += textPath(f5, 'a fork of XMCL', x, 388, 24, P.mute).svg;
  }
  b += '</g>';
  return svg(1280, 640, b, launcher ? 'Zenith Launcher' : 'Zenith');
}
files['banner.svg'] = banner(false);
files['banner-launcher.svg'] = banner(true);

mkdirSync(out, { recursive: true });
for (const [n, c] of Object.entries(files)) writeFileSync(path.join(out, n), c);
console.log('wrote', Object.keys(files).join(', '));
