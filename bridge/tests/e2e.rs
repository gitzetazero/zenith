//! Runs the REAL zenith-client through the bridge's own command and relay, with a fake session.
//! `ZENITH_E2E_BUILD` is an unpacked Zenith build, `ZENITH_E2E_PREPARED` its prepared resources
//! (writable: the client writes `horizon-data.bin` there). A window opens for a few seconds.
//!   cargo test --manifest-path bridge/Cargo.toml --test e2e -- --ignored --nocapture
use std::path::PathBuf;
use zenith_bridge::args::LaunchArgs;
use zenith_bridge::play::{READY_LINE, Stream, client_cmd, run_game};

#[test]
#[ignore = "needs ZENITH_E2E_BUILD and ZENITH_E2E_PREPARED"]
fn real_client_starts_and_exits_cleanly() {
    let build = PathBuf::from(std::env::var("ZENITH_E2E_BUILD").expect("ZENITH_E2E_BUILD"));
    let prepared =
        PathBuf::from(std::env::var("ZENITH_E2E_PREPARED").expect("ZENITH_E2E_PREPARED"));
    let game = std::env::temp_dir().join(format!("zenith-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&game).unwrap();
    let argv: Vec<String> = [
        "C:\\jdk\\bin\\javaw.exe",
        "--username",
        "ZenithTest",
        "--version",
        "26.3",
        "--gameDir",
        &game.to_string_lossy(),
        "--assetsDir",
        "C:/x/assets",
        "--uuid",
        "069a79f444e94726a5befca90e38aaf5",
        "--accessToken",
        "FAKE",
        "--width",
        "640",
        "--height",
        "360",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let a = LaunchArgs::parse(&argv).unwrap();
    let mut cmd = client_cmd(&build, &a, &prepared);
    // draws this many frames, then quits by itself (about 15 s at 60 fps)
    let frames = std::env::var("ZENITH_E2E_FRAMES").unwrap_or_else(|_| "900".into());
    cmd.args(["--frames", &frames]);
    let mut lines = Vec::new();
    let code = run_game(cmd, &a.session_json(), &mut |s: Stream, l: &str| {
        println!("[{s:?}] {l}");
        lines.push(l.to_string());
    })
    .expect("the client starts");
    assert_eq!(lines.first().map(String::as_str), Some(READY_LINE));
    assert!(
        lines.iter().any(|l| l.starts_with("Horizon")),
        "the client's own output is relayed"
    );
    assert_eq!(code, 0, "{lines:?}");
    let _ = std::fs::remove_dir_all(&game);
}
