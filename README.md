<img src="branding/png/banner.png" alt="Zenith: Minecraft Java en Rust, entra a más servidores. Fork de Horizon." width="100%">

# Zenith

Zenith es un fork de [Horizon](https://github.com/horizon-rust/horizon), Minecraft: Java Edition 26.3 reescrito en Rust, que se mantiene al día con los arreglos de Horizon y busca entrar a más servidores.

> **Estado: temprano, privado primero.** El repositorio y la Release `nightly` (con el exe del puente) son públicos, pero no hay anuncio y todo es experimental y sin verificar con la interfaz real de XMCL. Lo que sigue describe lo que existe hoy, no una promesa.

## Crédito a Horizon

Todo el motor es obra del equipo de Horizon y sus colaboradores. Zenith solo añade una capa encima y recibe su trabajo por sincronización. Si te interesa el proyecto de fondo, está en **[github.com/horizon-rust/horizon](https://github.com/horizon-rust/horizon)**: ve a apoyarlo a él.

## Qué es distinto hoy

- **Nombre:** el juego muestra "Zenith" y ya no ofrece los servidores de Horizon.
- **Servidores más antiguos:** el traductor de versiones viene activado por defecto, con un selector de versión por servidor. Hay adaptadores de 1.20.2 a 26.2; los de 1.20.2 a 1.21.7 son experimentales.
  Vienen del [PR #8718 de Horizon](https://github.com/horizon-rust/horizon/pull/8718), que sigue en borrador y **no se ha probado contra servidores reales**.
- **Todo lo demás es Horizon.** La `main` de Zenith ya contiene la `main` de Horizon hasta el 2026-11-04, y un workflow diario (`zenith-sync`) fusiona las nuevas compilaciones nocturnas de Horizon. Se ha ejecutado una sola vez y no había nada nuevo que fusionar, así que el camino de fusión y comprobación todavía no está probado.

**Solo cliente.** Zenith no compila, empaqueta ni publica el servidor dedicado de Horizon; el servidor integrado del modo un jugador va dentro del cliente.

## Cómo se juega

Experimental, solo Windows. Hace falta una cuenta de Microsoft que sea dueña de Minecraft: Java Edition.

1. En la página de Releases (pre-release `nightly`) descarga `Zenith-Launcher-Setup-x64.exe` y ejecútalo (se instala solo para tu usuario, sin administrador; Windows SmartScreen avisará porque no está firmado). Incluye el lanzador oficial [XMCL](https://xmcl.app/en/) sin modificar y el puente de Zenith. Alternativa sin instalador: `Zenith-Launcher-Portable-x64.zip`.
2. Abre «Zenith Launcher». La primera vez XMCL te pide su carpeta de datos: elígela, cierra XMCL y abre «Zenith Launcher» otra vez (así se crea la instancia «Zenith»).
3. Inicia sesión con la cuenta de Microsoft que tiene Java Edition y pulsa Jugar en la instancia «Zenith».

Los pasos completos, las actualizaciones y los límites están en [`zenith/README.md`](zenith/README.md). Todavía no se ha verificado contra la interfaz real de XMCL ni con una instalación real del instalador.

## Qué está planeado

Nada de esto existe todavía:

- Presets de FPS (máximo, equilibrado, calidad).
- Shaders que funcionen bien.
- Un launcher propio en Rust, traducción de XMCL.

El detalle, el orden y los riesgos están en [`ZENITH.md`](ZENITH.md) y en el
[diseño](docs/superpowers/specs/2026-11-04-zenith-design.md).

## Compilar desde el código

Hace falta Rust 1.90+, Python 3.10+, JDK 25 y, en Linux, `libasound2-dev libudev-dev`.
`horizon-prepare` construye los recursos del juego a partir de **tu propia copia** de Minecraft: Java Edition 26.3 (comprobada contra `research-lock.json`) y nunca la descarga.
No subas nunca archivos del juego al repositorio.

1. **Cliente:** juega 26.3 una vez con el launcher oficial. Zenith lee `~/.minecraft` (o `--minecraft-dir` / `HORIZON_MINECRAFT_DIR`).
2. **Jar del servidor:** el jar del servidor 26.3 (`downloads.server.url` en `versions/26.3/26.3.json`), pasado una vez con `--server-jar` o `HORIZON_SERVER_JAR`.

```sh
git clone https://github.com/gitzetazero/zenith.git
cd zenith
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar   # necesita JDK 25
cargo run --release -p horizon-prepare -- sounds   # opcional: sonidos de tu instalación del launcher
cargo run --release -p horizon-client
```

Los archivos preparados quedan en `.local/`, ignorado por git: no los subas ni los compartas.
Los crates, binarios y variables `HORIZON_*` conservan su nombre a propósito (ver `ZENITH.md`).
Opciones, controles y arquitectura: [docs/native-client.md](docs/native-client.md).

## Más información

- [`ZENITH.md`](ZENITH.md): reglas del fork, qué archivos son de Zenith y cuáles de Horizon.
- [`docs/superpowers/specs/`](docs/superpowers/specs/) y [`docs/superpowers/plans/`](docs/superpowers/plans/): el diseño y el plan de trabajo.
- Documentación de Horizon (en inglés): [docs/README.md](docs/README.md) y [ROADMAP.md](ROADMAP.md), que describen Horizon, no Zenith.

## Licencia y avisos

[MIT](LICENSE), con el copyright de Horizon y el de los colaboradores de Zenith. Las dependencias conservan sus licencias ([avisos](THIRD_PARTY_NOTICES.md)).

Minecraft es una marca de Mojang Synergies AB. Zenith no está afiliado a Mojang ni a Microsoft, ni respaldado por ellos. No contiene código, recursos ni datos de Minecraft: necesitas tu propia copia de Java Edition 26.3.
