// The Windows build is the one that ships first (zip); the Linux .tar.gz is not installed yet.
#![cfg(windows)]

use horizon_release_sign::{SigningKey, sha256_hex, sums_file};
use std::cell::{Cell, RefCell};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use zenith_bridge::builds::{MIN_BUILT, STAMP, Stamp, Store, asset_name, ensure_latest_after};

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

/// Build number `n`: built `n` seconds past midnight of 2026-11-05, with the commit `n` repeated.
fn stamp_of(n: u32) -> String {
    format!(
        "commit={}\nbuilt=2026-11-05T03:{:02}:{:02}Z\n",
        format!("{n:02x}").repeat(20),
        n / 60,
        n % 60
    )
}

/// The version build `n` installs as.
fn ver(n: u32) -> String {
    format!("nightly-2026-11-05-{}", &format!("{n:02x}").repeat(4)[..7])
}

struct Release {
    archive: Vec<u8>,
    stamp: Vec<u8>,
    sums: String,
    sig: String,
}

/// `listed` is the stamp the signed sums describe, `stamp` the one served.
fn release_with(
    key: &SigningKey,
    channel: &str,
    archive: Vec<u8>,
    stamp: &str,
    listed: Option<&str>,
) -> Release {
    let mut files = vec![
        (asset_name().to_string(), sha256_hex(&archive)),
        ("horizon-linux-x86_64.tar.gz".into(), sha256_hex(b"linux")),
    ];
    if let Some(listed) = listed {
        files.push((STAMP.into(), sha256_hex(listed.as_bytes())));
    }
    let sums = sums_file(&files);
    let sig = key.sign(channel, sums.as_bytes());
    Release {
        archive,
        stamp: stamp.as_bytes().to_vec(),
        sums,
        sig,
    }
}

fn release(key: &SigningKey, channel: &str, archive: Vec<u8>, n: u32) -> Release {
    let stamp = stamp_of(n);
    release_with(key, channel, archive, &stamp, Some(&stamp))
}

/// A fake GitHub serving `rel`; every URL asked for is recorded in `calls`.
fn serve<'a>(rel: &'a Release, calls: &'a RefCell<Vec<String>>) -> Box<Fetch<'a>> {
    Box::new(move |url| {
        calls.borrow_mut().push(url.to_string());
        Ok(Some(if url.ends_with("SHA256SUMS.sig") {
            rel.sig.clone().into_bytes()
        } else if url.ends_with("SHA256SUMS") {
            rel.sums.clone().into_bytes()
        } else if url.ends_with(asset_name()) {
            rel.archive.clone()
        } else if url.ends_with(STAMP) {
            rel.stamp.clone()
        } else {
            return Ok(None);
        }))
    })
}

fn update(s: &Store, key: &SigningKey, f: &Fetch) -> Result<PathBuf, String> {
    ensure_latest_after(s, &[key.public()], f, Duration::ZERO)
}

fn install_r(s: &Store, key: &SigningKey, r: &Release) -> Result<PathBuf, String> {
    s.install(
        &r.archive,
        &r.stamp,
        r.sums.as_bytes(),
        &r.sig,
        &[key.public()],
    )
}

/// Installs build `n` (its exe says `tag`) directly.
fn install(s: &Store, key: &SigningKey, n: u32, tag: &str) -> PathBuf {
    install_r(s, key, &release(key, "nightly", good(tag), n)).unwrap()
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

fn log(s: &Store) -> String {
    std::fs::read_to_string(s.root.join("logs/bridge.log")).unwrap_or_default()
}

fn exe(dir: &std::path::Path) -> String {
    std::fs::read_to_string(dir.join("horizon-client.exe")).unwrap()
}

#[test]
fn installs_signed_archive_and_strips_the_root_folder() {
    let (s, k) = (store(), key());
    let r = release(&k, "nightly", good("one"), 1);
    let dir = update(&s, &k, &*serve(&r, &RefCell::default())).unwrap();
    assert_eq!(dir, s.root.join("builds").join(ver(1)));
    assert_eq!(exe(&dir), "one");
    assert!(dir.join("tools/a.txt").exists());
    assert!(!dir.join("horizon-windows-x64").exists());
    // The signed stamp is kept inside the build.
    assert_eq!(
        std::fs::read_to_string(dir.join(STAMP)).unwrap(),
        stamp_of(1)
    );
    assert_eq!(s.current(), Some(dir));
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn bad_signature_keeps_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    let r = release(&key(), "nightly", good("new"), 2);
    assert_eq!(update(&s, &k, &*serve(&r, &RefCell::default())), Ok(cur));
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn tampered_archive_keeps_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    let mut r = release(&k, "nightly", good("new"), 2);
    r.archive = good("evil");
    assert_eq!(update(&s, &k, &*serve(&r, &RefCell::default())), Ok(cur));
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn signature_for_other_channel_refused() {
    let (s, k) = (store(), key());
    let r = release(&k, "v1", good("new"), 1);
    assert!(update(&s, &k, &*serve(&r, &RefCell::default())).is_err());
    assert!(s.current().is_none());
    assert!(builds(&s).is_empty());
}

#[test]
fn newer_stamp_is_installed() {
    let (s, k) = (store(), key());
    install(&s, &k, 1, "old");
    let r = release(&k, "nightly", good("new"), 2);
    let dir = update(&s, &k, &*serve(&r, &RefCell::default())).unwrap();
    assert_eq!(dir, s.root.join("builds").join(ver(2)));
    assert_eq!(exe(&dir), "new");
    assert_eq!(builds(&s), [ver(1), ver(2)]);
}

#[test]
fn older_stamp_is_a_downgrade_and_refused() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 5, "five");
    let r = release(&k, "nightly", good("four"), 4);
    let calls = RefCell::default();
    assert_eq!(update(&s, &k, &*serve(&r, &calls)), Ok(cur.clone()));
    assert!(log(&s).contains("downgrade refused"), "{}", log(&s));
    assert!(!calls.borrow().iter().any(|u| u.ends_with(asset_name())));
    assert_eq!(builds(&s), [ver(5)]);
    assert_eq!(s.current(), Some(cur));
    // Installing it directly is refused too.
    let err = install_r(&s, &k, &r).unwrap_err();
    assert!(err.contains("downgrade refused"), "{err}");
}

#[test]
fn same_stamp_downloads_no_archive() {
    let (s, k) = (store(), key());
    let r = release(&k, "nightly", good("one"), 1);
    let calls = RefCell::default();
    let first = update(&s, &k, &*serve(&r, &calls)).unwrap();
    calls.borrow_mut().clear();
    assert_eq!(update(&s, &k, &*serve(&r, &calls)), Ok(first));
    assert!(
        !calls.borrow().iter().any(|u| u.ends_with(asset_name())),
        "{:?}",
        calls.borrow()
    );
    assert!(!log(&s).contains("downgrade"));
}

#[test]
fn stamp_missing_from_the_signed_sums_is_refused() {
    let (s, k) = (store(), key());
    let r = release_with(&k, "nightly", good("new"), &stamp_of(1), None);
    assert!(update(&s, &k, &*serve(&r, &RefCell::default())).is_err());
    assert!(install_r(&s, &k, &r).unwrap_err().contains(STAMP));
    assert!(s.current().is_none());
    assert!(builds(&s).is_empty());
}

#[test]
fn tampered_stamp_is_refused() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    // Signed sums describe build 2; a newer-looking stamp is served instead.
    let r = release_with(&k, "nightly", good("new"), &stamp_of(9), Some(&stamp_of(2)));
    assert_eq!(update(&s, &k, &*serve(&r, &RefCell::default())), Ok(cur));
    assert!(
        install_r(&s, &k, &r)
            .unwrap_err()
            .contains("signed checksum")
    );
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn malformed_stamps_are_refused() {
    let good = stamp_of(1);
    let c = "ab".repeat(20);
    let bad = [
        String::new(),
        "commit=abc\nbuilt=2026-11-05T00:00:01Z\n".to_string(),
        format!("built=2026-11-05T00:00:01Z\ncommit={c}\n"),
        format!("commit={c}\nbuilt=2026-11-05 00:00:01\n"),
        format!("commit={c}\nbuilt=2026-11-05T00:00:01Z"),
        format!("{good}extra=1\n"),
        format!("commit={}\nbuilt=2026-11-05T00:00:01Z\n", c.to_uppercase()),
        good.replace('\n', "\r\n"),
    ];
    for text in bad {
        let (s, k) = (store(), key());
        let r = release_with(&k, "nightly", good_zip(), &text, Some(&text));
        assert!(
            install_r(&s, &k, &r).unwrap_err().contains("malformed"),
            "{text:?}"
        );
        assert!(update(&s, &k, &*serve(&r, &RefCell::default())).is_err());
        assert!(builds(&s).is_empty());
    }
}

fn good_zip() -> Vec<u8> {
    good("x")
}

fn refused(entry: &str) {
    let (s, k) = (store(), key());
    let evil = zip_of(&[
        (entry, "x"),
        ("horizon-windows-x64/horizon-client.exe", "x"),
    ]);
    let r = release(&k, "nightly", evil, 1);
    let err = install_r(&s, &k, &r).unwrap_err();
    assert!(err.contains("unsafe path"), "{entry}: {err}");
    assert!(s.current().is_none());
    assert!(builds(&s).is_empty(), "{entry}: {:?}", builds(&s));
    assert!(!s.root.join("evil.txt").exists());
}

#[test]
fn zip_slip_refused() {
    for entry in [
        "horizon-windows-x64/../../evil.txt",
        "/evil.txt",
        "\\evil.txt",
        "horizon-windows-x64\\..\\..\\evil.txt",
        "..\\evil.txt",
        "C:evil.txt",
        "C:\\evil.txt",
        "horizon-windows-x64/a.txt:stream",
        "horizon-windows-x64/NUL",
        "horizon-windows-x64/con.txt",
        "horizon-windows-x64/COM1",
        "horizon-windows-x64/lpt9.log",
        "horizon-windows-x64/trailing.",
    ] {
        refused(entry);
    }
}

#[test]
fn keeps_only_two_builds() {
    let (s, k) = (store(), key());
    for n in [1, 2, 3] {
        install(&s, &k, n, "x");
    }
    assert_eq!(builds(&s), [ver(2), ver(3)]);
    assert_eq!(s.current(), Some(s.root.join("builds").join(ver(3))));
}

#[test]
fn a_pinned_build_survives_pruning() {
    let (s, k) = (store(), key());
    install(&s, &k, 1, "x");
    s.set_pinned(Some(&ver(1))).unwrap();
    install(&s, &k, 2, "x");
    install(&s, &k, 3, "x");
    assert_eq!(builds(&s), [ver(1), ver(2), ver(3)]);
}

#[test]
fn installing_the_current_or_pinned_build_again_leaves_it_alone() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "first");
    std::fs::write(cur.join("marker"), "m").unwrap();
    // Same stamp, different archive: its folder is the one in use, so it is not replaced.
    let r = release(&k, "nightly", good("second"), 1);
    assert_eq!(install_r(&s, &k, &r), Ok(cur.clone()));
    assert!(cur.join("marker").exists());
    assert_eq!(exe(&cur), "first");
    assert_eq!(s.current(), Some(cur));
}

#[test]
fn a_failed_install_leaves_current_alone() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    let no_client = zip_of(&[("horizon-windows-x64/other.exe", "x")]);
    let r = release(&k, "nightly", no_client, 2);
    assert!(install_r(&s, &k, &r).is_err());
    assert_eq!(s.current(), Some(cur));
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn stale_tmp_folders_are_cleaned() {
    let (s, k) = (store(), key());
    install(&s, &k, 1, "old");
    let stale = s.root.join("builds").join(format!("{}.tmp", ver(2)));
    std::fs::create_dir_all(&stale).unwrap();
    std::fs::write(stale.join("junk"), "x").unwrap();
    // An install of the same version starts over; a later one prunes any other leftovers.
    install(&s, &k, 2, "new");
    assert_eq!(builds(&s), [ver(1), ver(2)]);
    let other = s.root.join("builds").join("leftover.tmp");
    std::fs::create_dir_all(&other).unwrap();
    install(&s, &k, 3, "newer");
    assert_eq!(builds(&s), [ver(2), ver(3)]);
}

#[test]
fn invalid_current_or_pinned_files_are_ignored() {
    let (s, k) = (store(), key());
    install(&s, &k, 1, "old");
    std::fs::write(s.root.join("current.txt"), "..\\..\\x").unwrap();
    assert_eq!(s.current(), None);
    std::fs::write(s.root.join("pinned.txt"), "../escape").unwrap();
    assert_eq!(s.pinned(), None);
}

#[test]
fn network_error_returns_current() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    assert_eq!(update(&s, &k, &|_| Err("offline".into())), Ok(cur));
    assert!(log(&s).contains("offline"));
}

#[test]
fn network_error_without_build_is_error() {
    let (s, k) = (store(), key());
    assert!(update(&s, &k, &|_| Err("offline".into())).is_err());
}

#[test]
fn pinned_version_does_not_update() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    s.set_pinned(Some(&ver(1))).unwrap();
    assert_eq!(s.pinned(), Some(ver(1)));
    let calls = RefCell::default();
    let r = release(&k, "nightly", good("new"), 2);
    assert_eq!(update(&s, &k, &*serve(&r, &calls)), Ok(cur));
    assert!(calls.borrow().is_empty());
    s.set_pinned(None).unwrap();
    assert_eq!(s.pinned(), None);
}

#[test]
fn offline_fails_after_one_try_saying_so() {
    let s = store();
    let n = Cell::new(0);
    let f = |_: &str| -> Result<Option<Vec<u8>>, String> {
        n.set(n.get() + 1);
        Err(format!(
            "{} or GitHub is not answering (dns)",
            zenith_bridge::builds::OFFLINE
        ))
    };
    let e = update(&s, &key(), &f).unwrap_err();
    assert!(e.starts_with("no internet connection"), "{e}");
    assert_eq!(n.get(), 1);
}

#[test]
fn a_mismatching_window_keeps_current_and_retries_once() {
    let (s, k) = (store(), key());
    let cur = install(&s, &k, 1, "old");
    let new = release(&k, "nightly", good("new"), 3);
    let stale = release(&k, "nightly", good("stale"), 2);

    // First try: the new archive and stamp are up but the sums are still the previous build's.
    let tries = Cell::new(0);
    let f = |url: &str| -> Result<Option<Vec<u8>>, String> {
        let first = tries.get() == 0;
        if url.ends_with(STAMP) {
            tries.set(tries.get() + 1);
        }
        let sums_from = if first { &stale } else { &new };
        let mut out = serve(sums_from, &RefCell::default())(url)?;
        if url.ends_with(asset_name()) {
            out = Some(new.archive.clone());
        }
        if url.ends_with(STAMP) {
            out = Some(new.stamp.clone());
        }
        Ok(out)
    };
    let dir = update(&s, &k, &f).unwrap();
    assert_ne!(dir, cur);
    assert_eq!(tries.get(), 2);
    assert_eq!(exe(&dir), "new");

    // A window that never closes: the current build, after exactly one retry.
    let s2 = store();
    let cur2 = install(&s2, &k, 1, "old");
    let n = Cell::new(0);
    let g = |url: &str| -> Result<Option<Vec<u8>>, String> {
        if url.ends_with(STAMP) {
            n.set(n.get() + 1);
            return Ok(Some(new.stamp.clone()));
        }
        serve(&stale, &RefCell::default())(url)
    };
    assert_eq!(update(&s2, &k, &g), Ok(cur2));
    assert_eq!(n.get(), 2);
}

/// Against the real GitHub Release: `ZENITH_LIVE_TEST=1 cargo test --test builds -- --ignored live`.
/// Downloads ~40 MB into a temporary ZENITH_HOME.
#[test]
#[ignore]
fn live_release_install() {
    if std::env::var("ZENITH_LIVE_TEST").as_deref() != Ok("1") {
        eprintln!("ZENITH_LIVE_TEST=1 not set: skipped");
        return;
    }
    let s = store();
    let keys = zenith_bridge::builds::embedded_keys();
    let t = std::time::Instant::now();
    let dir =
        zenith_bridge::builds::ensure_latest(&s, &keys, zenith_bridge::builds::http_fetch).unwrap();
    eprintln!("first call: {:?}, {}", t.elapsed(), dir.display());
    assert!(dir.join("horizon-client.exe").is_file());
    let stamp = s.installed_stamp().expect("the stamp parses");
    eprintln!("stamp: {stamp:?}");
    assert_eq!(dir.file_name().unwrap().to_string_lossy(), stamp.version());

    let t = std::time::Instant::now();
    let downloads = Cell::new(0);
    let again = zenith_bridge::builds::ensure_latest(&s, &keys, |u| {
        if u.ends_with(asset_name()) {
            downloads.set(downloads.get() + 1);
        }
        zenith_bridge::builds::http_fetch(u)
    })
    .unwrap();
    eprintln!("second call: {:?}", t.elapsed());
    assert_eq!(again, dir);
    assert_eq!(
        downloads.get(),
        0,
        "the second call downloaded the archive again"
    );
    let _ = std::fs::remove_dir_all(&s.root);
}

fn rebuilt(commit_n: u32, built: &str) -> String {
    format!(
        "commit={}\nbuilt={built}\n",
        format!("{commit_n:02x}").repeat(20)
    )
}

#[test]
fn same_commit_rebuilt_later_downloads_nothing_and_refreshes_the_stamp() {
    let (s, k) = (store(), key());
    let first = install(&s, &k, 1, "one");
    let later = rebuilt(1, "2026-11-05T09:00:00Z");
    let r = release_with(&k, "nightly", good("rebuild"), &later, Some(&later));
    for _ in 0..2 {
        let calls = RefCell::default();
        assert_eq!(update(&s, &k, &*serve(&r, &calls)), Ok(first.clone()));
        assert!(!calls.borrow().iter().any(|u| u.ends_with(asset_name())));
    }
    assert_eq!(std::fs::read_to_string(first.join(STAMP)).unwrap(), later);
    assert_eq!(exe(&first), "one");
    assert_eq!(builds(&s), [ver(1)]);
}

#[test]
fn stamp_times_are_validated() {
    let now = 1_793_864_000; // 2026-11-05T07:33:20Z
    let c = "ab".repeat(20);
    let at = |built: &str| Stamp::parse_at(&format!("commit={c}\nbuilt={built}\n"), now);
    for bad in [
        "9999-99-99T99:99:99Z",
        "2026-13-05T03:00:00Z",
        "2026-02-30T03:00:00Z",
        "2027-02-29T03:00:00Z",
        "2026-11-05T24:00:00Z",
        "2026-11-05T03:60:00Z",
        "2026-11-05T03:00:60Z",
        "2101-01-01T00:00:00Z",
        "2026-11-07T03:00:00Z", // +2 days
        "2026-11-05T02:56:38Z", // before MIN_BUILT
        "2026-01-01T00:00:00Z",
    ] {
        assert!(at(bad).is_err(), "{bad}");
    }
    for ok in [
        "2026-11-05T08:33:20Z", // +1 hour
        MIN_BUILT,
        "2028-02-29T03:00:00Z", // leap day, but in the future of this clock only if > 1 day
    ] {
        // the leap-day case is judged against a later clock
        let clock = if ok.starts_with("2028") {
            1_900_000_000
        } else {
            now
        };
        assert!(
            Stamp::parse_at(&format!("commit={c}\nbuilt={ok}\n"), clock).is_ok(),
            "{ok}"
        );
    }
}

#[test]
fn a_stamp_older_than_the_first_stamped_release_is_refused_on_a_fresh_install() {
    let (s, k) = (store(), key());
    let old = rebuilt(1, "2026-11-05T02:56:38Z");
    let r = release_with(&k, "nightly", good("old"), &old, Some(&old));
    assert!(update(&s, &k, &*serve(&r, &RefCell::default())).is_err());
    assert!(install_r(&s, &k, &r).is_err());
    assert!(s.current().is_none() && builds(&s).is_empty());
}
