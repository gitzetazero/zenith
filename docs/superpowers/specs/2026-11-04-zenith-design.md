# Zenith: diseño del fork de Horizon

## Contexto

Zenith es un fork de [horizon-rust/horizon](https://github.com/horizon-rust/horizon), Minecraft: Java Edition 26.3 reescrito en Rust (licencia MIT). Nace por un problema real: el usuario no pudo entrar a `<servidor de prueba>` ("¡Cliente no compatible! Por favor, usa 1.7.2-26.2"). Horizon ya trae un traductor de versiones, pero está apagado por defecto (`older_servers_enabled()`, `crates/horizon-client/src/net.rs:564`).

Objetivo: un cliente con más compatibilidad (entre otras cosas, servidores de versiones más antiguas), que se use desde [XMCL](https://xmcl.app/en/) y después desde una traducción completa de XMCL a Rust, con más FPS y shaders que funcionen, recibiendo siempre los parches de Horizon.

### Qué responde hoy ese servidor (medido con pings de estado, sin login)

- Es un proxy Velocity con una sola dirección y anuncia `Velocity 1.7.2-26.3`. La captura decía 26.2, así que lo más probable es que se haya actualizado desde entonces. **Puede que hoy ya se pueda entrar sin traductor; falta probarlo.**
- Devuelve el número de versión que se le declara: 777, 776, 775, 767 y 47 respondieron igual en la última medición. Su ping no sirve para saber qué versión usar.
- Con pings seguidos corta las conexiones a partir del cuarto. Con pausas de 15 s respondió a todos. Parece un límite de frecuencia.

## Decisiones del usuario

- **Nombre:** Zenith.
- **Launcher:** XMCL traducido por completo a Rust (1:1), funcionando con el fork. Hasta entonces, XMCL normal con un puente. Los cuatro frentes pedidos antes siguen siendo la meta: más opciones de juego, mods y shaders más fáciles, apariencia y diagnóstico.
- **Versiones:** el juego sigue siendo una sola versión por dentro y traduce para los servidores, como ViaFabric. No se juega "de verdad" en versiones antiguas.
- **Servidores objetivo:** 1.20.2 o más, incluidas las redes que aceptan rangos. El mínimo pedido al principio, 1.21.8, es obligatorio.
- **Público:** privado primero (el usuario y sus amigos), preparado para abrirse después.
- **Repositorio:** público en GitHub, sin anunciarlo.
- **Descargas:** GitHub Releases públicas sin anunciar, antes de revisar el tema legal. El usuario acepta que cualquiera con el enlace pueda bajarlas.
- **Publicación:** automática. Si la sincronización pasa las comprobaciones, se publica y se instala sola.
- **Rendimiento:** más FPS sin instalar nada. No se cargan `.jar` de Sodium ni similares, y no hay catálogo de mods de rendimiento.
- **Shaders:** Complementary, BSL, Bliss o Photon, y Sildur's o ligeros.
- **Parches:** Zenith se mantiene al día con los del repo original.

## Objetivos, no objetivos y anti-objetivos

**Objetivos**
- Abrir XMCL, pulsar Jugar y estar en Zenith con la cuenta de Microsoft del jugador.
- Entrar sin configurar nada a servidores de 1.20.2 a 26.3, incluido `<servidor de prueba>`.
- Traer los parches de Horizon cada día, con mínima intervención manual.
- Mejorar FPS y shaders con mejoras medibles.

**No objetivos por ahora**
- Servidores que solo aceptan versiones anteriores a 1.20.2 (por ejemplo, 1.8 PvP).
- Jugar con las reglas o el mundo de una versión antigua.
- Ejecutar mods Java en el motor de Rust.
- Mejorar el launcher de Rust de Horizon: queda en el repo sin tocar y Zenith no lo distribuye.
- Web, registro de mods o servicios propios.

**Sería un fracaso aunque funcione**
- Que traer parches de Horizon sea tan doloroso que el fork se quede atrás.
- Que Zenith toque o sobrescriba los mundos de la instalación de Horizon del usuario.
- Que el juego contacte servicios de Horizon sin que el jugador lo pida.
- Que Zenith arranque para una cuenta que no posee el juego.
- Publicar archivos de Mojang o shader packs de terceros en el repo.

## Reglas del fork (decisión de Claude)

Son lo que hace viable la sincronización. Irán en `ZENITH.md`.

1. **No renombrar lo interno.** Crates, rutas, binarios internos y variables `HORIZON_*` siguen igual. Renombrarlos daría conflictos en cada sincronización.
2. **Código de Zenith en `zenith/`, con su propio workspace de Cargo** (como hace `launcher/`), para no tocar el `Cargo.toml` ni el `Cargo.lock` de Horizon. En archivos de Horizon, solo enganches pequeños marcados con `// zenith:`, sin reformatear nada.
3. **Primero mirar arriba.** Antes de arreglar algo, comprobar con `gh pr list --search` que Horizon no lo está haciendo ya.
4. **Lo genérico se propone a Horizon.** Reduce nuestra diferencia. Cada PR externo necesita el visto bueno del usuario y sigue las reglas de agentes de su `AGENTS.md`.
5. **Se heredan sus reglas de seguridad:** `tools/check_public_tree.py`, nada de archivos de Mojang en el repo, y carpetas temporales `HORIZON_HOME` / `HORIZON_LAUNCHER_HOME` en cualquier ejecución de prueba.

## Fase 1: base del fork

**Primera comprobación, porque decide el resto.** `xtask package` necesita `exporters/`, que Horizon compila contra los jars de Mojang en su Mac (`.github/workflows/game-build.yml`). Los runners gratuitos no tienen esos jars. Plan: reutilizar los `exporters/` del archivo publicado de la misma nightly. Según `docs/launcher-preparation.md` son solo clases originales de Horizon, y Zenith no modifica sus fuentes. Si no se pueden obtener, la publicación automática no funciona en runners gratuitos y se vuelve al usuario con las alternativas (su PC como runner o publicación manual).

**Repositorio**
- Fork público en GitHub como `gitzetazero/zenith`, remoto `upstream`, clonado en `C:\Users\<usuario>\Documents\horizon`.
- Desactivar los workflows heredados con `gh workflow disable`, sin borrarlos: dependen de runners y secretos de Horizon, y borrarlos daría conflictos.

**Sincronización** (`.github/workflows/zenith-sync.yml`, diario, en runners gratuitos de GitHub)
- Toma la última etiqueta `nightly-source/*` alcanzable desde `upstream/main`. Esas etiquetas son permanentes, y se excluyen las nightlies con PRs sin integrar. No sigue cada commit: Horizon hace miles a la semana.
- Fusiona con merge en una rama temporal y ejecuta fmt, clippy y tests.
- Comprueba que `horizon-client --launch-api` sigue ofreciendo las opciones que usa el puente de la fase 2. Esa API es experimental y este es el aviso temprano si cambia.
- Si todo pasa, empuja a `main` y dispara la publicación. Si hay conflicto o algo falla, abre un PR con la lista de archivos y no toca `main` ni publica.
- Usa un token con permiso de workflows, porque `GITHUB_TOKEN` no puede empujar los cambios de Horizon en `.github/workflows`.

**Publicación automática** (`.github/workflows/zenith-release.yml`)
- Compila el juego y el puente para Windows y Linux, firma y publica como máximo una build al día.
- Límite aceptado: nadie abre el juego antes de publicar. El smoke test de Horizon cubre su commit, no nuestra fusión, y un fallo que solo se vea jugando llegará a todos.
- Dos frenos baratos: la variable de repo `ZENITH_AUTOPUBLISH=off` detiene las publicaciones, y el puente conserva la build anterior para volver atrás.

**Identidad del juego** (mínimo cambio posible)
- Firma: clave propia para los archivos de Zenith, verificada por el puente con `crates/horizon-release-sign`. La privada es un secreto de Actions.
- Nombre visible: sustituir "Horizon" por "Zenith" en un solo punto, al cargar los textos del juego, sin editar los archivos de idioma.
- Tráfico del juego (decisión de Claude): lista blanca de destinos. Permitidos: Microsoft, Xbox y Mojang, y GitHub para las builds. Solo cuando el jugador lo pide: relay de amigos. Apagados: telemetría y ping "I'm playing" (`crates/horizon-telemetry`, `crates/horizon-client/src/heartbeat.rs`) y cosméticos (`HORIZON_COSMETICS=0`, hoy activos por defecto y ligados al UUID). Se prefieren variables de entorno fijadas por el puente antes que cambiar código.
- Se mantienen `LICENSE`, `THIRD_PARTY_NOTICES.md`, el aviso de no afiliación con Mojang y Microsoft, y el crédito a Horizon.

Como Zenith no distribuye el launcher de Rust, ya no hace falta tocar `launcher/`. Esto elimina la zona con más riesgo de conflictos.

**Terminado cuando:**
- Una nightly real llega a `main` y se publica sola.
- El archivo publicado supera la verificación de firma.

## Fase 2: XMCL como launcher, con un puente

**Por qué es viable.** Horizon documenta cómo arrancarlo desde launchers externos ([`docs/api/launch-guide.md`](https://github.com/horizon-rust/horizon/blob/main/docs/api/launch-guide.md), API experimental 0.1.0). XMCL (MIT, Electron) permite por instancia un "comando previo" que se antepone a la orden de arranque (`prependCommand`, `xmcl-keystone-ui/src/views/BaseSettingJava.vue:157`, `packages/core/launch.ts:701`). No hay que modificar XMCL.

**Inicio de sesión.** Lo hace XMCL, sin modificar y con su propio registro, como con cualquier otro juego. Zenith no incluye ni copia ningún ID de aplicación, así que esta fase no necesita aprobación de Mojang.

**El puente:** `zenith/bridge/`, un programa pequeño en Rust (`zenith-bridge.exe`). XMCL descarga Minecraft 26.3 y Java, y ejecuta `zenith-bridge <java> … net.minecraft.client.main.Main --username … --uuid … --accessToken … --gameDir …`. El puente no arranca Java. Hace esto:
1. **Lee los argumentos** de XMCL.
2. **Comprueba que la cuenta posee el juego** (paso 0 de la guía de Horizon): rechaza si `--userType` no es `msa` y pregunta a los servicios de Minecraft con ese token. Las cuentas offline, ely.by o LittleSkin no arrancan Zenith.
3. **Instala o actualiza Zenith** desde las Releases, verificando la firma. Conserva la build anterior.
4. **Prepara los recursos:** `horizon-prepare --check` y, si falla, `--prepare --complete --progress json` con los archivos que XMCL ya descargó y su Java (`--java-home`). Solo para ese dueño descarga el jar del servidor, que la preparación necesita. Hay una sola carpeta preparada por versión de Minecraft. La primera vez tarda minutos, así que debe verse una ventana de progreso.
5. **Arranca el juego:** `horizon-client --resources P --game-dir G --home G --width W --height H --session-stdin`. La sesión va por entrada estándar, nunca por la línea de comandos. Reenvía la salida, devuelve el código de salida y escribe la ruta del informe de fallo en el formato que XMCL detecta (`Crash report saved to:`).

**Primero una prueba desechable**, con la nightly actual de Horizon sin cambios, antes de construir el puente de verdad. Debe responder:
- si XMCL acepta el comando previo y qué argumentos exactos pasa;
- si deja de marcar la instancia como "arrancando" sin un log de Java;
- si funciona su pestaña de shaders en una instancia vanilla;
- si `--minecraft-dir` sirve con la estructura de carpetas de XMCL.

**Puesta en marcha.** `zenith-bridge setup` deja listo el puente en una carpeta fija y crea la instancia "Zenith" de 26.3 con el comando previo. Si el formato de instancia de XMCL no lo permite con seguridad, se queda en tres pasos manuales documentados.

**Qué da esta fase y qué no**

| Lo que pediste | Con el puente | Con el launcher en Rust (fase 6) |
| --- | --- | --- |
| Apariencia | La de XMCL | Marca y tema de Zenith |
| Opciones de juego | Instancias de XMCL y ajustes dentro del juego | Presets de FPS y ajustes de Zenith en el launcher |
| Mods y shaders | Shaders y resource packs desde Modrinth y CurseForge (por confirmar en la prueba) | Pestaña de mods filtrada, con `.hmod` |
| Diagnóstico | Lo que XMCL detecte de fallos y logs | Panel propio |

**Lo que se pierde respecto al launcher de Horizon**, salvo que se añada luego:
- Compilar mods nativos (`.hnmod`).
- Portar modpacks.
- Alojar servidores.
- Actualizaciones con la lista de versiones de Horizon.

Además, la pestaña de mods de XMCL ofrecerá mods Java que no funcionan en Zenith.

**Límite heredado.** XMCL pasa el token por la línea de comandos al puente, como hace con cualquier juego. Otros programas de ese usuario podrían leerlo mientras dura el arranque. Desaparece en la fase 6.

**Tests.** El analizador de argumentos con líneas reales de XMCL, el rechazo de cuentas sin propiedad o no `msa`, y la comprobación de que el token nunca aparece en la orden de `horizon-client`.

**Terminado cuando:**
- En un perfil limpio de XMCL, con la cuenta del usuario, Jugar abre Zenith en un servidor online.
- Una cuenta offline se rechaza con un mensaje claro.
- Una build con firma mala se rechaza.
- Una captura de tráfico de 10 minutos de juego muestra solo destinos de la lista blanca.
- `%APPDATA%\horizon` y `%APPDATA%\horizon-launcher` quedan sin cambios (comparación antes y después).

## Fase 3: entrar a los servidores

**Elegir versión.** Hoy se elige por coincidencia exacta del número que anuncia el servidor (`older_server`, `net.rs:570`). Nuevo orden:
1. La versión elegida a mano para ese servidor, si la hay.
2. El número anunciado, si es uno que sabemos traducir. Se reutilizan `ServerVersion::from_protocol` y `ALL` (`crates/horizon-protocol/src/adapter.rs`).
3. Si el servidor rechaza durante el login (`net.rs:686`) con un mensaje que incluye un rango como "1.7.2-26.2": un único reintento con la versión más nueva que sabemos traducir dentro de ese rango.

No se sondea con pings por versión: `<servidor de prueba>` responde que sí a todas y corta los pings rápidos, así que no aportaría nada y podría activar protecciones. Sin rango en el mensaje no hay reintento, y el error sugiere usar el selector.

**Cambios**
- `older_servers_enabled()` pasa a estar activo por defecto; `HORIZON_OLDER_SERVERS=0` lo apaga.
- La lista de servidores (`crates/horizon-client/src/multiplayer.rs:261`) usa la misma lógica, para no marcar como incompatible un servidor al que sí se puede entrar.
- Selector por servidor ("Automática", "26.2", …) en la pantalla de edición. Se guarda en un archivo aparte de Zenith indexado por dirección, para no alterar `servers.dat`, que debe seguir siendo compatible con vanilla. Las versiones de 1.20.2 a 1.21.7 se marcan como experimentales.
- Mostrar "Entrando como 26.2" al conectar. Si la traducción falla, el error dice con qué versión se intentó.

**Integrar 1.20.2 a 1.21.8.** Hoy el traductor cubre de 1.21.9 a 26.2. El resto está en [horizon-rust/horizon#8718](https://github.com/horizon-rust/horizon/pull/8718): borrador con conflictos, +18.944 líneas, en desarrollo activo. Se fusiona su rama en Zenith y se resuelven los conflictos. Cuando Horizon lo integre, la sincronización lo absorbe. Mientras tanto, cada cambio de esa rama se vuelve a traer a mano.

**Sin copiar código de Via.** ViaVersion y ViaFabricPlus son GPL-3; Zenith sigue siendo MIT.

**Límite conocido.** El juego sigue siendo 26.3 por dentro. En servidores antiguos puede haber diferencias visibles, y algunos anticheats podrían reaccionar.

**Comprobación**
- Tests de `crates/horizon-protocol` y tests nuevos del selector, con la respuesta real de `<servidor de prueba>` y el mensaje de la captura como datos de prueba.
- Matriz con servidores vanilla reales en local, en modo offline, uno por protocolo: 1.20.2, 1.20.4, 1.20.6, 1.21.1, 1.21.3, 1.21.4, 1.21.5, 1.21.6, 1.21.8, 1.21.10, 1.21.11, 26.1.2, 26.2 y 26.3. Con `horizon-ctl`: entrar, moverse, romper y poner un bloque, chat, inventario, cambiar de dimensión, morir y reaparecer. Se ejecuta a mano, no en cada sincronización, y los jars no se suben al repo.
- `<servidor de prueba>` en automático y también forzando 26.2 con el selector, porque si entra directo como 26.3 la prueba no ejercita el traductor.

**Terminado cuando (decisión de Claude):**
- De 1.21.8 a 26.3, toda la matriz pasa. Es obligatorio.
- De 1.20.2 a 1.21.7, se hace lo posible y lo que falle queda listado como problema conocido.
- El usuario entra a `<servidor de prueba>` desde XMCL sin configurar nada más y juega 30 minutos sin expulsiones. Una expulsión por anticheat se investiga; si viene de ser 26.3 por dentro y no se puede arreglar, se documenta y decide el usuario.

## Fase 4: más FPS

- Medir primero: `tools/perf_bench.py` en el PC del usuario, Zenith contra la misma nightly de Horizon.
- Entregable: presets "Máximo FPS", "Equilibrado" y "Calidad" sobre los ajustes que ya existen (`docs/graphics.md`). Se eligen en el juego, y en el launcher cuando exista la fase 6.
- El motor solo se toca si el benchmark demuestra la mejora, y primero se propone a Horizon, que ya trabaja en esto. Así no se pierde al sincronizar.
- No se promete un porcentaje antes de medir: depende del PC, y el punto de partida se fija en la primera medición.

## Fase 5: shaders

- Partir del código existente (`crates/horizon-shaders`, `crates/horizon-client/src/game/shader_packs.rs`), no reescribir.
- Primero comprobar su estado real ejecutándolo: la hoja de ruta y `docs/shaders.md` no coinciden del todo, y hay fallos abiertos como [#9137](https://github.com/horizon-rust/horizon/issues/9137).
- Orden: Sildur's o ligeros → BSL → Complementary → Bliss y Photon.
- Listo por pack: carga sin errores en Windows, mundo normal, Nether y End, escena fija capturada con `horizon-ctl` y comparada a ojo con el mismo pack en Iris sobre una instancia Java de XMCL, sin caída de FPS inaceptable para el usuario.
- Los packs nunca se suben al repo: se descargan en local (licencias de cada pack). Los datos de prueba del repo son packs propios, como `crates/horizon-shaders/tests/packs/terrain-test`.

## Fase 6: XMCL traducido a Rust (launcher Zenith)

**Decisión del usuario: traducción completa 1:1.** Es la pieza más grande de todo el proyecto: unos 7,8 MB de TypeScript y Vue en unos 1.400 archivos, sin contar tests. Mayor que el launcher de Horizon entero (3,3 MB). Mientras dura, se sigue jugando con XMCL normal y el puente, así que nadie espera. Tendrá su propio plan detallado, con maquetas para aprobar antes de construir.

**Condición para empezar (decisión de Claude):** tener la aprobación de Mojang. Sin ella, el launcher resultante no podría iniciar sesión, y no conviene invertir en esto antes de saberlo.

**Bloqueos que solo puede resolver el usuario**
- **ID de aplicación propio.** Un launcher nuevo no puede usar el de XMCL, que está en `xmcl-runtime/user/pluginOfficialUserApi.ts:12`, ni el de otra aplicación. Ese valor no se traduce. El usuario registra una app en Azure y pide a Mojang la aprobación ("App ID review"; Horizon documenta que tarda de días a semanas, pasos en `docs/microsoft-account.md`). Conviene pedirla al empezar la fase 1, porque no depende de nada.
- **Clave propia de la API de CurseForge**, por el mismo motivo.
- **Si Mojang lo rechaza:** Zenith sigue funcionando con XMCL normal y el puente, y esta fase se replantea con el usuario. No se bloquea nada más.

**Decisiones de Claude, revisables en el plan de esta fase**
- **Repositorio:** `gitzetazero/zenith-launcher`, nuevo, conservando la licencia MIT y el copyright de XMCL. La base queda fijada en la última versión de XMCL al empezar (hoy v0.73.1).
- **Interfaz:** GPUI Fast, el mismo framework del launcher de Horizon. Vuetify no existe en Rust, así que primero se construye una biblioteca de componentes equivalente. "1:1" significa mismas pantallas, flujos y opciones, no igualdad de píxeles.
- **Reutilización:** los archivos de idioma de XMCL se conservan. Antes de escribir algo de cero se mira el launcher de Horizon (MIT), que ya resuelve cuentas, descargas y Modrinth en Rust. El puente de la fase 2 pasa a ser una biblioteca, y el token deja de ir por la línea de comandos.

**El inventario es el contrato.** `docs/xmcl-parity.md` lista cada uno de los 52 servicios, cada pantalla y cada paquete `@xmcl/*`, con su estado: pendiente, portado o excluido. Solo el usuario puede aprobar una exclusión. Excepciones propuestas desde el principio:
- `xmcl-electron-app` y `deskgap-app`, porque los sustituye el propio programa en Rust.
- La telemetría de XMCL y su canal de actualización, porque se sustituyen por los de Zenith.

**Orden, para que cada etapa sea usable**
1. Ventana, componentes, temas, idiomas y ajustes.
2. Cuenta de Microsoft, instancias y arranque de Zenith. Desde aquí ya puede sustituir a XMCL para jugar a Zenith.
3. Descargas: Minecraft, Java, Forge, Fabric, Quilt, OptiFine y espejos, con instancias Java.
4. Recursos: mods, shaders, resource packs y mundos, Modrinth y CurseForge, y `.hmod`.
5. Modpacks: importar y exportar.
6. El resto, incluidos multijugador P2P y cuentas de terceros (Yggdrasil, ely.by, LittleSkin).

Lo propio de Zenith se añade cuando exista su equivalente: presets de FPS, diagnóstico, avisos de qué mods funcionan en cada instancia, y marca y tema.

**Cuentas sin propiedad.** Las cuentas offline o de terceros nunca arrancan el motor de Zenith. Si se portan para instancias Java se decide con el usuario al llegar a esa etapa.

**Método.** Los tests de XMCL (unos 2 MB) se traducen también y sirven de especificación.

**Mantenerse al día con XMCL.** Ya no se puede fusionar su código. Un workflow semanal abre un issue con los commits nuevos de XMCL desde la base (en el último mes, 41 commits y 3 versiones), y se decide cuáles traducir.

**Terminado cuando:**
- El inventario no tiene pendientes.
- Los tests traducidos pasan.
- Cada pantalla se ha comparado con XMCL a 1200x720 y 800x400 (su ventana por defecto y su mínima).
- El usuario lo usa en lugar de XMCL y da el visto bueno.

## Orden y alcance

Orden (decisión de Claude): Fase 1 → 2 → 3 → 4 → 5 → 6. El puente va antes que el traductor porque permite jugar y probar Zenith con la cuenta del usuario sin esperar a Mojang. La traducción va la última porque es la más grande, necesita la aprobación de Mojang y nada depende de ella: los presets de FPS se eligen en el juego hasta entonces.

Aprobar este plan aprueba el diseño completo y permite construir las fases 1, 2 y 3, **incluidos crear el fork público y publicar builds automáticamente**. Cada una de las demás tendrá su plan detallado antes de empezar.

## Lo que necesita el usuario

1. **Volver a probar `<servidor de prueba>`** con Horizon tal cual: el servidor ya anuncia 26.3.
2. **Instalar XMCL** en este PC e iniciar sesión con su cuenta, para la prueba desechable de la fase 2. No se ha comprobado si ya está instalado.
3. **Crear el token de sincronización** (permisos de contenido y workflows) y guardarlo como secreto del repo, sin pegarlo en el chat.
4. **Generar la clave de firma** con el comando que le dé Claude y guardarla como secreto. La privada nunca va al chat ni al repo.
5. **Registrar la app de Azure y pedir aprobación a Mojang**, cuanto antes aunque sea para la fase 6, porque tarda. También una clave de la API de CurseForge. Claude da los pasos; las cuentas son suyas.
6. **Aprobar cada PR hacia Horizon** y las maquetas del launcher.

## Riesgos

- **Publicación automática sin abrir el juego:** es el riesgo aceptado con más impacto. Si hay un fallo, se usa `ZENITH_AUTOPUBLISH=off` y se vuelve a la build anterior.
- **Tamaño de la traducción 1:1:** es varias veces mayor que el resto de lo que hay que construir y deja a Zenith manteniendo un launcher completo por su cuenta. Traducir también lo que no sirve en Zenith (como los instaladores de loaders para el motor de Rust) añade trabajo sin ganancia para ese motor. El inventario permite excluir partes si el usuario cambia de idea.
- **`exporters/`:** si no se pueden reutilizar, la publicación automática cambia (ver fase 1).
- **Launch API experimental:** puede cambiar sin aviso. Se detecta en la sincronización antes de publicar, y entonces hay que ajustar el puente.
- **Dependencia de XMCL:** el comando previo y el formato de argumentos son de XMCL y pueden cambiar en sus actualizaciones, que no controlamos hasta tener el launcher en Rust. XMCL también tiene telemetría propia, con ajuste para apagarla (`disableTelemetry`). Conviene apagarla si no se quiere enviar.
- **Ritmo de Horizon:** los conflictos más probables estarán en el traductor. Si la sincronización se atasca, se pausa la fase en curso y se atiende primero, porque mantenerse al día es un objetivo y no un extra.
- **#8718 sin terminar:** puede cambiar mucho o no integrarse. Si se abandona, ese código pasa a ser nuestro y 1.21.8 depende de ello. De 1.21.9 a 26.2 no depende de él.
- **Traductor experimental:** activarlo por defecto puede provocar expulsiones. Por eso hay interruptor, selector y mensajes claros.
- **Normas de los servidores:** entrar con un cliente no oficial puede ir contra las reglas de un servidor. El usuario debería revisar las de `<servidor de prueba>` antes de la prueba de 30 minutos.
- **Cambio de versión base:** el usuario ya tiene `26.4-snapshot-6` instalado. Cuando Horizon pase a otra versión, llegará como un parche grande en la sincronización, y el puente debe seguir la versión que declare `horizon-prepare --launch-api`.
- **Legal:** el repo original tiene issues abiertos sobre sus afirmaciones de "clean room" ([#1555](https://github.com/horizon-rust/horizon/issues/1555), [#6741](https://github.com/horizon-rust/horizon/issues/6741)). No se han evaluado aquí y el fork los heredaría desde la primera Release pública. Conviene consultar a un abogado antes de anunciarlo.
- **Nombre, antes de anunciarlo:** `zenith-rust` está libre en GitHub, pero "Zenith" ya se usa en otros proyectos de Minecraft, incluido un repo de cheats. Revisarlo entonces.

## Verificación global

- `git log` confirma que `main` contiene la última nightly y `git diff <nightly>..main --stat` muestra una diferencia pequeña fuera de `zenith/`.
- `python3 tools/pr_ready.py` pasa en cada cambio. Necesita JDK 25, que falta en este PC (Rust 1.99 y Python 3.12 ya están).
- Las pruebas de `ZENITH.md` se ejecutan en Windows. Horizon solo declara sus harnesses en Linux y macOS, así que Windows es el hueco a vigilar.
- Después de cada fase, el usuario lo prueba en su PC.

## Sin comprobar todavía

Nada se ha compilado ni ejecutado: esto sale de leer los repos en GitHub y de los pings al servidor. Sin confirmar:
- Si hoy se puede entrar a `<servidor de prueba>` como 26.3, porque un ping no prueba el login.
- Por qué cambió el rango respecto a la captura.
- Qué responde Velocity a una versión que no soporta. De memoria, su versión máxima, en cuyo caso el paso 2 del selector ya resolvería el caso de la captura.
- Todo lo que debe responder la prueba desechable de la fase 2. Mientras no se haga, la viabilidad con XMCL es una deducción a partir de su código, no un hecho.
- De dónde obtener `exporters/`.
- El tiempo real de compilación en GitHub.
- Si GPUI Fast cubre todo lo que XMCL hace con Vuetify y Electron. Se valida con la primera etapa de la fase 6.
