//! Signed Zenith builds: fetched from the `nightly` Release of the fork, verified against the
//! embedded keys, and unpacked under `<root>/builds/<version>/`.

use horizon_release_sign::{
    PublicKey, SIGNATURE, SUMS, check, nightly_digest, nightly_version, sha256_hex,
};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const REPO: &str = "gitzetazero/zenith";
const MAX_DOWNLOAD: u64 = 1 << 30;

/// The keys the nightly's checksums are signed with, built in.
pub fn embedded_keys() -> Vec<PublicKey> {
    horizon_release_sign::parse_keys(include_str!("../zenith-release-keys.txt")).unwrap_or_default()
}

pub fn asset_name() -> &'static str {
    if cfg!(windows) {
        "horizon-windows-x64.zip"
    } else {
        "horizon-linux-x86_64.tar.gz"
    }
}

pub fn api_url() -> String {
    format!("https://api.github.com/repos/{REPO}/releases/tags/nightly")
}

pub fn asset_url(name: &str) -> String {
    format!("https://github.com/{REPO}/releases/download/nightly/{name}")
}

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    /// `ZENITH_HOME`, else `%APPDATA%\zenith`.
    pub fn from_env() -> Store {
        let root = match std::env::var_os("ZENITH_HOME") {
            Some(h) => h.into(),
            None => PathBuf::from(std::env::var_os("APPDATA").unwrap_or_else(|| ".".into()))
                .join("zenith"),
        };
        Store { root }
    }

    pub fn builds(&self) -> PathBuf {
        self.root.join("builds")
    }

    fn read(&self, file: &str) -> Option<String> {
        let text = std::fs::read_to_string(self.root.join(file)).ok()?;
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    fn current_version(&self) -> Option<String> {
        self.read("current.txt")
    }

    /// The installed build in use, if its folder is still there.
    pub fn current(&self) -> Option<PathBuf> {
        let dir = self.builds().join(self.current_version()?);
        dir.is_dir().then_some(dir)
    }

    pub fn pinned(&self) -> Option<String> {
        self.read("pinned.txt")
    }

    pub fn set_pinned(&self, version: Option<&str>) -> Result<(), String> {
        let file = self.root.join("pinned.txt");
        match version {
            Some(v) => {
                valid_version(v)?;
                write_atomic(&file, v)
            }
            None => match std::fs::remove_file(&file) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
                _ => Ok(()),
            },
        }
    }

    /// A GUI-subsystem exe has no console, so warnings go to `<root>/logs/bridge.log`.
    pub fn log(&self, msg: &str) {
        let logs = self.root.join("logs");
        let _ = std::fs::create_dir_all(&logs);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs.join("bridge.log"))
        {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let _ = writeln!(f, "[{secs}] {msg}");
        }
    }

    /// Verifies `archive` against the nightly-signed `sums`, unpacks it as `version` and makes it
    /// the current build. Nothing is unpacked unless the signature and the digest check out.
    pub fn install(
        &self,
        version: &str,
        archive: &[u8],
        sums: &[u8],
        signature: &str,
        keys: &[PublicKey],
    ) -> Result<PathBuf, String> {
        valid_version(version)?;
        let asset = asset_name();
        let expected = nightly_digest(asset, Some(sums), Some(signature), keys)?;
        check(asset, &expected, &sha256_hex(archive))?;
        if !asset.ends_with(".zip") {
            // shortcut: Windows first; the Linux .tar.gz needs flate2+tar here, when Linux is wired up.
            return Err(format!("installing {asset} is not supported yet"));
        }

        let builds = self.builds();
        let tmp = builds.join(format!("{version}.tmp"));
        let dest = builds.join(version);
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let done = extract_zip(archive, &tmp).and_then(|()| {
            if !tmp.join("horizon-client.exe").is_file() {
                return Err("the archive has no horizon-client.exe".to_string());
            }
            let _ = std::fs::remove_dir_all(&dest);
            std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())
        });
        if let Err(e) = done {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(e);
        }

        let previous = self.current_version();
        write_atomic(&self.root.join("current.txt"), version)?;
        self.prune(&[Some(version), previous.as_deref(), self.pinned().as_deref()]);
        Ok(dest)
    }

    fn prune(&self, keep: &[Option<&str>]) {
        let Ok(dir) = std::fs::read_dir(self.builds()) else {
            return;
        };
        for entry in dir.flatten() {
            let name = entry.file_name();
            if !keep.contains(&name.to_str()) {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
}

// The version names a folder, so it can't be a path.
fn valid_version(v: &str) -> Result<(), String> {
    let ok = !v.is_empty()
        && !v.starts_with('.')
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        Ok(())
    } else {
        Err(format!("{v:?} is not a valid build version"))
    }
}

fn write_atomic(file: &Path, text: &str) -> Result<(), String> {
    let tmp = file.with_extension("tmp");
    let go = || -> std::io::Result<()> {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, file)
    };
    go().map_err(|e| format!("{}: {e}", file.display()))
}

/// Unpacks into `dest`, dropping the archive's single root folder. An entry that would land
/// outside `dest` fails the whole archive (zip-slip).
// shortcut: no cap on the unpacked size; the archive is already signed, so it is ours.
fn extract_zip(archive: &[u8], dest: &Path) -> Result<(), String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(archive)).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for i in 0..zip.len() {
        let f = zip.by_index(i).map_err(|e| e.to_string())?;
        let path = f
            .enclosed_name()
            .ok_or_else(|| format!("unsafe path in the archive: {}", f.name()))?;
        paths.push(path);
    }
    let root = paths.first().and_then(|p| p.components().next());
    let strip = root.is_some()
        && paths.iter().all(|p| p.components().next() == root)
        && paths.iter().any(|p| p.components().count() > 1);
    for (i, path) in paths.iter().enumerate() {
        let rel: PathBuf = if strip {
            path.components().skip(1).collect()
        } else {
            path.clone()
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(rel);
        let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(dir) = out.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The newest signed build, installed if it isn't yet. Any failure with a build already installed
/// keeps that build (a warning goes to the log); without one it is an error.
pub fn ensure_latest(
    store: &Store,
    keys: &[PublicKey],
    fetch: impl Fn(&str) -> Result<Option<Vec<u8>>, String>,
) -> Result<PathBuf, String> {
    ensure_latest_after(store, keys, fetch, Duration::from_secs(5))
}

/// [`ensure_latest`] with the wait before its one retry. The workflow replaces the archives,
/// then SHA256SUMS, then its signature, so for a moment they can disagree: that is "not ready".
pub fn ensure_latest_after(
    store: &Store,
    keys: &[PublicKey],
    fetch: impl Fn(&str) -> Result<Option<Vec<u8>>, String>,
    retry_delay: Duration,
) -> Result<PathBuf, String> {
    if let Some(pinned) = store.pinned() {
        let dir = store.builds().join(&pinned);
        return if dir.is_dir() {
            Ok(dir)
        } else {
            Err(format!("the pinned build {pinned} is not installed"))
        };
    }
    match update(store, keys, &fetch, retry_delay) {
        Ok(dir) => Ok(dir),
        Err(e) => match store.current() {
            Some(current) => {
                store.log(&format!(
                    "warning: no update ({e}); keeping the current build"
                ));
                Ok(current)
            }
            None => Err(e),
        },
    }
}

fn update(
    store: &Store,
    keys: &[PublicKey],
    fetch: &impl Fn(&str) -> Result<Option<Vec<u8>>, String>,
    retry_delay: Duration,
) -> Result<PathBuf, String> {
    let get =
        |url: String, what: &str| fetch(&url)?.ok_or_else(|| format!("{what} is not published"));
    let release: serde_json::Value =
        serde_json::from_slice(&get(api_url(), "the nightly release")?)
            .map_err(|e| e.to_string())?;
    let text = |k: &str| release[k].as_str().unwrap_or_default().to_string();
    let version = nightly_version(&text("name"), &text("published_at"));
    if store.current_version().as_deref() == Some(version.as_str()) {
        if let Some(current) = store.current() {
            return Ok(current);
        }
    }
    let mut last = String::new();
    for attempt in 0..2 {
        if attempt > 0 {
            std::thread::sleep(retry_delay);
        }
        let try_once = || -> Result<PathBuf, String> {
            let sums = get(asset_url(SUMS), SUMS)?;
            let sig = get(asset_url(SIGNATURE), SIGNATURE)?;
            let archive = get(asset_url(asset_name()), asset_name())?;
            let sig = String::from_utf8(sig).map_err(|_| format!("{SIGNATURE} is not text"))?;
            store.install(&version, &archive, &sums, &sig, keys)
        };
        match try_once() {
            Ok(dir) => return Ok(dir),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// The production fetch: a GET with the timeouts and User-Agent the bridge uses. 404 is `None`.
/// `GITHUB_TOKEN`, if set, goes only to the API host and is never logged.
pub fn http_fetch(url: &str) -> Result<Option<Vec<u8>>, String> {
    let agent = ureq::AgentBuilder::new()
        .https_only(true) // a redirect to plain http fails too
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .user_agent(concat!("zenith-bridge/", env!("CARGO_PKG_VERSION")))
        .build();
    let mut req = agent.get(url).set("Accept", "application/vnd.github+json");
    if url.starts_with("https://api.github.com/") {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if !token.is_empty() {
                req = req.set("Authorization", &format!("Bearer {token}"));
            }
        }
    }
    match req.call() {
        Ok(resp) => {
            let mut body = Vec::new();
            resp.into_reader()
                .take(MAX_DOWNLOAD + 1)
                .read_to_end(&mut body)
                .map_err(|e| e.to_string())?;
            if body.len() as u64 > MAX_DOWNLOAD {
                return Err("the download is too large".into());
            }
            Ok(Some(body))
        }
        Err(ureq::Error::Status(404, _)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
