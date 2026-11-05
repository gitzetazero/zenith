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
        v.push(PathBuf::from(h));
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        if dir.join("java.exe").is_file()
            && let Some(home) = dir.parent()
        {
            v.push(home.to_path_buf());
        }
    }
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

fn lower(p: &Path) -> String {
    p.to_string_lossy().replace('/', "\\").to_lowercase()
}

/// A usable Java 25+ home. `shim_dir` (`<home>\java`) is never probed: that is the bridge itself.
/// Order: Zenith's own runtime, `candidates`, then Mojang's runtime downloaded into
/// `<home>/java-runtime/<component>`. Call it only for an owner.
pub fn find_or_install(
    home: &Path,
    component: &str,
    candidates: &[PathBuf],
    fetch: Fetch,
    probe: &dyn Fn(&Path) -> Option<u32>,
) -> Result<PathBuf, String> {
    let shim = lower(&home.join("java"));
    let ok = |p: &Path| {
        let l = lower(p);
        !(l == shim || l.starts_with(&format!("{shim}\\")))
            && p.join("bin").join("java.exe").is_file()
            && probe(p).is_some_and(|m| m >= MIN_MAJOR)
    };
    let own = home.join("java-runtime").join(component);
    if ok(&own) {
        return Ok(own);
    }
    if let Some(c) = candidates.iter().find(|c| ok(c)) {
        return Ok(c.clone());
    }
    install(&own, component, fetch)?;
    if ok(&own) {
        Ok(own)
    } else {
        Err(format!(
            "El Java descargado de Mojang ({component}) no arranca o no es Java {MIN_MAJOR} o superior."
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
            "descarga rechazada: {url:?} no es una URL https de Mojang"
        )),
    }
}

fn get_checked(fetch: Fetch, url: &str, sha1: &str) -> Result<Vec<u8>, String> {
    allowed(url)?;
    let body = fetch(url)?.ok_or_else(|| format!("{url}: no encontrado"))?;
    if sha1_hex(&body) != sha1 {
        return Err(format!("{url}: sha1 no coincide"));
    }
    Ok(body)
}

fn str_of<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("el manifiesto de Mojang no tiene {key}"))
}

fn install(dest: &Path, component: &str, fetch: Fetch) -> Result<(), String> {
    allowed(INDEX)?;
    let index: Value =
        serde_json::from_slice(&fetch(INDEX)?.ok_or("el índice de Java de Mojang no existe")?)
            .map_err(|e| format!("índice de Java de Mojang: {e}"))?;
    let m = &index[PLATFORM][component][0]["manifest"];
    if m.is_null() {
        return Err(format!("Mojang no tiene {component} para {PLATFORM}"));
    }
    let manifest: Value =
        serde_json::from_slice(&get_checked(fetch, str_of(m, "url")?, str_of(m, "sha1")?)?)
            .map_err(|e| format!("manifiesto de {component}: {e}"))?;
    let files = manifest["files"]
        .as_object()
        .ok_or("el manifiesto de Java no lista archivos")?;
    // check every entry before writing anything
    let mut todo = Vec::new();
    for (name, f) in files {
        if f["type"] != "file" {
            continue;
        }
        let ok = !name.contains('\\')
            && Path::new(name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !ok {
            return Err(format!("ruta insegura {name:?} en el manifiesto de Java"));
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
        let dir = path.parent().ok_or("ruta sin carpeta")?;
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let tmp = path.with_extension("part");
        std::fs::write(&tmp, body)
            .and_then(|_| std::fs::rename(&tmp, &path))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}
