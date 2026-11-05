# Zenith + XMCL

Zenith es una build de Horizon (Minecraft en Rust) que se juega desde [XMCL](https://xmcl.app) (X Minecraft Launcher). XMCL se encarga de tu cuenta de Microsoft, de descargar Minecraft y Java y del botón Jugar. Un pequeño programa, `zenith-bridge`, hace de comando previo: cuando pulsas Jugar arranca Zenith en lugar de Java.

Es **experimental**.

## Requisitos

- Windows 10 u 11 (de momento solo Windows).
- Una cuenta de Microsoft que **tenga Minecraft: Java Edition**. El puente lo comprueba con los servicios de Minecraft; las cuentas sin licencia, offline, ely.by o LittleSkin se rechazan.
- XMCL, de xmcl.app o de sus releases en GitHub.

No se distribuye ningún archivo de Mojang: XMCL descarga Minecraft y el puente prepara los recursos de Zenith a partir de esa descarga.

## Instalación

1. Instala XMCL. Si ya lo has abierto alguna vez, mira la nota sobre la carpeta de datos más abajo.
2. En la pestaña Releases de este repositorio descarga `zenith-bridge-windows-x64.exe` y comprueba que su SHA-256 coincide con la entrada de `SHA256SUMS` (`certutil -hashfile zenith-bridge-windows-x64.exe SHA256`).
3. Ejecuta `zenith-bridge-windows-x64.exe setup` desde una consola. Instala el puente en la carpeta de datos de XMCL y crea la instancia **Zenith**. Si XMCL estaba abierto, ciérralo y ábrelo otra vez.
4. En XMCL inicia sesión con tu cuenta de Microsoft, elige la instancia Zenith y pulsa Jugar.

La primera vez XMCL descarga Minecraft 26.3 y Java (es normal) y Zenith prepara sus recursos: de unos 30 segundos a unos minutos.

### La carpeta de datos de XMCL debe ser corta

Con un comando previo, XMCL lanza el juego a través de `cmd.exe`, que rechaza líneas de más de 8191 caracteres. La orden de Minecraft 26.3 crece unos 82 caracteres por cada carácter de la ruta de datos de XMCL, así que esa ruta debe tener como máximo unos 19 caracteres (por ejemplo `C:\xmcl`). Si XMCL nunca se ha abierto, `setup` la fija en `C:\xmcl`. Si ya usa una ruta larga, `setup` no toca nada y te lo explica: cambia la carpeta de datos en los ajustes de XMCL a una ruta corta, o, si no tienes datos de XMCL que conservar, borra `%APPDATA%\xmcl\root`, y repite `setup`.

## Órdenes

- `zenith-bridge setup`: instala o repara todo. Si ya existía una instancia Zenith, conserva tus ajustes y guarda la anterior como `instance.json.bak`.
- `zenith-bridge status`: carpeta de datos, instancia, build instalada, versión fijada, caché de cuenta y últimas líneas del registro.
- `zenith-bridge use <versión>` fija una build ya instalada (para volver atrás si una versión falla); `zenith-bridge use latest` vuelve a las actualizaciones automáticas.
- `zenith-bridge help`.

Es un programa de ventana: si lo ejecutas haciendo doble clic, el resultado sale en un cuadro de diálogo; desde una consola sale por ella, pero la consola no espera a que termine (para un código de salida fiable usa `start /wait`).

## Actualizaciones

Automáticas: al pulsar Jugar el puente busca la última build, verifica su firma y su hash, y la instala. Sin conexión usa la ya instalada. Para quedarte en una versión, usa `use`.

## Solución de problemas

- Registro del puente: `%APPDATA%\zenith\logs\bridge.log` (también `zenith-bridge status`).
- "La línea de comandos es demasiado larga" en el registro de XMCL: la carpeta de datos es demasiado larga (ver arriba).
- El inicio de sesión de Microsoft lo gestiona XMCL, no Zenith.
- La instancia no aparece: XMCL solo lee las instancias al arrancar; ciérralo del todo y ábrelo de nuevo.
- Para empezar de cero los recursos de Zenith, borra `%APPDATA%\zenith\prepared`.

## Privacidad

Telemetría y cosméticos están desactivados. El puente solo contacta con `api.minecraftservices.com` (comprobar que tienes el juego), `github.com` (builds y actualizaciones) y los servidores de Mojang para el jar del servidor. Guarda en `%APPDATA%\zenith` un registro y una caché de qué cuenta posee el juego; no guarda tu token.

## Límites conocidos

- Experimental; no verificado con todas las versiones de XMCL.
- Solo Windows por ahora.
- Los mods de Java no funcionan: Zenith no ejecuta la JVM de Minecraft.
- Una instancia Zenith solo sirve para la versión de Minecraft que fija la build (hoy 26.3).
