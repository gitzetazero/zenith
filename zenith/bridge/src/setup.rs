//! `zenith-bridge setup`: installs the bridge as the "Java" of an XMCL instance called Zenith.
//! The instance's `java` points at `<home>\java\bin\java.exe` (SPIKE.md section 10): no cmd.exe, no length limit.

use crate::builds::Store;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const FALLBACK_MINECRAFT: &str = "26.3";

pub struct Env {
    /// `%APPDATA%`: XMCL keeps `xmcl\root` here.
    pub appdata: PathBuf,
    /// Zenith's home (`%APPDATA%\zenith` or `ZENITH_HOME`).
    pub home: PathBuf,
    pub xmcl_running: fn() -> bool,
}

impl Env {
    pub fn from_env(home: &Path) -> Env {
        Env {
            appdata: std::env::var_os("APPDATA").unwrap_or_default().into(),
            home: home.to_path_buf(),
            xmcl_running,
        }
    }

    pub fn java_exe(&self) -> PathBuf {
        self.home.join("java").join("bin").join("java.exe")
    }
}

fn xmcl_running() -> bool {
    let mut cmd = std::process::Command::new("tasklist");
    crate::play::hide_console(&mut cmd);
    cmd.output().is_ok_and(|o| {
        let t = String::from_utf8_lossy(&o.stdout).to_lowercase();
        t.contains("xmcl") || t.contains("x minecraft launcher")
    })
}

/// XMCL's data root as stored in `%APPDATA%\xmcl\root` (trimmed, like XMCL does), for display.
pub fn xmcl_root(appdata: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(appdata.join("xmcl").join("root")).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| PathBuf::from(text))
}

/// The root for `setup`. Only a missing file means "XMCL never ran"; anything odd is refused untouched.
fn strict_root(appdata: &Path) -> Result<PathBuf, String> {
    let file = appdata.join("xmcl").join("root");
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err("XMCL aún no está inicializado. Abre XMCL una vez, elige la carpeta de datos y vuelve a ejecutar «zenith-bridge setup». No he creado nada.".into());
        }
        Err(e) => {
            return Err(format!(
                "No puedo leer {} ({e}). No he cambiado nada.",
                file.display()
            ));
        }
    };
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("{} no es texto UTF-8. No he cambiado nada.", file.display()))?;
    let text = text.trim();
    if text.is_empty() {
        return Err(format!(
            "{} está vacío. Abre XMCL, elige la carpeta de datos y vuelve a ejecutar «setup». No he cambiado nada.",
            file.display()
        ));
    }
    let root = PathBuf::from(text);
    if !root.is_dir() {
        return Err(format!(
            "La carpeta de datos de XMCL ({}) no existe. Ábrela en XMCL o corrígela y vuelve a ejecutar «setup». No he cambiado nada.",
            root.display()
        ));
    }
    Ok(root)
}

/// Writes (or updates) `<root>/instances/Zenith/instance.json`; true if it changed. An existing
/// file keeps its other settings. `instance.json.bak` is written only if there is none yet.
pub fn write_instance(
    root: &Path,
    java_exe: &Path,
    minecraft: &str,
) -> Result<(PathBuf, bool), String> {
    let dir = root.join("instances").join("Zenith");
    let file = dir.join("instance.json");
    let old = std::fs::read_to_string(&file).ok();
    let parsed = old
        .as_deref()
        .and_then(|t| serde_json::from_str::<Value>(t).ok())
        .filter(Value::is_object);
    let mut v = parsed.clone().unwrap_or_else(|| json!({}));
    v["name"] = "Zenith".into();
    if !v["runtime"].is_object() {
        v["runtime"] = json!({});
    }
    v["runtime"]["minecraft"] = minecraft.into();
    v["java"] = java_exe.to_string_lossy().as_ref().into();
    v["prependCommand"] = "".into(); // an explicit empty string also overrides XMCL's global one
    if parsed.as_ref() == Some(&v) {
        return Ok((file, false));
    }
    let new = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let bak = dir.join("instance.json.bak");
    if old.is_some() && !bak.exists() {
        std::fs::copy(&file, &bak).map_err(|e| e.to_string())?;
    }
    std::fs::write(&file, new).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok((file, true))
}

/// Copies `src` to `dst` unless identical (atomically: temp file + rename); true if it copied.
fn install_copy(src: &[u8], dst: &Path) -> Result<bool, String> {
    if std::fs::read(dst).is_ok_and(|d| d == src) {
        return Ok(false);
    }
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let tmp = dst.with_extension("exe.new");
    std::fs::write(&tmp, src).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, dst).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!(
            "No puedo reemplazar {} ({e}): cierra el juego y vuelve a ejecutar setup. La copia anterior sigue en su sitio.",
            dst.display()
        )
    })?;
    Ok(true)
}

/// Returns the lines to show the player. `minecraft` runs only once the root is accepted
/// and gives the version to pin plus an optional note (offline fallback).
pub fn setup(
    env: &Env,
    exe: &Path,
    minecraft: impl FnOnce() -> (String, Option<String>),
) -> Result<Vec<String>, String> {
    let root = strict_root(&env.appdata)?;
    let mut out = vec![format!("Carpeta de datos de XMCL: {}", root.display())];

    let bytes = std::fs::read(exe).map_err(|e| format!("{}: {e}", exe.display()))?;
    let java = env.java_exe();
    let javaw = java.with_file_name("javaw.exe");
    // two independent copies, not links: XMCL tells files apart by inode
    let a = install_copy(&bytes, &java)?;
    let b = install_copy(&bytes, &javaw)?;
    out.push(if a || b {
        format!("«Java» de Zenith instalado en {}.", java.display())
    } else {
        "El «Java» de Zenith ya estaba instalado.".into()
    });

    let (version, note) = minecraft();
    out.extend(note);
    let (file, changed) = write_instance(&root, &java, &version)?;
    out.push(if changed {
        format!(
            "Instancia «Zenith» (Minecraft {version}) escrita en {}. Si ya existía, la primera versión quedó como instance.json.bak.",
            file.display()
        )
    } else {
        "La instancia «Zenith» ya estaba al día: no he cambiado nada.".into()
    });
    if (env.xmcl_running)() {
        out.push("XMCL está abierto: cierra XMCL del todo y ábrelo de nuevo; la instancia se aplica al reiniciarlo.".into());
    }
    out.push(String::new());
    out.push("Siguientes pasos:".into());
    out.push("1. Abre XMCL e inicia sesión con tu cuenta de Microsoft (debe tener Minecraft: Java Edition).".into());
    out.push("2. Elige la instancia «Zenith» y pulsa Jugar.".into());
    out.push(format!("3. La primera vez XMCL descarga Minecraft {version} (es normal; el Java que verá en la lista es el de Zenith) y Zenith prepara sus recursos y, si no hay un Java 25 instalado, descarga el de Mojang (de unos 30 segundos a unos minutos)."));
    Ok(out)
}

/// Lines for `zenith-bridge status`. `java` is an already-installed Java 25 for the preparer, if any.
pub fn status(env: &Env, java: Option<PathBuf>) -> Vec<String> {
    let store = Store {
        root: env.home.clone(),
    };
    let root = xmcl_root(&env.appdata);
    let yes = |b: bool| if b { "sí" } else { "no" };
    let mut out = vec![
        format!(
            "Carpeta de datos de XMCL: {}",
            root.as_ref()
                .map_or("no encontrada (XMCL no se ha abierto)".into(), |r| r
                    .display()
                    .to_string())
        ),
        format!(
            "Instancia Zenith: {}",
            yes(root
                .as_ref()
                .is_some_and(|r| r.join("instances/Zenith/instance.json").is_file()))
        ),
        format!(
            "«Java» de Zenith instalado: {}",
            yes(env.java_exe().is_file() && env.java_exe().with_file_name("javaw.exe").is_file())
        ),
        format!(
            "Java 25 para preparar recursos: {}",
            java.map_or(
                "ninguno (se descargará el de Mojang al jugar)".into(),
                |j| j.display().to_string()
            )
        ),
        format!(
            "Build instalada: {}",
            store
                .current()
                .and_then(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "ninguna".into())
        ),
        format!(
            "Versión fijada: {}",
            store
                .pinned()
                .unwrap_or_else(|| "ninguna (se actualiza sola)".into())
        ),
        format!(
            "Caché de cuenta (owners.json): {}",
            yes(env.home.join("owners.json").is_file())
        ),
        "Últimas líneas de bridge.log:".into(),
    ];
    let log = std::fs::read_to_string(env.home.join("logs").join("bridge.log")).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    for l in &lines[lines.len().saturating_sub(10)..] {
        let low = l.to_lowercase();
        out.push(if low.contains("token") || low.contains("bearer") {
            "  [línea omitida: podría contener un secreto]".into()
        } else {
            format!("  {l}")
        });
    }
    out
}

/// `zenith-bridge use <version|latest>`.
pub fn use_version(home: &Path, version: &str) -> Result<String, String> {
    let store = Store {
        root: home.to_path_buf(),
    };
    if version == "latest" {
        store.set_pinned(None)?;
        return Ok("Zenith vuelve a actualizarse solo a la última build.".into());
    }
    if !store.builds().join(version).is_dir() {
        let mut have: Vec<String> = std::fs::read_dir(store.builds())
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        have.sort();
        return Err(format!(
            "La versión {version} no está instalada. Instaladas: {}.",
            if have.is_empty() {
                "ninguna".into()
            } else {
                have.join(", ")
            }
        ));
    }
    store.set_pinned(Some(version))?;
    Ok(format!(
        "Zenith queda fijado en {version}. «zenith-bridge use latest» lo libera."
    ))
}

pub const HELP: &str = "zenith-bridge: Zenith (Horizon) para XMCL\n\n  setup            instala el «Java» de Zenith y crea la instancia «Zenith» en XMCL\n  status           muestra el estado\n  use <versión>    fija una build instalada (para volver atrás)\n  use latest       vuelve a actualizar sola\n  help             esta ayuda\n\nXMCL lo ejecuta solo al pulsar Jugar.";
