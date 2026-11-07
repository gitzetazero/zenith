# Zenith UI: tema Zenith, barra superior y Addons

Estado: borrador para aprobación. Fecha: 2026-11-07. Nada de esto está construido.

## Intención

Que Zenith se vea y se maneje como Laby Client (referencia: Laby Client v49 sobre Minecraft 26.3, capturado hoy)
con la marca de Zenith. El jugador abre el juego y ve un menú moderno; los extras se activan desde una
pantalla de Addons, dentro del juego, sin instalar nada ni usar otro programa.

Por qué: Zenith es el cliente del dueño y quiere una cara propia y cuidada, sin cargadores de mods ni terminal.

## Qué se construye

1. **Tema Zenith por defecto, tema Vanilla opcional.** Un ajuste "Tema" con dos valores: `Zenith` (por defecto, para
   todos) y `Vanilla` (el aspecto actual del juego). Cambia todos los menús. Sin tema Glass por ahora.
2. **Menú principal Zenith.** Fondo de panorama desenfocado y oscurecido (se reutiliza `menu_blur`), logo con
   marca Zenith, botones oscuros redondeados: "Un jugador" y "Multijugador" a ancho completo; "Screenshots" y
   "Realms" en una fila; una fila de iconos pequeños **vacía** (sin enlaces por ahora); "Opciones" y "Cerrar
   Minecraft". Abajo a la izquierda: "Zenith · Minecraft 26.3". Sin barra superior. Sin Discord, eventos, tienda,
   skin ni app: los huecos se dejan **vacíos**.
3. **Barra superior** en las pantallas de dentro (Un jugador, Multijugador, Addons, Opciones y sus subpantallas;
   no en el menú principal ni en el juego): pestañas "Un jugador", "Multijugador", "Addons", "Opciones"; a la derecha,
   iconos (vacíos por ahora) y la cuenta con su skin.
4. **Opciones y subpantallas con el estilo Zenith**: secciones con título, ajustes en dos columnas, deslizadores con
   acento azul, avisos al pasar el ratón, botón "Hecho" abajo. Pantallas a cubrir: Opciones, Gráficos, Controles,
   Música y sonido, Idioma, Chat, Paquetes de recursos, Accesibilidad, Telemetría, Créditos, Personalizar aspecto,
   En línea.
5. **Pantalla Addons** (en Laby, "Mods"): panel sobre el mundo o el fondo desenfocado, buscador, filtro de
   categoría, vista cuadrícula o lista, cada Addon con interruptor y, si tiene ajustes, una subpantalla con "Volver".
   Incluye un Addon **Aspecto** con el selector de tema y vistas previas (Zenith, Vanilla).
6. **Dos Addons:** **FPS** y **Teclas en pantalla**. Tienen interruptor y sus ajustes (posición, tamaño, color).
   Se apoyan en lo que ya existe: contadores de FPS y clics (`extras.rs`) y dibujo de HUD (`extras_hud.rs`).

## Objetivos / no-objetivos / antiobjetivos

Objetivos: parecido muy alto al diseño de Laby; que todo texto esté traducido (en y es); que cambiar de tema sea
inmediato; que el tema Vanilla deje el juego como hoy.

No-objetivos: mods de terceros, cargador de mods o plugins, Addons descargables, tema Glass, enlaces a Discord/web/
tienda, parecido al píxel con la tecnología web de Laby, funciones de cuenta o cosméticos de Laby.

Antiobjetivos (sería un fracaso aunque compile): que el tema Vanilla cambie algo respecto a hoy; que se pierda un
ajuste o un botón que existía; que el juego en sí (mundo, mecánicas, HUD de vanilla) cambie; textos fijos en inglés
en el código (regla de `AGENTS.md`).

## Decisiones tomadas por el dueño

- Aspecto Laby-con-marca-Zenith por defecto para todos, tema Vanilla opcional, sin Glass.
- La regla de paridad con vanilla vale para el **juego**, no para la interfaz, los menús ni los Addons.
- Menú principal sin barra superior; barra superior solo en las pantallas de dentro.
- Huecos de Discord, eventos, tienda y skin vacíos.
- Addons solo del dueño, incluidos en el juego (sin cargador). Primeros Addons: FPS y Teclas en pantalla.
- Empezar por el menú principal (resultado visible antes).

## Restricciones

- El juego está en Rust y dibuja su interfaz con su motor propio (`crates/zenith-client-render`: `ui.rs`,
  `widgets.rs`, `options_ui.rs`, `graphics_style.rs`, `screen_draw.rs`, `menu_blur.rs`). No hay web ni Java.
- Sin cadenas fijas: textos por claves de traducción (`Text::tr`, `lang::get`), en y es.
- Cada cambio del juego se compila (≈4 min en frío): fases pequeñas.
- Los tests de paridad con vanilla deben seguir pasando; se ejecutan con el tema Vanilla.
- Sin telemetría ni dependencias nuevas salvo necesidad demostrada.
- Plataforma: Windows primero.

## Fases (orden propuesto)

1. Sistema de tema (ajuste Zenith/Vanilla, tokens de color/forma en `zenith-theme`) y menú principal Zenith.
2. Barra superior y estilo de Opciones, Gráficos y Controles; después el resto de subpantallas.
3. Pantalla Addons y Addon Aspecto.
4. Addons FPS y Teclas en pantalla.

## Lo que deja al constructor

Colores exactos, radios, fuente (Laby usa una sans geométrica; Zenith ya usa Geist en el launcher), animaciones,
icono y logo concretos de la marca Zenith, diseño de las vistas previas de tema y detalle de los ajustes de cada Addon.

## Decisiones que tomé yo (revisa)

- El ajuste "Tema" se guarda en las opciones del cliente, no por mundo.
- Los iconos vacíos de la fila del menú y de la barra superior **no se dibujan** (en vez de dejar botones muertos).
- La pantalla de pausa dentro del mundo recibe el estilo Zenith en la misma fase que las Opciones.
- El logo de Zenith lo pongo como texto con la fuente del tema hasta que haya imagen de marca.
- Los tests de paridad con vanilla fuerzan el tema Vanilla.
- Los botones del menú principal son los que Zenith ya tiene hoy (no se añade "Realms" ni "Screenshots" si el
  juego no los tiene), colocados en la distribución de Laby.
- La cuenta de la barra superior muestra el nombre y la skin de la sesión actual; sin menú desplegable por ahora.
