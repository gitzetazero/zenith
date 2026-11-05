// The Windows build is the one that ships first (zip); the Linux .tar.gz is not installed yet.
#![cfg(windows)]

use horizon_release_sign::{SigningKey, sha256_hex, sums_file};
use std::cell::{Cell, RefCell};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use zenith_bridge::builds::{Store, asset_name, ensure_latest_after};

const TITLE: &str = "Nightly 2026-11-05 · b933efa";
const V1: &str = "nightly-2026-11-05-b933efa";

type Fetch<'a> = dyn Fn(&str) -> Result<Option<Vec<u8>>, String> + 'a;

fn store() -> Store {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("zenith-builds-test-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    Store { root }
}

fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, body) in entries {
        z.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(body.as_bytes()).unwrap();
    }
    z.finish().unwrap().into_inner()
}

fn good(tag: &str) -> Vec<u8> {
    zip_of(&[
        ("horizon-windows-x64/horizon-client.exe", tag),
        ("horizon-windows-x64/tools/a.txt", "a"),
    ])
}

struct Release {
    archive: Vec<u8>,
    sums: String,
    sig: String,
}

fn release(key: &SigningKey, channel: &str, archive: Vec<u8>) -> Release {
    let sums = sums_file(&[
        (asset_name().to_string(), sha256_hex(&archive)),
        ("horizon-linux-x86_64.tar.gz".into(), sha256_hex(b"linux")),
    ]);
    let sig = key.sign(channel, sums.as_bytes());
    Release { archive, sums, sig }
}

fn api(title: &str) -> Vec<u8> {
    format!(r#"{{"name":"{title}","published_at":"2026-11-05T01:09:08Z"}}"#).into_bytes()
}

/// A fake GitHub serving `rel`; every URL asked for is recorded in `calls`.
fn serve<'a>(rel: &'a Release, calls: &'a RefCell<Vec<String>>) -> Box<Fetch<'a>> {
    Box::new(move |url| {
        calls.borrow_mut().push(url.to_string());
        Ok(Some(if url.contains("/releases/tags/") {
            api(TITLE)
        } else if url.ends_with("SHA256SUMS.sig") {
            rel.sig.clone().into_bytes()
        } else if url.ends_with("SHA256SUMS") {
            rel.sums.clone().into_bytes()
        } else if url.ends_with(asset_name()) {
            rel.archive.clone()
        } else {
            return Ok(None);
        }))
    })
}

fn update(s: &Store, key: &SigningKey, f: &Fetch) -> Result<PathBuf, String> {
    ensure_latest_after(s, &[key.public()], f, Duration::ZERO)
}

fn install(s: &Store, key: &SigningKey, v: &str) -> PathBuf {
    let r = release(key, "nightly", good(v));
    s.install(v, &r.archive, r.sums.as_bytes(), &r.sig, &[key.public()])
        .unwrap()
}

fn key() -> SigningKey {
    SigningKey::generate().unwrap().0
}

fn builds(s: &Store) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(s.root.join("builds"))
        .map(|d| {
            d.map(|e| e.unwrap().file_name().to_string_lossy().into())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn installs_signed_archive_and_strips_the_root_folder() {
    let (s, k) = (store(), key());
    let r = release(&k, "nightly", good("one"));
    let dir = update(&s, &k, &*serve(&r, &RefCell::default())).unwrap();
    assert_eq!(dir, s.root.join("builds").join(V1));
    let exe = std::fs::read_to_string(dir.join("horizon-client.exe")).unwrap();
    assert_eq!(exe, "one");
    assert!(dir.join("tools/a.txt").exists());
    assert!(!dir.join("horizon-windows-x64").exists());
    assert_eq!(s.current(), Some(dir));
    assert_eq!(builds(&s), [V1]);
}

#[test]
fn bad_signature_keeps_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, "old");
    let r = release(&key(), "nightly", good("new"));
    assert_eq!(update(&s, &k, &*serve(&r, &RefCell::default())), Ok(cur));
    assert_eq!(builds(&s), ["old"]);
}

#[test]
fn tampered_archive_keeps_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, "old");
    let mut r = release(&k, "nightly", good("new"));
    r.archive = good("evil");
    assert_eq!(update(&s, &k, &*serve(&r, &RefCell::default())), Ok(cur));
    assert_eq!(builds(&s), ["old"]);
}

#[test]
fn signature_for_other_channel_refused() {
    let (s, k) = (store(), key());
    let r = release(&k, "v1", good("new"));
    assert!(update(&s, &k, &*serve(&r, &RefCell::default())).is_err());
    assert!(s.current().is_none());
    assert!(builds(&s).is_empty());
}

#[test]
fn zip_slip_refused() {
    let (s, k) = (store(), key());
    let evil = zip_of(&[
        ("horizon-windows-x64/../../evil.txt", "x"),
        ("horizon-windows-x64/horizon-client.exe", "x"),
    ]);
    let r = release(&k, "nightly", evil);
    let err = s.install(V1, &r.archive, r.sums.as_bytes(), &r.sig, &[k.public()]);
    assert!(err.is_err());
    assert!(!s.root.join("evil.txt").exists());
    assert!(s.current().is_none());
    assert!(builds(&s).is_empty(), "{:?}", builds(&s));
}

#[test]
fn keeps_only_two_builds() {
    let (s, k) = (store(), key());
    for v in ["a", "b", "c"] {
        install(&s, &k, v);
    }
    assert_eq!(builds(&s), ["b", "c"]);
    assert_eq!(s.current(), Some(s.root.join("builds/c")));
}

#[test]
fn network_error_returns_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, "old");
    assert_eq!(update(&s, &k, &|_| Err("offline".into())), Ok(cur));
    let log = std::fs::read_to_string(s.root.join("logs/bridge.log")).unwrap();
    assert!(log.contains("offline"));
}

#[test]
fn network_error_without_build_is_error() {
    let (s, k) = (store(), key());
    assert!(update(&s, &k, &|_| Err("offline".into())).is_err());
}

#[test]
fn same_version_does_not_download() {
    let (s, k) = (store(), key());
    let r = release(&k, "nightly", good("one"));
    let calls = RefCell::default();
    update(&s, &k, &*serve(&r, &calls)).unwrap();
    calls.borrow_mut().clear();
    update(&s, &k, &*serve(&r, &calls)).unwrap();
    assert_eq!(calls.borrow().len(), 1, "{:?}", calls.borrow());
    assert!(calls.borrow()[0].contains("api.github.com"));
}

#[test]
fn pinned_version_does_not_update() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, "old");
    s.set_pinned(Some("old")).unwrap();
    assert_eq!(s.pinned().as_deref(), Some("old"));
    let calls = RefCell::default();
    let r = release(&k, "nightly", good("new"));
    assert_eq!(update(&s, &k, &*serve(&r, &calls)), Ok(cur));
    assert!(calls.borrow().is_empty());
    s.set_pinned(None).unwrap();
    assert_eq!(s.pinned(), None);
}

#[test]
fn a_mismatching_window_keeps_current_and_retries_once() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, "old");
    let new = release(&k, "nightly", good("new"));
    let stale = release(&k, "nightly", good("stale"));

    // First try: the new archive is up but the sums are still the previous build's.
    let tries = Cell::new(0);
    let f = |url: &str| -> Result<Option<Vec<u8>>, String> {
        let first = tries.get() == 0;
        if url.ends_with(asset_name()) {
            tries.set(tries.get() + 1);
        }
        let sums_from = if first { &stale } else { &new };
        let mut out = serve(sums_from, &RefCell::default())(url)?;
        if url.ends_with(asset_name()) {
            out = Some(new.archive.clone());
        }
        Ok(out)
    };
    let dir = update(&s, &k, &f).unwrap();
    assert_ne!(dir, cur);
    assert_eq!(tries.get(), 2);
    let exe = std::fs::read_to_string(dir.join("horizon-client.exe")).unwrap();
    assert_eq!(exe, "new");

    // A window that never closes: the current build, after exactly one retry.
    let s2 = store();
    let cur2 = install(&s2, &k, "old");
    let n = Cell::new(0);
    let g = |url: &str| -> Result<Option<Vec<u8>>, String> {
        if url.ends_with(asset_name()) {
            n.set(n.get() + 1);
            return Ok(Some(new.archive.clone()));
        }
        serve(&stale, &RefCell::default())(url)
    };
    assert_eq!(update(&s2, &k, &g), Ok(cur2));
    assert_eq!(n.get(), 2);
}
