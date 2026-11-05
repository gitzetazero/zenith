//! Preparing Horizon's resources from the Minecraft files XMCL already downloaded.

use crate::args::LaunchArgs;
use crate::play::{Stream, hide_console, pump, tool};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The Minecraft version the build's preparer is pinned to (`horizon-prepare --launch-api`).
pub fn pinned_minecraft(build: &Path) -> Result<String, String> {
    let mut cmd = Command::new(tool(build, "horizon-prepare"));
    let out = hide_console(cmd.arg("--launch-api"))
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("horizon-prepare: {e}"))?;
    if !out.status.success() {
        return Err("horizon-prepare --launch-api failed".into());
    }
    serde_json::from_slice::<Value>(&out.stdout)
        .ok()
        .and_then(|v| v["minecraft"].as_str().map(str::to_string))
        .ok_or_else(|| "horizon-prepare --launch-api names no Minecraft version".into())
}

pub fn check_cmd(build: &Path, prepared: &Path) -> Command {
    let mut c = Command::new(tool(build, "horizon-prepare"));
    c.arg("--check").arg("--output").arg(prepared);
    c
}

/// `work` is a throwaway Horizon folder (see [`make_work`]): the preparer writes about 400 MB of
/// scratch under its `--root`, and the installed build must stay untouched.
pub fn prepare_cmd(
    build: &Path,
    a: &LaunchArgs,
    prepared: &Path,
    server_jar: &Path,
    work: &Path,
) -> Command {
    let mc = a.minecraft_root();
    let v = &a.version;
    let mut c = Command::new(tool(build, "horizon-prepare"));
    c.arg("--prepare")
        .arg("--root")
        .arg(work)
        .arg("--server-jar")
        .arg(server_jar)
        .arg("--version-json")
        .arg(mc.join("versions").join(v).join(format!("{v}.json")))
        .arg("--client-jar")
        .arg(mc.join("versions").join(v).join(format!("{v}.jar")))
        .arg("--libraries-dir")
        .arg(mc.join("libraries"))
        .arg("--assets-dir")
        .arg(&a.assets_dir)
        .arg("--java-home")
        .arg(a.java_home())
        .arg("--output")
        .arg(prepared)
        .args(["--complete", "--progress", "json"]);
    c
}

fn sha1_hex(bytes: &[u8]) -> String {
    Sha1::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The server jar from the version JSON's `downloads.server`, checked against its sha1. Call it only
/// for an owner. A jar already at `dest` with the right hash is reused.
pub fn ensure_server_jar(
    version_json: &Path,
    dest: &Path,
    fetch: impl Fn(&str) -> Result<Option<Vec<u8>>, String>,
) -> Result<(), String> {
    let text =
        std::fs::read(version_json).map_err(|e| format!("{}: {e}", version_json.display()))?;
    let v: Value = serde_json::from_slice(&text).map_err(|e| e.to_string())?;
    let server = &v["downloads"]["server"];
    let (Some(url), Some(want)) = (server["url"].as_str(), server["sha1"].as_str()) else {
        return Err("the version JSON has no server download".into());
    };
    let want = want.to_lowercase();
    if std::fs::read(dest).is_ok_and(|b| sha1_hex(&b) == want) {
        return Ok(());
    }
    let bytes = fetch(url)?.ok_or("the server jar is not published")?;
    if sha1_hex(&bytes) != want {
        return Err("the downloaded server jar does not match its sha1".into());
    }
    let tmp = dest.with_extension("jar.tmp");
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, dest)
    };
    write().map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", dest.display())
    })
}

/// XMCL passes the asset index by its sha1 and may not leave `indexes/<id>.json` (`34.json`) that
/// the preparer looks for: copy it. Best effort; the preparer says what is missing.
pub fn ensure_index_alias(version_json: &Path, assets: &Path) {
    let Ok(text) = std::fs::read(version_json) else {
        return;
    };
    let Ok(v) = serde_json::from_slice::<Value>(&text) else {
        return;
    };
    let (Some(id), Some(sha1)) = (
        v["assetIndex"]["id"].as_str(),
        v["assetIndex"]["sha1"].as_str(),
    ) else {
        return;
    };
    let dir = assets.join("indexes");
    let alias = dir.join(format!("{id}.json"));
    if !alias.exists() {
        let _ = std::fs::copy(dir.join(format!("{sha1}.json")), alias);
    }
}

/// A Horizon folder for the preparer to work in: the build without its programs and `.local`
/// (about 15 MB of exporters, tools and the lock). Rebuilt each time.
pub fn make_work(build: &Path, work: &Path) -> Result<(), String> {
    fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(to)?;
        for e in std::fs::read_dir(from)? {
            let e = e?;
            let (src, dst) = (e.path(), to.join(e.file_name()));
            if e.file_type()?.is_dir() {
                copy(&src, &dst)?;
            } else {
                std::fs::copy(&src, &dst)?;
            }
        }
        Ok(())
    }
    let _ = std::fs::remove_dir_all(work);
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let skip: Vec<PathBuf> = ["horizon-client", "horizon-prepare", "horizon-ctl"]
        .iter()
        .map(|n| tool(Path::new(""), n))
        .collect();
    for e in std::fs::read_dir(build)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let name = PathBuf::from(e.file_name());
        if name == Path::new(".local") || skip.contains(&name) {
            continue;
        }
        let dst = work.join(&name);
        let done = if e.file_type().is_ok_and(|t| t.is_dir()) {
            copy(&e.path(), &dst)
        } else {
            std::fs::copy(e.path(), dst).map(|_| ())
        };
        done.map_err(|e| format!("{}: {e}", name.display()))?;
    }
    Ok(())
}

/// Runs the preparer, relaying its `--progress json` messages as plain lines.
pub fn run_prepare(mut cmd: Command, emit: &mut dyn FnMut(Stream, &str)) -> Result<(), String> {
    hide_console(&mut cmd);
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("horizon-prepare: {e}"))?;
    let mut failure = None;
    let code = pump(
        child,
        &mut |stream, line| match (stream, serde_json::from_str::<Value>(line)) {
            (Stream::Out, Ok(v)) => {
                if let Some(m) = v["message"].as_str() {
                    emit(Stream::Out, m);
                }
                if let Some(e) = v["error"].as_str() {
                    failure = Some(e.to_string());
                    emit(Stream::Err, e);
                }
            }
            _ => emit(stream, line),
        },
    )
    .map_err(|e| e.to_string())?;
    if code == 0 {
        Ok(())
    } else {
        Err(format!(
            "No se pudieron preparar los recursos de Horizon: {} (código {code}).",
            failure.as_deref().unwrap_or("mira el registro")
        ))
    }
}
