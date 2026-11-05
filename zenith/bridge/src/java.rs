//! A Java 25 for `horizon-prepare`: an installed one, else Mojang's runtime (as Horizon's launcher fetches it).

use crate::prepare::sha1_hex;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

pub type Fetch<'a> = &'a dyn Fn(&str) -> Result<Option<Vec<u8>>, String>;

const INDEX: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
const PLATFORM: &str = "windows-x64"; // Windows only for now
const HOSTS: [&str; 3] = [
    "launchermeta.mojang.com",
    "piston-meta.mojang.com",
    "piston-data.mojang.com",
];
pub const MIN_MAJOR: u32 = 25;

/// The major in `java -version` output (`openjdk version "25.0.5" ...`, `java version "1.8.0"` is 8).
pub fn parse_major(text: &str) -> Option<u32> {
    let rest = &text[text.find("version \"")? + 9..];
    let v = rest.split('"').next()?;
    let mut parts = v.split(['.', '_', '-', '+']);
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

/// Runs `<home>/bin/java.exe -version`.
pub fn probe_major(home: &Path) -> Option<u32> {
    let mut c = Command::new(home.join("bin").join("java.exe"));
    let out = crate::play::hide_console(c.arg("-version"))
        .stdin(Stdio::null())
        .output()
        .ok()?;
    parse_major(&format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    ))
}

/// Java homes worth probing on this machine: JAVA_HOME, PATH, the usual install folders.
pub fn system_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(h) = std::env::var_os("JAVA_HOME") {
        v.extend(absolute(PathBuf::from(h)));
    }
    v.extend(path_candidates(
        &std::env::var_os("PATH").unwrap_or_default(),
        &|p| p.is_file(),
    ));
    for pf in ["ProgramFiles", "ProgramW6432"] {
        let Some(pf) = std::env::var_os(pf) else {
            continue;
        };
        for vendor in ["Eclipse Adoptium", "Java", "Microsoft", "Zulu", "BellSoft"] {
            for e in std::fs::read_dir(Path::new(&pf).join(vendor))
                .into_iter()
                .flatten()
                .flatten()
            {
                v.push(e.path());
            }
        }
    }
    v
}

/// The path without surrounding quotes, only if absolute: a relative one would run whatever the
/// current folder holds.
fn absolute(p: PathBuf) -> Option<PathBuf> {
    let s = p.to_string_lossy();
    let q = PathBuf::from(s.trim().trim_matches('"'));
    q.is_absolute().then_some(q)
}

/// Java homes for the absolute `PATH` entries that hold a `java.exe` (`is_file` is injected for tests).
pub fn path_candidates(path: &std::ffi::OsStr, is_file: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    std::env::split_paths(path)
        .filter_map(absolute)
        .filter(|d| is_file(&d.join("java.exe")))
        .filter_map(|d| d.parent().map(Path::to_path_buf))
        .collect()
}

/// A Mojang runtime component name such as `java-runtime-epsilon`; it becomes a folder name.
pub fn valid_component(c: &str) -> bool {
    !c.is_empty()
        && c != "."
        && !c.contains("..")
        && c.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn lower(p: &Path) -> String {
    p.to_string_lossy().replace('/', "\\").to_lowercase()
}

fn usable(home: &Path, p: &Path, probe: &dyn Fn(&Path) -> Option<u32>) -> bool {
    // `<home>\java` holds the bridge itself: never run it as a probe
    let shim = lower(&home.join("java"));
    let l = lower(p);
    !(l == shim || l.starts_with(&format!("{shim}\\")))
        && p.join("bin").join("java.exe").is_file()
        && probe(p).is_some_and(|m| m >= MIN_MAJOR)
}

/// A usable Java 25+ home that is already on this machine: Zenith's own runtime, then `candidates`.
pub fn find_installed(
    home: &Path,
    component: &str,
    candidates: &[PathBuf],
    probe: &dyn Fn(&Path) -> Option<u32>,
) -> Option<PathBuf> {
    if !valid_component(component) {
        return None;
    }
    std::iter::once(home.join("java-runtime").join(component))
        .chain(candidates.iter().cloned())
        .find(|c| usable(home, c, probe))
}

/// [`find_installed`], else Mojang's runtime downloaded into `<home>/java-runtime/<component>`.
/// Call it only for an owner.
pub fn find_or_install(
    home: &Path,
    component: &str,
    candidates: &[PathBuf],
    fetch: Fetch,
    probe: &dyn Fn(&Path) -> Option<u32>,
) -> Result<PathBuf, String> {
    if !valid_component(component) {
        return Err(format!("invalid Java component name: {component:?}"));
    }
    if let Some(j) = find_installed(home, component, candidates, probe) {
        return Ok(j);
    }
    let own = home.join("java-runtime").join(component);
    install(&own, component, fetch)?;
    if usable(home, &own, probe) {
        Ok(own)
    } else {
        Err(format!(
            "The Java downloaded from Mojang ({component}) does not start or is not Java {MIN_MAJOR} or newer."
        ))
    }
}

fn allowed(url: &str) -> Result<(), String> {
    let host = url
        .strip_prefix("https://")
        .map(|r| r.split(['/', '?', '#']).next().unwrap_or(""));
    match host {
        Some(h) if HOSTS.contains(&h) => Ok(()),
        _ => Err(format!(
            "download refused: {url:?} is not an https URL of Mojang"
        )),
    }
}

fn get_checked(fetch: Fetch, url: &str, sha1: &str) -> Result<Vec<u8>, String> {
    allowed(url)?;
    let body = fetch(url)?.ok_or_else(|| format!("{url}: not found"))?;
    if sha1_hex(&body) != sha1 {
        return Err(format!("{url}: sha1 does not match"));
    }
    Ok(body)
}

fn str_of<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("Mojang's manifest has no {key}"))
}

fn install(dest: &Path, component: &str, fetch: Fetch) -> Result<(), String> {
    allowed(INDEX)?;
    let index: Value =
        serde_json::from_slice(&fetch(INDEX)?.ok_or("Mojang's Java index does not exist")?)
            .map_err(|e| format!("Mojang's Java index: {e}"))?;
    let m = &index[PLATFORM][component][0]["manifest"];
    if m.is_null() {
        return Err(format!("Mojang has no {component} for {PLATFORM}"));
    }
    let manifest: Value =
        serde_json::from_slice(&get_checked(fetch, str_of(m, "url")?, str_of(m, "sha1")?)?)
            .map_err(|e| format!("manifiesto de {component}: {e}"))?;
    let files = manifest["files"]
        .as_object()
        .ok_or("the Java manifest lists no files")?;
    // check every entry before writing anything
    let mut todo = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (name, f) in files {
        if f["type"] != "file" {
            continue;
        }
        // NTFS: `name:stream` writes a hidden stream; names differing only by case are one file
        if name.contains(':') || !seen.insert(name.to_lowercase()) {
            return Err(format!(
                "invalid or repeated file name {name:?} in the Java manifest"
            ));
        }
        let ok = !name.contains('\\')
            && Path::new(name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !ok {
            return Err(format!("unsafe path {name:?} in the Java manifest"));
        }
        let raw = &f["downloads"]["raw"];
        let (url, sha1) = (str_of(raw, "url")?, str_of(raw, "sha1")?);
        allowed(url)?;
        todo.push((dest.join(name), url, sha1));
    }
    for (path, url, sha1) in todo {
        if std::fs::read(&path).is_ok_and(|b| sha1_hex(&b) == sha1) {
            continue;
        }
        let body = get_checked(fetch, url, sha1)?;
        let dir = path.parent().ok_or("path without a folder")?;
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
        tmp_name.push(".part");
        let tmp = path.with_file_name(tmp_name);
        std::fs::write(&tmp, body)
            .and_then(|_| std::fs::rename(&tmp, &path))
            .map_err(|e| {
                let _ = std::fs::remove_file(&tmp);
                format!("{}: {e}", path.display())
            })?;
    }
    Ok(())
}
