# Zenith Launcher: guía del jugador

**Zenith Launcher** es el programa que abre Zenith. Por ahora es el launcher [XMCL](https://xmcl.app) sin modificar, más un pequeño programa de Zenith, el **puente**. XMCL se encarga de tu cuenta de Microsoft, de descargar Minecraft y del botón Jugar. Cuando pulsas Jugar en la instancia «Zenith», el puente arranca Zenith en lugar de Java.

Es un proyecto independiente y **no está afiliado a XMCL (Voxelum), a Mojang ni a Microsoft**. La licencia MIT de XMCL va dentro del paquete (carpeta `licencias`). Se está preparando un launcher propio de Zenith; este paquete es provisional.

Es **experimental**: solo se ha comprobado con pruebas automáticas y con el contenido de los archivos publicados, no ejecutando el instalador en un Windows real.

## Requisitos

- Windows 10 u 11 (solo Windows por ahora).
- Una cuenta de Microsoft con **Minecraft: Java Edition**. El puente lo comprueba con los servicios de Minecraft; las cuentas sin licencia, offline, ely.by o LittleSkin se rechazan.

Zenith no distribuye ningún archivo de Mojang: XMCL descarga Minecraft y el puente prepara los recursos de Zenith a partir de esa descarga.

## Instalar

1. En [Releases](https://github.com/gitzetazero/zenith/releases/tag/nightly), descarga `Zenith-Launcher-Setup-x64.exe`. Si quieres, comprueba su SHA-256 con la línea que le corresponde en `SHA256SUMS` (`certutil -hashfile Zenith-Launcher-Setup-x64.exe SHA256`). Eso solo detecta una descarga corrupta.
2. Ejecútalo. **El instalador no está firmado para Windows**, así que SmartScreen avisará («Windows protegió su PC»: Más información, Ejecutar de todas formas). Se instala solo para tu usuario, sin administrador, en `%LOCALAPPDATA%\Programs\Zenith Launcher`, con acceso directo en el menú Inicio. Admite `/S` (silencioso) y `/D=ruta`.
3. Abre «Zenith Launcher». **La primera vez** XMCL te pide su carpeta de datos: elígela, ciérralo del todo y abre «Zenith Launcher» otra vez. Ahora se crea la instancia **Zenith**.
4. En XMCL inicia sesión con tu cuenta de Microsoft, elige la instancia **Zenith** y pulsa Jugar.

La primera vez, XMCL descarga Minecraft 26.3 y Zenith prepara sus recursos: de unos 30 segundos a unos minutos. Si no tienes Java 25, Zenith descarga el de Mojang para esa preparación.

**Sin instalador:** `Zenith-Launcher-Portable-x64.zip` trae lo mismo en una carpeta. Descomprímela y abre `Zenith Launcher.cmd`. XMCL no tiene modo portátil: sus datos y los de Zenith (`%APPDATA%\xmcl` y `%APPDATA%\zenith`) viven en tu perfil aunque la carpeta esté en un pendrive.

**Desinstalar:** «Aplicaciones instaladas» de Windows. Solo borra lo que instaló. Nunca toca `%APPDATA%\xmcl` (tus mundos e instancias) ni `%APPDATA%\zenith`.

Si ya tenías XMCL, el instalador no lo toca: instala otra copia y reutiliza los datos de XMCL, así que verás tus instancias de siempre más la de Zenith.

## El «Java 25» que aparece en XMCL

En la lista de Java de XMCL verás un Java 25.0.5 en `%APPDATA%\zenith\java\bin`. Es el puente: responde como Java y solo acepta lanzar Zenith. **No lo uses en otras instancias**: se negará a arrancar con un mensaje claro y no hará nada más.

## Órdenes del puente (avanzado)

- `zenith-bridge start`: lo que ejecuta el acceso directo. Prepara la instancia y abre XMCL.
- `zenith-bridge setup`: instala o repara la instancia y el «Java» de Zenith, sin abrir XMCL. Guarda tu configuración anterior como `instance.json.bak`.
- `zenith-bridge status`: carpeta de datos, instancia, build instalada, versión fijada y últimas líneas del registro.
- `zenith-bridge use <versión>`: fija una build instalada (para volver atrás si una versión falla). `zenith-bridge use latest` vuelve a las actualizaciones automáticas.
- `zenith-bridge help`.

Es un programa de ventana: sin consola, el resultado sale en un cuadro de diálogo.

## Actualizaciones

- **El juego:** automáticas. Al pulsar Jugar, el puente busca la última build y la instala solo si pasa las comprobaciones: la firma de `SHA256SUMS`, el hash del archivo y un sello de compilación firmado, con el que rechaza una build más antigua que la instalada. Sin conexión usa la que ya tienes.
- **El puente instalado:** se renueva instalando un instalador más nuevo (o descomprimiendo el zip portátil nuevo) y abriendo «Zenith Launcher».
- **XMCL:** se actualiza con su propio actualizador. Cambia solo `resources\app.asar`; el puente y tus datos no se tocan. Windows pedirá permiso de administrador al instalar. Después de actualizarse, XMCL se abre sin pasar por «Zenith Launcher» (no importa: la instancia ya está lista). No lo hemos probado ejecutando XMCL, solo leyendo su código (v0.73.1).

## Solución de problemas

- El registro del puente está en `%APPDATA%\zenith\logs\bridge.log` (también con `zenith-bridge status`).
- El inicio de sesión de Microsoft lo gestiona XMCL, no Zenith.
- Si la instancia no aparece, cierra XMCL del todo y ábrelo otra vez: solo lee las instancias al arrancar.
- SmartScreen o el antivirus pueden marcar el instalador, el zip o un `java.exe` desconocido: ni el instalador ni el puente están firmados para Windows. Puedes comprobar sus hashes con `SHA256SUMS`.
- Para rehacer los recursos de Zenith, borra `%APPDATA%\zenith\prepared`.

## Privacidad

- La telemetría y los cosméticos de Zenith vienen desactivados, también si abres el juego sin el puente. La telemetría de XMCL es cosa de XMCL: puedes desactivarla en Ajustes, General.
- El puente solo contacta con: `api.minecraftservices.com` (comprobar que tu cuenta tiene el juego), `github.com` y su CDN de descargas (builds y actualizaciones), y `launchermeta.mojang.com`, `piston-meta.mojang.com` y `piston-data.mojang.com` (el jar del servidor y, si no tienes Java 25, el Java de Mojang). Todo por https. Las builds llevan firma Ed25519 y los archivos de Mojang se comprueban con sha1.
- Guarda en `%APPDATA%\zenith` un registro y una caché de qué cuenta posee el juego, no tu token.

## Límites conocidos

- Experimental; no verificado con la interfaz real de XMCL ni con todas sus versiones.
- El instalador y el zip no están firmados para Windows.
- Incluye una versión fija de XMCL (la de `zenith/launcher/xmcl-pin.txt`).
- Solo Windows.
- Los mods de Java no funcionan: Zenith no ejecuta la JVM de Minecraft.
- Una instancia Zenith solo sirve para la versión de Minecraft de su build (hoy 26.3).
- La ruta de la carpeta de datos de XMCL no debe pasar de unos 300 caracteres.

Para servidores y versiones, mira [`SERVIDORES.md`](../SERVIDORES.md).
