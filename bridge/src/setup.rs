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
            return Err("XMCL is not initialised yet. Open XMCL once, choose the data folder and run \"zenith-bridge setup\" again. Nothing was created.".into());
        }
        Err(e) => {
            return Err(format!(
                "Cannot read {} ({e}). Nothing was changed.",
                file.display()
            ));
        }
    };
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("{} is not UTF-8 text. Nothing was changed.", file.display()))?;
    let text = text.trim();
    if text.is_empty() {
        return Err(format!(
            "{} is empty. Open XMCL, choose the data folder and run \"setup\" again. Nothing was changed.",
            file.display()
        ));
    }
    let root = PathBuf::from(text);
    if !root.is_dir() {
        return Err(format!(
            "XMCL's data folder ({}) does not exist. Open it in XMCL or fix it and run \"setup\" again. Nothing was changed.",
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
            "Cannot replace {} ({e}): close the game and run setup again. The previous copy is still in place.",
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
    let mut out = vec![format!("XMCL data folder: {}", root.display())];

    let bytes = std::fs::read(exe).map_err(|e| format!("{}: {e}", exe.display()))?;
    let java = env.java_exe();
    let javaw = java.with_file_name("javaw.exe");
    // two independent copies, not links: XMCL tells files apart by inode
    let a = install_copy(&bytes, &java)?;
    let b = install_copy(&bytes, &javaw)?;
    out.push(if a || b {
        format!("Zenith's \"Java\" installed in {}.", java.display())
    } else {
        "Zenith's \"Java\" was already installed.".into()
    });

    let (version, note) = minecraft();
    out.extend(note);
    let (file, changed) = write_instance(&root, &java, &version)?;
    out.push(if changed {
        format!(
            "\"Zenith\" instance (Minecraft {version}) written to {}. If it already existed, the first version was kept as instance.json.bak.",
            file.display()
        )
    } else {
        "The \"Zenith\" instance was already up to date: nothing was changed.".into()
    });
    if (env.xmcl_running)() {
        out.push("XMCL is open: close XMCL completely and open it again; the instance is applied on restart.".into());
    }
    out.push(String::new());
    out.push("Next steps:".into());
    out.push("1. Open XMCL and sign in with your Microsoft account (it must own Minecraft: Java Edition).".into());
    out.push("2. Choose the \"Zenith\" instance and press Play.".into());
    out.push(format!("3. The first time XMCL downloads Minecraft {version} (this is normal; the Java it shows in the list is Zenith's) and Zenith prepares its resources and, if no Java 25 is installed, downloads Mojang's (from about 30 seconds to a few minutes)."));
    Ok(out)
}

pub const XMCL_EXE: &str = "XMCL.exe";

/// `zenith-bridge start`: what the "Zenith Launcher" shortcut runs. Prepares the instance quietly like `setup`
/// (no network: the Minecraft version comes from the installed build) and starts the XMCL that sits next to the
/// bridge. Without XMCL's `root` file it only starts XMCL, whose own first run lets the player choose the data
/// folder (we never write that file). A running XMCL is never started again, killed or touched.
/// Returns the lines to show; empty means "all done, nothing to say".
pub fn start(
    env: &Env,
    exe: &Path,
    minecraft: impl FnOnce() -> (String, Option<String>),
    spawn: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<Vec<String>, String> {
    let xmcl = exe.parent().unwrap_or(Path::new("")).join(XMCL_EXE);
    if !xmcl.is_file() {
        return Err(format!(
            "Cannot find {XMCL_EXE} next to zenith-bridge.exe ({}). Reinstall Zenith Launcher.",
            xmcl.display()
        ));
    }
    let running = (env.xmcl_running)();
    let first_run = matches!(
        std::fs::metadata(env.appdata.join("xmcl").join("root")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound
    );
    if first_run {
        if running {
            return Ok(vec!["XMCL is already open. Choose its data folder, close it completely and open \"Zenith Launcher\" again so the Zenith instance is created.".into()]);
        }
        spawn(&xmcl)?;
        return Ok(vec!["First time: XMCL will ask for the data folder. Choose it, close XMCL completely and open \"Zenith Launcher\" again: this creates the Zenith instance.".into()]);
    }
    setup(env, exe, minecraft)?;
    if running {
        return Ok(vec!["XMCL is already open: the Zenith instance is applied when it restarts. Close it completely and open \"Zenith Launcher\" again.".into()]);
    }
    spawn(&xmcl)?;
    Ok(vec![])
}

/// Lines for `zenith-bridge status`. `java` is an already-installed Java 25 for the preparer, if any.
pub fn status(env: &Env, java: Option<PathBuf>) -> Vec<String> {
    let store = Store {
        root: env.home.clone(),
    };
    let root = xmcl_root(&env.appdata);
    let yes = |b: bool| if b { "yes" } else { "no" };
    let mut out = vec![
        format!(
            "XMCL data folder: {}",
            root.as_ref()
                .map_or("not found (XMCL has never been opened)".into(), |r| r
                    .display()
                    .to_string())
        ),
        format!(
            "Zenith instance: {}",
            yes(root
                .as_ref()
                .is_some_and(|r| r.join("instances/Zenith/instance.json").is_file()))
        ),
        format!(
            "Zenith's \"Java\" installed: {}",
            yes(env.java_exe().is_file() && env.java_exe().with_file_name("javaw.exe").is_file())
        ),
        format!(
            "Java 25 for preparing resources: {}",
            java.map_or(
                "none (Mojang's will be downloaded when you play)".into(),
                |j| j.display().to_string()
            )
        ),
        format!(
            "Installed build: {}",
            store
                .current()
                .and_then(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "none".into())
        ),
        format!(
            "Pinned version: {}",
            store
                .pinned()
                .unwrap_or_else(|| "none (updates itself)".into())
        ),
        format!(
            "Account cache (owners.json): {}",
            yes(env.home.join("owners.json").is_file())
        ),
        "Last lines of bridge.log:".into(),
    ];
    let log = std::fs::read_to_string(env.home.join("logs").join("bridge.log")).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    for l in &lines[lines.len().saturating_sub(10)..] {
        let low = l.to_lowercase();
        out.push(if low.contains("token") || low.contains("bearer") {
            "  [line omitted: it might contain a secret]".into()
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
        return Ok("Zenith updates itself to the latest build again.".into());
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
            "Version {version} is not installed. Installed: {}.",
            if have.is_empty() {
                "none".into()
            } else {
                have.join(", ")
            }
        ));
    }
    store.set_pinned(Some(version))?;
    Ok(format!(
        "Zenith is pinned to {version}. \"zenith-bridge use latest\" releases it."
    ))
}

pub const HELP: &str = "zenith-bridge: Zenith (Horizon) for XMCL\n\n  start            prepares the \"Zenith\" instance and opens XMCL (what the shortcut does)
  setup            installs Zenith's \"Java\" and creates the \"Zenith\" instance in XMCL\n  status           shows the state\n  use <version>    pins an installed build (to go back)\n  use latest       updates itself again\n  help             this help\n\nXMCL runs it by itself when you press Play.";
