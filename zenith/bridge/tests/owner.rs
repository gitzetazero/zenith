use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use zenith_bridge::owner::{OwnerCheck, check, gate};

/// Answers one request with `status`/`body` and returns the base URL.
fn serve_once(status: &str, body: &str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    std::thread::spawn(move || {
        let (mut c, _) = l.accept().unwrap();
        let mut buf = [0u8; 4096];
        let n = c.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        assert!(req.starts_with("GET /entitlements/mcstore "), "{req}");
        assert!(
            req.to_lowercase().contains("authorization: bearer tok"),
            "{req}"
        );
        c.write_all(reply.as_bytes()).unwrap();
    });
    url
}

fn is(r: OwnerCheck, want: &str) {
    let got = match r {
        OwnerCheck::Owner => "owner",
        OwnerCheck::NotOwner => "not_owner",
        OwnerCheck::InvalidToken => "invalid_token",
        OwnerCheck::Unreachable(_) => "unreachable",
    };
    assert_eq!(got, want);
}

#[test]
fn owner_on_game_minecraft() {
    let url = serve_once("200 OK", r#"{"items":[{"name":"game_minecraft"}]}"#);
    is(check(&url, "tok"), "owner");
}

#[test]
fn owner_on_product_minecraft() {
    let url = serve_once(
        "200 OK",
        r#"{"items":[{"name":"x"},{"name":"product_minecraft"}]}"#,
    );
    is(check(&url, "tok"), "owner");
}

#[test]
fn not_owner_on_empty_items() {
    is(
        check(&serve_once("200 OK", r#"{"items":[]}"#), "tok"),
        "not_owner",
    );
}

#[test]
fn invalid_token_on_401_and_403() {
    is(
        check(&serve_once("401 Unauthorized", "{}"), "tok"),
        "invalid_token",
    );
    is(
        check(&serve_once("403 Forbidden", "{}"), "tok"),
        "invalid_token",
    );
}

#[test]
fn unreachable_on_503_and_429() {
    let url = serve_once("503 Service Unavailable", "{}");
    is(check(&url, "tok"), "unreachable");
    let url = serve_once("429 Too Many Requests", "{}");
    is(check(&url, "tok"), "unreachable");
}

#[test]
fn unreachable_when_refused() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    drop(l);
    is(check(&url, "tok"), "unreachable");
}

const DAY: u64 = 86_400;

fn cache(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zenith-bridge-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.join("owners.json")
}

fn down() -> OwnerCheck {
    OwnerCheck::Unreachable("offline".into())
}

#[test]
fn gate_allows_owner_and_caches() {
    let c = cache("a");
    gate(OwnerCheck::Owner, "AB-CD", &c, 1000).unwrap();
    // cached under the dashless lowercase UUID, so a later offline run passes
    gate(down(), "abcd", &c, 1000 + DAY).unwrap();
}

#[test]
fn gate_refuses_not_owner_even_with_cache() {
    let c = cache("b");
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    let e = gate(OwnerCheck::NotOwner, "u", &c, 1001).unwrap_err();
    assert!(e.contains("Java Edition"), "{e}");
}

#[test]
fn gate_unreachable_with_recent_cache_passes() {
    let c = cache("c");
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    gate(down(), "u", &c, 1000 + 29 * DAY).unwrap();
}

#[test]
fn gate_unreachable_with_old_cache_refuses() {
    let c = cache("d");
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    assert!(gate(down(), "u", &c, 1000 + 30 * DAY).is_err());
}

#[test]
fn gate_unreachable_without_cache_refuses() {
    let c = cache("e");
    assert!(gate(down(), "u", &c, 1000).is_err());
    gate(OwnerCheck::Owner, "other", &c, 1000).unwrap();
    assert!(gate(down(), "u", &c, 1000).is_err());
}

#[test]
fn gate_future_dated_entry_is_not_accepted_offline() {
    let c = cache("f");
    gate(OwnerCheck::Owner, "u", &c, 1_000_000).unwrap();
    assert!(gate(down(), "u", &c, 500_000).is_err());
}

#[test]
fn gate_corrupt_cache_is_empty_then_rewritten() {
    let c = cache("g");
    std::fs::create_dir_all(c.parent().unwrap()).unwrap();
    std::fs::write(&c, "{not json").unwrap();
    assert!(gate(down(), "u", &c, 1000).is_err());
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    gate(down(), "u", &c, 1001).unwrap();
}

#[test]
fn gate_write_is_atomic_and_leaves_no_temp_file() {
    let c = cache("h");
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    gate(OwnerCheck::Owner, "u", &c, 2000).unwrap();
    let names: Vec<_> = std::fs::read_dir(c.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["owners.json"]);
}

#[test]
fn gate_not_owner_removes_the_positive_entry() {
    let c = cache("i");
    gate(OwnerCheck::Owner, "u", &c, 1000).unwrap();
    gate(OwnerCheck::Owner, "keep", &c, 1000).unwrap();
    assert!(gate(OwnerCheck::NotOwner, "u", &c, 1001).is_err());
    assert!(gate(down(), "u", &c, 1002).is_err());
    gate(down(), "keep", &c, 1002).unwrap();
}

#[test]
fn offline_profile_plays_where_an_owner_signed_in_recently() {
    let c = cache("offline-ok");
    gate(OwnerCheck::Owner, "real", &c, 1000).unwrap();
    gate(
        OwnerCheck::InvalidToken,
        "offline-uuid",
        &c,
        1000 + 29 * DAY,
    )
    .unwrap();
}

#[test]
fn offline_profile_is_refused_without_a_recent_owner() {
    let c = cache("offline-none");
    let e = gate(OwnerCheck::InvalidToken, "offline-uuid", &c, 1000).unwrap_err();
    assert!(e.contains("Offline profiles"), "{e}");
    gate(OwnerCheck::Owner, "real", &c, 1000).unwrap();
    assert!(
        gate(
            OwnerCheck::InvalidToken,
            "offline-uuid",
            &c,
            1000 + 31 * DAY
        )
        .is_err()
    );
}

#[test]
fn a_valid_account_that_does_not_own_the_game_is_still_refused() {
    let c = cache("offline-notowner");
    gate(OwnerCheck::Owner, "real", &c, 1000).unwrap();
    assert!(gate(OwnerCheck::NotOwner, "someone-else", &c, 1001).is_err());
}
