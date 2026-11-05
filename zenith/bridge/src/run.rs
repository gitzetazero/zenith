//! The bridge's whole flow, with its outside world injected so tests can drive it.

use crate::args::LaunchArgs;
use crate::builds::{Store, ensure_latest};
use crate::owner::{self, OwnerCheck};
use crate::play::{Stream, client_cmd, hide_console, run_game};
use crate::prepare::{
    check_cmd, ensure_index_alias, ensure_server_jar, make_work, pinned_minecraft, prepare_cmd,
    run_prepare,
};
use horizon_release_sign::PublicKey;
use std::path::PathBuf;
use std::process::Stdio;

pub type Fetch = Box<dyn Fn(&str) -> Result<Option<Vec<u8>>, String>>;
pub type Emit = Box<dyn Fn(Stream, &str)>;

pub struct Deps {
    /// `%APPDATA%\zenith`: builds, prepared resources, owners cache, logs.
    pub home: PathBuf,
    pub keys: Vec<PublicKey>,
    /// Asks the Minecraft services whether the token's account owns the game.
    pub check_owner: Box<dyn Fn(&str) -> OwnerCheck>,
    pub fetch: Fetch,
    /// One line for the bridge's stdout or stderr (XMCL's game log).
    pub emit: Emit,
    /// A refusal or failure, for a dialog: a GUI-subsystem exe has no console to read.
    pub show_error: Box<dyn Fn(&str)>,
}

/// `argv` is XMCL's command line without the bridge's own path. Returns the exit code.
pub fn run(argv: &[String], d: &Deps) -> i32 {
    let store = Store {
        root: d.home.clone(),
    };
    match go(argv, d, &store) {
        Ok(code) => code,
        Err(e) => {
            store.log(&e);
            (d.emit)(Stream::Err, &format!("Zenith: {e}"));
            (d.show_error)(&e);
            1
        }
    }
}

fn go(argv: &[String], d: &Deps, store: &Store) -> Result<i32, String> {
    let a = LaunchArgs::parse(argv)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_secs());
    owner::gate(
        (d.check_owner)(&a.access_token),
        &a.uuid,
        &d.home.join("owners.json"),
        now,
    )?;

    let build = ensure_latest(store, &d.keys, |u| (d.fetch)(u))?;
    let pinned = pinned_minecraft(&build)?;
    if pinned != a.version {
        return Err(format!(
            "Esta instancia debe ser Minecraft {pinned} (la de XMCL es {}).",
            a.version
        ));
    }

    let prepared = d.home.join("prepared").join(&pinned);
    let mut check = check_cmd(&build, &prepared);
    let ready = hide_console(&mut check)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ready {
        (d.emit)(
            Stream::Out,
            "Zenith: preparando los recursos de Horizon (la primera vez tarda unos minutos).",
        );
        let mc = a.minecraft_root();
        let version_json = mc
            .join("versions")
            .join(&pinned)
            .join(format!("{pinned}.json"));
        let jar = d.home.join("server-jars").join(format!("{pinned}.jar"));
        ensure_server_jar(&version_json, &jar, |u| (d.fetch)(u))?;
        ensure_index_alias(&version_json, &a.assets_dir)?;
        if a.java_home().is_empty() {
            return Err("XMCL passed a Java path without a folder; cannot prepare Horizon.".into());
        }
        let work = d
            .home
            .join("work")
            .join(build.file_name().unwrap_or_default());
        make_work(&build, &work).inspect_err(|_| {
            let _ = std::fs::remove_dir_all(&work);
        })?;
        let done = run_prepare(
            prepare_cmd(&build, &a, &prepared, &jar, &work),
            &mut |s, l| (d.emit)(s, l),
        );
        let _ = std::fs::remove_dir_all(&work); // ~400 MB of scratch
        done?;
    }

    run_game(
        client_cmd(&build, &a, &prepared),
        &a.session_json(),
        &mut |s, l| (d.emit)(s, l),
    )
    .map_err(|e| format!("no se pudo iniciar horizon-client: {e}"))
}
