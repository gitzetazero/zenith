# Prueba desechable con XMCL (forma estática)

Tarea 5 del plan de las fases 1 a 3. No se manejó la interfaz de XMCL: nadie podía hacer clic en el equipo y no había aprobación para controlar la pantalla. Se hizo lo que sí se puede hacer sin ella: leer el código de XMCL, ejecutar su generador de argumentos y su manera de lanzar procesos contra el JSON real de la versión 26.3, y probar la nightly real de Zenith. Todo lo que de verdad necesita la interfaz está en el apartado "Lo que NO se pudo verificar".

- XMCL: `Voxelum/x-minecraft-launcher`, commit `aeb5240` (rama principal el 2026-11-05), XMCL 0.73.1, `@xmcl/core` 2.16.3.
- Zenith: Release `nightly` de `gitzetazero/zenith` ("Nightly 2026-11-05 · b933efa").
- Todo lo descargado y generado vive fuera del repositorio, en `%TEMP%\zenith-spike`.
- Fixture: `zenith/bridge/tests/fixtures/xmcl-args.txt`.

## Resumen de lo importante

1. **Bloqueante para el plan.** En Windows, con un comando previo XMCL lanza con `shell: true`, es decir, a través de `cmd.exe`, cuyo límite es 8191 caracteres. La orden de Minecraft 26.3 mide unos 6900 caracteres más 82 veces la longitud de la raíz de datos de XMCL. Con la raíz por defecto (`%APPDATA%\xmcl`, unos 35 a 40 caracteres) la orden tiene unos 9500 a 10 000 caracteres y **`cmd.exe` la rechaza con "La línea de comandos es demasiado larga" sin llegar a ejecutar el puente**. Solo funciona con una raíz de unos 19 caracteres o menos (por ejemplo `C:\xmcl`). Medido, no supuesto.
2. **El puente debe ser un ejecutable de subsistema Windows** (`#![windows_subsystem = "windows"]`). Con `shell: true` y `detached: true`, un ejecutable de consola pierde su salida estándar: XMCL no la recibe y nunca verá `LWJGL Version: ` ni `Crash report saved to:`. Medido.
3. **Hay dos supuestos de la tarea 6 que son falsos para 26.3.** No existe `--userType` y `--width/--height` no aparecen con `-1`. Además `--clientId` y `--xuid` llegan como texto literal sin sustituir (`${clientid}`, `${auth_xuid}`).

## 1. Argumentos exactos que pasa XMCL

### Cómo se genera

`xmcl-runtime/launch/LaunchService.ts`, `#generateOptions()` (líneas 192 a 290) construye las opciones y `launch()` llama a `launch(op)` de `@xmcl/core`, que a su vez usa `generateArguments` (`packages/core/launch.ts:770`). Lo que pasa XMCL:

| Opción de `@xmcl/core` | Valor |
| --- | --- |
| `gamePath` | carpeta de la instancia, `<raíz>/instances/<nombre>` (también es el `cwd`) |
| `resourcePath` | la raíz de datos de XMCL (`getPath()`) |
| `javaPath` | `javaw.exe` junto al Java elegido, si existe y se puede ejecutar; si no, `java.exe` |
| `gameProfile` | `{ name, id }` de la cuenta |
| `accessToken` | token de la cuenta (si no hay, un hexadecimal aleatorio) |
| `userType`, `properties`, `features` | **no se pasan**: `userType` queda en `msa` pero la plantilla de 26.3 no lo usa; `features` queda vacío |
| `launcherName`, `launcherBrand` | `XMCL (0.73.1)` |
| `extraJVMArgs`, `extraMCArgs` | `vmOptions` y `mcOptions` de la instancia o los globales |
| `server` | `{ ip, port }` si la instancia tiene `server` |
| `quickPlayMultiplayer` | `host[:port]` si hay `server` |
| `quickPlaySingleplayer` | `options.world` si se pide |
| `resolution` | `inst.resolution ?? globalResolution` |
| `useHashAssetsIndex` | `true` |
| `prependCommand` | la cadena, partida con `normalizeCommandLine` |
| `extraExecOption` | `{ shell: prepend && prepend.length > 0, detached: true, cwd: gamePath, env }` |
| `prechecks` | `[]` (el de `@xmcl/core` no se usa; las comprobaciones son middlewares de XMCL) |

### Cómo se ejecutó

`@xmcl/core` se empaquetó con esbuild (con un sustituto vacío de `@xmcl/unzip`, que solo se usa para extraer nativos) y se llamó a `generateArguments` y a `launch` con las mismas opciones. Para ver lo que recibe el puente de verdad se compiló un ejecutable de prueba (`zenith-bridge.exe` en `…\Jose Ñ\bridge dir\`, ruta con espacios y con Ñ) que vuelca `GetCommandLineW` y `std::env::args()` a un archivo. El JSON de versión fue una copia del 26.3 real (solo lectura) y las rutas apuntaban a carpetas temporales.

Comando principal:

```
node run.cjs        # generateArguments en 10 escenarios
node real2.cjs      # core.launch() de verdad (spawn con shell: true, detached: true) contra el ejecutable de volcado
```

### Disposición de argv

Con comando previo, `argv` del puente es: ruta del puente, `javaw.exe`, argumentos de la JVM, clase principal, argumentos del juego, extras. El comando previo va **primero y no sustituye a Java**: Java pasa a ser el primer argumento. `javaw.exe` sí se sustituye cuando existe (XMCL lo hace siempre en Windows si el archivo está). Por eso el plan acierta en que `argv[0]` (tras quitar el nombre del puente) es Java, pero es `javaw.exe`.

El fixture es exactamente lo que recibe el puente, sin su propia ruta (43 argumentos, uno por línea). Orden:

```
 1  <raíz>\jre\java-runtime-epsilon\bin\javaw.exe          (barras invertidas)
 2  -XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump
 3  -XX:StackShadowPages=32
 4  --enable-native-access=ALL-UNNAMED
 5  --add-exports
 6  java.base/jdk.internal.misc=ALL-UNNAMED
 7  -Djava.library.path=<raíz>/versions/26.3/26.3-natives/java          (barras normales)
 8  -Djna.tmpdir=…/26.3-natives/jna
 9  -Dorg.lwjgl.system.SharedLibraryExtractPath=…/26.3-natives/lwjgl
10  -Dio.netty.native.workdir=…/26.3-natives/netty
11  -Dminecraft.launcher.brand=XMCL (0.73.1)            (un solo argumento, con espacio)
12  -Dminecraft.launcher.version=XMCL (0.73.1)
13  -cp
14  <82 jars separados por ';'>…;<raíz>/versions/26.3/26.3.jar
15  -DlibraryDirectory=<raíz>/libraries
16-22 -Xmx2G -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20
    -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M     (DEFAULT_EXTRA_JVM_ARGS)
23  net.minecraft.client.main.Main
24-25  --username <nombre>
26-27  --version 26.3
28-29  --gameDir <raíz>/instances/<instancia>
30-31  --assetsDir <raíz>/assets
32-33  --assetIndex e787e8afd149a4cefa89d341b91f5aed8f5d93a0   (el sha1 del índice, no "34")
34-35  --uuid <uuid sin guiones>
36-37  --accessToken <token>
38-39  --clientId ${clientid}                  <- texto literal
40-41  --xuid ${auth_xuid}                     <- texto literal
42-43  --versionType release
```

Detalles medidos, con el JSON de 26.3:

- **No hay `--userType`.** La plantilla `arguments.game` de 26.3 no lo contiene. La suposición de la tarea 6 ("rechaza si `--userType` no es `msa`") no se puede cumplir. Lo único que distingue una cuenta sin propiedad es el resultado de preguntar a los servicios de Minecraft. Para ely.by y LittleSkin XMCL añade `-javaagent:<jar>=<servidor>` y `-Dauthlibinjector.side=client` a los argumentos de JVM (código en `#generateOptions` y `generateArguments`), lo que sí permite rechazarlas antes de la red.
- **`--clientId ${clientid}` y `--xuid ${auth_xuid}`** salen literales, porque XMCL no define esas variables y `format()` deja la clave si no hay valor. El puente debe tratar cualquier valor que empiece por `${` como ausente (xuid `None`).
- **`--width` y `--height` no existen por defecto.** La plantilla los condiciona a la característica `has_custom_resolution`, que XMCL no activa, y XMCL solo los añade al final cuando hay `resolution`. No se ve ningún `-1`. Medido:
  - sin resolución: no hay `--width` ni `--height`;
  - `resolution: {}` (objeto vacío): tampoco;
  - `{ width: 854, height: 480 }`: al final `--height 480 --width 854` (**primero la altura**);
  - `{ fullscreen: true }`: al final `--fullscreen`.
- **Servidor.** Con `server` en la instancia salen los dos juntos y en este orden, tras los demás: `--quickPlayMultiplayer host[:puerto]`, `--server host` y `--port N` (sin `--port` si no hay puerto). Un mundo da `--quickPlaySingleplayer <carpeta>` (con espacios en una sola pieza: `New World`). No se usa `--quickPlayPath`.
- **Extras.** `mcOptions` van después de `--versionType release`, antes de quickPlay/server/resolución.
- `--demo` solo si no hay cuenta.
- **Rutas.** Mezcla de separadores: `javaw.exe` va con `\`; todo lo demás (`--gameDir`, `--assetsDir`, `-cp`, nativos) con `/`. El `cwd` del proceso es `--gameDir`.

### Raíz de Minecraft y disposición en disco

`--assetsDir` es siempre `<raíz>/assets`, así que **la raíz es el padre de `--assetsDir`**. Bajo esa raíz XMCL usa la disposición clásica de `MinecraftFolder`: `versions/26.3/26.3.json`, `versions/26.3/26.3.jar` (el último elemento de `-cp`), `libraries/…` (Maven) y `assets/indexes`, `assets/objects`. El juego vive en `<raíz>/instances/<nombre>` (`--gameDir`), no en la raíz. Es la misma disposición que usa el launcher de Horizon (`horizon-prepare --minecraft-dir` la acepta tal cual, y con ella se preparó más abajo).

Dos matices de XMCL que afectan al puente:

- `--assetIndex` es el sha1 del índice. XMCL instala `assets/indexes/<sha1>.json` y crea un alias `assets/indexes/34.json` con una operación `link` (`packages/installer/assets.ts:104`). El alias no se pudo comprobar en disco sin instalar el juego: si no existiera, `--minecraft-dir` no encontraría el índice y habría que pasar `--assets-dir` con el índice por su nombre.
- Cada instancia lleva uniones (`libraries/`, `versions/`) hacia la raíz, creadas por `pluginLaunchPrecheck.ts`. No cambian lo que recibe el puente.

### Variantes medidas

Con el mismo experimento, solo cambian los extremos. La última parte del argv en cada caso:

```
Servidor + puerto:   … --versionType release --quickPlayMultiplayer play.example.invalid:25566 --server play.example.invalid --port 25566
Servidor sin puerto: … --versionType release --quickPlayMultiplayer play.example.invalid --server play.example.invalid
Mundo:               … --versionType release --quickPlaySingleplayer New World
Resolución:          … --versionType release --height 480 --width 854
Pantalla completa:   … --versionType release --fullscreen
Extras:              … --versionType release --foo "bar baz"
```

## 2. Comando previo: citas, división y ejecución

### División (`xmcl-runtime/launch/utils/cmd.ts`)

`splitCommandLine` usa `/[^\s"']+|"([^"]*)"|'([^']*)'/g`. Reglas medidas:

| Valor del campo | Resultado |
| --- | --- |
| `"C:\ruta con espacios\zenith-bridge.exe"` (comillas dobles) | una sola pieza, sin comillas. Funciona. |
| `'C:\ruta con espacios\zenith-bridge.exe'` (simples) | igual |
| `C:\Users\Jose Ñ\bridge dir\zenith-bridge.exe` (sin comillas) | **se parte en 3 piezas** (`…\Jose`, `Ñ\bridge`, `dir\zenith-bridge.exe`) y falla: `""C:\…\Jose"" no se reconoce como un comando interno o externo`. |
| comillas vacías `""` | no se tratan como cadena vacía (la rama `match[1]` es falsa y se añade el texto `""`) |

La Ñ y otros caracteres no ASCII pasan intactos en ambos sentidos (verificado con el volcado: el puente recibe `Jose Ñ` correctamente).

(Superseded: ver apartado 10) **Para `setup` (tarea 10): `prependCommand` debe llevar la ruta del puente entre comillas dobles.** Con argumentos propios del puente se pueden añadir detrás.

### Ejecución (`@xmcl/core` `launch()`)

Con `prependCommand` no vacío, `extraExecOption.shell` es `true`. `launch()` entonces envuelve **cada** argumento entre comillas dobles (`args.map(a => `"${a}"`)`) y llama a `spawn(args[0], args.slice(1), { shell: true, detached: true, cwd, env })`. En Windows Node lanza `cmd.exe /d /s /c "<todo junto>"`. Consecuencias medidas:

- **Límite de 8191 caracteres de `cmd.exe`.** Esto es lo más grave. Se ejecutó `launch()` real con la orden de 26.3 variando solo la longitud de la raíz (ruta del puente de 70 caracteres):

  | Longitud de la raíz | Línea de comandos | Resultado |
  | --- | --- | --- |
  | 5 (`C:\zx`) | 6960 | el puente se ejecuta |
  | 19 | 8122 | el puente se ejecuta |
  | 20 | 8204 | **"La línea de comandos es demasiado larga."**, el puente no arranca, código 1 |
  | 22 | 8370 | igual |
  | 36 (una carpeta temporal normal) | ~11 000 | igual |

  La línea crece 82 caracteres por cada carácter de la raíz (82 apariciones de la raíz en la ruta de las librerías y los nativos) y 1 por cada carácter de la ruta del puente. La raíz por defecto de XMCL (`%APPDATA%\xmcl`) mide unos 35 a 40 caracteres, así que **falla**. Sin comando previo no hay `shell` y no hay este límite (el límite de `CreateProcess` es 32 767). No se encontró en XMCL ninguna medida contra esto (`grep` de "too long", "8191", "E2BIG": nada).
- Dentro de `cmd.exe`, **`%VAR%` se expande**: un argumento `100%ZXTEST%` llegó como `100EXPANDED`. Un `"` dentro de un argumento se pierde (`q"uote` llegó como `quote`). `&` entre comillas pasa bien. Un nombre de instancia o ruta con `%…%` o `"` se corrompe antes de llegar al puente.
- **`process.pid` es el de `cmd.exe`, no el del puente.** `kill()` sin fuerza (`process.kill()`) solo termina `cmd.exe` y deja vivos al puente y al juego. `kill(…, force)` sí usa `taskkill /pid … /f /t` (árbol completo) (`LaunchService.ts:733`). El comportamiento del botón de la interfaz depende de cuál se use (sin verificar).
- `detached: true` más `shell: true` hacen que `cmd.exe` se cree sin consola.

### La salida estándar del puente (necesita subsistema Windows)

Medido con `child_process.spawn` y las mismas opciones que XMCL (`shell: true`, `detached: true`):

| Ejecutable | stdout que ve el padre |
| --- | --- |
| consola, `shell` + `detached` (**el caso de XMCL**) | `""` (vacío, **se pierde**) |
| subsistema `windows`, `shell` + `detached` | `LWJGL Version: zenith` |
| consola, `shell` sin `detached` | `LWJGL Version: zenith` |
| consola, sin `shell`, con `detached` | `LWJGL Version: zenith` |

Es decir: con la combinación que usa XMCL, un puente de consola no puede hablarle a XMCL. La causa probable es que `cmd.exe` no tiene consola (`DETACHED_PROCESS`), un programa de consola hijo recibe una consola nueva y su salida va allí, no a la tubería (inferencia; la ventana no se observó). Con `#![windows_subsystem = "windows"]` la salida llega. **Decisión:** el puente va con `#![windows_subsystem = "windows"]`, lanza `horizon-client` con `CREATE_NO_WINDOW` y tuberías, y reenvía stdout y stderr. La preparación en consola propia (`CREATE_NEW_CONSOLE`, tarea 9) no tiene este problema porque es un proceso aparte con su propia consola.

## 3. Instancia: dónde vive y qué necesita

- **Raíz de datos:** `%APPDATA%\xmcl\root` es un archivo de texto con la ruta (`xmcl-runtime/app/LauncherApp.ts:348`). Si no existe, es el primer arranque y la interfaz pide la ruta (`bootstrap`). Si falla la lectura, usa `%APPDATA%\xmcl`. `LAUNCHER_NAME` es `xmcl`.
- **Instancias gestionadas:** `<raíz>/instances/<nombre>/instance.json`. Al arrancar, `InstanceService` lee la carpeta `instances/` entera (`readdirEnsured`) y además `%APPDATA%\xmcl\instances.json` (lista de instancias externas, grupos y selección). Una carpeta nueva con `instance.json` se detecta **solo al arrancar XMCL** (se cargan una vez al inicio); XMCL debe reiniciarse. Si el nombre del campo `name` no coincide con el nombre de la carpeta, XMCL **renombra la carpeta** a `filenamify(name)` (`InstanceService.ts`, función `loadInstance`).
- **Esquema** (`packages/instance/instance.ts`, zod 4): todos los campos tienen valor por defecto o son opcionales. Se ejecutó `InstanceSchema.parse` y `createInstance` (empaquetados con esbuild) con el JSON mínimo `{ "name": "Zenith", "runtime": { "minecraft": "26.3" }, "prependCommand": "\"C:\\ruta\\zenith-bridge.exe\"" }` y se aceptó, completando `edition: "java"`, el resto de `runtime` vacío, `version: ""`, fechas a 0, etc. Si falta `runtime.minecraft`, XMCL pone la última versión. Lo cual cuadra con el plan: bastan `name`, `runtime.minecraft` y `prependCommand`.
- **`prependCommand` por instancia:** campo `prependCommand` (cadena, opcional) de `instance.json`; se edita en el ajuste de Java de la instancia (`BaseSettingJava.vue:157`). Se usa `inst.prependCommand ?? globalPrependCommand` (`xmcl-runtime-api/src/util/launch.ts:140`); el global (`globalPrependCommand`, por defecto `''`) vale para todas las instancias que no definan el suyo.
- **Java:** `java` es opcional en el esquema; si falta, la interfaz elige o instala el Java que pide el JSON de versión (`javaVersion: java-runtime-epsilon`, 25). Esto no se pudo comprobar.
- **Lo que XMCL descarga y el puente necesita:** todo lo listado antes, bajo `<raíz>/versions`, `libraries`, `assets`, más `jre/<componente>/bin` para el Java gestionado.

## 4. Detección de "arrancó" y de fallos (`createMinecraftProcessWatcher`, `packages/core/launch.ts:524`)

Solo se vigila **stdout** (stderr no interviene en la detección). Por cada fragmento de salida, en este orden de `else if`:

1. Contiene `---- Minecraft Crash Report ----`: guarda el fragmento como informe.
2. Contiene `Crash report saved to:`: la ruta es lo que sigue a `Crash report saved to: #@!@# ` (se saltan exactamente `len("Crash report saved to: #@!@# ")` caracteres desde el inicio de la marca). Después `LaunchService` corta en el último `.txt`.
3. Contiene `Crash report saved to ` (sin dos puntos): la ruta es lo que sigue.
4. Si aún no está listo y contiene cualquiera de `Missing metadata in pack`, `Registering resource reload listener`, `Reloading ResourceManager`, `LWJGL Version: `, `OpenAL initialized.`: emite `minecraft-window-ready` (una sola vez).
5. Si aún no está listo y contiene ` Preparing level `: igual.
6. Contiene `Failed to start the minecraft server`: guarda el fragmento como informe.

Salida: `exit` del proceso. Se marca como fallo si el código no es 0, o hay informe, o hay ruta de informe. Si hay informe y el código es 0, lo cambia a 1. **Las comprobaciones son por fragmento de salida y por subcadena**; lo normal es imprimir cada línea en una escritura aparte. El texto exacto del puente debe ser `Crash report saved to: #@!@# <ruta>` (la forma sin `#@!@#` también se acepta en la rama 3, pero la rama 2 gana si hay dos puntos).

El stderr se acumula en `errorLogs` y se emite como `minecraft-stderr`; solo se muestra en el diálogo de fallo y en el registro. Nada de lo anterior se pudo comprobar en la interfaz (qué muestra con código 1, etc.).

Riesgos para un programa que no es Java:

- **No hay comprobación de Java sobre el comando previo.** El middleware `java-validation` (`pluginLaunchPrecheck.ts`) valida el Java de la instancia (`input.java`), no el comando previo. Si el Java es válido, el puente no estorba. Se comprobó que `launch()` usa `prechecks: []`.
- `process.unref()` es solo para que XMCL pueda salir sin esperar; no cambia nada para el puente.
- Tras `spawn`, si `process.pid` no es un número, XMCL espera 1 s al evento `error`. Con `cmd.exe` siempre hay `pid`, así que un puente que no existe o falla **no produce un error de "Java inválido"**: aparece como una salida con código 1 y el mensaje de `cmd.exe` por stderr.
- XMCL decodifica la salida adivinando la codificación (`guessEncodingByBuffer`): conviene que el puente escriba en UTF-8 y en líneas completas.
- `stdin` queda abierta como tubería (XMCL nunca escribe ni cierra); el puente no debe leer de ahí.
- Hay un middleware de idioma (`sync-language`) que puede escribir `options.txt`, solo si el usuario lo activa.

## 5. Launch API de la nightly

Descarga (`gh release download nightly -R gitzetazero/zenith`), comprobación de firma con la clave de `zenith/bridge/zenith-release-keys.txt`, y desempaquetado en carpeta propia:

```
$ cargo run -q -p horizon-release-sign -- verify --tag nightly --key "<clave>" <carpeta con SHA256SUMS, SHA256SUMS.sig, zip y tar.gz>
nightly: 2 files match SHA256SUMS            (código de salida 0)
```

(Hay que tener también el `.tar.gz` de Linux: `SHA256SUMS` lo lista y la verificación falla si falta, no es opcional.) El zip pesa 36,9 MB, desempaquetado 100 MB, y **trae una carpeta raíz `horizon-windows-x64/`** con `horizon-client.exe`, `horizon-prepare.exe`, `horizon-ctl.exe`, `research-lock.json`, `exporters/`, `mods/`, `tools/`, `crates/`, `LICENSE` y `PLAY.txt`.

`horizon-client.exe --launch-api` (código 0):

```json
{ "crash_report_line": "Horizon: crash report saved to ", "exit_codes": [0,1,2,101],
  "horizon": "Nightly 2026-11-05 · b933efa", "launch_api": "0.1.0", "minecraft": "26.3",
  "options": [ "--resources DIR", "--game-dir DIR", "--home DIR", "--player-dir DIR", "--width N", "--height N",
               "--fullscreen", "--session-file PATH", "--session-stdin", "--quickPlaySingleplayer WORLD",
               "--quickPlayMultiplayer HOST[:PORT]", "--launch-api" ] }
```

`horizon-prepare.exe --launch-api` (código 0):

```json
{ "launch_api": "0.1.0", "horizon": "nightly-2026-11-05", "minecraft": "26.3", "java_major": 25,
  "schema": 7, "output_version": 3,
  "options": [ "--prepare", "--server-jar PATH", "--minecraft-dir PATH", "--version-json PATH", "--client-jar PATH",
               "--libraries-dir PATH", "--assets-dir PATH", "--java-home PATH", "--root PATH", "--output PATH",
               "--check", "--complete", "--progress json", "--launch-api" ],
  "exit_codes": [0,1,2] }
```

Notas: los dos campos `horizon` tienen formato distinto (`Nightly 2026-11-05 · b933efa` frente a `nightly-2026-11-05`), así que no se pueden comparar entre sí. `--root PATH` existe en `horizon-prepare` y no está en la guía; ver el aviso sobre `.local` más abajo.

## 6. Preparación y prueba de humo

### Preparación (solo lectura sobre la carpeta de Minecraft del launcher de Horizon)

```
horizon-prepare.exe --prepare --server-jar <…>\server-jars\26.3.jar \
    --minecraft-dir <…>\horizon-launcher\minecraft --java-home "C:\Program Files\Eclipse Adoptium\jdk-25.0.5.8-hotspot" \
    --output %TEMP%\zenith-spike\prepared --complete --progress json
```

- Salida de código 0 en 31 s (el programa dice "Done in 26.6 s"). 72 líneas JSON por stdout, stderr vacío, sin errores. Terminó con `{"fraction": 1.000}`.
- Mensajes relevantes: importó el cliente 26.3 desde la carpeta de Minecraft, "Checked the game files in 1.4 s", exportó estados de bloques, compiló los exportadores (cliente y servidor, unos 3 s cada uno), copió assets y sonidos (4962 archivos, 377,3 MB).
- **No descargó nada**: la preparación solo pidió archivos locales (comprobado por la salida; no se midió la red con otra herramienta).
- Tamaño de la carpeta preparada: **595,3 MB, 28 178 archivos**. Además:
  - la primera arrancada del cliente añadió `horizon-data.bin` (120 MB) dentro de esa carpeta (ver abajo), así que la carpeta debe ser escribible y queda en unos 715 MB;
  - la preparación escribió **398,7 MB (1947 archivos) en `<build>\.local\minecraft\26.3`**, dentro de la carpeta de la build desempaquetada, no en `--output`. Ya existe `--root PATH` para mover esto; no se probó. Si el puente guarda las builds en una carpeta que no debe modificarse, hay que pasar `--root`.
- `horizon-prepare.exe --check --output …\prepared`: código 0, sin salida.

### Arranque (humo)

Variables `HORIZON_HOME`, `HORIZON_NO_AUDIO=1`, `HORIZON_NO_HEARTBEAT=1`, `HORIZON_TELEMETRY_URL=off`; argumentos `--resources <prepared> --game-dir <game> --home <home> --width 640 --height 360 --session-stdin` y la sesión falsa `{"name":"ZenithTest","uuid":"…","access_token":"FAKE","user_type":"msa"}` por entrada estándar.

- Seguía vivo a los 25 s. **stdout vacío**; todo sale por stderr, en texto, empezando por `Horizon Nightly 2026-11-05 · b933efa`.
- `logs/latest.log` (en la carpeta `--home`): renderizador en Vulkan sobre la GPU discreta, `HDR output HDR10 (PQ)`, `core-ready after 876 ms`, `loading screen shown after 1739 ms`, `world renderer ready after 1935 ms`, `first frame after 83 ms`, `resources loaded in 4.9s; 35723 block states, 1658 items, 67 biomes`, `rest-ready after 4866 ms`. La única línea de error: `Failed to retrieve profile key pair: InvalidSession`, esperable con un token falso.
- **No hay una línea que diga "pantalla de título"**: llegó a "loading screen shown", "world renderer ready" y "rest-ready", y el proceso siguió vivo. No se pudo ver la ventana (sin control de pantalla). Es consistente con haber llegado al título, no es una prueba.
- A los 17 s escribió `horizon-data.bin` (120 MB) en la carpeta preparada (`wrote horizon-data.bin in 17451 ms`, hilo `Prepared data file`). Terminar el proceso antes de ese punto, o dejar la carpeta de solo lectura, tiene un efecto no probado.
- Se terminó a la fuerza (`taskkill /T /F`); el código de salida que recogió el intérprete (143) no es el del cliente. El comportamiento al cerrarse la ventana no se probó.
- Código de salida de opciones rechazadas: `--width abc` y `--bogus` dan código 2 y un mensaje (`Horizon: --width: invalid digit found in string`), como dice la guía.
- Creó en `--game-dir`: `config/`, `options.txt` y `horizon-graphics.txt`.

## 7. Lo que NO se pudo verificar sin la interfaz de XMCL

- Que la interfaz de XMCL acepte un `instance.json` escrito a mano. Solo se comprobó, con el esquema zod empaquetado, que el JSON mínimo se analiza sin error, y por el código que `InstanceService` carga todas las carpetas de `instances/` al arrancar. No se comprobó que aparezca en la lista, que el botón de jugar funcione ni que XMCL no lo reescriba de forma inesperada.
- El botón de jugar y el camino completo desde la interfaz: `instanceLaunch.ts`, comprobaciones previas del lado de la interfaz, descarga de Java 25 y de 26.3, inicio de sesión. Solo se reprodujo lo que hace el servicio de lanzamiento una vez recibidas las opciones.
- Que XMCL deje de marcar la instancia como "arrancando" al ver `LWJGL Version: zenith`, y que detecte `Crash report saved to: #@!@# <ruta>`: solo la lógica (apartado 4), no la reacción de la interfaz.
- Qué muestra XMCL cuando el puente sale con 1 y escribe en stderr.
- Que no se vea una ventana de consola con un ejecutable de consola, y que con subsistema Windows tampoco (solo se midió la salida estándar; una ventana no se observa sin pantalla).
- Que la pestaña de shaders de XMCL funcione en una instancia vanilla y escriba en `shaderpacks/` (el código crea `shaderpacks/` en la instancia con `ensureDir`, `InstanceService.ts:~380`, pero no se probó el flujo).
- Los índices de assets con alias (`34.json` junto a `<sha1>.json`): solo se leyó el código; no se instaló el juego con XMCL.
- Si `horizon-prepare --minecraft-dir <raíz de XMCL>` funciona con una instalación real de XMCL: se probó con la disposición del launcher de Horizon, que es la misma por el código, pero no con archivos instalados por XMCL.
- El comportamiento de "Detener" de la interfaz (`kill` frente a `forceKill` y los procesos hijos).
- Cuentas offline, ely.by o LittleSkin: solo se leyó el código.
- Cualquier medida de red (que `horizon-client` no abra conexiones inesperadas) y la prueba en un servidor online: es la tarea 10.

## 8. Suposiciones de las tareas 6 a 10 que la evidencia contradice

| Tarea | El plan supone | Lo medido | Corrección |
| --- | --- | --- | --- |
| 6 | `--width -1 --height -1` aparecen y significan "sin tamaño" (`minus_one_size_is_none`) | No aparecen nunca por defecto. Con resolución: `--height H --width W` (altura primero). Con pantalla completa: `--fullscreen`. | El test debe ser "sin `--width`/`--height` es `None`". Tolerar `-1` por si llega. Añadir `fullscreen: bool` a `LaunchArgs` y a `client_cmd` (`--fullscreen` está en la Launch API). |
| 6 | El puente rechaza si `--userType` no es `msa` (spec, fase 2, punto 2) | `--userType` no existe en 26.3. | Quitar esa comprobación. Dejar solo el aviso de propiedad por red (token de cuentas offline: 32 hex aleatorios, los servicios responden 401, es `NotOwner`). Rechazar además si hay `-javaagent:` de authlib-injector en los argumentos de la JVM. `user_type` de la sesión es siempre `msa`. |
| 6 | `--xuid` es opcional y viene o no viene | Siempre viene, como texto literal `${auth_xuid}` (y `--clientId ${clientid}`). | Tratar cualquier valor que empiece por `${` como ausente. `xuid` es `None`. |
| 6 | `--server a.b --port 25566` y `quickPlayMultiplayer` son alternativas | Con servidor en la instancia XMCL pasa **ambos** a la vez, en el orden `--quickPlayMultiplayer`, `--server`, `--port`. | Dar prioridad a `--quickPlayMultiplayer`. Los otros dos solo si falta ese (por si una versión antigua de XMCL o de Minecraft los usa solos). |
| 6 | `argv[0]` es Java | Sí, pero es `javaw.exe` y no `java.exe`, y con ruta mixta (`\` en Java, `/` en el resto). | `--java-home` = padre del padre de `bin` (vale para `javaw.exe`). Los tests de rutas con `\` y `/` mezclados. |
| 6 | `minecraft_root()` es el padre de `assets_dir` | Correcto. | Sin cambios. Notar que `--assetIndex` es un sha1, no el nombre del índice. |
| 6 | `parse` ignora lo desconocido | Cierto, y conviene: hay 40 argumentos de JVM y el classpath. | Sin cambios. |
| 9 | `run_game` debe forzar o evitar consolas con `CREATE_NO_WINDOW` (la tarea 5 preguntaba si hacía falta `windows_subsystem`) | Hace falta `#![windows_subsystem = "windows"]` **en el puente**; con consola la salida estándar se pierde y XMCL nunca ve `LWJGL Version: zenith`. | `main.rs` con `#![windows_subsystem = "windows"]` en Windows. Los errores van a stderr y, con subsistema Windows, además a `bridge.log` (stderr no se verá en ninguna consola). |
| 9 | `prepare_cmd` no usa `--root` | La preparación escribe unos 400 MB en `<build>\.local` si no se le da `--root`. | Pasar `--root <home>/prepared-root/<versión>` (o la carpeta que se decida) para no escribir dentro de la build instalada; comprobar qué hace exactamente antes. |
| 9 | La carpeta preparada es de solo lectura tras prepararse | El cliente escribe `horizon-data.bin` (120 MB) en ella en la primera arrancada. | Debe ser escribible. Una primera arrancada tarda además unos 17 s en segundo plano. |
| 8 | `install` extrae el zip en `builds/<v>` y `current()` es esa carpeta | El zip trae una carpeta raíz `horizon-windows-x64/`. | El ejecutable está en `builds/<v>/horizon-windows-x64/horizon-client.exe`; o se quita la carpeta raíz al extraer. Verificar también que `enclosed_name()` tolera esa estructura. |
| 8 | `SHA256SUMS` lista lo que descarga el puente | `SHA256SUMS` lista también el `.tar.gz` de Linux; `verify` exige tener todos los archivos listados. | El puente verifica con `check` sobre el archivo concreto, no con el comando `verify` de la carpeta. |
| 10 (Superseded: ver apartado 10) | `setup` escribe `instance.json` y funciona con cualquier raíz | **Con la raíz por defecto, `cmd.exe` rechaza la orden (apartado 2).** | Decisión pendiente antes de la tarea 10 (ver abajo). |
| 10 | Una ruta con espacios necesita comillas (`quotes_path_with_spaces`) | Confirmado: comillas dobles. Sin ellas se parte en piezas. | Ya previsto. Escribir `"<ruta>"` en `prependCommand`. La prueba debe usar una ruta con Ñ. |
| 10 | XMCL carga la instancia nada más escribirla | Se carga al arrancar XMCL; hay que reiniciarlo (ya está previsto en el README). | Sin cambios, pero `setup` debe avisar si XMCL está en marcha. |

### Decisión pendiente por el hallazgo del límite de `cmd.exe`

(Superseded: ver apartado 10; se eligió el puente como «Java» de la instancia.)

No se resuelve aquí. Opciones, de más barata a más cara, ninguna probada con XMCL real:

1. Exigir una raíz de datos corta (unos 19 caracteres o menos para 26.3; por ejemplo `C:\xmcl`), elegida en el primer arranque de XMCL, y que `zenith-bridge setup` lo compruebe leyendo `%APPDATA%\xmcl\root` y lo explique. El margen depende de la ruta del puente (cada carácter de más resta uno) y de la cantidad de librerías de la versión; `setup` debe generar la orden con las mismas reglas que XMCL y comprobar que no pasa de 8191 caracteres, en vez de fiarse de un número fijo.
2. Una unión de directorio (`mklink /J`) corta que apunte a la raíz real y escribir esa ruta corta en `%APPDATA%\xmcl\root`. Habría que comprobar que XMCL lo respeta y que no resuelve el enlace.
3. Pedir a XMCL que no use `shell: true` en Windows cuando el comando previo es un ejecutable (cambio en `@xmcl/core`/`LaunchService`). Contradice "no hay que modificar XMCL" del diseño.
4. Abandonar `prependCommand` para esta fase y pasar a la fase 6 (launcher propio).

Hasta que se decida, la tarea 10 no debería escribirse como está. Ver el apartado 10: una cuarta vía (el puente como `java` de la instancia, sin `cmd.exe`) medida después, y que además corrige el límite de 19 caracteres (se midió con un token de 10 caracteres).

## 9. Efectos secundarios de esta prueba (para ser honestos)

- Se creó la carpeta `C:\zx` (tres subcarpetas y una copia del JSON de versión, ~45 KB) como raíz corta de pruebas, y no se pudo borrar desde esta sesión (el borrado fue bloqueado por una comprobación de seguridad). Hay que borrarla a mano: `rmdir /s /q C:\zx`.
- Al probar opciones rechazadas (`--width abc`, `--bogus`) **sin** definir `HORIZON_HOME`, `horizon-client` escribió su registro en `%APPDATA%\horizon\logs` (el valor por defecto): rotó el `latest.log` anterior a `2026-11-04-4.log.gz`, creó `2026-11-05-1.log.gz` y reescribió `latest.log`. Eso va en contra de la regla de no tocar `%APPDATA%\horizon*` y **deja la lista de archivos de `%APPDATA%\horizon` distinta a la de antes de la prueba**; la comparación del paso 6 de la tarea 10 debe partir de un perfil que se anote de nuevo. No se modificó nada más allá de eso (el contenido del registro anterior se conservó comprimido; `%APPDATA%\horizon-launcher` no tuvo ningún cambio). No se intentó deshacerlo para no escribir más allí.
- Todas las descargas, desempaquetados, la carpeta preparada, la de la build y el clon de XMCL están en `%TEMP%\zenith-spike`. No se borran para reutilizarlos en tareas posteriores.

## 10. Alternativa: el puente como "java" de la instancia (sin cmd.exe)

Hallazgo que motiva esto: con `prependCommand` XMCL usa `shell: true` (cmd.exe, límite 8191) y pasa el token **real** de Minecraft, un JWT que puede medir más de mil caracteres. La idea: no usar `prependCommand` y poner en el campo `java` de `instance.json` una copia del puente que se hace pasar por Java. Entonces XMCL lanza `<java> -Xmx… -cp … net.minecraft.client.main.Main <args>` **sin shell** y manda el límite de `CreateProcess` (32 767).

Prueba estática, como la anterior: ningún clic en la interfaz. Se leyó el código del mismo clon de XMCL (`aeb5240`, solo lectura) y se ejecutaron sus funciones reales (empaquetadas con esbuild) contra un "java" falso. Entorno: Windows 11 (10.0.26300), Node v24.22.1. Todo en una carpeta temporal; no se tocó `%APPDATA%\horizon*`, `%APPDATA%\xmcl`, `C:\xmcl` ni `%APPDATA%\zenith`. Se reproduce con `zenith/bridge/tests/fixtures/java-shim-spike/` (ver "Cómo reproducirlo").

### Veredicto

**Viable con salvedades.** El mecanismo de XMCL (resolución de Java, validación, cambio a `javaw.exe`, lanzamiento sin shell, lectura de stdout) funciona con un ejecutable que responde a `-version`, y la línea de comandos deja unos 21 000 caracteres de margen. Quedan abiertas cosas que necesitan la interfaz de XMCL (último apartado) y dos consecuencias de diseño que no tienen que ver con el límite: XMCL ya no descarga un Java 25 para el puente, y otras instancias podrían elegir el shim como Java. El diseño con `prependCommand` queda **en duda seria**: con `C:\xmcl` falla en cuanto el token pase de unos 1090 caracteres, y la longitud real no se pudo medir.

### Cómo decide XMCL, con evidencia de código

**Sin shell, salvo con comando previo.** `LaunchService.#generateOptions` (`xmcl-runtime/launch/LaunchService.ts:228`): `shell: prepend && prepend.length > 0`. `launch()` de `@xmcl/core` (`packages/core/launch.ts:688-698`) solo cita los argumentos si `extraExecOption.shell` es verdadero, y después hace `spawn(args[0], args.slice(1), { cwd, detached: true, env })`. `inst.prependCommand ?? globalPrependCommand` (`xmcl-runtime-api/src/util/launch.ts:140`) usa el global solo si el de la instancia es `undefined`, y `normalizeCommandLine('')` devuelve `''` (falso), así que **un `"prependCommand": ""` explícito en la instancia anula también un comando previo global**; el esquema acepta la cadena vacía (`packages/instance/instance.ts:273`). Medido más abajo: sin shell.

**Qué ejecuta XMCL para saber qué es "Java"** (`packages/installer/java.ts`, `resolveJavaWithDiagnostic`): comprueba que el archivo existe y ejecuta `execFile(<ruta>, ['-version'], { windowsHide: true })`. Lee la versión de **stderr y, si no, de stdout** (`parseJavaVersionOutput`), **sin mirar el código de salida** (solo lo guarda para diagnóstico). `parseJavaVersion` exige que el texto contenga `openjdk`, `java version` o `java_version` como palabra, y un número. No lee `<home>/release` salvo para rellenar un registro inválido después de fallar (`JavaService.ts:170`). No usa `-XshowSettings:properties` (grep sin resultados) y no mira el nombre del archivo, ni el vendor, ni que esté en `bin/` bajo un home. Medido con `parseJavaVersion`:

| Salida | Resultado |
| --- | --- |
| `openjdk version "25"` | `{version:"25", majorVersion:25}` |
| `java version "25"` | igual |
| 3 líneas de Temurin con `"25.0.5"` | `{version:"25.0.5", majorVersion:25}` |
| `25.0.5`, `version "25.0.5"`, un banner propio o nada | `undefined` (no es Java) |

`resolveJava` sobre un ejecutable que solo responde a `-version` (`fakejava.rs`) da `25.0.5 (major 25)` **tanto de consola como de subsistema Windows** (con stdout y stderr en tuberías, el de subsistema Windows también escribe). Un binario ajeno (`whoami.exe`) da `undefined`: XMCL **rechaza lo desconocido**; solo acepta lo que hable "java". Un archivo inexistente da `ENOENT`.

**Extra opcional.** `JavaService.resolveJava` llama a `getJavaArch` (`detectJVMArch.ts`): `execFile(<java>, ['JavaProxy','-jvm_info'], { cwd: <datos de XMCL> })` y espera JSON con `java_arch` (`"64"` pasa a `x64`). Si falla solo avisa y deja la arquitectura sin definir (cosmético: se ve en la lista de Java). El shim contesta `{"java_arch":"64"}`.

**Cuánta versión exige XMCL.** El middleware `java-validation` (`pluginLaunchPrecheck.ts:99-149`) **no compara versiones**: solo exige que `resolveJava` devuelva algo. La comparación está en la interfaz: `getAutoOrManuallJava` (`xmcl-runtime-api/src/util/java.ts`) la hace contra el requisito del JSON de versión (`javaVersion.majorVersion`; para 26.3: `java-runtime-epsilon`, **25**, regla `>=25`) y `getInstanceJavaIssue` (`instanceJavaDiagnose.ts`) abre el diálogo "incompatible" si no se cumple. Se ejecutaron las funciones reales de selección (`validate.cjs`; el último punto reproduce a mano `getInstanceJavaIssue`):

```
shim 25 (java.exe)         resolveJava: 25.0.5 (major 25) | arch: 64 | calidad: 0 (Matched)  problema: null
shim 25 (javaw.exe)        resolveJava: 25.0.5 (major 25) | arch: 64 | calidad: 0 (Matched)  problema: null
shim que dice 21           resolveJava: 21.0.5 (major 21) | arch: 64 | calidad: 2            problema: incompatible
binario ajeno (whoami.exe) resolveJava: undefined                                           problema: invalid
ruta inexistente           X_OK: ENOENT, resolveJava: undefined                              problema: invalid
```

Con la lista de Java de XMCL vacía (`all` vacío) el resultado es `invalid` **incluso con un Java real**, porque `getAutoOrManuallJava` devuelve `noJava` sin mirar el campo. Es transitorio: `resolveJava` añade el Java a la lista y la interfaz recalcula (`watch` en `instanceJava.ts`). Ese refresco no se pudo ver sin la interfaz.

### El cambio a `javaw.exe` y los nombres del shim

`#generateOptions` (líneas 205-206) calcula `join(dirname(javaPath), 'javaw.exe')` y, si existe y `access(X_OK)` (en Windows: que exista), lanza **ese** en lugar del campo, sea cual sea el nombre del campo. En cambio `java-validation` y la interfaz sondean `options.java`, el del campo. Por tanto:

- **`java.exe`** debe existir: es lo que se sondea con `-version` y lo que va en el campo `java`.
- **`javaw.exe`** debe estar **en la misma carpeta**: es lo que recibe los argumentos del juego. Puede ser el mismo binario: se probó con `java.exe` de consola + `javaw.exe` de subsistema Windows (A1), solo `java.exe` de consola (A2) y ambos de subsistema Windows (A3). En los tres se lanza y XMCL ve la salida. Sin `javaw.exe` se lanza `java.exe`.

### Lo que recibe el shim y cómo se trata su salida

Se ejecutó el `core.launch()` real de `@xmcl/core` con las mismas opciones que XMCL (`shell: false`, `detached: true`, `cwd`, `env`), un token falso de 1200 caracteres y el `createMinecraftProcessWatcher` real (el que usa `LaunchService`), contra el shim (`e2e.cjs`, resumido):

```
A1 java.exe consola + javaw.exe winsub  lanzado=jre\bin\javaw.exe   ready=true code=0 stdout="LWJGL Version: zenith" stderr="stderr line from shim" argc=43 cwdOk=true pidIgualAlDeNode=true
A2 solo java.exe (consola)              lanzado=jre2\bin\java.exe   ready=true code=0 stdout="LWJGL Version: zenith" …
A3 java.exe y javaw.exe winsub          lanzado=jre3\bin\javaw.exe  ready=true code=0 stdout="LWJGL Version: zenith" …
```

- **La salida estándar llega en los tres casos**, también con el ejecutable de consola, a diferencia del caso `shell: true` del apartado 2. `minecraft-window-ready` se dispara con `LWJGL Version: `; stderr llega aparte.
- **`argv` del shim:** `argv[0]` es la ruta de `javaw.exe` que pasa XMCL y a continuación van los 42 argumentos del fixture `xmcl-args.txt` (que lleva `javaw.exe` como primera línea, 43 en total). Tras normalizar la raíz son idénticos, salvo el uuid y el token, que en el fixture eran de relleno (`#34`, `#36`). Respecto a `prependCommand` **no hay una ruta del puente antes**: `argv[1]` ya es `-XX:HeapDumpPath=…`.
- **`process.pid` es el del shim** (el `PID` que vuelca el shim coincide con el `pid` de Node). No hay `cmd.exe` en medio, así que Detener de XMCL (`process.kill()`) ya alcanza al puente; que el árbol del juego muera con él depende del objeto de trabajo del puente (`d628dd28a`).
- **Sin shell no hay expansión ni pérdida de caracteres.** Con la ruta `…/Jose Ñ/java dir/bin/javaw.exe` y argumentos extra `100%ZXTEST%`, `a&b`, `q"uote` y uno con tabulador, el shim recibió exactamente `["--x","100%ZXTEST%","--y","a&b","--z","q\"uote","--w","tab\there"]` y la ruta con Ñ y espacios intacta. (Con `cmd.exe`, apartado 2, `%…%` se expandía y las comillas se perdían.)
- Un `-version` tarda unos 12,6 ms (media de 20, subsistema Windows).

### Longitud de línea medida (la razón de este apartado)

Misma orden de 26.3 (82 jars) con `core.generateArguments`, token falso de longitud variable (no se pudo obtener un JWT real), raíz de datos `C:\Users\Some Long User Name\AppData\Roaming\xmcl` (49 caracteres) y Java en `C:\Users\Some Long User Name\AppData\Roaming\zenith\java\bin\javaw.exe`. "Sin shell" es el `GetCommandLineW` del shim real, corregido a la ruta de Java larga (+2 por las comillas que libuv pone a una ruta con espacios; incertidumbre de unos 3 caracteres). "Con prepend + shell" es la longitud del comando que recibe `cmd.exe` (ruta del puente de unos 70 caracteres, como en la prueba anterior):

| Token | Sin shell (CreateProcess, límite 32 767) | Margen | Con prepend + shell (cmd.exe, límite 8191) |
| --- | --- | --- | --- |
| 10 | 10 444 | 22 323 | 10 531 |
| 700 | 11 134 | 21 633 | 11 221 |
| **1200** | **11 634** | **21 133** | **11 721** |
| 1500 | 11 934 | 20 833 | 12 021 |

- `cmd.exe` real, raíz larga, token 1200, con prepend: **código 1, "La línea de comandos es demasiado larga.", el programa no se ejecutó.**
- Sin shell cabe con holgura. La línea crece 82 caracteres por carácter de raíz, así que haría falta una raíz de unos 300 caracteres para llegar a 32 767 (antes pesaría `MAX_PATH` en otros sitios).
- **Corrige el apartado 2.** El límite de "unos 19 caracteres" se midió con un token de 10 caracteres. Con la raíz corta recomendada y el diseño actual:

  ```
  prepend+shell, raíz C:\xmcl (7), token 10:   comando=7108, código 0, shim ejecutado: true
  prepend+shell, raíz C:\xmcl (7), token 1200: comando=8298, código 1, shim ejecutado: false
  ```

  El token cuenta carácter a carácter: con `C:\xmcl` el diseño con `prependCommand` solo funciona con tokens de hasta unos 1090 caracteres (8191 − 7108 + 10). La longitud del token real es desconocida aquí.

### Qué pasa con la interfaz y los ajustes de XMCL (leído, no ejecutado)

- **XMCL no reemplaza nuestro campo `java` por su cuenta.** La interfaz lo usa tal cual si está asignado (`getAutoOrManuallJava`: "User assigned java path, revalidate it"; `instanceVersionInstall.ts:44-47`: con `inst.java` definido no se instala ni se busca otro; `InstallCoordinator.ts:150-166`: "A pinned instance Java wins when it resolves"). El único reemplazo automático está en el middleware y **solo si la ruta cuelga de `<raíz>/jre/<componente>`** (`getManagedJavaComponent`): si `resolveJava` falla, XMCL quita ese Java y **reinstala el de Mojang**. **El shim no debe estar bajo `<raíz de XMCL>/jre`.** Fuera de ahí, si falla, el error es `launchNoProperJava`: visible y sin reemplazo.
- **El Java global** (`globalJava`) solo desempata la **elección automática** cuando `instance.java` está vacío (`selectJavaByPreference`, parámetro `preferredPath`); no pisa un campo asignado. No hay ajuste de XMCL que cambiar.
- **Reemplazos manuales** del campo (los hace el usuario): el botón de usar el Java recomendado en el diálogo de problema de Java (`HomeLaunchStatusDialog.vue:192-196`), la página de Java de la instancia y las herramientas del agente (`tools.ts:1450-1456`). Si el shim contestara mal a `-version`, ese diálogo ofrecería justo sustituirlo.
- **Lista de Java.** El shim aparecerá en la lista de Java de XMCL (caché en `%APPDATA%\xmcl\java.json`, revalidada en cada arranque con `-version`: `refreshLocalJava`), como 25.0.5 `x64`. XMCL solo busca en rutas conocidas (Mojang, Oracle, OpenJDK, Adoptium, Zulu, `JAVA_HOME`, registro, `where java`, `<raíz>/jre/*`); una carpeta propia como `%APPDATA%\zenith\java` no se descubre sola, entra por la instancia.
- **Riesgo medido con la función de selección:** una instancia **sin Java propio** de otra versión (por ejemplo 1.21.4, que pide `>=21`) puede elegir el shim por ser "Java 25" si está antes en la lista:

  ```
  lista [shim25, real21] -> elige S/bin/java.exe     lista [real21, shim25] -> elige R/bin/java.exe
  ```

  Se evita haciendo que el modo juego del shim **rechace** lo que no sea su versión de Minecraft (salida ≠ 0 con un mensaje claro) o delegue en un Java real. No probado en la interfaz.
- **Descarga de Java.** Con el campo asignado y resoluble, XMCL **no descarga** `java-runtime-epsilon` (ver arriba). Consecuencia: el puente ya no recibe un Java 25 en `argv` para `horizon-prepare --java-home`; tendría que aportar o localizar el suyo. No es un fallo del shim sino del diseño.

### Condiciones exactas de la alternativa

1. Carpeta propia **fuera de `<raíz de XMCL>/jre`**, por ejemplo `%APPDATA%\zenith\java\bin\`, con `java.exe` y `javaw.exe` (dos copias independientes; no enlaces duros, por el `ino` que usa `refreshLocalJava` para no duplicar). Opcional: `..\release` con `JAVA_VERSION="25.0.5"` (solo se lee tras un fallo).
2. `instance.json`: `java` con la ruta absoluta de `…\bin\java.exe` (en JSON las barras invertidas van duplicadas) y `"prependCommand": ""` (anula también el global; `shell` queda en falso).
3. Un solo ejecutable de subsistema Windows vale para los dos nombres (medido). El modo se decide por el nombre del archivo (`java`/`javaw`) o por `argv[1]` (`-version`, `JavaProxy`).
4. Respuestas a los sondeos: rápidas, código 0, sin red ni escrituras.
   - `-version` (único argumento): por stderr (o stdout) `openjdk version "25.0.5" <fecha> LTS` y las dos líneas de runtime; el major debe ser **mayor o igual que `javaVersion.majorVersion` del JSON** (25 para 26.3).
   - `JavaProxy -jvm_info`: `{"java_arch":"64"}` por stdout (opcional).
   - Cualquier otra invocación es un lanzamiento de juego.
5. Modo juego: leer `argv` (fixture), arrancar Zenith, escribir por **stdout** `LWJGL Version: …` al estar listo y `Crash report saved to: #@!@# <ruta>` si falla; stderr se acumula para el diálogo de error; salir con el código del juego.
6. Windows bloquea un `javaw.exe` en ejecución: la copia solo se puede reemplazar con el juego cerrado. Hay que decidir quién la repone tras una actualización del puente (`setup`, o el propio modo `-version` si no hay juego en marcha).

### Lo que NO se pudo verificar sin la interfaz

- Que XMCL cargue una instancia escrita a mano con `java` y `prependCommand: ""` y que no los reescriba (por ejemplo al abrir la página de Java de la instancia y guardar).
- Cómo se ven la página de Java, la lista de Java global y el botón Jugar con el shim (versión, arquitectura, ausencia de avisos), y el refresco tras el estado transitorio de lista vacía.
- Que no haya ventana de consola ni parpadeo con el shim (consola o subsistema Windows); solo se midió la salida estándar.
- Qué muestra XMCL cuando el shim sale con código ≠ 0, y su reacción a `Crash report saved to:`.
- El token real de Minecraft: su longitud no se obtuvo (nadie puede iniciar sesión aquí); los 1200 caracteres son una suposición. La conclusión sobre `cmd.exe` depende de ella; la del shim no (21 000 caracteres de margen).
- El botón Detener (`kill` normal y forzado) y el árbol de procesos.
- Que otra instancia no elija el shim en la práctica y que ningún camino normal ofrezca sustituirlo.
- Que una versión de XMCL distinta de `aeb5240` (0.73.1) valide Java igual.
- Antivirus o SmartScreen ante un `java.exe` que no es de un proveedor conocido.

### Cómo reproducirlo

```
sh zenith/bridge/tests/fixtures/java-shim-spike/build.sh <clon de XMCL, solo lectura> <carpeta de trabajo> <26.3.json>
# necesita rustc, node 24 y esbuild (ESBUILD_DIR=<ruta a node_modules/esbuild> si no está instalado)
```

Compila `fakejava.rs` (consola, subsistema Windows y la variante "dice Java 21"), empaqueta con esbuild `@xmcl/core`, `packages/installer/java.ts` y `xmcl-runtime-api/src/util/java.ts` del clon y ejecuta `e2e.cjs` (lanzamiento, largos, `cmd.exe`), `validate.cjs` (resolución, calidad, selección en otras instancias) y `probe.cjs`. Solo escribe en la carpeta de trabajo.
