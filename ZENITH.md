# Guía para quien mantiene Zenith

Esta guía es para quien mantiene o colabora en el código. Si solo quieres jugar, lee el [README](README.md).

## Cómo funciona el fork

Zenith nace de [Horizon](https://github.com/horizon-rust/horizon) (Minecraft en Rust, licencia MIT). Zenith añade una capa encima y recibe el trabajo de Horizon por sincronización. Reglas:

1. **No renombrar lo interno.** Crates, rutas, binarios internos y variables `HORIZON_*` siguen igual. Renombrarlos daría conflictos en cada sincronización.
2. **El código propio va en `zenith/`**, con su propio workspace de Cargo (como `launcher/`), para no tocar el `Cargo.toml` ni el `Cargo.lock` de Horizon. En los archivos de Horizon solo caben enganches pequeños marcados con `// zenith:`, sin reformatear nada.
3. **Primero mirar arriba.** Antes de arreglar algo, comprueba con `gh pr list -R horizon-rust/horizon --search` que Horizon no lo esté haciendo ya.
4. **Lo genérico se propone a Horizon.** Reduce la diferencia entre los dos.
5. **Seguridad heredada:** `tools/check_public_tree.py` debe pasar, no se sube ningún archivo de Mojang, y toda ejecución de prueba usa carpetas temporales en `HORIZON_HOME` y `HORIZON_LAUNCHER_HOME`.

## Sincronización con Horizon

El workflow `zenith-sync` (`.github/workflows/zenith-sync.yml`) corre cada día, o a mano:

1. Elige la última compilación nocturna de Horizon (`zenith/ci/pick_nightly.py`) y, si no está ya en `main`, la fusiona.
2. Si hay conflictos, no toca `main`: sube una rama `sync/...` y abre un PR que lista los archivos en conflicto.
3. Si no, comprueba el árbol fusionado (compilación, pruebas, clippy), publica `main` y lanza `zenith-release`.

Para que `README.md` y los demás archivos propios no se pisen, `.gitattributes` los marca con `merge=ours` y el workflow activa el driver (`git config merge.ours.driver true`). Si añades un archivo propio que Horizon también tenga, márcalo igual.

Estado: se ha ejecutado una sola vez y no había nada nuevo que fusionar, así que el camino de fusión y comprobación no está probado.

### Qué es de quién

**Archivos propios de Zenith** (Horizon no los tiene, no hay conflicto posible): `ZENITH.md`, `SERVIDORES.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `branding/**`, `zenith/**`, `docs/superpowers/**`, `.github/ISSUE_TEMPLATE/**`, `.github/PULL_REQUEST_TEMPLATE/**` y los workflows `zenith-*.yml`.

**Archivos propios que Horizon también tiene** y que `merge=ours` protege: `README.md`, `CONTRIBUTING.md`.

**Archivos de Horizon que se dejan como están** (reescribirlos daría conflicto en cada sincronización): `ROADMAP.md`, `AGENTS.md`, `CLAUDE.md`, `docs/**` (menos `docs/superpowers/`), `.github/pull_request_template.md` y `THIRD_PARTY_NOTICES.md`. Describen el proyecto de Horizon.

## Qué cambia Zenith en el juego

- Nombre en pantalla «Zenith» y sin los servidores de Horizon.
- Traductor de versiones de servidor, activado por defecto, con selector por servidor (`crates/horizon-client/src/zenith_versions.rs`). Viene del PR #8718 de Horizon, que sigue en borrador. Ver [`SERVIDORES.md`](SERVIDORES.md).
- Puente y launcher (`zenith/bridge`, `zenith/launcher`): comprueba que la cuenta tiene el juego, instala las builds firmadas y arranca el cliente.
- Solo cliente: `horizon-server`, sus archivos de empaquetado y `docs/hosting` no se compilan ni se publican. El servidor integrado del modo un jugador va dentro del cliente. La única edición en archivos de Horizon es la lista `TOOLS` de `xtask/src/main.rs` (marcada `// zenith:`).

### Traductor de versiones (PR #8718)

Está integrado desde el commit `cc3ee16e4664eeced430dc05d68074602d79ba63` de la rama `feature/older-servers-1.21.8`. Para volver a traerlo:

```sh
git fetch https://github.com/TheApple05/horizon.git feature/older-servers-1.21.8
git merge --no-ff FETCH_HEAD
```

Cuando Horizon fusione el PR, la sincronización puede dar conflictos en `crates/horizon-protocol/src/adapter*`, `item.rs` y `tools/generate_registry_id_adapter.py`: resuélvelos quedándote con la versión de Horizon. Al fusionar se añadieron a mano `..BEFORE_26_1` en 1.21.8 y `dropped_components: &[]` en `v1_21_8_ids.rs`, y `item.rs` solo guarda el parche de componentes sin adaptador activo.

## Compilar, probar y publicar

```sh
cargo build --release -p horizon-client              # el juego
cargo test -p horizon-protocol -p horizon-client     # protocolo y selector de versiones
cd zenith/bridge && cargo test                       # el puente (otro workspace)
python tools/check_public_tree.py                    # comprobación del árbol público
python -m unittest discover zenith/ci               # pruebas de la sincronización
```

Publicar: el workflow `zenith-release` (`workflow_dispatch`, con la fecha UTC) compila `main`, firma `SHA256SUMS` y publica `nightly-<fecha>` y la release móvil `nightly`. La variable de repositorio `ZENITH_AUTOPUBLISH=off` lo detiene. El instalador se construye con `zenith/launcher/build.sh` (ver el encabezado del script).

Los nombres de binarios y variables siguen siendo los de Horizon: `horizon-client`, `horizon-prepare`, `HORIZON_HOME`, `HORIZON_OLDER_SERVERS`, etc.

## Pruebas que fallan a propósito o por conocidas

- `entities::drop_motion_tests::drops_match_vanillas_motion_bit_for_bit` falla también en Horizon (el último bit de un `f64` difiere en algunos equipos).
- Zenith tiene su propio `README.md`, así que fallan `tests/test_parity.py` y `tests/test_parity_gaps.py::test_the_readme_agrees_with_itself` (esperan las tablas de paridad de Horizon). La sincronización no las ejecuta; si corres toda la suite de Python, sáltalas. `tools/parity_gaps.py` tampoco encontrará su bloque en el README.

## Estado

Verificado: versiones firmadas y pruebas de manipulación, pruebas del puente, prueba de humo del cliente real con propietario inyectado.

Sin verificar: Play real desde el launcher, la respuesta real de la API de propietario, el camino de fusión de la sincronización, los adaptadores de servidor contra servidores reales (la matriz real no se ha ejecutado) y Linux.

## Launcher propio (fase 6)

Estado: empezando. Se está traduciendo XMCL v0.73.1 (MIT, TypeScript/Vue/Electron) a Rust como programa propio: <https://github.com/gitzetazero/zenith-launcher>. No sirve todavía para jugar y no tiene inicio de sesión. Etapas (el detalle está en `docs/xmcl-parity.md` de ese repositorio):

1. Ventana, componentes, temas, idiomas y ajustes.
2. Cuenta de Microsoft, instancias y arranque de Zenith. Desde aquí sustituye al puente y a XMCL.
3. Descargas: Minecraft, Java, Forge, Fabric, Quilt, OptiFine, espejos.
4. Recursos: mods, shaders, resource packs, mundos, Modrinth, CurseForge, `.hmod`.
5. Modpacks: importar y exportar.
6. El resto: multijugador P2P y cuentas de terceros.

El inicio de sesión real necesita una app de Azure propia y la aprobación de Mojang (`docs/microsoft-account.md`). Hasta entonces solo hay cuentas offline de desarrollo, que nunca son una forma de jugar. El instalador y el zip portátil que empaquetan XMCL sin modificar son provisionales: se retirarán cuando la etapa 2 esté lista.

## Aviso legal

Zenith no está afiliado, respaldado ni patrocinado por Mojang AB ni por Microsoft. Minecraft es una marca de Mojang Synergies AB. Este repositorio no contiene archivos de Mojang.
