use serde_json::Value;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zenith_bridge::args::LaunchArgs;
use zenith_bridge::owner::OwnerCheck;
use zenith_bridge::play::{Stream, client_cmd};
use zenith_bridge::prepare::{ensure_server_jar, prepare_cmd};
use zenith_bridge::run::{Deps, run};

const TOKEN: &str = "SECRET-TOKEN-123";
const JAR: &[u8] = b"pretend server jar";

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn args(game: &str, assets: &str, extra: &[&str]) -> LaunchArgs {
    let mut v = s(&[
        "C:\\jdk\\bin\\javaw.exe",
        "--username",
        "Alex",
        "--version",
        "26.3",
        "--gameDir",
        game,
        "--assetsDir",
        assets,
        "--uuid",
        "abc",
        "--accessToken",
        TOKEN,
    ]);
    v.extend(extra.iter().map(|x| x.to_string()));
    LaunchArgs::parse(&v).unwrap()
}

fn strs(c: &std::process::Command) -> Vec<String> {
    c.get_args()
        .map(|a| a.to_string_lossy().replace("\\", "/"))
        .collect()
}

fn env_of(c: &std::process::Command) -> Vec<(String, String)> {
    c.get_envs()
        .filter_map(|(k, v)| {
            Some((
                k.to_string_lossy().into_owned(),
                v?.to_string_lossy().into_owned(),
            ))
        })
        .collect()
}

#[test]
fn client_cmd_never_has_token() {
    let a = args(
        "C:/g",
        "C:/x/assets",
        &["--width", "854", "--height", "480"],
    );
    let c = client_cmd(Path::new("B"), &a, Path::new("P"));
    assert!(!strs(&c).iter().any(|x| x.contains(TOKEN)));
    assert!(
        !env_of(&c)
            .iter()
            .any(|(k, v)| k.contains(TOKEN) || v.contains(TOKEN))
    );
}

#[test]
fn client_cmd_args() {
    let a = args("C:/g", "C:/x/assets", &[]);
    let c = client_cmd(Path::new("B"), &a, Path::new("P"));
    assert_eq!(
        strs(&c),
        s(&[
            "--resources",
            "P",
            "--game-dir",
            "C:/g",
            "--home",
            "C:/g",
            "--session-stdin"
        ])
    );
    let a = args(
        "C:/g",
        "C:/x/assets",
        &[
            "--width",
            "854",
            "--height",
            "480",
            "--quickPlayMultiplayer",
            "h:1",
            "--quickPlaySingleplayer",
            "W w",
        ],
    );
    let v = strs(&client_cmd(Path::new("B"), &a, Path::new("P")));
    for pair in [
        ["--width", "854"],
        ["--height", "480"],
        ["--quickPlayMultiplayer", "h:1"],
        ["--quickPlaySingleplayer", "W w"],
    ] {
        assert!(v.windows(2).any(|w| w == pair), "{pair:?} in {v:?}");
    }
    let a = args("C:/g", "C:/x/assets", &["--fullscreen"]);
    assert!(strs(&client_cmd(Path::new("B"), &a, Path::new("P"))).contains(&"--fullscreen".into()));
}

#[test]
fn client_cmd_env() {
    let a = args("C:/g", "C:/x/assets", &[]);
    let e = env_of(&client_cmd(Path::new("B"), &a, Path::new("P")));
    for (k, v) in [
        ("HORIZON_TELEMETRY_URL", "off"),
        ("HORIZON_NO_HEARTBEAT", "1"),
        ("HORIZON_COSMETICS", "0"),
        ("HORIZON_HOME", "C:/g"),
    ] {
        assert!(e.contains(&(k.into(), v.into())), "{k} in {e:?}");
    }
}

#[test]
fn prepare_cmd_args() {
    let a = args("C:/g", "C:/x/assets", &[]);
    let c = prepare_cmd(
        Path::new("B"),
        &a,
        Path::new("P"),
        Path::new("J.jar"),
        Path::new("W"),
    );
    let v = strs(&c);
    for pair in [
        ["--server-jar", "J.jar"],
        ["--version-json", "C:/x/versions/26.3/26.3.json"],
        ["--client-jar", "C:/x/versions/26.3/26.3.jar"],
        ["--libraries-dir", "C:/x/libraries"],
        ["--assets-dir", "C:/x/assets"],
        ["--java-home", "C:/jdk"],
        ["--output", "P"],
        ["--root", "W"],
        ["--progress", "json"],
    ] {
        assert!(v.windows(2).any(|w| w == pair), "{pair:?} in {v:?}");
    }
    assert!(v.contains(&"--prepare".into()) && v.contains(&"--complete".into()));
    assert!(!v.iter().any(|x| x.contains(TOKEN)));
}

fn sha1_hex(b: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    Sha1::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

fn version_json(dir: &Path, sha1: &str) -> PathBuf {
    let p = dir.join("26.3.json");
    std::fs::create_dir_all(dir).unwrap();
    let j = serde_json::json!({
        "assetIndex": { "id": "34", "sha1": "idx" },
        "downloads": { "server": { "url": "https://x.invalid/server.jar", "sha1": sha1 } }
    });
    std::fs::write(&p, j.to_string()).unwrap();
    p
}

fn tmp(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("zenith-play-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn server_jar_downloaded_and_verified() {
    let d = tmp("jar");
    let vj = version_json(&d.join("v"), &sha1_hex(JAR));
    let dest = d.join("server-jars/26.3.jar");
    ensure_server_jar(&vj, &dest, |u| {
        assert_eq!(u, "https://x.invalid/server.jar");
        Ok(Some(JAR.to_vec()))
    })
    .unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), JAR);
    // already there and matching: no download
    ensure_server_jar(&vj, &dest, |_| panic!("downloaded again")).unwrap();
}

#[test]
fn server_jar_with_wrong_hash_is_refused() {
    let d = tmp("badjar");
    let vj = version_json(&d.join("v"), &sha1_hex(JAR));
    let dest = d.join("server-jars/26.3.jar");
    let err = ensure_server_jar(&vj, &dest, |_| Ok(Some(b"other".to_vec()))).unwrap_err();
    assert!(err.contains("sha1"), "{err}");
    assert!(!dest.exists());
}

// ---- the whole flow, against a fake horizon-client / horizon-prepare ----

struct Fx {
    home: PathBuf,
    build: PathBuf,
    argv: Vec<String>,
    out: Rc<RefCell<Vec<(Stream, String)>>>,
    errors: Rc<RefCell<Vec<String>>>,
    fetched: Rc<RefCell<Vec<String>>>,
}

fn fake_exe() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let name = format!("fake_horizon{}", std::env::consts::EXE_SUFFIX);
    exe.parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join(name)
}

/// `behavior` is the fake's behavior.json.
fn fx(name: &str, behavior: Value, extra: &[&str]) -> Fx {
    let root = tmp(name);
    let home = root.join("zenith");
    let build = home.join("builds/test");
    std::fs::create_dir_all(&build).unwrap();
    std::fs::write(home.join("current.txt"), "test").unwrap();
    for tool in ["horizon-client", "horizon-prepare"] {
        let dest = build.join(format!("{tool}{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(fake_exe(), dest).unwrap();
    }
    std::fs::write(build.join("research-lock.json"), "{}").unwrap();
    std::fs::write(build.join("behavior.json"), behavior.to_string()).unwrap();
    let mc = root.join("mc");
    version_json(&mc.join("versions/26.3"), &sha1_hex(JAR));
    let game = mc.join("instances/Z");
    std::fs::create_dir_all(&game).unwrap();
    let mut argv = s(&[
        "C:\\jdk\\bin\\javaw.exe",
        "-Xmx2G",
        "--username",
        "Alex",
        "--version",
        "26.3",
        "--gameDir",
        &game.to_string_lossy(),
        "--assetsDir",
        &mc.join("assets").to_string_lossy(),
        "--uuid",
        "abc",
        "--accessToken",
        TOKEN,
    ]);
    argv.extend(extra.iter().map(|x| x.to_string()));
    Fx {
        home,
        build,
        argv,
        out: Default::default(),
        errors: Default::default(),
        fetched: Default::default(),
    }
}

impl Fx {
    fn run(&self, owner: fn() -> OwnerCheck) -> i32 {
        let (out, errors, fetched) = (self.out.clone(), self.errors.clone(), self.fetched.clone());
        let deps = Deps {
            home: self.home.clone(),
            keys: vec![],
            check_owner: Box::new(move |_| owner()),
            fetch: Box::new(move |u| {
                fetched.borrow_mut().push(u.to_string());
                if u.ends_with("server.jar") {
                    Ok(Some(JAR.to_vec()))
                } else {
                    Err("offline".into())
                }
            }),
            emit: Box::new(move |st, l| out.borrow_mut().push((st, l.to_string()))),
            show_error: Box::new(move |m| errors.borrow_mut().push(m.to_string())),
        };
        run(&self.argv, &deps)
    }

    fn records(&self) -> Vec<Value> {
        std::fs::read_to_string(self.build.join("record.log"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn stdout(&self) -> Vec<String> {
        self.out
            .borrow()
            .iter()
            .filter(|(s, _)| *s == Stream::Out)
            .map(|(_, l)| l.clone())
            .collect()
    }
}

fn owner() -> OwnerCheck {
    OwnerCheck::Owner
}

fn has_flag(r: &Value, flag: &str) -> bool {
    r["args"].as_array().unwrap().iter().any(|a| a == flag)
}

#[test]
fn first_run_prepares_then_starts_the_game() {
    let f = fx(
        "happy",
        serde_json::json!({ "minecraft": "26.3", "stdout": ["hello"], "exit": 7 }),
        &["--width", "854", "--height", "480"],
    );
    assert_eq!(f.run(owner), 7, "exit code passes through");
    let recs = f.records();
    let prep = recs
        .iter()
        .find(|r| has_flag(r, "--prepare"))
        .expect("prepared");
    assert!(has_flag(prep, "--complete"));
    assert!(f.home.join("server-jars/26.3.jar").is_file());
    let client = recs.iter().find(|r| r["tool"] == "horizon-client").unwrap();
    let session: Value = serde_json::from_str(client["stdin"].as_str().unwrap()).unwrap();
    assert_eq!(session["name"], "Alex");
    assert_eq!(session["access_token"], TOKEN);
    for r in &recs {
        let all = format!("{}{}", r["args"], r["env"]);
        assert!(!all.contains(TOKEN), "token leaked to {}", r["tool"]);
    }
    let env = |k: &str| {
        client["env"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e[0] == k)
            .map(|e| e[1].clone())
    };
    assert_eq!(env("HORIZON_TELEMETRY_URL"), Some("off".into()));
    assert_eq!(env("HORIZON_NO_HEARTBEAT"), Some("1".into()));
    assert!(
        client["args"]
            .as_array()
            .unwrap()
            .windows(2)
            .any(|w| w[0] == "--width" && w[1] == "854")
    );
    let out = f.stdout();
    assert!(
        out.contains(&"Importing the game files".to_string()),
        "{out:?}"
    );
    let ready = out
        .iter()
        .position(|l| l == "LWJGL Version: zenith")
        .expect("ready line");
    let hello = out.iter().position(|l| l == "hello").expect("forwarded");
    assert!(ready < hello);
    let log = std::fs::read_to_string(f.home.join("logs/bridge.log")).unwrap_or_default();
    assert!(!log.contains(TOKEN));
}

#[test]
fn second_run_skips_preparation() {
    let f = fx("second", serde_json::json!({ "minecraft": "26.3" }), &[]);
    assert_eq!(f.run(owner), 0);
    let before = f.fetched.borrow().len();
    std::fs::remove_file(f.build.join("record.log")).unwrap();
    assert_eq!(f.run(owner), 0);
    assert!(!f.records().iter().any(|r| has_flag(r, "--prepare")));
    assert!(
        !f.fetched
            .borrow()
            .iter()
            .skip(before)
            .any(|u| u.ends_with("server.jar"))
    );
}

#[test]
fn refused_account_reaches_nothing() {
    let f = fx("refused", serde_json::json!({ "minecraft": "26.3" }), &[]);
    assert_eq!(f.run(|| OwnerCheck::NotOwner), 1);
    assert!(f.fetched.borrow().is_empty(), "no network after a refusal");
    assert!(f.records().is_empty(), "nothing was run");
    assert!(f.errors.borrow()[0].contains("posea Minecraft"));
}

#[test]
fn wrong_minecraft_version_is_refused() {
    let f = fx("wrongver", serde_json::json!({ "minecraft": "27.0" }), &[]);
    assert_eq!(f.run(owner), 1);
    assert!(
        f.errors.borrow()[0].contains("Esta instancia debe ser Minecraft 27.0"),
        "{:?}",
        f.errors
    );
    assert!(
        !f.records()
            .iter()
            .any(|r| has_flag(r, "--prepare") || r["tool"] == "horizon-client")
    );
}

#[test]
fn crash_line_translated() {
    let f = fx(
        "crash",
        serde_json::json!({ "minecraft": "26.3", "stderr": ["Horizon: crash report saved to C:\\x y\\c.txt"], "exit": 1 }),
        &[],
    );
    assert_eq!(f.run(owner), 1);
    assert!(
        f.stdout()
            .contains(&"Crash report saved to: #@!@# C:\\x y\\c.txt".to_string()),
        "{:?}",
        f.stdout()
    );
}

#[test]
fn failed_preparation_does_not_start_the_game() {
    let f = fx(
        "prepfail",
        serde_json::json!({ "minecraft": "26.3", "prepare_fails": true }),
        &[],
    );
    assert_eq!(f.run(owner), 1);
    assert!(f.errors.borrow()[0].contains("boom"), "{:?}", f.errors);
    assert!(!f.records().iter().any(|r| r["tool"] == "horizon-client"));
}
