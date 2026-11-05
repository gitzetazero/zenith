use std::cell::RefCell;
use std::path::{Path, PathBuf};
use zenith_bridge::java::{find_or_install, parse_major, path_candidates, valid_component};
use zenith_bridge::prepare::sha1_hex;

type Fetched = Result<Option<Vec<u8>>, String>;

fn tmp(tag: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("zenith-java-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fake_java(home: &Path) {
    std::fs::create_dir_all(home.join("bin")).unwrap();
    std::fs::write(home.join("bin/java.exe"), b"x").unwrap();
}

fn probe_is(m: u32) -> impl Fn(&Path) -> Option<u32> {
    move |_| Some(m)
}

const INDEX: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// A fake Mojang: index -> manifest -> files. The file entry can be made hostile.
fn mojang(file_name: &str, file_url: &str, body: &[u8], sha: String) -> impl Fn(&str) -> Fetched {
    let manifest = serde_json::json!({ "files": {
        file_name: { "type": "file", "downloads": { "raw": { "url": file_url, "sha1": sha } } },
        "bin/java.exe": { "type": "file", "downloads": { "raw": { "url": "https://piston-data.mojang.com/j", "sha1": sha1_hex(b"JAVA") } } },
    }})
    .to_string();
    let index = serde_json::json!({ "windows-x64": { "java-runtime-epsilon": [
        { "manifest": { "url": "https://piston-meta.mojang.com/m.json", "sha1": sha1_hex(manifest.as_bytes()) } }
    ]}})
    .to_string();
    let (file_url, body) = (file_url.to_string(), body.to_vec());
    move |u: &str| {
        Ok(Some(if u == INDEX {
            index.clone().into_bytes()
        } else if u == "https://piston-meta.mojang.com/m.json" {
            manifest.clone().into_bytes()
        } else if u == "https://piston-data.mojang.com/j" {
            b"JAVA".to_vec()
        } else if u == file_url {
            body.clone()
        } else {
            panic!("unexpected fetch {u}")
        }))
    }
}

#[test]
fn parses_java_version_output() {
    assert_eq!(
        parse_major("openjdk version \"25.0.5\" 2026-01-20 LTS\nx"),
        Some(25)
    );
    assert_eq!(parse_major("java version \"1.8.0_301\""), Some(8));
    assert_eq!(parse_major("openjdk version \"21\" 2023-09-19"), Some(21));
    assert_eq!(parse_major("hello"), None);
}

#[test]
fn own_runtime_then_candidates_in_order_without_network() {
    let home = tmp("order");
    let (cand, cand2) = (home.join("c1"), home.join("c2"));
    fake_java(&cand);
    fake_java(&cand2);
    let nofetch = |u: &str| -> Fetched { panic!("fetched {u}") };
    let got = find_or_install(
        &home,
        "java-runtime-epsilon",
        &[cand.clone(), cand2],
        &nofetch,
        &probe_is(25),
    )
    .unwrap();
    assert_eq!(got, cand);
    let own = home.join("java-runtime/java-runtime-epsilon");
    fake_java(&own);
    let got = find_or_install(
        &home,
        "java-runtime-epsilon",
        &[cand],
        &nofetch,
        &probe_is(25),
    )
    .unwrap();
    assert_eq!(got, own);
}

#[test]
fn old_java_is_not_accepted_and_the_shim_is_never_probed() {
    let home = tmp("shim");
    let shim_home = home.join("java"); // <home>\java\bin\java.exe is the bridge itself
    fake_java(&shim_home);
    let old = home.join("old");
    fake_java(&old);
    let probed = RefCell::new(Vec::new());
    let probe = |p: &Path| {
        probed.borrow_mut().push(p.to_path_buf());
        Some(if p == old { 21 } else { 25 })
    };
    let body = b"LIB";
    let f = mojang(
        "lib/x.dll",
        "https://piston-data.mojang.com/x",
        body,
        sha1_hex(body),
    );
    let got = find_or_install(
        &home,
        "java-runtime-epsilon",
        &[shim_home.clone(), old.clone()],
        &f,
        &probe,
    )
    .unwrap();
    assert_eq!(got, home.join("java-runtime/java-runtime-epsilon"));
    assert!(!probed.borrow().contains(&shim_home));
}

#[test]
fn downloads_mojang_runtime_and_verifies_hashes() {
    let home = tmp("dl");
    let body = b"LIB";
    let f = mojang(
        "lib/x.dll",
        "https://piston-data.mojang.com/x",
        body,
        sha1_hex(body),
    );
    let got = find_or_install(&home, "java-runtime-epsilon", &[], &f, &probe_is(25)).unwrap();
    assert_eq!(std::fs::read(got.join("lib/x.dll")).unwrap(), body);
    assert_eq!(std::fs::read(got.join("bin/java.exe")).unwrap(), b"JAVA");
}

#[test]
fn bad_hash_host_or_path_is_refused() {
    let body = b"LIB";
    let good = sha1_hex(body);
    let cases = [
        (
            "lib/x.dll",
            "https://piston-data.mojang.com/x",
            sha1_hex(b"other"),
        ),
        ("lib/x.dll", "https://evil.invalid/x", good.clone()),
        ("lib/x.dll", "http://piston-data.mojang.com/x", good.clone()),
        (
            "lib/x.dll",
            "https://piston-data.mojang.com@evil.invalid/x",
            good.clone(),
        ),
        ("../x.dll", "https://piston-data.mojang.com/x", good.clone()),
        ("C:/x.dll", "https://piston-data.mojang.com/x", good.clone()),
        ("a\\b.dll", "https://piston-data.mojang.com/x", good),
    ];
    for (name, url, sha) in cases {
        let home = tmp("bad");
        let inner = mojang(name, url, body, sha);
        // a hostile host must be refused before it is ever fetched
        let f = |u: &str| -> Fetched {
            assert!(
                u.starts_with("https://piston-") || u.starts_with("https://launchermeta."),
                "fetched {u}"
            );
            inner(u)
        };
        let r = find_or_install(&home, "java-runtime-epsilon", &[], &f, &probe_is(25));
        assert!(r.is_err(), "{name} {url}");
        assert!(!home.join("java-runtime/x.dll").exists());
    }
}

#[test]
fn relative_path_entries_are_never_probed() {
    let looked = RefCell::new(Vec::new());
    let is_file = |p: &Path| {
        looked.borrow_mut().push(p.to_path_buf());
        true
    };
    let abs = std::env::temp_dir().join("jdk25").join("bin");
    let path = std::env::join_paths([
        PathBuf::from("."),
        PathBuf::from("bin"),
        PathBuf::from(""),
        abs.clone(),
    ])
    .unwrap();
    let got = path_candidates(&path, &is_file);
    assert_eq!(got, vec![abs.parent().unwrap().to_path_buf()]);
    assert!(looked.borrow().iter().all(|p| p.is_absolute()));
}

#[test]
fn component_names_are_validated() {
    for ok in ["java-runtime-epsilon", "jre_1.2"] {
        assert!(valid_component(ok), "{ok}");
    }
    for bad in ["", ".", "..", "../x", "a/b", "a\\b", "C:x", "a b", "x..y"] {
        assert!(!valid_component(bad), "{bad:?}");
    }
    let home = tmp("comp");
    let nofetch = |u: &str| -> Fetched { panic!("fetched {u}") };
    let r = find_or_install(&home, "../evil", &[], &nofetch, &probe_is(25));
    assert!(r.is_err());
    assert!(!home.join("java-runtime").exists());
}

#[test]
fn colliding_or_stream_file_names_are_refused() {
    let body = b"LIB";
    for name in ["lib/x.dll:evil", "bin/JAVA.EXE"] {
        let home = tmp("names");
        let f = mojang(
            name,
            "https://piston-data.mojang.com/x",
            body,
            sha1_hex(body),
        );
        let r = find_or_install(&home, "java-runtime-epsilon", &[], &f, &probe_is(25));
        assert!(r.is_err(), "{name}");
    }
}

#[test]
fn failed_download_leaves_no_part_file() {
    let home = tmp("part");
    let f = mojang(
        "lib/x.dll",
        "https://piston-data.mojang.com/x",
        b"LIB",
        sha1_hex(b"other"),
    );
    assert!(find_or_install(&home, "java-runtime-epsilon", &[], &f, &probe_is(25)).is_err());
    let left: Vec<_> = walk(&home)
        .into_iter()
        .filter(|p| p.to_string_lossy().ends_with(".part"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

fn walk(d: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            v.extend(walk(&p));
        } else {
            v.push(p);
        }
    }
    v
}
