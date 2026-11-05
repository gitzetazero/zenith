// Generates every SVG in branding/ (deterministic: same input -> same bytes).
// Letters of the wordmark are hand-built stroke geometry (no font). Only the Spanish tagline
// is text; it is converted to path outlines with opentype.js from Inter (SIL OFL 1.1).
// Usage: BRAND_DEPS=<dir with node_modules> node branding/tools/build-svg.mjs
import { createRequire } from 'node:module';
import { writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const deps = process.env.BRAND_DEPS;
if (!deps) throw new Error('Set BRAND_DEPS to a directory where `npm i opentype.js @fontsource/inter` was run');
const req = createRequire(path.join(path.resolve(deps), 'x.js'));
const opentype = req('opentype.js');
const interFile = w => req.resolve(`@fontsource/inter/files/inter-latin-${w}-normal.woff`);
const inter = w => {
  const b = (req('node:fs')).readFileSync(interFile(w));
  return opentype.parse(b.buffer.slice(b.byteOffset, b.byteOffset + b.byteLength));
};

const out = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const P = { ink: '#14163A', night: '#0E1030', white: '#F2F1FF', mute: '#A9A9D6', amber: '#FFB938', violet: '#5B4BFF', deep: '#231B8F' };
const S = 15; // stroke weight in cap-height-100 units

// ---- glyphs: [width, [path d...]] in a box 100 tall; strokes get clipped to the box ----
const G = {
  Z: [64, ['M0 7.5H64', 'M0 92.5H64', 'M62 -4L2 104']],
  E: [54, ['M7.5 0V100', 'M0 7.5H54', 'M0 50H46', 'M0 92.5H54']],
  N: [70, ['M7.5 0V100', 'M62.5 0V100', 'M7.5 -6L62.5 106']],
  I: [15, ['M7.5 0V100']],
  T: [62, ['M0 7.5H62', 'M31 0V100']],
  H: [68, ['M7.5 0V100', 'M60.5 0V100', 'M0 50H68']],
  L: [52, ['M7.5 0V100', 'M0 92.5H52']],
  A: [70, ['M-2 108L35 -4L72 108', 'M16 70H54']],
  U: [66, ['M7.5 0V67A25.5 25.5 0 0 0 58.5 67V0']],
  C: [64, ['M49.3 19.9A24.5 42.5 0 1 0 49.3 80.1']],
  R: [62, ['M7.5 0V100', 'M0 7.5H36A18.5 18.5 0 0 1 36 44.5H7.5', 'M32 44.5L58 108']],
};
const GAP = 9, SPACE = 44;

function word(text, { x = 0, y = 0, cap = 100, color = null, track = 0, id = 'w' }) {
  const k = cap / 100;
  let cx = 0, body = '', clips = new Map();
  for (const ch of text) {
    if (ch === ' ') { cx += SPACE; continue; }
    const [w, ds] = G[ch];
    if (!clips.has(w)) clips.set(w, `${id}c${w}`);
    body += `<g transform="translate(${cx} 0)"><g clip-path="url(#${clips.get(w)})"><path d="${ds.join('')}"/></g></g>`;
    cx += w + GAP + track;
  }
  cx -= GAP + track;
  const defs = [...clips].map(([w, i]) => `<clipPath id="${i}"><rect x="0" y="0" width="${w}" height="100"/></clipPath>`).join('');
  const stroke = color ? `stroke="${color}"` : 'class="t"';
  const svg = `<defs>${defs}</defs><g transform="translate(${r(x)} ${r(y)}) scale(${k})" fill="none" ${stroke} stroke-width="${S}" stroke-linejoin="miter" stroke-miterlimit="10">${body}</g>`;
  return { svg, width: cx * k };
}
const r = n => Math.round(n * 100) / 100;

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

// ---- the mark (256 box) ----
// horizon arc: circle r=250 centred (128,436), top at y=186
const arc = `M0 220.8A250 250 0 0 1 256 220.8V256H0Z`;
const defsColor = `<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${P.deep}"/><stop offset="1" stop-color="${P.violet}"/></linearGradient><linearGradient id="sun" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFC95C"/><stop offset="1" stop-color="#FF9A3C"/></linearGradient></defs>`;
const markBody = `<rect width="256" height="256" rx="58" fill="url(#bg)"/><circle cx="128" cy="104" r="46" fill="url(#sun)"/><clipPath id="tc"><rect width="256" height="256" rx="58"/></clipPath><path d="${arc}" fill="#F4F1FF" clip-path="url(#tc)"/>`;
const monoSvg = c => svg(256, 256, `<mask id="m"><rect width="256" height="256" rx="58" fill="#fff"/><circle cx="128" cy="104" r="46" fill="#000"/><path d="${arc}" fill="#000"/></mask><rect width="256" height="256" fill="${c}" mask="url(#m)"/>`, 'Zenith', '');
const badge = `<circle cx="206" cy="206" r="44" fill="${P.night}"/><circle cx="206" cy="206" r="44" fill="none" stroke="#F4F1FF" stroke-width="8"/><path d="M194 186L226 206L194 226Z" fill="${P.amber}"/>`;

function svg(w, h, inner, title, extra = '') {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-label="${title}"><title>${title}</title>${extra}${inner}</svg>\n`;
}
const darkCss = `<style>.t{stroke:${P.ink}}.f{fill:${P.ink}}@media (prefers-color-scheme:dark){.t{stroke:${P.white}}.f{fill:${P.white}}}</style>`;
const markG = (x, y, s, body = markBody) => `<g transform="translate(${x} ${y}) scale(${s / 256})">${body}</g>`;

const files = {};
files['logo-mark.svg'] = svg(256, 256, defsColor + markBody, 'Zenith');
files['logo-mark-mono-black.svg'] = monoSvg('#000');
files['logo-mark-mono-white.svg'] = monoSvg('#fff');
files['logo-mark-launcher.svg'] = svg(256, 256, defsColor + markBody + badge, 'Zenith Launcher');

// wordmark: mark 128 + ZENITH cap 56
{
  const z = word('ZENITH', { x: 168, y: 36, cap: 56, id: 'a' });
  files['logo-wordmark.svg'] = svg(168 + Math.ceil(z.width) + 8, 128, defsColor + markG(0, 0, 128) + z.svg, 'Zenith', darkCss);
  const l = word('ZENITH LAUNCHER', { x: 168, y: 46, cap: 36, id: 'b', track: 2 });
  files['logo-wordmark-launcher.svg'] = svg(168 + Math.ceil(l.width) + 8, 128, defsColor + markG(0, 0, 128, markBody + badge) + l.svg, 'Zenith Launcher', darkCss);
}

// banners
const f6 = inter(600), f5 = inter(500);
function banner(launcher) {
  let b = `<defs><radialGradient id="glow" cx="0.5" cy="1" r="0.8"><stop offset="0" stop-color="#5B4BFF" stop-opacity=".55"/><stop offset="1" stop-color="#5B4BFF" stop-opacity="0"/></radialGradient>
<linearGradient id="sunb" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFC95C"/><stop offset="1" stop-color="#FF9A3C"/></linearGradient>
<radialGradient id="halo" cx="0.5" cy="0.5" r="0.5"><stop offset="0.35" stop-color="#FFB938" stop-opacity=".30"/><stop offset="1" stop-color="#FFB938" stop-opacity="0"/></radialGradient>
<linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${P.deep}"/><stop offset="1" stop-color="${P.violet}"/></linearGradient>
<linearGradient id="sun" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFC95C"/><stop offset="1" stop-color="#FF9A3C"/></linearGradient>
<linearGradient id="hz" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2A2A70"/><stop offset="1" stop-color="${P.night}"/></linearGradient></defs>`;
  b += `<rect width="1280" height="640" fill="${P.night}"/><rect width="1280" height="640" fill="url(#glow)"/>`;
  // sun + halo on the right, above the horizon
  b += `<circle cx="1060" cy="170" r="230" fill="url(#halo)"/><circle cx="1060" cy="170" r="96" fill="url(#sunb)"/>`;
  b += `<path d="M0 560C300 520 980 520 1280 560V640H0Z" fill="url(#hz)"/><path d="M0 560C300 520 980 520 1280 560" fill="none" stroke="#F4F1FF" stroke-opacity=".55" stroke-width="3"/>`;
  b += markG(100, 150, 200);
  const x = 350;
  if (!launcher) {
    const z = word('ZENITH', { x, y: 168, cap: 96, color: P.white, id: 'z', track: 3 });
    b += z.svg;
    b += textPath(f6, 'Minecraft en Rust, en su punto más alto.', x, 342, 36, P.white).svg;
    b += textPath(f5, 'fork de Horizon', x, 392, 24, P.mute).svg;
  } else {
    const z = word('ZENITH', { x, y: 148, cap: 84, color: P.white, id: 'z', track: 3 });
    b += z.svg;
    b += word('LAUNCHER', { x, y: 268, cap: 44, color: P.amber, id: 'l', track: 12 }).svg;
    b += textPath(f6, 'El lanzador de Zenith', x, 358, 32, P.white).svg;
    b += textPath(f5, 'fork de XMCL', x, 404, 24, P.mute).svg;
  }
  return svg(1280, 640, b, launcher ? 'Zenith Launcher' : 'Zenith');
}
files['banner.svg'] = banner(false);
files['banner-launcher.svg'] = banner(true);

mkdirSync(out, { recursive: true });
for (const [n, c] of Object.entries(files)) writeFileSync(path.join(out, n), c);
console.log('wrote', Object.keys(files).join(', '));
