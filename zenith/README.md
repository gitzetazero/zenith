# Zenith + XMCL

Zenith es una build de Horizon (Minecraft en Rust) que se juega desde [XMCL](https://xmcl.app) (X Minecraft Launcher). XMCL se encarga de tu cuenta de Microsoft, de descargar Minecraft y del botón Jugar. `zenith-bridge` se instala como el «Java» de una instancia llamada **Zenith**: cuando pulsas Jugar, XMCL lo ejecuta creyendo que es Java 25 y el puente arranca Zenith en su lugar.

Es **experimental** y **no está verificado con la interfaz real de XMCL** (solo con su código y con pruebas automáticas).

## Requisitos

- Windows 10 u 11 (de momento solo Windows).
- Una cuenta de Microsoft que **tenga Minecraft: Java Edition**. El puente lo comprueba con los servicios de Minecraft; las cuentas sin licencia, offline, ely.by o LittleSkin se rechazan.
- XMCL, de xmcl.app o de sus releases en GitHub, **abierto al menos una vez** (para que elija su carpeta de datos).

No se distribuye ningún archivo de Mojang: XMCL descarga Minecraft y el puente prepara los recursos de Zenith a partir de esa descarga.

## Instalación

1. Instala XMCL, ábrelo una vez, elige la carpeta de datos y ciérralo.
2. En la pestaña Releases de este repositorio, en la pre-release `nightly`, descarga `zenith-bridge-windows-x64.exe` y comprueba que su SHA-256 coincide con su línea de `SHA256SUMS` (`certutil -hashfile zenith-bridge-windows-x64.exe SHA256`). Puedes dejarlo en cualquier carpeta. Ojo: comparar el hash con el `SHA256SUMS` de la misma release solo detecta una descarga corrupta, no una release sustituida; las builds del juego que el puente descarga después sí se verifican con una firma Ed25519. El exe no está firmado para Windows, así que SmartScreen puede avisar.
3. Desde una consola, ejecuta `zenith-bridge-windows-x64.exe setup`. Copia el puente a `%APPDATA%\zenith\java\bin` (como `java.exe` y `javaw.exe`) y crea la instancia **Zenith** en la carpeta de datos de XMCL. Si XMCL estaba abierto, ciérralo del todo y ábrelo otra vez: la instancia se aplica al reiniciarlo.
4. En XMCL inicia sesión con tu cuenta de Microsoft, elige la instancia Zenith y pulsa Jugar.

La primera vez XMCL descarga Minecraft 26.3 (es normal) y Zenith prepara sus recursos: de unos 30 segundos a unos minutos. Si no tienes un Java 25 instalado, Zenith descarga el runtime de Java de Mojang para esa preparación.

El límite de longitud de la ruta de la carpeta de datos de XMCL es de unos 300 caracteres (el límite de 32767 de CreateProcess en Windows): el puente no usa un comando previo (que obligaría a pasar por `cmd.exe`, con su límite de 8191 caracteres) sino el campo Java de la instancia.

## Por qué aparece un «Java 25» en XMCL

La lista de Java de XMCL mostrará un Java 25.0.5 en `%APPDATA%\zenith\java\bin`: es el puente. Responde a `-version` como Java y solo acepta lanzar el Minecraft de Zenith. **No lo uses en otras instancias**: si lo haces, el puente se niega a arrancar con un mensaje claro («no es un Minecraft de Zenith») y no hace nada más.

## Órdenes

- `zenith-bridge setup`: instala o repara todo. Si ya existía una instancia Zenith, conserva tus otros ajustes y guarda la primera versión como `instance.json.bak`. Con el juego abierto no puede reemplazar el puente: ciérralo y repite.
- `zenith-bridge status`: carpeta de datos, instancia, «Java» instalado, Java 25 disponible, build instalada, versión fijada, caché de cuenta y últimas líneas del registro.
- `zenith-bridge use <versión>` fija una build ya instalada (para volver atrás si una versión falla); `zenith-bridge use latest` vuelve a las actualizaciones automáticas.
- `zenith-bridge help`.

Es un programa de ventana: haciendo doble clic o sin consola, el resultado sale en un cuadro de diálogo; desde una consola sale por ella, pero la consola no espera a que termine (para un código de salida fiable usa `start /wait`).

## Actualizaciones

Automáticas: al pulsar Jugar el puente busca la última build y la instala tras verificarla él mismo: la firma de `SHA256SUMS`, el hash del archivo y un sello de compilación firmado (`zenith-build.txt`) con el que rechaza una build más antigua que la instalada. Sin conexión usa la ya instalada. Para quedarte en una versión, usa `use`. Las copias de `java.exe`/`javaw.exe` son el puente que ejecutaste con `setup`: renovarlas exige descargar un `zenith-bridge-windows-x64.exe` más nuevo (comprobando su hash) y ejecutar `setup` otra vez.

## Solución de problemas

- Registro del puente: `%APPDATA%\zenith\logs\bridge.log` (también `zenith-bridge status`).
- El inicio de sesión de Microsoft lo gestiona XMCL, no Zenith.
- La instancia no aparece: XMCL solo lee las instancias al arrancar; ciérralo del todo y ábrelo de nuevo.
- Windows SmartScreen o el antivirus pueden marcar un `java.exe` desconocido: es el puente; puedes comprobar su hash con el de `SHA256SUMS`.
- Para rehacer los recursos de Zenith, borra `%APPDATA%\zenith\prepared`.

## Privacidad

- Telemetría y cosméticos de Zenith están desactivados por defecto, también si ejecutas `horizon-client.exe` sin el puente (solo se activan con `HORIZON_TELEMETRY_URL` o `HORIZON_COSMETICS=1`). La telemetría de XMCL es cosa de XMCL: puedes desactivarla en Ajustes, General, «desactivar telemetría» (`disableTelemetry`).
- El puente contacta solo con: `api.minecraftservices.com` (comprobar que tu cuenta tiene el juego); `github.com` y su CDN de descargas (builds y actualizaciones); `launchermeta.mojang.com`, `piston-meta.mojang.com` y `piston-data.mojang.com` (el jar del servidor y, si no hay un Java 25, el runtime de Java de Mojang). Todo por https. La API de propiedad de la cuenta y los índices de Mojang dependen solo de TLS; las builds del juego llevan firma Ed25519 y los archivos de Mojang (jar del servidor, runtime de Java) se comprueban con sha1. El exe del puente no está firmado para Windows (SmartScreen puede avisar).
- Guarda en `%APPDATA%\zenith` un registro y una caché de qué cuenta posee el juego; no guarda tu token.

## Límites conocidos

- Experimental; no verificado con la interfaz real de XMCL ni con todas sus versiones.
- Solo Windows por ahora.
- Los mods de Java no funcionan: Zenith no ejecuta la JVM de Minecraft.
- Una instancia Zenith solo sirve para la versión de Minecraft que fija la build (hoy 26.3).
