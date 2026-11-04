# Zenith, fases 1 a 3: plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un fork de Horizon que recibe sus parches a diario, se publica firmado, se juega desde XMCL con la cuenta del jugador y entra a servidores de 1.20.2 a 26.3.

**Architecture:** Fork público con merges desde las nightlies de Horizon. Lo propio vive en `zenith/` (workspace de Cargo aparte) y en archivos nuevos con enganches mínimos en los de Horizon. XMCL sin modificar inicia sesión y descarga Minecraft; su "comando previo" ejecuta `zenith-bridge`, que comprueba la propiedad, instala la build firmada, prepara recursos y arranca `horizon-client` según la Launch API de Horizon.

**Tech Stack:** Rust, `ureq`, `serde_json`, `zip`, `horizon-release-sign`; GitHub Actions; Python solo para herramientas de CI y pruebas.

**Spec:** `docs/superpowers/specs/2026-11-04-zenith-design.md`

## Global Constraints

- Repos: fork `gitzetazero/zenith`, remoto `upstream` = `horizon-rust/horizon`, clon en `C:\Users\<usuario>\Documents\horizon`.
- No renombrar crates, rutas, binarios internos ni variables `HORIZON_*`.
- Código propio en `zenith/`. Si tiene que compilar dentro de un crate de Horizon: archivo nuevo más un enganche de pocas líneas marcado `// zenith:`. Sin reformatear código ajeno. No tocar `Cargo.toml` ni `Cargo.lock` raíz, ni `launcher/`.
- Nada de archivos de Mojang ni shader packs en el repo. `tools/check_public_tree.py` debe pasar.
- Cualquier ejecución de prueba usa carpetas temporales. `%APPDATA%\horizon` y `%APPDATA%\horizon-launcher` nunca se escriben.
- El token nunca aparece en un log, en la orden de `horizon-client` ni en el chat. Secretos solo como secretos del repo, creados por el usuario.
- Ningún ID de aplicación de Microsoft en Zenith: inicia sesión XMCL. No se salta la comprobación de propiedad.
- Sin código de ViaVersion ni ViaFabricPlus (GPL-3). Zenith sigue siendo MIT.
- Textos visibles nuevos con clave de traducción, en inglés y español.
- Antes de arreglar algo de Horizon: `gh pr list --repo horizon-rust/horizon --search <tema>`.
- Antes de `git commit`, `python3 tools/check_public_tree.py`. Los commits terminan con `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

- **Rutas con espacios o no ASCII** (usuario `José Ñ`, ver Horizon #6009) en Java, carpeta de juego y comando previo: deben funcionar. Tests en tareas 6 y 10.
- **Sin internet con build instalada:** el juego arranca igual para una cuenta ya verificada; sin verificación previa se rechaza con un mensaje claro. Tests en tareas 7 y 8.
- **Descarga cortada o firma mala:** la build actual sigue intacta y se juega con ella. Tests en tarea 8.
- **Expulsiones con números que no son un rango** ("Baneado hasta 2026-11-05", "Reinicio en 1.5 minutos"): no provocan reintento. Tests en tarea 11.
- **Instancia de XMCL con otra versión o cuenta offline:** mensaje claro, sin arrancar. XMCL pasa `msa` por defecto, así que `--userType` no basta. Tests en tareas 7 y 9.

---

# Fase 1: base del fork

### Task 1: Fork, clon y reglas

**Files:**
- Create: `ZENITH.md`, `docs/superpowers/specs/2026-11-04-zenith-design.md` (ya copiado)

**Interfaces:**
- Produces: remotos `origin` y `upstream`; workflows heredados desactivados.

- [ ] **Step 1: Crear el fork**

Run: `gh repo fork horizon-rust/horizon --fork-name zenith --clone=false --default-branch-only`
Expected: `gh repo view gitzetazero/zenith --json isFork --jq .isFork` imprime `true`.

- [ ] **Step 2: Convertir la carpeta actual en el clon** (`docs/superpowers` ya existe y no choca)

```bash
git init -b main
git remote add origin https://github.com/gitzetazero/zenith.git
git remote add upstream https://github.com/horizon-rust/horizon.git
git fetch origin main
git checkout -B main --track origin/main
git config core.hooksPath .githooks
git config rerere.enabled true
```
Expected: `git status` muestra solo `docs/superpowers/` sin seguir.

- [ ] **Step 3: Desactivar los workflows heredados**

Activar Actions con `gh api -X PUT repos/gitzetazero/zenith/actions/permissions -F enabled=true`. Luego `gh workflow disable <id>` para cada `id` de `gh workflow list --all --json id --jq '.[].id' -R gitzetazero/zenith`.
Expected: `gh workflow list --all -R gitzetazero/zenith --json state --jq '[.[]|select(.state=="active")]|length'` imprime `0`.

- [ ] **Step 4: Escribir `ZENITH.md`** con las cinco reglas del fork copiadas del spec, el crédito a Horizon y el aviso de no afiliación con Mojang y Microsoft.

- [ ] **Step 5: Instalar lo que falta para compilar**

Run: `winget install --id EclipseAdoptium.Temurin.25.JDK -e` y abrir un shell nuevo.
Expected: `java --version` dice 25. Si `cargo build -p horizon-release-sign` falla por el enlazador, instalar lo que pida antes de seguir.

- [ ] **Step 6: Commit y push**

```bash
git add ZENITH.md docs/superpowers
git commit -m "docs: Zenith's design, plan and fork rules"
git push origin main
```

### Task 2: Sincronización diaria

Requiere que el usuario haya creado el secreto `ZENITH_SYNC_TOKEN` (permisos de contenido y workflows sobre `gitzetazero/zenith`).

**Files:**
- Create: `zenith/ci/pick_nightly.py`, `zenith/ci/test_pick_nightly.py`, `.github/workflows/zenith-sync.yml`

**Interfaces:**
- Produces: `python3 zenith/ci/pick_nightly.py --ref upstream/main` imprime `{"tag": "nightly-source/2026-11-04-6475f62", "release": "nightly", "date": "2026-11-04", "short": "6475f62"}`, o `null` si ninguna sirve. `release` es la Release de Horizon de la que la tarea 3 saca `exporters/`.

- [ ] **Step 1: Escribir los tests fallidos** (stdlib `unittest`)

```python
def test_parses_title(self):
    self.assertEqual(parse_title("Nightly 2026-11-04 · 6475f62"), ("2026-11-04", "6475f62"))
def test_rejects_other_titles(self):
    self.assertIsNone(parse_title("Horizon launcher 0.2.41"))
def test_picks_newest_reachable(self):
    releases = [("nightly", "Nightly 2026-11-04 · aaaaaaa"), ("nightly-2026-11-03", "Nightly 2026-11-03 · bbbbbbb")]
    self.assertEqual(pick(releases, lambda short: short == "bbbbbbb")["short"], "bbbbbbb")
def test_none_when_nothing_reachable(self):
    self.assertIsNone(pick([("nightly", "Nightly 2026-11-04 · aaaaaaa")], lambda s: False))
```

- [ ] **Step 2: Comprobar que fallan.** Run: `python3 -m unittest zenith/ci/test_pick_nightly.py`. Expected: FAIL, `parse_title` no existe.

- [ ] **Step 3: Implementar** `parse_title(title) -> tuple[str, str] | None` y `pick(releases, reachable) -> dict | None`. El `main` lista con `gh release list -R horizon-rust/horizon --json tagName,name` las etiquetas `nightly` y `nightly-*`, y `reachable(short)` comprueba con `git merge-base --is-ancestor nightly-source/<date>-<short> <ref>`. Así se excluyen las nightlies con PRs sin integrar.

- [ ] **Step 4: Comprobar que pasan.** Run: el mismo. Expected: OK.

- [ ] **Step 5: Escribir `zenith-sync.yml`**

Disparadores: `schedule: "17 6 * * *"` y `workflow_dispatch`. `runs-on: ubuntu-22.04`. Checkout de `main` con `token: ${{ secrets.ZENITH_SYNC_TOKEN }}`, `fetch-depth: 0`. Pasos, en orden:
1. Instalar `libasound2-dev libudev-dev`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`.
2. `git remote add upstream …; git fetch upstream main --tags`; ejecutar `pick_nightly.py`. Terminar sin error si es `null` o si la etiqueta ya está en `main`.
3. `git merge --no-edit <tag>` con identidad `zenith-sync`. Si hay conflicto: `git merge --abort`, empujar la etiqueta como rama `sync/<date>-<short>` y `gh pr create --base main` con los archivos de `git diff --name-only --diff-filter=U` en el cuerpo. Terminar en rojo.
4. Comprobar: `python3 tools/check_public_tree.py`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`. Si falla algo: empujar la fusión a `sync/<date>-<short>`, abrir PR con el paso fallido y terminar en rojo.
5. `git push origin HEAD:main`.
6. Desactivar cualquier workflow activo cuyo archivo no empiece por `zenith-`, porque los nuevos de Horizon llegan activos.
7. Si `vars.ZENITH_AUTOPUBLISH != 'off'`: `gh workflow run zenith-release.yml -f release=<release> -f date=<date>`.

- [ ] **Step 6: Probar de verdad.** Run: `gh workflow run zenith-sync.yml` y `gh run watch`. Expected: `git log origin/main` contiene la etiqueta de la nightly, o se abre un PR con la causa. Si algo falla por limitaciones del runner, corregir el workflow antes de seguir.

- [ ] **Step 7: Commit y push** de los tres archivos, con `feat(zenith): daily sync from Horizon's nightlies`.

### Task 3: Publicación firmada

Requiere que el usuario haya ejecutado, una vez, la orden que genera la clave. La privada no se muestra:

```bash
mkdir -p zenith/bridge && cargo run -q -p horizon-release-sign -- keygen --public-out zenith/bridge/zenith-release-keys.txt | gh secret set RELEASE_SIGNING_KEY -R gitzetazero/zenith
```

**Files:**
- Create: `.github/workflows/zenith-release.yml`, `zenith/bridge/zenith-release-keys.txt`

**Interfaces:**
- Consumes: `release` y `date` de la tarea 2.
- Produces: Releases `nightly` y `nightly-<date>` en `gitzetazero/zenith`, título `Nightly <date> · <short>` (short del commit de Zenith), con `horizon-windows-x64.zip`, `horizon-linux-x86_64.tar.gz`, `SHA256SUMS`, `SHA256SUMS.sig`, firmados para el canal `nightly`.

- [ ] **Step 1: Primera comprobación, porque decide el resto**

Run: `gh release download nightly -R horizon-rust/horizon -p horizon-linux-x86_64.tar.gz -D <temporal>`, `tar tzf` y buscar `exporters/manifest.json`.
Expected: `exporters/server`, `exporters/client` y `manifest.json` están, con clases solo bajo `dev/horizonlab/parity`. **Si no están, parar y volver al usuario** con las alternativas del spec: su PC como runner o publicación manual.

- [ ] **Step 2: Escribir `zenith-release.yml`**

`workflow_dispatch` con entradas `release`, `date`. Si existe ya `nightly-<date>`, terminar (como máximo una al día). Jobs:
- `build`, matriz `ubuntu-22.04` y `windows-latest`, con `dir` y `asset` iguales a los de `.github/workflows/game-build.yml:94-108` y `rustflags: -C target-feature=+crt-static` en Windows. Pasos: descargar `exporters/` de la Release `release` de Horizon a `exporters/`; `cargo build --release --locked -p horizon-client -p horizon-prepare`; el mismo comando de empaquetado (`cargo run --locked -p xtask -- package …`) y compresión de `game-build.yml:273-291`; subir artefacto.
- `sign`, en GitHub: solo descarga los artefactos y ejecuta `tools/ci/sign_release.sh nightly dist/<assets>` con `RELEASE_SIGNING_KEY`.
- `publish`: `cargo run --locked -p horizon-release-sign -- verify --tag nightly --key "$(cat zenith/bridge/zenith-release-keys.txt)" dist`; crear `nightly-<date>` y mover `nightly`. Las notas incluyen el aviso de no afiliación. Antes de todo, `vars.ZENITH_AUTOPUBLISH == 'off'` detiene el job.

- [ ] **Step 3: Probar de verdad.** Run: `gh workflow run zenith-release.yml -f release=nightly -f date=<hoy>`, `gh run watch`. Expected: `gh release view nightly -R gitzetazero/zenith --json assets` lista los cuatro archivos.

- [ ] **Step 4: Comprobar la firma como lo hará el puente.** Run: descargar los archivos a una carpeta temporal y `cargo run -q -p horizon-release-sign -- verify --tag nightly --key "$(cat zenith/bridge/zenith-release-keys.txt)" <carpeta>`. Expected: salida correcta. Alterar un byte del zip. Expected: la misma orden falla.

- [ ] **Step 5: Commit y push** con `feat(zenith): signed release workflow`.

### Task 4: Identidad del juego

**Files:**
- Create: `crates/horizon-client-core/src/zenith_brand.rs`
- Modify: `crates/horizon-client-core/src/lang.rs` (`Language::load`, enganche), el `lib.rs` del crate (`pub mod`), `crates/horizon-client/src/game.rs:17310,21347,21870-21875`, `crates/horizon-client/src/game/no_resources.rs:97`, `crates/horizon-client/src/game/split/windows.rs:565`

**Interfaces:**
- Produces: `pub fn apply(map: &mut HashMap<String, String>, code: &str)`, `pub fn rebrand(text: &str) -> String` y `pub const WINDOW_TITLE: &str = "Zenith"`. La tarea 12 amplía `apply`.

- [ ] **Step 1: Escribir los tests fallidos**

```rust
#[test] fn replaces_whole_word() { assert_eq!(rebrand("Horizon Graphics"), "Zenith Graphics"); }
#[test] fn keeps_spanish_word() { assert_eq!(rebrand("Horizonte Horizon."), "Horizonte Zenith."); }
#[test] fn keeps_lowercase() { assert_eq!(rebrand("horizon-client"), "horizon-client"); }
```

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test -p horizon-client-core zenith_brand`. Expected: FAIL.

- [ ] **Step 3: Implementar.** `rebrand` sustituye "Horizon" solo si el carácter siguiente no es alfanumérico. `apply` la aplica a cada valor. Llamar a `apply(&mut map, code)` al final de `Language::load`, sobre `map`, con `// zenith:`. Títulos de ventana con `WINDOW_TITLE`. En `game.rs:21870-21875`, no llamar a `add_official_server` ni `add_partner_servers`, con `// zenith:`.

- [ ] **Step 4: Comprobar que pasan.** Run: `cargo test -p horizon-client-core zenith_brand && cargo check -p horizon-client`. Expected: PASS.

- [ ] **Step 5: Commit** con `feat(zenith): visible name and no offered Horizon servers`.

**Fase 1 terminada cuando:** una nightly real llega a `main` y se publica sola, y el archivo publicado supera la verificación de firma.

---

# Fase 2: XMCL con el puente

### Task 5: Prueba desechable con XMCL

Requiere que el usuario instale XMCL e inicie sesión. **Nada de esta tarea se conserva salvo sus conclusiones.** El programa de prueba vive en el scratchpad.

**Files:**
- Create: `zenith/bridge/SPIKE.md`, `zenith/bridge/tests/fixtures/xmcl-args.txt`

**Interfaces:**
- Produces: la línea real de argumentos, con token y UUID sustituidos por valores falsos. Las tareas 6 y 9 la usan y **se corrigen si difiere** de lo que este plan supone.

- [ ] **Step 1: Compilar un programa de prueba** con `rustc`: escribe sus argumentos en un archivo enmascarando el valor tras `--accessToken`, y arranca `horizon-client` con `--resources` de la nightly que el usuario ya tiene preparada, `--game-dir`/`--home` de la instancia y `--session-stdin`.

- [ ] **Step 2: Con el usuario:** crear en XMCL una instancia de Minecraft 26.3 con el programa como comando previo, pulsar Jugar. Expected: abre Horizon con la cuenta del usuario.

- [ ] **Step 3: Responder en `SPIKE.md`, con evidencia:**
  - Argumentos exactos, incluidos `--width -1` y las rutas.
  - Si XMCL deja de marcar "arrancando" al imprimir `LWJGL Version: zenith` (`packages/core/launch.ts:638`).
  - Si detecta `Crash report saved to: #@!@# <ruta>`.
  - Qué muestra XMCL cuando el programa sale con 1 y escribe en stderr.
  - Si su pestaña de shaders funciona en una instancia vanilla y escribe en `shaderpacks/`.
  - Si `--version-json`, `--client-jar`, `--libraries-dir` y `--assets-dir` se resuelven como `<raíz>/versions/<v>/<v>.json`, `<raíz>/versions/<v>/<v>.jar`, `<raíz>/libraries`, `<raíz>/assets`, siendo `<raíz>` el padre de `--assetsDir`.
  - Si el comando previo admite comillas para una ruta con espacios (`xmcl-runtime/launch/utils/cmd.ts`).
  - Si hace falta `#![windows_subsystem = "windows"]` y `CREATE_NO_WINDOW` para no abrir consolas.
  - Si XMCL carga una instancia creada a mano en `<raíz>/instances/<nombre>/instance.json` con solo `name`, `runtime.minecraft` y `prependCommand`.

- [ ] **Step 4: Decidir.** Si XMCL no puede arrancar el juego así, **parar y volver al usuario**. Si funciona, commit de `SPIKE.md` y el fixture con `docs(zenith): XMCL spike findings`.

### Task 6: Puente, argumentos

**Files:**
- Create: `zenith/bridge/Cargo.toml` (con su propio `[workspace]`), `zenith/bridge/src/args.rs`, `zenith/bridge/src/main.rs`, `zenith/bridge/tests/args.rs`

**Interfaces:**
- Produces:

```rust
pub struct LaunchArgs { pub java: PathBuf, pub version: String, pub username: String, pub uuid: String,
    pub access_token: String, pub xuid: Option<String>, pub game_dir: PathBuf, pub assets_dir: PathBuf,
    pub width: Option<u32>, pub height: Option<u32>, pub join: Option<String>, pub world: Option<String> }
impl LaunchArgs {
    pub fn parse(argv: &[String]) -> Result<Self, String>; // argv[0] es Java
    pub fn session_json(&self) -> String; // name, uuid, access_token, user_type "msa", xuid
    pub fn minecraft_root(&self) -> PathBuf; // padre de assets_dir
}
```
`Debug` se implementa a mano y enmascara `access_token`.

- [ ] **Step 1: Escribir los tests fallidos**

```rust
#[test] fn parses_real_xmcl_line() { /* fixture: version, username, uuid, game_dir, java */ }
#[test] fn minus_one_size_is_none() { /* --width -1 --height -1 => None */ }
#[test] fn server_and_port_become_join() { /* --server a.b --port 25566 => "a.b:25566" */ }
#[test] fn quickplay_multiplayer_becomes_join() {}
#[test] fn quickplay_singleplayer_becomes_world() {}
#[test] fn spaces_and_accents_in_paths() { /* C:\Users\José Ñ\... intactos */ }
#[test] fn missing_token_is_an_error() {}
#[test] fn debug_hides_token() { assert!(!format!("{a:?}").contains("SECRET")); }
#[test] fn session_json_has_required_fields() {}
```

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test --manifest-path zenith/bridge/Cargo.toml --test args`. Expected: FAIL.

- [ ] **Step 3: Implementar `LaunchArgs`.** Recorre los argumentos sin regex; ignora lo desconocido, porque XMCL pasa JVM args y classpath. Dependencias: `serde_json`, `ureq` con las features de la raíz, `zip = { version = "9.1.3", default-features = false, features = ["deflate"] }`, `horizon-release-sign = { path = "../../crates/horizon-release-sign" }`.

- [ ] **Step 4: Comprobar que pasan.** Run: el mismo. Expected: PASS.

- [ ] **Step 5: Añadir a `zenith-sync.yml`** `cargo test --manifest-path zenith/bridge/Cargo.toml --locked`. Commit con `feat(zenith): bridge reads XMCL's launch arguments`.

### Task 7: Puente, propiedad del juego

**Files:**
- Create: `zenith/bridge/src/owner.rs`, `zenith/bridge/tests/owner.rs`

**Interfaces:**
- Produces:

```rust
pub const MINECRAFT_API: &str = "https://api.minecraftservices.com";
pub enum OwnerCheck { Owner, NotOwner, Unreachable(String) }
pub fn check(base_url: &str, token: &str) -> OwnerCheck;
pub fn gate(result: OwnerCheck, uuid: &str, cache: &Path, now_secs: u64) -> Result<(), String>;
```

Reglas de `check`, sobre `GET {base}/entitlements/mcstore` con `Authorization: Bearer`, iguales a `launcher/src/account.rs:397`: 200 con un item `game_minecraft` o `product_minecraft` es `Owner`; 200 sin ellos, 401 o 403 es `NotOwner`; error de red, 429 o 5xx es `Unreachable`.

Reglas de `gate` (decisión de Claude): `Owner` guarda `uuid → now` en `owners.json`. `Unreachable` deja pasar si hay verificación de ese UUID con menos de 30 días; si no, error. `NotOwner` da error, con el mensaje "Inicia sesión en XMCL con una cuenta de Microsoft que posea Minecraft: Java Edition".

- [ ] **Step 1: Escribir los tests fallidos.** Con `TcpListener` local que responde una vez: `owner_on_game_minecraft`, `owner_on_product_minecraft`, `not_owner_on_empty_items`, `not_owner_on_401`, `unreachable_on_503`, `unreachable_when_refused`. Sin red: `gate_allows_owner_and_caches`, `gate_refuses_not_owner_even_with_cache`, `gate_unreachable_with_recent_cache_passes`, `gate_unreachable_with_old_cache_refuses`, `gate_unreachable_without_cache_refuses`.

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test --manifest-path zenith/bridge/Cargo.toml --test owner`. Expected: FAIL.

- [ ] **Step 3: Implementar** `check` y `gate`.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Commit** con `feat(zenith): bridge refuses accounts that don't own the game`.

### Task 8: Puente, builds firmadas

**Files:**
- Create: `zenith/bridge/src/builds.rs`, `zenith/bridge/tests/builds.rs`

**Interfaces:**
- Consumes: `horizon_release_sign::{nightly_digest, parse_keys, check, sha256_hex, PublicKey}`, `zenith-release-keys.txt` con `include_str!`.
- Produces:

```rust
pub const REPO: &str = "gitzetazero/zenith";
pub fn asset_name() -> &'static str; // horizon-windows-x64.zip, horizon-linux-x86_64.tar.gz
pub struct Store { pub root: PathBuf } // %APPDATA%\zenith o ZENITH_HOME
impl Store {
    pub fn current(&self) -> Option<PathBuf>;
    pub fn pinned(&self) -> Option<String>;
    pub fn set_pinned(&self, version: Option<&str>) -> Result<(), String>;
    pub fn install(&self, version: &str, archive: &[u8], sums: &[u8], signature: &str,
                   keys: &[PublicKey]) -> Result<PathBuf, String>;
}
pub fn ensure_latest(store: &Store, keys: &[PublicKey],
                     fetch: impl Fn(&str) -> Result<Option<Vec<u8>>, String>) -> Result<PathBuf, String>;
```

`ensure_latest` lee el título de la Release `nightly` en la API de GitHub. Si coincide con la versión instalada o hay versión fijada, no descarga. Si falla la red o la verificación y hay `current`, avisa en el log y devuelve `current`; sin `current`, error. `install` verifica antes de extraer, extrae a `builds/<v>.tmp`, renombra, escribe `current.txt` con archivo temporal y `rename`, y conserva solo la actual y la anterior.

- [ ] **Step 1: Escribir los tests fallidos** con un `SigningKey::generate()` de `horizon-release-sign`: `installs_signed_archive`, `bad_signature_keeps_current`, `tampered_archive_keeps_current`, `signature_for_other_channel_refused`, `zip_slip_refused`, `keeps_only_two_builds`, `network_error_returns_current`, `network_error_without_build_is_error`, `same_version_does_not_download`, `pinned_version_does_not_update`.

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test --manifest-path zenith/bridge/Cargo.toml --test builds`. Expected: FAIL.

- [ ] **Step 3: Implementar.** Extraer con `enclosed_name()` del crate `zip`. En Linux, `flate2` y `tar`, solo bajo `cfg(unix)`.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Commit** con `feat(zenith): bridge installs signed builds`.

### Task 9: Puente, preparar y arrancar

**Files:**
- Create: `zenith/bridge/src/prepare.rs`, `zenith/bridge/src/play.rs`, `zenith/bridge/tests/play.rs`
- Modify: `zenith/bridge/src/main.rs`, `.github/workflows/zenith-release.yml`, `.github/workflows/zenith-sync.yml`, y crear `zenith/ci/check_launch_api.py`

**Interfaces:**
- Consumes: `LaunchArgs`, `owner::{check, gate}`, `builds::{Store, ensure_latest}`.
- Produces:

```rust
pub fn pinned_minecraft(build: &Path) -> Result<String, String>; // horizon-prepare --launch-api
pub fn check_cmd(build: &Path, prepared: &Path) -> Command;
pub fn prepare_cmd(build: &Path, a: &LaunchArgs, prepared: &Path, server_jar: &Path) -> Command;
pub fn ensure_server_jar(version_json: &Path, dest: &Path,
                         fetch: impl Fn(&str) -> Result<Option<Vec<u8>>, String>) -> Result<(), String>;
pub fn client_cmd(build: &Path, a: &LaunchArgs, prepared: &Path) -> Command;
pub fn run_game(cmd: Command, session_json: &str) -> std::io::Result<i32>;
```

- [ ] **Step 1: Escribir los tests fallidos** sobre las órdenes, sin ejecutar nada:
  - `client_cmd_never_has_token`: ningún argumento ni variable contiene el token.
  - `client_cmd_args`: `--resources P --game-dir G --home G --session-stdin`, con `--width`, `--height`, `--quickPlayMultiplayer` y `--quickPlaySingleplayer` solo si existen.
  - `client_cmd_env`: `HORIZON_TELEMETRY_URL=off`, `HORIZON_NO_HEARTBEAT=1`, `HORIZON_COSMETICS=0`, `HORIZON_HOME=G`.
  - `prepare_cmd_args`: `--prepare --server-jar J --version-json … --client-jar … --libraries-dir … --assets-dir … --java-home <padre de bin> --output P --complete`.
  - `other_version_refused`: instancia 1.21.8 con build de 26.3 da "Esta instancia debe ser Minecraft 26.3".
  - `crash_line_translated`: `Horizon: crash report saved to C:\x y\c.txt` da `Crash report saved to: #@!@# C:\x y\c.txt`.

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test --manifest-path zenith/bridge/Cargo.toml --test play`. Expected: FAIL.

- [ ] **Step 3: Implementar.**
  - `prepared` es `<home>/prepared/<versión de Minecraft>`, una por versión; `--check` decide si hay que preparar.
  - La preparación se abre en una ventana de consola propia en Windows (`CREATE_NEW_CONSOLE`), en texto normal, para que se vea el progreso.
  - `ensure_server_jar` lee `downloads.server.url` del JSON de versión y solo se llama tras pasar `gate`.
  - `run_game` escribe la sesión por stdin y la cierra, imprime `LWJGL Version: zenith` en stdout, reenvía stdout y stderr, traduce la línea de fallo y devuelve el código de salida.
  - `main` encadena: `parse` → `check` + `gate` → `ensure_latest` → `pinned_minecraft` y comparación de versión → preparar → `run_game`. Los errores van a stderr y salida 1.
  - Aplicar lo decidido en `SPIKE.md` sobre ventanas de consola. Log en `<home>/logs/bridge.log`, sin token.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Añadir el aviso temprano y la publicación.**
  - `check_launch_api.py <horizon-client>` falla si `launch_api` no empieza por `0.1` o falta alguna de `--resources --game-dir --home --width --height --session-stdin --quickPlayMultiplayer --quickPlaySingleplayer`. Se ejecuta en `zenith-sync.yml`, tras los tests.
  - `zenith-release.yml` también compila el puente y publica `zenith-bridge-windows-x64.exe` y `zenith-bridge-linux-x86_64`, incluidos en `SHA256SUMS`.

- [ ] **Step 6: Commit** con `feat(zenith): bridge prepares and starts the game`.

### Task 10: Puesta en marcha y prueba completa en Windows

**Files:**
- Create: `zenith/bridge/src/setup.rs`, `zenith/bridge/tests/setup.rs`, `zenith/README.md`
- Modify: `zenith/bridge/src/main.rs`, `zenith/bridge/src/builds.rs`

**Interfaces:**
- Produces: `pub fn write_instance(game_data: &Path, exe: &Path, minecraft: &str) -> Result<PathBuf, String>`; órdenes `zenith-bridge setup`, `zenith-bridge use <versión|latest>`.

- [ ] **Step 1: Escribir los tests fallidos**: `writes_instance_json` (`name` "Zenith", `runtime.minecraft`, `prependCommand`), `quotes_path_with_spaces` según `SPIKE.md`, `never_overwrites_existing_instance`.

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test --manifest-path zenith/bridge/Cargo.toml --test setup`. Expected: FAIL.

- [ ] **Step 3: Implementar.** `setup` copia el puente a `<home>/zenith-bridge.exe`, lee la raíz de datos de `%APPDATA%\xmcl\root`, instala la build, y escribe `<raíz>/instances/Zenith/instance.json`. El puente se actualiza a sí mismo al arrancar: renombra el ejecutable en uso a `.old` y deja el nuevo verificado en su lugar, con efecto en el siguiente arranque. `use` fija o libera la versión para volver atrás.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Escribir `zenith/README.md`**: instalar XMCL, descargar el puente, `zenith-bridge setup`, reiniciar XMCL, apagar la telemetría de XMCL si se quiere, y los límites del spec (mods Java, funciones que se pierden).

- [ ] **Step 6: Prueba completa con el usuario, en un perfil limpio de XMCL.** Antes, guardar la lista de archivos con tamaño y fecha de `%APPDATA%\horizon` y `%APPDATA%\horizon-launcher`. Comprobar:
  - Jugar abre Zenith en un servidor online.
  - Una cuenta offline se rechaza con mensaje claro.
  - Alterar `zenith-release-keys.txt` en una build de prueba hace que se rechace la descarga.
  - Tras 10 minutos de juego, `ipconfig /displaydns` no contiene `playhorizon.dev`, y las conexiones de `horizon-client` (`Get-NetTCPConnection`) van solo a destinos de la lista blanca.
  - Las listas de archivos de Horizon son idénticas.

- [ ] **Step 7: Commit** con `feat(zenith): bridge setup and player guide`.

**Fase 2 terminada cuando:** el paso 6 pasa entero.

---

# Fase 3: entrar a los servidores

### Task 11: Elegir versión y reintentar

**Files:**
- Create: `crates/horizon-client/src/zenith_versions.rs`
- Modify: el `lib.rs` del crate (`pub mod`), `crates/horizon-client/src/net.rs:216,564,570-578,603,644-648,686`, `crates/horizon-client/src/multiplayer.rs:261-268`, `crates/horizon-client/src/game.rs:21870` (llamar a `init`)

**Interfaces:**
- Consumes: `ServerVersion::{ALL, name, from_protocol, protocol}`.
- Produces:

```rust
pub fn init(player_dir: &Path); // zenith-servers.json junto a servers.dat
pub fn range_upper(text: &str) -> Option<(u32, u32, u32)>;
pub fn retry_version(text: &str) -> Option<ServerVersion>;
pub fn override_for(address: &str) -> Option<ServerVersion>;
pub fn set_override(address: &str, version: Option<ServerVersion>) -> std::io::Result<()>;
pub fn choose(address: &str, forced: Option<ServerVersion>, ping: impl FnOnce() -> Option<i32>) -> Option<ServerVersion>;
pub fn with_version(text: String, version: Option<ServerVersion>) -> String; // "… (26.2)"
```

`choose` sigue este orden: `forced`; `override_for`; `ServerVersion::from_protocol(ping())`. `Current` se devuelve como `None`. No se hace ningún ping si hay `forced` o `override`.

- [ ] **Step 1: Escribir los tests fallidos**

```rust
#[test] fn screenshot_message() { assert_eq!(range_upper("¡Cliente no compatible! Por favor, usa 1.7.2-26.2"), Some((26, 2, 0))); }
#[test] fn english_and_en_dash() { assert_eq!(range_upper("Please use 1.8 – 1.21.8"), Some((1, 21, 8))); }
#[test] fn date_is_not_a_range() { assert_eq!(range_upper("Baneado hasta 2026-11-05"), None); }
#[test] fn lone_decimal_is_not_a_range() { assert_eq!(range_upper("Reinicio en 1.5 minutos"), None); }
#[test] fn lone_version_is_not_a_range() { assert_eq!(range_upper("Usa 1.21.4"), None); }
#[test] fn retries_with_newest_inside_range() { assert_eq!(retry_version("usa 1.7.2-26.2"), Some(ServerVersion::V26_2)); }
#[test] fn current_in_range_gives_next_older() { assert_eq!(retry_version("Velocity 1.7.2-26.3"), Some(ServerVersion::V26_2)); }
#[test] fn range_too_old_gives_none() { assert_eq!(retry_version("usa 1.7.2-1.12.2"), None); }
#[test] fn forced_skips_ping() { /* ping que hace panic si se llama */ }
#[test] fn override_skips_ping() {}
#[test] fn ping_of_current_is_none() {}
#[test] fn unknown_ping_is_none() {}
#[test] fn override_survives_restart() { /* init en carpeta temporal, set, leer archivo */ }
#[test] fn corrupt_overrides_file_is_ignored() {}
```

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test -p horizon-client zenith_versions`. Expected: FAIL.

- [ ] **Step 3: Implementar el módulo.** Sin regex: buscar `versión separador versión`, donde versión exige al menos un punto y el separador es `-` o `–` con espacios opcionales. `retry_version` elige en `ALL` la más nueva, estrictamente anterior a `Current`, que no supere el límite. Versiones comparadas como tuplas numéricas a partir de `name()`.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Enganches, cada uno con `// zenith:`**
  - `net.rs:564`: activo salvo `HORIZON_OLDER_SERVERS=0`.
  - `net.rs:570`: `older_server` delega en `choose`.
  - `net.rs:603` y `216`: `run` recibe `forced: Option<ServerVersion>`; la llamada pasa `None`.
  - `net.rs:686`: si `forced` es `None`, la conexión no era local ni de amigo, y `retry_version(&reason.text)` da una versión, `return run(None, …, Some(v))`. Es un único reintento, porque `forced` ya está fijado.
  - Cierre `e` de `net.rs:650`: `Reason::plain(with_version(connection_lost(err, lang), adapted))`.
  - `multiplayer.rs:261`: también es unible si `override_for(&address)` existe o `retry_version(&version)` da una versión.

- [ ] **Step 6: Comprobar.** Run: `cargo test -p horizon-client -- net:: multiplayer:: zenith_versions`. Expected: PASS. Si un test de Horizon asumía el traductor apagado, ajustarlo y anotarlo en el mensaje de commit.

- [ ] **Step 7: Commit** con `feat(zenith): choose the server's version and retry once on a named range`.

### Task 12: Selector y mensajes visibles

**Files:**
- Create: `crates/horizon-client/assets/zenith/lang/en_us.json`, `crates/horizon-client/assets/zenith/lang/es_es.json`
- Modify: `crates/horizon-client-core/src/zenith_brand.rs` (`apply` añade estas claves), `crates/horizon-client/src/multiplayer.rs:102-110,782-790,1523,1539,1658-1677`, `crates/horizon-client/src/net.rs:644-648`

**Interfaces:**
- Consumes: `override_for`, `set_override`, `ServerVersion::ALL`.
- Produces: claves `zenith.multiplayer.version` ("Versión: %s"), `zenith.multiplayer.version.auto` ("Automática"), `zenith.multiplayer.version.experimental` ("%s (experimental)"), `zenith.multiplayer.joining_as` ("Entrando como %s").

- [ ] **Step 1: Escribir los tests fallidos**: `cycle_goes_auto_then_newest_to_oldest_then_auto`; `label_marks_below_1_21_8_experimental`, que da "1.21.4 (experimental)" y "1.21.8" sin marca; `apply_merges_keys_in_spanish_and_falls_back_to_english`.

- [ ] **Step 2: Comprobar que fallan.** Run: `cargo test -p horizon-client -p horizon-client-core -- zenith_versions zenith_brand`. Expected: FAIL.

- [ ] **Step 3: Implementar.** `next_version(Option<ServerVersion>) -> Option<ServerVersion>` y `version_label(Option<ServerVersion>) -> String` en `zenith_versions.rs`. Campo `version: Option<ServerVersion>` en `Screen::Edit`. Botón con id `61` en `quarter(h) + 93.0`, entre "Server Resource Packs" y "Done", con el mismo estilo. Se carga en `1539`, cambia en `1658` y se guarda con `set_override` junto a `accept_textures`. Tras `c.adapt(version)` en `net.rs:647`, `status(send, …joining_as…)`.

- [ ] **Step 4: Comprobar que pasan.** Expected: PASS.

- [ ] **Step 5: Mirar el botón en el juego.** Ejecutar el cliente con carpetas temporales, abrir Editar servidor y hacer una captura. Expected: cabe sin solaparse a 1280x720 y con la escala de interfaz más alta que el juego permita a esa resolución.

- [ ] **Step 6: Commit** con `feat(zenith): per-server version selector`.

### Task 13: Integrar 1.20.2 a 1.21.8

**Files:**
- Modify: lo que traiga [horizon-rust/horizon#8718](https://github.com/horizon-rust/horizon/pull/8718), sobre todo `crates/horizon-protocol/src/adapter*`; `ZENITH.md`

**Interfaces:**
- Produces: nuevas variantes en `ServerVersion`, que el selector ve por `ALL` sin más cambios.

- [ ] **Step 1: Comprobar el estado.** Run: `gh pr view 8718 --repo horizon-rust/horizon --json state,isDraft,mergeable,headRefOid`. Si ya está integrado en una nightly, saltar al paso 4.

- [ ] **Step 2: Fusionar.**

```bash
git fetch https://github.com/TheApple05/horizon.git feature/older-servers-1.21.8
git merge --no-ff FETCH_HEAD
```
Resolver los conflictos conservando el comportamiento de `main` fuera del traductor y de los enganches `// zenith:`.

- [ ] **Step 3: Comprobar.** Run: `cargo test -p horizon-protocol && cargo test -p horizon-client zenith_versions`. Expected: PASS, y `retry_version("usa 1.7.2-1.21.8")` da la variante de 1.21.7/1.21.8. Añadir ese test.

- [ ] **Step 4: Anotar en `ZENITH.md`** el commit integrado y cómo volver a traer la rama.

- [ ] **Step 5: Commit** con `feat(zenith): older-server adapters down to 1.20.2 (Horizon #8718)`.

### Task 14: Matriz de versiones

**El paso 2 necesita permiso expreso del usuario:** arrancar servidores vanilla exige aceptar el EULA de Minecraft (`eula=true`), y esa aceptación es suya.

**Files:**
- Create: `zenith/tools/matrix.py`, `zenith/tools/test_matrix.py`, `zenith/tools/matrix-results.md`

**Interfaces:**
- Produces: `python3 zenith/tools/matrix.py --client <exe> --ctl <exe> --resources <dir> --java 17=<ruta> --java 21=<ruta> --java 25=<ruta> [--only 1.21.8] --accept-eula`. Sale con 1 si falla algo de 1.21.8 a 26.3. Sin `--accept-eula` no arranca nada.

- [ ] **Step 1: Escribir los tests fallidos**: `required_versions_are_1_21_8_and_up`, `best_effort_failure_exits_zero`, `required_failure_exits_one`, `refuses_without_accept_eula`.

- [ ] **Step 2: Implementar.** Versiones: 1.20.2, 1.20.4, 1.20.6, 1.21.1, 1.21.3, 1.21.4, 1.21.5, 1.21.6, 1.21.8, 1.21.10, 1.21.11, 26.1.2, 26.2 y 26.3. Jars de `downloads.server.url` en `.local/matrix/`, que no se sube. Cada servidor en carpeta temporal y puerto propio, con `online-mode=false`. Cliente con `--quickPlayMultiplayer`, carpetas temporales y `HORIZON_NO_AUDIO=1`. Con `horizon-ctl` (ver `docs/horizon-ctl-commands.md`): entrar, moverse, romper y poner un bloque, chat, inventario, cambiar de dimensión, morir y reaparecer. Escribe la tabla en `matrix-results.md`.

- [ ] **Step 3: Comprobar los tests.** Run: `python3 -m unittest zenith/tools/test_matrix.py`. Expected: OK.

- [ ] **Step 4: Con el permiso del usuario, ejecutar la matriz.** Expected: pasan de 1.21.8 a 26.3. Lo que falle de 1.20.2 a 1.21.7 queda listado como problema conocido en `ZENITH.md`. Un fallo obligatorio se arregla antes de seguir, buscando primero en #8718 y en Horizon.

- [ ] **Step 5: Commit** con `test(zenith): version matrix against vanilla servers`.

### Task 15: `<servidor de prueba>` de verdad

Solo el usuario puede hacerlo, con su cuenta. Antes conviene que revise las normas del servidor sobre clientes no oficiales.

- [ ] **Step 1:** Publicar la build con todo lo anterior. Expected: la instancia Zenith de XMCL se actualiza sola.

- [ ] **Step 2: Con el usuario, sin configurar nada:** entrar a `<servidor de prueba>`. Expected: entra. Si entra como 26.3, el traductor no se ha usado.

- [ ] **Step 3:** Forzar 26.2 con el selector y volver a entrar, para ejercitar el traductor. Expected: entra.

- [ ] **Step 4:** Jugar 30 minutos. Expected: sin expulsiones. Una expulsión por anticheat se investiga; si viene de ser 26.3 por dentro y no se puede arreglar, se documenta en `ZENITH.md` y decide el usuario.

**Fase 3 terminada cuando:** la tarea 14 pasa en la parte obligatoria y la 15 pasa entera.

---

## Pasos que solo puede hacer el usuario

| Antes de | Qué | Cómo |
| --- | --- | --- |
| Tarea 2 | Crear `ZENITH_SYNC_TOKEN` | Token de GitHub con permisos de contenido y workflows; guardarlo como secreto, sin pegarlo en el chat |
| Tarea 3 | Generar la clave de firma | La orden al inicio de la tarea 3 |
| Tarea 5 | Instalar XMCL e iniciar sesión | [xmcl.app](https://xmcl.app/en/) |
| Tarea 14 | Aceptar el EULA para los servidores de prueba | Decirlo en el chat |
| Tarea 15 | Jugar en `<servidor de prueba>` | Revisando antes sus normas |
| Fase 6 (no bloquea este plan) | App de Azure y aprobación de Mojang | Conviene empezar ya, porque tarda |
