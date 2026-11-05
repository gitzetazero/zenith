<img src="zenith/branding/png/banner.png" alt="Zenith: Minecraft en Rust, en su punto más alto. Fork de Horizon." width="100%">

# Zenith

## Qué es

Zenith es Minecraft: Java Edition 26.3 escrito de nuevo en Rust. Se instala, entras con tu cuenta de Microsoft y juegas, solo o en servidores. Además de los servidores 26.3, puede entrar a servidores de versiones anteriores (de 1.20.2 en adelante).

> **Estado: temprano y experimental.** No hay anuncio todavía. Lo que sigue describe lo que existe hoy, no una promesa.

## Download and install

Windows only for now. Download **Zenith Launcher** from the [zenith-launcher Releases](https://github.com/gitzetazero/zenith-launcher/releases) page (a fork of XMCL with Zenith built in):

1. Get `Zenith-Launcher-Setup-x64.exe` (per-user installer, no administrator) or `Zenith-Launcher-Portable-x64.exe`, and check it against `SHA256SUMS`.
2. Run it. Windows will warn you ("Windows protected your PC") because the build is not code-signed: choose "More info", then "Run anyway".
3. First-run steps, updates and limits are in [`zenith/README.md`](zenith/README.md). Note: Microsoft sign-in in Zenith Launcher needs an Azure app approved by Mojang that does not exist yet, so for now it shows a message instead of signing in.

(The older interim installer on the [`nightly` release](https://github.com/gitzetazero/zenith/releases/tag/nightly) bundles unmodified XMCL and is being withdrawn.)

## Requisitos

- Windows 10 u 11 de 64 bits.
- Una tarjeta gráfica con Vulkan o DirectX 12.
- Una cuenta de Microsoft que tenga **Minecraft: Java Edition**.
- Tu propia copia de los archivos del juego. Zenith no incluye código ni recursos de Minecraft: el launcher descarga el juego de Mojang con tu cuenta y Zenith prepara sus recursos en tu equipo a partir de esa copia.

## Cómo jugar

1. Install and open "Zenith Launcher".
2. La primera vez te pide elegir una carpeta de datos. Elígela, cierra el launcher y ábrelo otra vez: así se crea la instancia «Zenith».
3. Inicia sesión con tu cuenta de Microsoft.
4. Elige la instancia «Zenith» y pulsa Jugar. La primera vez descarga y prepara los recursos: tarda desde unos segundos hasta unos minutos.
5. Para jugar en un servidor, añádelo en Multijugador como siempre. Zenith detecta su versión. Más en [`zenith/SERVERS.md`](zenith/SERVERS.md).

## Qué trae

- El motor del juego, escrito en Rust.
- Multijugador con traducción de versiones: entra a servidores de 1.20.2 a 26.2 además de 26.3, con un selector de versión para cada servidor. **Aún sin probar con servidores reales**: mira la tabla en [`zenith/SERVERS.md`](zenith/SERVERS.md).
- Actualizaciones automáticas del juego, con firma comprobada antes de instalar.
- Sin telemetría y sin cosméticos por defecto.

**En camino** (todavía no existe): shaders que funcionen bien, presets de rendimiento y un launcher propio hecho en Rust.

## Servidores y versiones soportadas

| Versión del servidor | Cómo se entra | Estado |
|---|---|---|
| 26.3 | Nativo | Nativo (26.3) |
| 1.21.7 a 26.2 | Traducido | Traducido |
| 1.20.2 a 1.21.6 | Traducido | Traducido, experimental |

No se puede entrar a servidores anteriores a 1.20.2, a Bedrock ni a servidores que exigen mods. **Ninguna versión traducida se ha probado todavía contra un servidor real.** La tabla completa, cómo se elige la versión y cómo apagar el traductor están en [`zenith/SERVERS.md`](zenith/SERVERS.md).

## Compilar desde el código

Hace falta Rust 1.90 o más, Python 3.10 o más, JDK 25 y, en Linux, `libasound2-dev libudev-dev`.
La preparación construye los recursos del juego a partir de **tu propia copia** de Minecraft: Java Edition 26.3 y nunca la descarga. No subas nunca archivos del juego al repositorio.

1. **Cliente:** juega 26.3 una vez con el launcher oficial. Zenith lee `~/.minecraft` (o la ruta de `--minecraft-dir` / `HORIZON_MINECRAFT_DIR`).
2. **Jar del servidor:** el jar del servidor 26.3, pasado una vez con `--server-jar` o `HORIZON_SERVER_JAR`.

```sh
git clone https://github.com/gitzetazero/zenith.git
cd zenith
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
cargo run --release -p horizon-client
```

Los archivos preparados quedan en `.local/`, que git ignora: no los subas ni los compartas. Los nombres internos (`horizon-client`, `HORIZON_*`) no se han cambiado a propósito; ver [`ZENITH.md`](ZENITH.md).

## Problemas conocidos

- Todo es experimental y solo se ha probado con pruebas automáticas, no con el instalador en un Windows real.
- Los servidores traducidos no se han probado con servidores reales.
- El instalador y el programa que prepara el juego no están firmados para Windows: SmartScreen y algunos antivirus avisarán.
- Solo Windows. Los mods de Java no funcionan: Zenith no ejecuta la JVM de Minecraft.
- Una instancia de Zenith solo sirve para la versión del juego que trae (hoy 26.3).

¿Algo más? [Abre un aviso](https://github.com/gitzetazero/zenith/issues/new/choose).

## Más información

- [`zenith/SERVERS.md`](zenith/SERVERS.md): versiones de servidor y cómo se elige.
- [`zenith/README.md`](zenith/README.md): guía del jugador con el launcher.
- [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md) y [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md): cómo colaborar y cómo avisar de un fallo de seguridad.
- [`ZENITH.md`](ZENITH.md): guía para quien mantiene el proyecto.
- [`zenith/branding/BRAND.md`](zenith/branding/BRAND.md): logotipo y colores.

## Créditos y licencia

Zenith es un fork de [Horizon](https://github.com/horizon-rust/horizon). Gracias al equipo de Horizon y a sus colaboradores: el motor es obra suya, y Zenith recibe sus mejoras por sincronización. Si te gusta lo que hacen, apóyalos a ellos.

El código es [MIT](LICENSE). Los avisos de copyright originales se conservan en el archivo de licencia, junto al de los colaboradores de Zenith. Las dependencias conservan sus licencias ([avisos](THIRD_PARTY_NOTICES.md)).

Minecraft es una marca de Mojang Synergies AB. Zenith no está afiliado a Mojang ni a Microsoft, ni respaldado por ellos. No contiene código, recursos ni datos de Minecraft: necesitas tu propia copia de Java Edition.
