# Herramientas de marca

Todo lo de `branding/` (SVG, PNG, ICO) se regenera con dos scripts de Node (24 o más). No se sube
`node_modules`: las dependencias se instalan en un directorio temporal cualquiera.

```sh
mkdir /tmp/zenith-brand-deps && cd /tmp/zenith-brand-deps
npm init -y
npm i @resvg/resvg-js opentype.js @fontsource/inter     # Windows, Linux y macOS, sin instalar nada del sistema

cd <raíz del repo>
export BRAND_DEPS=/tmp/zenith-brand-deps                # PowerShell: $env:BRAND_DEPS = "..."
node branding/tools/build-svg.mjs                       # escribe los SVG de branding/
node branding/tools/render.mjs                          # escribe branding/png/ y branding/icons/
```

- `build-svg.mjs` genera los SVG de forma determinista. Las letras de `ZENITH` y `LAUNCHER` son
  trazos geométricos dibujados a mano (sin fuente). Solo el eslogan de los banners es texto: se
  convierte a contornos (paths) con opentype.js a partir de Inter (SIL OFL 1.1), así que los SVG no
  dependen de ninguna fuente instalada.
- `render.mjs` rasteriza con resvg sin cargar fuentes del sistema y arma los `.ico` (contenedor ICO
  con capas PNG de 16, 24, 32, 48, 64, 128 y 256 px) con código propio.
- Los PNG no llevan metadatos de texto ni rutas locales.
