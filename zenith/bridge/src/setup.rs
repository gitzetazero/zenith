//! `zenith-bridge setup`: puts the bridge and a "Zenith" instance where XMCL will find them.

use crate::builds::Store;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const LIMIT: usize = 8191; // cmd.exe's command line limit (SPIKE.md section 2)
// Measured with Minecraft 26.3: 8122 chars at root 19 / bridge 70 => 6494 + 82 per root char + 1 per bridge char.
const BASE: usize = 6494;
const MARGIN: usize = 100; // other versions have more libraries; this errs on refusing
const BRIDGE_NAME: &str = "zenith-bridge.exe";
pub const FALLBACK_MINECRAFT: &str = "26.3";

/// Whether XMCL's launch line (run through cmd.exe because of the prepend command) fits.
pub fn command_fits(root_len: usize, bridge_path_len: usize) -> bool {
    BASE + 82 * root_len + bridge_path_len + MARGIN <= LIMIT
}

/// `<root>\zenith-bridge.exe` for a root of `n` characters.
fn root_fits(fits: fn(usize, usize) -> bool, root: &Path) -> bool {
    let n = root.to_string_lossy().chars().count();
    fits(n, n + 1 + BRIDGE_NAME.len())
}

fn max_root_len(fits: fn(usize, usize) -> bool) -> usize {
    (1..100)
        .take_while(|&r| fits(r, r + 1 + BRIDGE_NAME.len()))
        .last()
        .unwrap_or(0)
}

/// XMCL splits this field on spaces unless the path is in double quotes (SPIKE.md section 2).
pub fn quote_prepend(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

pub struct Env {
    /// `%APPDATA%`: XMCL keeps `xmcl\root` here.
    pub appdata: PathBuf,
    /// Where the data root goes when XMCL never ran (`C:\xmcl`).
    pub default_root: PathBuf,
    pub fits: fn(usize, usize) -> bool,
    pub xmcl_running: fn() -> bool,
}

impl Env {
    pub fn from_env() -> Env {
        Env {
            appdata: std::env::var_os("APPDATA").unwrap_or_default().into(),
            default_root: PathBuf::from(r"C:\xmcl"),
            fits: command_fits,
            xmcl_running,
        }
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

/// XMCL's data root as stored in `%APPDATA%\xmcl\root` (trimmed, like XMCL does).
pub fn xmcl_root(appdata: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(appdata.join("xmcl").join("root")).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| PathBuf::from(text))
}

/// Writes (or updates) `<root>/instances/Zenith/instance.json`; true if it changed.
/// An existing file keeps its other settings; its previous content is saved as `instance.json.bak`.
pub fn write_instance(root: &Path, exe: &Path, minecraft: &str) -> Result<(PathBuf, bool), String> {
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
    v["prependCommand"] = quote_prepend(exe).into();
    if parsed.as_ref() == Some(&v) {
        return Ok((file, false));
    }
    let new = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if old.is_some() {
        std::fs::copy(&file, dir.join("instance.json.bak")).map_err(|e| e.to_string())?;
    }
    std::fs::write(&file, new).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok((file, true))
}

/// Copies the running bridge to `dst` unless it is already identical; true if it copied.
fn install_bridge(src: &Path, dst: &Path) -> Result<bool, String> {
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    if std::fs::read(dst).is_ok_and(|d| d == bytes) {
        return Ok(false);
    }
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let tmp = dst.with_extension("exe.new");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, dst).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!(
            "no se pudo reemplazar {} (¿hay una partida abierta?): {e}",
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
    let mut out = Vec::new();
    let (root, created) = match xmcl_root(&env.appdata) {
        Some(root) => {
            if !root_fits(env.fits, &root) {
                return Err(format!(
                    "La carpeta de datos de XMCL ({}) es demasiado larga. Con un comando previo, XMCL lanza el juego a través de cmd.exe, que rechaza líneas de más de {LIMIT} caracteres, y Minecraft 26.3 necesita una carpeta de datos de como máximo {} caracteres (por ejemplo C:\\xmcl).\n\
                     No he cambiado nada. Qué hacer: en XMCL, Ajustes, cambia la carpeta de datos a una ruta corta y vuelve a ejecutar «zenith-bridge setup». Si no tienes datos de XMCL que conservar, borra el archivo %APPDATA%\\xmcl\\root y vuelve a ejecutar «setup».",
                    root.display(),
                    max_root_len(env.fits)
                ));
            }
            (root, false)
        }
        None => {
            let root = env.default_root.clone();
            if !root_fits(env.fits, &root) {
                return Err(format!("{} es demasiado larga para XMCL.", root.display()));
            }
            let conf = env.appdata.join("xmcl");
            std::fs::create_dir_all(&conf)
                .and_then(|_| std::fs::create_dir_all(&root))
                .and_then(|_| std::fs::write(conf.join("root"), root.to_string_lossy().as_bytes()))
                .map_err(|e| format!("no se pudo preparar la carpeta de datos de XMCL: {e}"))?;
            (root, true)
        }
    };
    out.push(if created {
        format!(
            "XMCL aún no se había abierto: he fijado su carpeta de datos en {}.",
            root.display()
        )
    } else {
        format!("Carpeta de datos de XMCL: {}", root.display())
    });

    let dst = root.join(BRIDGE_NAME);
    out.push(if install_bridge(exe, &dst)? {
        format!("Puente instalado en {}.", dst.display())
    } else {
        "El puente ya estaba instalado.".into()
    });

    let (version, note) = minecraft();
    out.extend(note);
    let (file, changed) = write_instance(&root, &dst, &version)?;
    out.push(if changed {
        format!(
            "Instancia «Zenith» (Minecraft {version}) escrita en {}. Si ya existía, la anterior quedó como instance.json.bak.",
            file.display()
        )
    } else {
        "La instancia «Zenith» ya estaba al día: no he cambiado nada.".into()
    });
    if (env.xmcl_running)() {
        out.push(
            "XMCL está abierto: ciérralo del todo y ábrelo de nuevo para que cargue la instancia."
                .into(),
        );
    }
    out.push(String::new());
    out.push("Siguientes pasos:".into());
    out.push("1. Abre XMCL e inicia sesión con tu cuenta de Microsoft (debe tener Minecraft: Java Edition).".into());
    out.push("2. Elige la instancia «Zenith» y pulsa Jugar.".into());
    out.push(format!("3. La primera vez XMCL descarga Minecraft {version} y Java (es normal) y Zenith prepara sus recursos (de unos 30 segundos a unos minutos)."));
    Ok(out)
}

/// Lines for `zenith-bridge status`.
pub fn status(env: &Env, home: &Path) -> Vec<String> {
    let store = Store {
        root: home.to_path_buf(),
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
            yes(home.join("owners.json").is_file())
        ),
        "Últimas líneas de bridge.log:".into(),
    ];
    let log = std::fs::read_to_string(home.join("logs").join("bridge.log")).unwrap_or_default();
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

pub const HELP: &str = "zenith-bridge: Zenith (Horizon) para XMCL\n\n  setup            instala el puente y crea la instancia «Zenith» en XMCL\n  status           muestra el estado\n  use <versión>    fija una build instalada (para volver atrás)\n  use latest       vuelve a actualizar sola\n  help             esta ayuda\n\nXMCL lo ejecuta solo al pulsar Jugar.";
