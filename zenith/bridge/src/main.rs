// A console exe loses its stdout under XMCL's `shell: true` + `detached: true` (SPIKE.md), so this is a GUI-subsystem exe.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;
use std::path::PathBuf;
use zenith_bridge::args::LaunchArgs;

fn home() -> PathBuf {
    if let Some(h) = std::env::var_os("ZENITH_HOME") {
        return h.into();
    }
    let base = std::env::var_os("APPDATA").unwrap_or_else(|| ".".into());
    PathBuf::from(base).join("zenith")
}

/// A GUI-subsystem exe has no console, so every error also goes to `<home>/logs/bridge.log`.
fn fail(msg: &str) -> ! {
    eprintln!("Zenith: {msg}");
    let logs = home().join("logs");
    let _ = std::fs::create_dir_all(&logs);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs.join("bridge.log"))
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let _ = writeln!(f, "[{secs}] {msg}");
    }
    std::process::exit(1);
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = LaunchArgs::parse(&argv) {
        fail(&e);
    }
}
