# Zenith

Zenith es un fork de [Horizon](https://github.com/horizon-rust/horizon) (Minecraft en Rust, licencia MIT). Todo el mérito del motor es del equipo de Horizon; Zenith solo añade una capa encima y mantiene la sincronización con su trabajo.

## Reglas del fork

1. **No renombrar lo interno.** Crates, rutas, binarios internos y variables `HORIZON_*` siguen igual. Renombrarlos daría conflictos en cada sincronización.
2. **Código de Zenith en `zenith/`, con su propio workspace de Cargo** (como hace `launcher/`), para no tocar el `Cargo.toml` ni el `Cargo.lock` de Horizon. En archivos de Horizon, solo enganches pequeños marcados con `// zenith:`, sin reformatear nada.
3. **Primero mirar arriba.** Antes de arreglar algo, comprobar con `gh pr list --search` que Horizon no lo está haciendo ya.
4. **Lo genérico se propone a Horizon.** Reduce nuestra diferencia. Cada PR externo necesita el visto bueno del usuario y sigue las reglas de agentes de su `AGENTS.md`.
5. **Se heredan sus reglas de seguridad:** `tools/check_public_tree.py`, nada de archivos de Mojang en el repo, y carpetas temporales `HORIZON_HOME` / `HORIZON_LAUNCHER_HOME` en cualquier ejecución de prueba.

## Aviso legal

Zenith no está afiliado, respaldado ni patrocinado por Mojang AB ni por Microsoft. Minecraft es una marca de Mojang AB. Este repositorio no contiene archivos de Mojang.
