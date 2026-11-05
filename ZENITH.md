# Zenith

Zenith es un fork de [Horizon](https://github.com/horizon-rust/horizon) (Minecraft en Rust, licencia MIT). Todo el mérito del motor es del equipo de Horizon; Zenith solo añade una capa encima y mantiene la sincronización con su trabajo.

## Reglas del fork

1. **No renombrar lo interno.** Crates, rutas, binarios internos y variables `HORIZON_*` siguen igual. Renombrarlos daría conflictos en cada sincronización.
2. **Código de Zenith en `zenith/`, con su propio workspace de Cargo** (como hace `launcher/`), para no tocar el `Cargo.toml` ni el `Cargo.lock` de Horizon. En archivos de Horizon, solo enganches pequeños marcados con `// zenith:`, sin reformatear nada.
3. **Primero mirar arriba.** Antes de arreglar algo, comprobar con `gh pr list --search` que Horizon no lo está haciendo ya.
4. **Lo genérico se propone a Horizon.** Reduce nuestra diferencia. Cada PR externo necesita el visto bueno del usuario y sigue las reglas de agentes de su `AGENTS.md`.
5. **Se heredan sus reglas de seguridad:** `tools/check_public_tree.py`, nada de archivos de Mojang en el repo, y carpetas temporales `HORIZON_HOME` / `HORIZON_LAUNCHER_HOME` en cualquier ejecución de prueba.

## Marca

**Archivos propios de Zenith:** `README.md` (protegido con `merge=ours` en `.gitattributes`: la sincronización nunca lo pisa), `ZENITH.md`, `docs/superpowers/**` y `zenith/**`.

**Archivos de Horizon que se dejan como están a propósito**, porque reescribirlos daría conflicto en cada sincronización: `ROADMAP.md`, `CONTRIBUTING.md`, `AGENTS.md`, `CLAUDE.md`, `docs/**` y `THIRD_PARTY_NOTICES.md`. `CONTRIBUTING.md`, `AGENTS.md` y `CLAUDE.md` describen las reglas de Horizon; el contrato de Zenith es este `ZENITH.md`.

## Aviso legal

Zenith no está afiliado, respaldado ni patrocinado por Mojang AB ni por Microsoft. Minecraft es una marca de Mojang AB. Este repositorio no contiene archivos de Mojang.

## Known failing upstream tests

- `entities::drop_motion_tests::drops_match_vanillas_motion_bit_for_bit` fails on upstream 83ad384f4 too (last bit of one f64 differs on this machine).

## Tests de Python que fallan por nuestro README

Zenith tiene su propio `README.md`, así que dos tests de Horizon que lo leen fallan a propósito: `tests/test_parity.py` y `tests/test_parity_gaps.py::test_the_readme_agrees_with_itself` (esperan las tablas y badges de paridad de Horizon). La sincronización diaria no ejecuta esos tests; si se ejecuta la suite completa de Python, hay que saltarlos. `tools/parity_gaps.py` también escribe en el README y no encontrará su bloque.

## Servidores 1.20.2 a 1.21.8 (Horizon #8718)

Integrado el PR horizon-rust/horizon#8718 (borrador de TheApple05, adaptadores nativos para servidores antiguos)
en el commit `cc3ee16e4664eeced430dc05d68074602d79ba63` de la rama `feature/older-servers-1.21.8`.

Volver a traerlo:

```bash
git fetch https://github.com/TheApple05/horizon.git feature/older-servers-1.21.8
git merge --no-ff FETCH_HEAD
```

La selección sigue siendo la de Zenith (`zenith_versions::choose`, activa por defecto);
`HORIZON_OLDER_SERVERS=0` la apaga. Las versiones por debajo de 1.21.8 salen como "(experimental)".

Cuando Horizon fusione el PR, la sincronización con upstream puede dar conflictos en
`crates/horizon-protocol/src/adapter*`, `item.rs` y `tools/generate_registry_id_adapter.py`: resolverlos
quedándose con la versión de upstream. Al fusionar se añadieron a mano `..BEFORE_26_1` en 1.21.8 y
`dropped_components: &[]` en `v1_21_8_ids.rs`, y `item.rs` solo guarda el parche de componentes sin adaptador activo.

## Solo cliente

Zenith es solo el cliente: el servidor dedicado (`horizon-server`, los archivos `horizon-server-*`, `package-server`, `docs/hosting`) no se compila, empaqueta ni publica. Su código sigue en el repo sin tocar para evitar conflictos al sincronizar; la única edición es la lista `TOOLS` de `xtask/src/main.rs` (marcada `// zenith:`), que ya no exige `horizon-server`. El servidor integrado de un jugador va dentro del cliente.

## Estado

Verificado: versiones firmadas y pruebas de manipulación, pruebas del puente, prueba de humo del cliente real con propietario inyectado.

Sin verificar: Play real desde XMCL, respuesta real de la API de propietario, el camino de fusión de la sincronización, adaptadores 1.20.2-1.21.8 contra servidores reales, Linux.
