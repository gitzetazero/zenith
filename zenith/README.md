# Zenith + XMCL

Zenith es una build de Horizon (Minecraft en Rust) que se juega desde [XMCL](https://xmcl.app) (X Minecraft Launcher). XMCL se encarga de tu cuenta de Microsoft, de descargar Minecraft y del botón Jugar. `zenith-bridge` se instala como el «Java» de una instancia llamada **Zenith**: cuando pulsas Jugar, XMCL lo ejecuta creyendo que es Java 25 y el puente arranca Zenith en su lugar.

**Zenith Launcher (powered by XMCL)** es el paquete que lo junta todo: el XMCL oficial, sin modificar, más el puente. Es un proyecto independiente, **no afiliado a XMCL (Voxelum), Mojang ni Microsoft**; el crédito y la licencia MIT de XMCL van dentro del paquete (carpeta `licencias`).

Es **experimental** y **no está verificado con la interfaz real de XMCL ni con una instalación real del instalador** (solo con su código, con pruebas automáticas y con el contenido de los archivos publicados).

## Requisitos

- Windows 10 u 11 (de momento solo Windows).
- Una cuenta de Microsoft que **tenga Minecraft: Java Edition**. El puente lo comprueba con los servicios de Minecraft; las cuentas sin licencia, offline, ely.by o LittleSkin se rechazan.

No se distribuye ningún archivo de Mojang: XMCL descarga Minecraft y el puente prepara los recursos de Zenith a partir de esa descarga.

## Instalación

1. En la pestaña Releases de este repositorio, en la pre-release `nightly`, descarga `Zenith-Launcher-Setup-x64.exe` y comprueba que su SHA-256 coincide con su línea de `SHA256SUMS` (`certutil -hashfile Zenith-Launcher-Setup-x64.exe SHA256`). Ojo: comparar con el `SHA256SUMS` de la misma release solo detecta una descarga corrupta, no una release sustituida; las builds del juego que el puente descarga después sí se verifican con una firma Ed25519.
2. Ejecútalo. **El instalador no está firmado para Windows**, así que SmartScreen avisará («Windows protegió su PC»: Más información, Ejecutar de todas formas). Se instala solo para tu usuario (sin administrador) en `%LOCALAPPDATA%\Programs\Zenith Launcher`, con acceso directo en el menú Inicio (y en el escritorio si lo marcas) y entrada en «Aplicaciones instaladas». Admite `/S` (silencioso) y `/D=ruta`.
3. Abre «Zenith Launcher» (hace `zenith-bridge.exe start`). **La primera vez** XMCL te pide su carpeta de datos: elígela, cierra XMCL del todo y abre «Zenith Launcher» otra vez. Ahora el puente crea la instancia **Zenith** (y copia el «Java» de Zenith a `%APPDATA%\zenith\java\bin`) antes de abrir XMCL; no escribe nada en XMCL hasta que XMCL ha creado su carpeta de datos.
4. En XMCL inicia sesión con tu cuenta de Microsoft (la que tiene Java Edition), elige la instancia **Zenith** y pulsa Jugar.

**Alternativa portátil:** `Zenith-Launcher-Portable-x64.zip` trae lo mismo en una carpeta (`Zenith-Launcher-Portable`); descomprímela y abre `Zenith Launcher.cmd` (equivale a `zenith-bridge.exe start`). XMCL no tiene modo portátil: sus datos (`%APPDATA%\xmcl\root` y la carpeta que elijas) y los de Zenith (`%APPDATA%\zenith`) viven en tu perfil aunque la carpeta esté en un pendrive. Las instrucciones también están en `README-PORTABLE.txt` dentro del zip.

**Desinstalar:** «Aplicaciones instaladas» (o `Desinstalar.exe`). Borra solo los archivos que instaló en su carpeta, sus accesos directos y sus dos claves de registro (HKCU). **Nunca** toca `%APPDATA%\xmcl`, la carpeta de datos de XMCL (tus mundos, tus instancias) ni `%APPDATA%\zenith`.

Si ya tenías XMCL, el instalador no lo toca: instala otra copia en su carpeta y reutiliza los datos de XMCL (`%APPDATA%\xmcl\root`), así que verás tus instancias de siempre más la de Zenith.

La primera vez que juegas XMCL descarga Minecraft 26.3 (es normal) y Zenith prepara sus recursos: de unos 30 segundos a unos minutos. Si no tienes un Java 25 instalado, Zenith descarga el runtime de Java de Mojang para esa preparación.

**Solo el puente (avanzado):** `zenith-bridge-windows-x64.exe` sigue publicado, para el actualizador del propio juego y para quien ya usa su XMCL: `zenith-bridge-windows-x64.exe setup` hace lo mismo que `start` sin abrir XMCL (hace falta que XMCL se haya abierto una vez).

El límite de longitud de la ruta de la carpeta de datos de XMCL es de unos 300 caracteres (el límite de 32767 de CreateProcess en Windows): el puente no usa un comando previo (que obligaría a pasar por `cmd.exe`, con su límite de 8191 caracteres) sino el campo Java de la instancia.

## Por qué aparece un «Java 25» en XMCL

La lista de Java de XMCL mostrará un Java 25.0.5 en `%APPDATA%\zenith\java\bin`: es el puente. Responde a `-version` como Java y solo acepta lanzar el Minecraft de Zenith. **No lo uses en otras instancias**: si lo haces, el puente se niega a arrancar con un mensaje claro («no es un Minecraft de Zenith») y no hace nada más.

## Órdenes

- `zenith-bridge start`: lo que ejecuta el acceso directo «Zenith Launcher». Prepara la instancia como `setup`, sin red (la versión de Minecraft sale de la build instalada), y abre el `XMCL.exe` que está junto al puente. Si XMCL nunca se ha abierto (no existe `%APPDATA%\xmcl\root`) solo abre XMCL y te dice que lo cierres tras elegir la carpeta y vuelvas a abrir «Zenith Launcher». Si XMCL ya está abierto no lo cierra ni lo toca: te dice que lo reinicies.
- `zenith-bridge setup`: instala o repara todo. Si ya existía una instancia Zenith, conserva tus otros ajustes y guarda la primera versión como `instance.json.bak`. Con el juego abierto no puede reemplazar el puente: ciérralo y repite.
- `zenith-bridge status`: carpeta de datos, instancia, «Java» instalado, Java 25 disponible, build instalada, versión fijada, caché de cuenta y últimas líneas del registro.
- `zenith-bridge use <versión>` fija una build ya instalada (para volver atrás si una versión falla); `zenith-bridge use latest` vuelve a las actualizaciones automáticas.
- `zenith-bridge help`.

Es un programa de ventana: haciendo doble clic o sin consola, el resultado sale en un cuadro de diálogo; desde una consola sale por ella, pero la consola no espera a que termine (para un código de salida fiable usa `start /wait`).

## Actualizaciones

- **El juego (Zenith):** automático. Al pulsar Jugar el puente busca la última build y la instala tras verificarla él mismo: la firma de `SHA256SUMS`, el hash del archivo y un sello de compilación firmado (`zenith-build.txt`) con el que rechaza una build más antigua que la instalada. Sin conexión usa la ya instalada. Para quedarte en una versión, usa `use`.
- **El puente instalado** (las copias `java.exe`/`javaw.exe` y el `zenith-bridge.exe` de la carpeta del launcher): se renueva instalando un `Zenith-Launcher-Setup-x64.exe` más nuevo (o descomprimiendo el zip portátil nuevo) y abriendo «Zenith Launcher» (`start` rehace las copias).
- **El launcher (XMCL):** se actualiza con el actualizador propio de XMCL. He leído su código (v0.73.1: `xmcl-electron-app/main/pluginAutoUpdate.ts` y `main/utils/updater.ts`): en Windows consulta `api.xmcl.app` y, si la release trae `app-<versión>-win.asar`, descarga solo ese `app.asar` a `%APPDATA%\xmcl\pending_update` y, al instalar, un `.bat` (con elevación forzada: Windows pedirá permiso de administrador aunque la carpeta sea tuya) cierra `XMCL.exe`, hace `copy /Y` sobre `resources\app.asar` y vuelve a abrir XMCL. Por tanto solo se sustituye `resources\app.asar`; el puente y los demás archivos no se tocan, y XMCL no borra nada de la carpeta. Si una release no trae el asar de Windows, XMCL solo abre xmcl.app para descargar a mano (no toca nada). Consecuencias honestas: tras actualizarse, XMCL reabre sin pasar por «Zenith Launcher» (da igual: la instancia ya está preparada); si vuelves a ejecutar el instalador de Zenith, `app.asar` vuelve a la versión incluida en él; la desinstalación borra `app.asar` aunque sea el actualizado, porque es un archivo de la lista. No lo he probado ejecutando XMCL, solo leyendo su código.

## Solución de problemas

- Registro del puente: `%APPDATA%\zenith\logs\bridge.log` (también `zenith-bridge status`).
- El inicio de sesión de Microsoft lo gestiona XMCL, no Zenith.
- La instancia no aparece: XMCL solo lee las instancias al arrancar; ciérralo del todo y ábrelo de nuevo.
- Windows SmartScreen o el antivirus pueden marcar el instalador, el zip o un `java.exe` desconocido: ni el instalador ni el puente están firmados para Windows; puedes comprobar sus hashes con `SHA256SUMS`.
- Para rehacer los recursos de Zenith, borra `%APPDATA%\zenith\prepared`.

## Privacidad

- Telemetría y cosméticos de Zenith están desactivados por defecto, también si ejecutas `horizon-client.exe` sin el puente (solo se activan con `HORIZON_TELEMETRY_URL` o `HORIZON_COSMETICS=1`). La telemetría de XMCL es cosa de XMCL: puedes desactivarla en Ajustes, General, «desactivar telemetría» (`disableTelemetry`).
- El puente contacta solo con: `api.minecraftservices.com` (comprobar que tu cuenta tiene el juego); `github.com` y su CDN de descargas (builds y actualizaciones); `launchermeta.mojang.com`, `piston-meta.mojang.com` y `piston-data.mojang.com` (el jar del servidor y, si no hay un Java 25, el runtime de Java de Mojang). Todo por https. La API de propiedad de la cuenta y los índices de Mojang dependen solo de TLS; las builds del juego llevan firma Ed25519 y los archivos de Mojang (jar del servidor, runtime de Java) se comprueban con sha1. Ni el instalador ni el puente están firmados para Windows (SmartScreen puede avisar).
- Guarda en `%APPDATA%\zenith` un registro y una caché de qué cuenta posee el juego; no guarda tu token.

## Límites conocidos

- Experimental; no verificado con la interfaz real de XMCL ni con todas sus versiones, ni ejecutando el instalador en un Windows real (comprobado solo su contenido y el del zip).
- El instalador y el zip no están firmados para Windows (SmartScreen avisa).
- Incluye una versión fija de XMCL (la de `zenith/launcher/xmcl-pin.txt`); XMCL se actualiza luego por su cuenta.
- Solo Windows por ahora.
- Los mods de Java no funcionan: Zenith no ejecuta la JVM de Minecraft.
- Una instancia Zenith solo sirve para la versión de Minecraft que fija la build (hoy 26.3).
