use std::path::PathBuf;
use zenith_bridge::args::LaunchArgs;

fn fixture() -> Vec<String> {
    include_str!("fixtures/xmcl-args.txt")
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// Minimal valid command line; `extra` is appended.
fn base(extra: &[&str]) -> Vec<String> {
    let mut v = s(&[
        "C:\\x\\bin\\javaw.exe",
        "--username",
        "Alex",
        "--version",
        "26.3",
        "--gameDir",
        "C:/x/instances/Z",
        "--assetsDir",
        "C:/x/assets",
        "--uuid",
        "abc",
        "--accessToken",
        "SECRET",
    ]);
    v.extend(extra.iter().map(|x| x.to_string()));
    v
}

#[test]
fn parses_real_xmcl_line() {
    let a = LaunchArgs::parse(&fixture()).unwrap();
    assert_eq!(
        a.java,
        PathBuf::from("C:\\fake\\xmcl\\jre\\java-runtime-epsilon\\bin\\javaw.exe")
    );
    assert_eq!(a.version, "26.3");
    assert_eq!(a.username, "FakeName");
    assert_eq!(a.uuid, "11111111222233334444555555555555");
    assert_eq!(a.access_token, "FAKE_TOKEN");
    assert_eq!(a.game_dir, PathBuf::from("C:/fake/xmcl/instances/Zenith"));
    assert_eq!(a.minecraft_root(), PathBuf::from("C:/fake/xmcl"));
    assert_eq!(a.java_home(), "C:\\fake\\xmcl\\jre\\java-runtime-epsilon");
    assert_eq!((a.width, a.height, a.fullscreen), (None, None, false));
    assert_eq!((a.join, a.world), (None, None));
}

#[test]
fn unsubstituted_placeholders_are_absent() {
    let a = LaunchArgs::parse(&fixture()).unwrap();
    assert_eq!(a.xuid, None);
    let a = LaunchArgs::parse(&base(&["--xuid", "12345"])).unwrap();
    assert_eq!(a.xuid.as_deref(), Some("12345"));
}

#[test]
fn no_size_flags_means_none_and_minus_one_is_tolerated() {
    let a = LaunchArgs::parse(&base(&[])).unwrap();
    assert_eq!((a.width, a.height), (None, None));
    let a = LaunchArgs::parse(&base(&["--width", "-1", "--height", "-1"])).unwrap();
    assert_eq!((a.width, a.height), (None, None));
}

#[test]
fn height_then_width() {
    let a = LaunchArgs::parse(&base(&["--height", "480", "--width", "854"])).unwrap();
    assert_eq!((a.width, a.height), (Some(854), Some(480)));
    assert!(LaunchArgs::parse(&base(&["--width", "abc"])).is_err());
}

#[test]
fn fullscreen_flag() {
    assert!(
        LaunchArgs::parse(&base(&["--fullscreen"]))
            .unwrap()
            .fullscreen
    );
}

#[test]
fn server_and_port_become_join() {
    let a = LaunchArgs::parse(&base(&["--server", "a.b", "--port", "25566"])).unwrap();
    assert_eq!(a.join.as_deref(), Some("a.b:25566"));
    let a = LaunchArgs::parse(&base(&["--server", "a.b"])).unwrap();
    assert_eq!(a.join.as_deref(), Some("a.b"));
}

#[test]
fn quickplay_multiplayer_becomes_join_and_wins() {
    let a = LaunchArgs::parse(&base(&[
        "--quickPlayMultiplayer",
        "p.example:25566",
        "--server",
        "p.example",
        "--port",
        "25566",
    ]))
    .unwrap();
    assert_eq!(a.join.as_deref(), Some("p.example:25566"));
}

#[test]
fn quickplay_singleplayer_becomes_world() {
    let a = LaunchArgs::parse(&base(&["--quickPlaySingleplayer", "New World"])).unwrap();
    assert_eq!(a.world.as_deref(), Some("New World"));
}

#[test]
fn spaces_and_accents_in_paths() {
    let mut v = base(&[]);
    v[0] = "C:\\Users\\José Ñ\\jre\\bin\\javaw.exe".into();
    v[6] = "C:/Users/José Ñ/inst ances/Z".into();
    v[8] = "C:/Users/José Ñ/assets".into();
    let a = LaunchArgs::parse(&v).unwrap();
    assert_eq!(a.game_dir, PathBuf::from("C:/Users/José Ñ/inst ances/Z"));
    assert_eq!(a.minecraft_root(), PathBuf::from("C:/Users/José Ñ"));
    assert_eq!(a.java_home(), "C:\\Users\\José Ñ\\jre");
}

#[test]
fn missing_token_is_an_error() {
    let v: Vec<String> = base(&[])
        .into_iter()
        .filter(|x| x != "--accessToken" && x != "SECRET")
        .collect();
    let e = LaunchArgs::parse(&v).unwrap_err();
    assert!(e.contains("accessToken"), "{e}");
    // an unsubstituted placeholder is not a token either
    let v = s(&["j", "--accessToken", "${auth_access_token}"]);
    assert!(LaunchArgs::parse(&v).is_err());
}

#[test]
fn empty_argv_is_an_error() {
    assert!(LaunchArgs::parse(&[]).is_err());
}

#[test]
fn authlib_injector_accounts_are_refused() {
    let mut v = base(&[]);
    v.insert(
        1,
        "-javaagent:C:/x/authlib-injector.jar=https://skin.example".into(),
    );
    assert!(LaunchArgs::parse(&v).unwrap_err().contains("Microsoft"));
}

#[test]
fn debug_hides_token() {
    let a = LaunchArgs::parse(&base(&[])).unwrap();
    assert!(!format!("{a:?}").contains("SECRET"));
}

#[test]
fn session_json_has_required_fields() {
    let a = LaunchArgs::parse(&base(&["--xuid", "99"])).unwrap();
    let v: serde_json::Value = serde_json::from_str(&a.session_json()).unwrap();
    assert_eq!(v["name"], "Alex");
    assert_eq!(v["uuid"], "abc");
    assert_eq!(v["access_token"], "SECRET");
    assert_eq!(v["user_type"], "msa");
    assert_eq!(v["xuid"], "99");
    let a = LaunchArgs::parse(&base(&[])).unwrap();
    let v: serde_json::Value = serde_json::from_str(&a.session_json()).unwrap();
    assert!(v.get("xuid").is_none());
}
