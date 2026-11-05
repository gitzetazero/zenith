// A console exe loses its stdout under XMCL's `shell: true` + `detached: true` (SPIKE.md), so this is a GUI-subsystem exe.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;
use std::path::PathBuf;
use zenith_bridge::args::{LaunchArgs, collect_args};

fn home() -> PathBuf {
    if let Some(h) = std::env::var_os("ZENITH_HOME") {
        return h.into();
    }
    let base = std::env::var_os("APPDATA").unwrap_or_else(|| ".".into());
    PathBuf::from(base).join("zenith")
}

/// A GUI-subsystem exe has no console, so every error also goes to `<home>/logs/bridge.log`.
fn log(msg: &str) {
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
}

fn fail(msg: &str) -> ! {
    eprintln!("Zenith: {msg}");
    log(msg);
    std::process::exit(1);
}

fn main() {
    // the panic message only: never the arguments, which carry the access token
    std::panic::set_hook(Box::new(|info| log(&format!("panic: {info}"))));
    let argv = collect_args(std::env::args_os().skip(1));
    if let Err(e) = LaunchArgs::parse(&argv) {
        fail(&e);
    }
}
