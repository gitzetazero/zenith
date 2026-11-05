use std::cell::RefCell;
use std::path::{Path, PathBuf};
use zenith_bridge::setup::{Env, setup, start, use_version, write_instance};

fn tmp(tag: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("zenith-setup-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn env(base: &Path) -> Env {
    Env {
        appdata: base.join("appdata"),
        home: base.join("zenith home Ñ"),
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

/// A good XMCL root (with a space in it) recorded in `root`.
fn good_root(base: &Path, e: &Env) -> PathBuf {
    let root = base.join("mi xmcl");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(root_file(e).parent().unwrap()).unwrap();
    std::fs::write(root_file(e), format!("{}\r\n", root.display())).unwrap();
    root
}

fn instance(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(root.join("instances/Zenith/instance.json")).unwrap())
        .unwrap()
}

fn nothing_written(base: &Path, e: &Env) {
    assert!(!e.home.exists(), "home created");
    assert!(!base.join("xmcl").exists());
}

#[test]
fn absent_root_is_refused_and_nothing_is_created() {
    let base = tmp("absent");
    let e = env(&base);
    let err = setup(&e, &fake_exe(&base), || panic!("no network")).unwrap_err();
    assert!(err.contains("Open XMCL once"), "{err}");
    assert!(!e.appdata.exists());
    nothing_written(&base, &e);
}

#[test]
fn odd_roots_are_refused_and_untouched() {
    let cases: [(&str, Vec<u8>); 4] = [
        ("empty", b"  \r\n".to_vec()),
        ("utf8", vec![0xff, 0xfe, 0x41]),
        ("missing", b"Z:\\no\\existe\\xmcl".to_vec()),
        ("dirnotfile", b"".to_vec()),
    ];
    for (tag, content) in cases {
        let base = tmp(tag);
        let e = env(&base);
        std::fs::create_dir_all(root_file(&e).parent().unwrap()).unwrap();
        if tag == "dirnotfile" {
            std::fs::create_dir(root_file(&e)).unwrap(); // reading it fails with something other than NotFound
        } else {
            std::fs::write(root_file(&e), &content).unwrap();
        }
        let err = setup(&e, &fake_exe(&base), || panic!("no network")).unwrap_err();
        assert!(err.contains("Nothing was changed"), "{tag}: {err}");
        if tag != "dirnotfile" {
            assert_eq!(std::fs::read(root_file(&e)).unwrap(), content);
        }
        nothing_written(&base, &e);
    }
}

#[test]
fn good_root_installs_shim_and_instance() {
    let base = tmp("good");
    let e = env(&base);
    let root = good_root(&base, &e);
    let out = setup(&e, &fake_exe(&base), mc).unwrap();
    assert_eq!(std::fs::read(e.java_exe()).unwrap(), b"bridge bytes");
    assert_eq!(
        std::fs::read(e.java_exe().with_file_name("javaw.exe")).unwrap(),
        b"bridge bytes"
    );
    let v = instance(&root);
    assert_eq!(v["name"], "Zenith");
    assert_eq!(v["runtime"]["minecraft"], "26.3");
    assert_eq!(v["prependCommand"], "");
    // the Ñ and the space survive JSON
    assert_eq!(v["java"], e.java_exe().to_string_lossy().as_ref());
    assert!(out.join("\n").contains("Microsoft"));
    // nothing under XMCL's own folders but the instance
    assert!(!root.join("jre").exists());
}

#[test]
fn identical_rerun_changes_nothing() {
    let base = tmp("again");
    let e = env(&base);
    good_root(&base, &e);
    let exe = fake_exe(&base);
    setup(&e, &exe, mc).unwrap();
    let out = setup(&e, &exe, mc).unwrap().join("\n");
    assert!(out.contains("was already up to date"), "{out}");
    assert!(out.contains("was already installed"), "{out}");
}

#[test]
fn offline_note_is_shown() {
    let base = tmp("note");
    let e = env(&base);
    good_root(&base, &e);
    let out = setup(&e, &fake_exe(&base), || {
        ("26.3".into(), Some("no internet connection".into()))
    })
    .unwrap();
    assert!(out.contains(&"no internet connection".to_string()));
}

#[test]
fn running_xmcl_is_reported_with_restart() {
    let base = tmp("running");
    let mut e = env(&base);
    e.xmcl_running = || true;
    good_root(&base, &e);
    let out = setup(&e, &fake_exe(&base), mc).unwrap().join("\n");
    assert!(out.contains("open it again"), "{out}");
}

#[cfg(windows)]
#[test]
fn locked_shim_is_reported_and_old_copy_kept() {
    use std::os::windows::fs::OpenOptionsExt;
    let base = tmp("locked");
    let e = env(&base);
    good_root(&base, &e);
    let exe = fake_exe(&base);
    setup(&e, &exe, mc).unwrap();
    std::fs::write(&exe, b"newer bridge").unwrap();
    // like a running exe: open without sharing delete/write
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(e.java_exe().with_file_name("javaw.exe"))
        .unwrap();
    let _lock2 = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(e.java_exe())
        .unwrap();
    let err = setup(&e, &exe, mc).unwrap_err();
    assert!(err.contains("close the game"), "{err}");
    assert_eq!(std::fs::read(e.java_exe()).unwrap(), b"bridge bytes");
    assert!(!e.java_exe().with_extension("exe.new").exists());
}

#[test]
fn existing_instance_keeps_unknown_fields_and_first_bak() {
    let base = tmp("keep");
    let java = Path::new(r"C:\Users\José Ñ\zenith\java\bin\java.exe");
    let dir = base.join("instances/Zenith");
    std::fs::create_dir_all(&dir).unwrap();
    let old = r#"{"name":"Zenith","runtime":{"minecraft":"1.0","forge":"x"},"maxMemory":4096,"prependCommand":"foo"}"#;
    std::fs::write(dir.join("instance.json"), old).unwrap();
    let (file, changed) = write_instance(&base, java, "26.3").unwrap();
    assert!(changed);
    assert_eq!(
        std::fs::read_to_string(dir.join("instance.json.bak")).unwrap(),
        old
    );
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(v["maxMemory"], 4096);
    assert_eq!(v["runtime"]["forge"], "x");
    assert_eq!(v["runtime"]["minecraft"], "26.3");
    assert_eq!(v["prependCommand"], "");
    assert_eq!(v["java"], java.to_string_lossy().as_ref());
    // the raw file has the backslashes escaped
    assert!(
        std::fs::read_to_string(&file)
            .unwrap()
            .contains(r"C:\\Users\\José Ñ")
    );
    // a later change must not overwrite the first .bak
    write_instance(&base, java, "26.4").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("instance.json.bak")).unwrap(),
        old
    );
    let (_, changed) = write_instance(&base, java, "26.4").unwrap();
    assert!(!changed);
}

#[test]
fn unreadable_instance_is_backed_up_not_lost() {
    let base = tmp("junk");
    let dir = base.join("instances/Zenith");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("instance.json"), "{not json").unwrap();
    write_instance(&base, Path::new("j.exe"), "26.3").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("instance.json.bak")).unwrap(),
        "{not json"
    );
}

#[test]
fn use_pins_installed_builds_only() {
    let home = tmp("use");
    std::fs::create_dir_all(home.join("builds/v1")).unwrap();
    let err = use_version(&home, "v2").unwrap_err();
    assert!(err.contains("v1"), "{err}");
    use_version(&home, "v1").unwrap();
    assert_eq!(
        std::fs::read_to_string(home.join("pinned.txt")).unwrap(),
        "v1"
    );
    use_version(&home, "latest").unwrap();
    assert!(!home.join("pinned.txt").exists());
}

/// A fake install folder: the bridge with an XMCL.exe beside it.
fn installed(base: &Path) -> PathBuf {
    let dir = base.join("Zenith Launcher");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("XMCL.exe"), b"xmcl").unwrap();
    let exe = dir.join("zenith-bridge.exe");
    std::fs::write(&exe, b"bridge bytes").unwrap();
    exe
}

#[test]
fn start_without_root_only_starts_xmcl_and_creates_nothing() {
    let base = tmp("start-absent");
    let e = env(&base);
    let exe = installed(&base);
    let spawned = RefCell::new(vec![]);
    let out = start(
        &e,
        &exe,
        || panic!("no network"),
        |p| {
            spawned.borrow_mut().push(p.to_path_buf());
            Ok(())
        },
    )
    .unwrap()
    .join(
        "
",
    );
    assert_eq!(*spawned.borrow(), vec![exe.with_file_name("XMCL.exe")]);
    assert!(out.contains("First time") && out.contains("again"), "{out}");
    assert!(!e.appdata.exists(), "XMCL's root file must stay XMCL's");
    nothing_written(&base, &e);
}

#[test]
fn start_with_root_prepares_quietly_then_starts_xmcl() {
    let base = tmp("start-good");
    let e = env(&base);
    let root = good_root(&base, &e);
    let exe = installed(&base);
    let n = RefCell::new(0);
    let out = start(&e, &exe, mc, |_| {
        *n.borrow_mut() += 1;
        Ok(())
    })
    .unwrap();
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(*n.borrow(), 1);
    assert_eq!(instance(&root)["name"], "Zenith");
    assert_eq!(std::fs::read(e.java_exe()).unwrap(), b"bridge bytes");
    // idempotent
    start(&e, &exe, mc, |_| Ok(())).unwrap();
}

#[test]
fn start_never_starts_a_running_xmcl() {
    for with_root in [false, true] {
        let base = tmp("start-running");
        let mut e = env(&base);
        e.xmcl_running = || true;
        if with_root {
            good_root(&base, &e);
        }
        let exe = installed(&base);
        let out = start(&e, &exe, mc, |_| panic!("must not spawn"))
            .unwrap()
            .join(
                "
",
            );
        assert!(out.contains("again"), "{out}");
    }
}

#[test]
fn start_refuses_odd_root_and_missing_xmcl() {
    let base = tmp("start-odd");
    let e = env(&base);
    let exe = installed(&base);
    std::fs::create_dir_all(root_file(&e).parent().unwrap()).unwrap();
    std::fs::write(root_file(&e), r"Z:\no\existe").unwrap();
    let err = start(&e, &exe, mc, |_| panic!("must not spawn")).unwrap_err();
    assert!(err.contains("Nothing was changed"), "{err}");
    std::fs::remove_file(exe.with_file_name("XMCL.exe")).unwrap();
    let err = start(&e, &exe, mc, |_| panic!("must not spawn")).unwrap_err();
    assert!(err.contains("Reinstall"), "{err}");
}
