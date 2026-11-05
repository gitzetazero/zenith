use std::path::{Path, PathBuf};
use zenith_bridge::setup::{Env, command_fits, quote_prepend, setup, write_instance};

fn tmp(tag: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("zenith-setup-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn env(base: &Path, fits: fn(usize, usize) -> bool) -> Env {
    Env {
        appdata: base.join("appdata"),
        default_root: base.join("xmcl"),
        fits,
        xmcl_running: || false,
    }
}

fn fake_exe(base: &Path) -> PathBuf {
    let p = base.join("src-bridge.exe");
    std::fs::write(&p, b"bridge bytes").unwrap();
    p
}

fn mc() -> (String, Option<String>) {
    ("26.3".into(), None)
}

fn root_file(e: &Env) -> PathBuf {
    e.appdata.join("xmcl").join("root")
}

#[test]
fn command_fits_matches_the_measured_limit() {
    // bridge = <root>\zenith-bridge.exe
    assert!(command_fits(7, 7 + 18)); // C:\xmcl
    assert!(command_fits(19, 19 + 18)); // the last length measured to run
    assert!(!command_fits(20, 20 + 18)); // measured to fail
    assert!(!command_fits(36, 36 + 18));
}

#[test]
fn absent_root_is_created_with_instance() {
    let base = tmp("absent");
    let e = env(&base, |_, _| true);
    let exe = fake_exe(&base);
    let out = setup(&e, &exe, mc).unwrap();
    let root = std::fs::read_to_string(root_file(&e)).unwrap();
    assert_eq!(root, e.default_root.to_string_lossy());
    assert!(e.default_root.join("zenith-bridge.exe").is_file());
    let v: serde_json::Value = serde_json::from_slice(
        &std::fs::read(e.default_root.join("instances/Zenith/instance.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["name"], "Zenith");
    assert_eq!(v["runtime"]["minecraft"], "26.3");
    assert!(v.get("java").is_none());
    assert!(out.join("\n").contains("Microsoft"));
}

#[test]
fn existing_root_is_reused_even_with_trailing_newline() {
    let base = tmp("reuse");
    let e = env(&base, |_, _| true);
    let mine = base.join("mi xmcl");
    std::fs::create_dir_all(root_file(&e).parent().unwrap()).unwrap();
    std::fs::write(root_file(&e), format!("{}\r\n", mine.display())).unwrap();
    setup(&e, &fake_exe(&base), mc).unwrap();
    assert!(mine.join("instances/Zenith/instance.json").is_file());
    assert!(!e.default_root.exists());
}

#[test]
fn too_long_root_is_refused_and_untouched() {
    let base = tmp("long");
    let e = env(&base, command_fits);
    let long = base.join("a".repeat(60));
    std::fs::create_dir_all(root_file(&e).parent().unwrap()).unwrap();
    std::fs::write(root_file(&e), long.to_string_lossy().as_bytes()).unwrap();
    let err = setup(&e, &fake_exe(&base), || {
        panic!("no network before refusing")
    })
    .unwrap_err();
    assert!(err.contains("8191"), "{err}");
    assert_eq!(
        std::fs::read_to_string(root_file(&e)).unwrap(),
        long.to_string_lossy()
    );
    assert!(!long.exists());
}

#[test]
fn quotes_path_with_spaces_and_enie() {
    let p = Path::new(r"C:\Users\Jose Ñ\bridge dir\zenith-bridge.exe");
    assert_eq!(quote_prepend(p), format!("\"{}\"", p.display()));
    let base = tmp("quote");
    let (file, _) = write_instance(&base, p, "26.3").unwrap();
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert_eq!(
        v["prependCommand"],
        "\"C:\\Users\\Jose Ñ\\bridge dir\\zenith-bridge.exe\""
    );
}

#[test]
fn existing_instance_is_kept_with_bak_and_user_fields() {
    let base = tmp("keep");
    let exe = Path::new("zenith-bridge.exe");
    let dir = base.join("instances/Zenith");
    std::fs::create_dir_all(&dir).unwrap();
    let old = r#"{"name":"Zenith","runtime":{"minecraft":"1.0"},"maxMemory":4096}"#;
    std::fs::write(dir.join("instance.json"), old).unwrap();
    let (file, changed) = write_instance(&base, exe, "26.3").unwrap();
    assert!(changed);
    assert_eq!(
        std::fs::read_to_string(dir.join("instance.json.bak")).unwrap(),
        old
    );
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(v["maxMemory"], 4096);
    assert_eq!(v["runtime"]["minecraft"], "26.3");
    // again: nothing changes
    let (_, changed) = write_instance(&base, exe, "26.3").unwrap();
    assert!(!changed);
}

#[test]
fn unreadable_instance_is_backed_up_not_lost() {
    let base = tmp("junk");
    let dir = base.join("instances/Zenith");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("instance.json"), "{not json").unwrap();
    write_instance(&base, Path::new("b.exe"), "26.3").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("instance.json.bak")).unwrap(),
        "{not json"
    );
}

#[test]
fn bridge_copy_is_replaced_when_different() {
    let base = tmp("copy");
    let e = env(&base, |_, _| true);
    let exe = fake_exe(&base);
    setup(&e, &exe, mc).unwrap();
    std::fs::write(&exe, b"newer bridge").unwrap();
    setup(&e, &exe, mc).unwrap();
    assert_eq!(
        std::fs::read(e.default_root.join("zenith-bridge.exe")).unwrap(),
        b"newer bridge"
    );
}

#[test]
fn use_pins_installed_builds_only() {
    let home = tmp("use");
    std::fs::create_dir_all(home.join("builds/v1")).unwrap();
    let err = zenith_bridge::setup::use_version(&home, "v2").unwrap_err();
    assert!(err.contains("v1"), "{err}");
    zenith_bridge::setup::use_version(&home, "v1").unwrap();
    assert_eq!(
        std::fs::read_to_string(home.join("pinned.txt")).unwrap(),
        "v1"
    );
    zenith_bridge::setup::use_version(&home, "latest").unwrap();
    assert!(!home.join("pinned.txt").exists());
}
