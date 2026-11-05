# Marca de Zenith

Zenith no está afiliado a Mojang ni a Microsoft. Minecraft es una marca de Mojang Synergies AB.
Zenith es un fork de [Horizon](https://github.com/horizon-rust/horizon) y su lanzador es un fork
de XMCL. La marca es original: no imita logotipos, tipografías ni recursos de Minecraft, de
Horizon ni de XMCL.

## La idea

*Zenith* es el punto más alto: el sol en su cenit, el punto justo sobre tu cabeza. El proyecto
base se llama Horizon, así que la idea es **subir por encima del horizonte**. El símbolo es un
disco solar en lo alto y, debajo, el arco claro de un horizonte curvo, dentro de un cuadrado de
esquinas redondeadas con un degradado de índigo (noche) a violeta (amanecer). Son dos formas
simples, así que se lee bien a 16 px.

El logotipo de texto (`ZENITH`, `LAUNCHER`) está dibujado a mano con trazos geométricos y
esquinas vivas. Es propio y no es una fuente.

## Archivos

| Archivo | Uso |
|---|---|
| `logo-mark.svg` | Símbolo, cuadrado. Funciona sobre fondo oscuro y claro (lleva su propio fondo). |
| `logo-mark-mono-black.svg`, `logo-mark-mono-white.svg` | Una sola tinta; el símbolo queda calado (transparente) sobre un cuadrado sólido. |
| `logo-mark-launcher.svg` | Símbolo con la insignia de "jugar" para Zenith Launcher. |
| `logo-wordmark.svg`, `logo-wordmark-launcher.svg` | Símbolo + `ZENITH` / `ZENITH LAUNCHER`. El texto cambia a claro con `prefers-color-scheme: dark`. |
| `banner.svg`, `banner-launcher.svg` | 1280x640 para README y redes. |
| `png/` | Exportaciones: `logo-mark-{16..1024}.png`, `banner.png`, `banner-launcher.png`, `social-preview.png`. |
| `icons/` | `zenith.ico`, `zenith-launcher.ico` (capas 16 a 256 px), `icon-512.png`, `icon-1024.png` (Electron/electron-builder). |
| `palette.json`, `palette.css` | Colores y variables CSS (tema oscuro y claro) para el tema del lanzador. |
| `tools/` | Scripts para regenerarlo todo (ver `tools/README.md`). |

## Paleta

Marca:

| Nombre | Hex | Uso |
|---|---|---|
| Primario | `#5B4BFF` | Violeta del degradado del símbolo, botones de marca |
| Primario profundo | `#231B8F` | Índigo del degradado, parte alta del símbolo |
| Acento (sol) | `#FFB938` | Disco solar, destacados; degradado hasta `#FF9A3C` |
| Noche | `#0E1030` | Fondo oscuro |
| Tinta | `#14163A` | Texto sobre fondo claro |
| Bruma | `#F4F1FF` | Arco del horizonte, fondo claro suave |

Tokens de interfaz:

| Token | Oscuro | Claro |
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

## Tamaños mínimos y espacio libre

- Símbolo: mínimo 16 px (favicon, bandeja); a 16 y 24 px usa siempre el símbolo solo, nunca el
  logotipo de texto.
- Logotipo con texto: que la altura de las letras mayúsculas sea de al menos 14 px.
- Espacio libre alrededor: al menos la cuarta parte del lado del símbolo (1/4 del ancho del
  símbolo) por todos los lados.
- La insignia de "jugar" del lanzador solo se usa a 32 px o más en material de marca; en iconos
  pequeños es aceptable porque la forma sigue siendo legible.

## Qué hacer y qué no

Sí:
- Usa los archivos SVG o PNG tal cual; escala proporcionalmente.
- Usa el símbolo sobre fondos oscuros o claros; ya lleva su fondo.
- Usa las variantes monocromas cuando solo hay una tinta (sellos, grabado, impresión en negro).

No:
- No deformes, gires, recortes ni cambies los colores del símbolo.
- No pongas sombras, brillos ni contornos sobre el símbolo.
- No lo combines con imágenes de bloques, creepers ni logotipos de Minecraft, Mojang o Microsoft.
- No uses el logotipo de Horizon ni el de XMCL en lugar del de Zenith, ni de forma que parezca un
  producto oficial de ellos.
- No sugieras que Zenith es un producto oficial de Mojang o Microsoft.

## Tipografía

Recomendada, no incluida: **Inter** (SIL Open Font License 1.1, <https://rsms.me/inter/>) para
interfaz y documentación, en pesos 500 y 600 para titulares. Si no está disponible, la pila
`system-ui, "Segoe UI", Roboto, sans-serif` es válida. El eslogan de los banners es Inter 600/500
convertido a contornos, de modo que los SVG no necesitan la fuente instalada; Inter se distribuye
con su licencia OFL (Copyright 2016 The Inter Project Authors).

## Subir la vista previa social en GitHub

La API de GitHub no tiene un endpoint para esto: es un paso manual. En el repositorio:
**Settings > Social preview > Edit > Upload an image** y elige `branding/png/social-preview.png`
(1280x640).

## Regenerar

Ver `tools/README.md`.
