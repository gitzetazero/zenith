//! The real binary, copied as `java.exe` / `javaw.exe`, the way XMCL will see it.
#![cfg(windows)]
use std::path::PathBuf;
use std::process::Command;

fn shim(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("zenith-shim-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let p = d.join(name);
    std::fs::copy(env!("CARGO_BIN_EXE_zenith-bridge"), &p).unwrap();
    p
}

fn run(exe: &PathBuf, args: &[&str]) -> std::process::Output {
    let home = exe.parent().unwrap().join("home");
    Command::new(exe)
        .args(args)
        .env("ZENITH_HOME", home)
        .env("ZENITH_NO_DIALOG", "1")
        .output()
        .unwrap()
}

#[test]
fn version_probe_looks_like_openjdk_25() {
    for name in ["java.exe", "JAVAW.EXE"] {
        let exe = shim(name);
        let out = run(&exe, &["-version"]);
        assert_eq!(out.status.code(), Some(0));
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.starts_with("openjdk version \"25.0.5\" "), "{err}");
        assert_eq!(err.lines().count(), 3, "{err}");
        assert!(out.stdout.is_empty());
        // no files written
        assert!(!exe.parent().unwrap().join("home").exists());
    }
}

#[test]
fn jvm_info_probe() {
    let out = run(&shim("java.exe"), &["JavaProxy", "-jvm_info"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "{\"java_arch\":\"64\"}"
    );
}

#[test]
fn game_launch_of_another_main_class_is_refused() {
    let exe = shim("javaw.exe");
    let out = run(
        &exe,
        &[
            "-Xmx1G",
            "com.example.Other",
            "--version",
            "1.21.4",
            "--username",
            "A",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no es un Minecraft de Zenith"), "{err}");
}
