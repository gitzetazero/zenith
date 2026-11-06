//! Signed Zenith builds: fetched from the `nightly` Release of the fork, verified against the
//! embedded keys, and unpacked under `<root>/builds/<version>/`.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use zenith_release_sign::{NIGHTLY, PublicKey, SIGNATURE, SUMS, check, sha256_hex, verify};

pub const REPO: &str = "gitzetazero/zenith";
const MAX_DOWNLOAD: u64 = 1 << 30;
/// Starts the error of a fetch that never reached the server; `update` does not wait to retry it.
pub const OFFLINE: &str = "no internet connection";

/// The keys the nightly's checksums are signed with, built in.
pub fn embedded_keys() -> Vec<PublicKey> {
    zenith_release_sign::parse_keys(include_str!("../zenith-release-keys.txt")).unwrap_or_default()
}

/// The signed SHA-256 of `asset`: `signature` must sign `sums` for the `nightly` channel (Zenith signs its
/// Nightly that way; Horizon's own Nightlies are signed per version since its SHA256SUMS.v2.sig).
fn nightly_digest(
    asset: &str,
    sums: Option<&[u8]>,
    signature: Option<&str>,
    keys: &[PublicKey],
) -> Result<String, String> {
    let (Some(sums), Some(signature)) = (sums, signature) else {
        return Err("The Nightly build is not signed: not installed.".into());
    };
    verify(NIGHTLY, sums, signature, keys)?
        .into_iter()
        .find(|(n, _)| n == asset)
        .map(|(_, hex)| hex)
        .ok_or_else(|| format!("{asset} is not in the signed checksums, so it was not installed."))
}

pub fn asset_name() -> &'static str {
    if cfg!(windows) {
        "horizon-windows-x64.zip"
    } else {
        "horizon-linux-x86_64.tar.gz"
    }
}

/// The build stamp: `commit=<40 hex>` and `built=<UTC time>`, signed through SHA256SUMS like the archive.
pub const STAMP: &str = "zenith-build.txt";

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
        valid_version(text).ok().map(|()| text.to_string())
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

    /// The stamp saved in the installed build's folder, if there is a valid one.
    pub fn installed_stamp(&self) -> Option<Stamp> {
        let text = std::fs::read_to_string(self.current()?.join(STAMP)).ok()?;
        Stamp::parse_at(&text, u64::MAX).ok()
    }

    /// Saves a newer stamp of the same commit in the installed build's folder (best effort).
    fn refresh_stamp(&self, dir: &Path, stamp: &Stamp) {
        let _ = write_atomic(&dir.join(STAMP), &stamp.text());
    }

    /// Verifies `archive` and the build `stamp` against the nightly-signed `sums` (the stamp before it
    /// is parsed), refuses a build older than the installed one, unpacks it as the version the stamp
    /// names and makes it the current build. Nothing is unpacked unless everything checks out.
    pub fn install(
        &self,
        archive: &[u8],
        stamp: &[u8],
        sums: &[u8],
        signature: &str,
        keys: &[PublicKey],
    ) -> Result<PathBuf, String> {
        let asset = asset_name();
        let expected = nightly_digest(asset, Some(sums), Some(signature), keys)?;
        check(asset, &expected, &sha256_hex(archive))?;
        let stamp = verified_stamp(stamp, sums, signature, keys)?;
        let version = stamp.version();
        valid_version(&version)?;
        if let Some(have) = self.installed_stamp() {
            if stamp.built < have.built {
                return Err(downgrade(&stamp, &have));
            }
        }
        if !asset.ends_with(".zip") {
            // shortcut: Windows first; the Linux .tar.gz needs flate2+tar here, when Linux is wired up.
            return Err(format!("installing {asset} is not supported yet"));
        }

        let builds = self.builds();
        let tmp = builds.join(format!("{version}.tmp"));
        let dest = builds.join(&version);
        // The same signed build is already the one in use (or pinned): never touch its folder.
        if dest.is_dir()
            && (self.current_version().as_deref() == Some(version.as_str())
                || self.pinned().as_deref() == Some(version.as_str()))
        {
            return Ok(dest);
        }
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let done = extract_zip(archive, &tmp).and_then(|()| {
            if !tmp.join("zenith.exe").is_file() && !tmp.join("horizon-client.exe").is_file() {
                return Err("the archive has no zenith.exe".to_string());
            }
            std::fs::write(tmp.join(STAMP), stamp.text()).map_err(|e| e.to_string())?;
            let _ = std::fs::remove_dir_all(&dest);
            std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())
        });
        if let Err(e) = done {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(e);
        }

        let previous = self.current_version();
        write_atomic(&self.root.join("current.txt"), &version)?;
        self.prune(&[
            Some(&version),
            previous.as_deref(),
            self.pinned().as_deref(),
        ]);
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

/// The first stamped release: nothing older is installed, not even on a fresh machine.
/// Bump it when a release is known to be vulnerable.
pub const MIN_BUILT: &str = "2026-11-05T02:56:39Z";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Seconds since 1970 of a well-formed `yyyy-mm-ddTHH:MM:SSZ`, None if any field is out of range.
fn epoch_secs(t: &str) -> Option<u64> {
    let n = |r: std::ops::Range<usize>| t[r].parse::<u64>().ok();
    let (y, mo, d, h, mi, s) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(2026..=2100).contains(&y) || !(1..=12).contains(&mo) || h >= 24 || mi >= 60 || s >= 60 {
        return None;
    }
    if !(1..=month_days[mo as usize - 1]).contains(&d) {
        return None;
    }
    let days = (1970..y)
        .map(|y| {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                366
            } else {
                365
            }
        })
        .sum::<u64>()
        + month_days[..mo as usize - 1].iter().sum::<u64>()
        + d
        - 1;
    Some(days * 86_400 + h * 3600 + mi * 60 + s)
}

/// What the signed `zenith-build.txt` says: the commit built and when (UTC, `yyyy-mm-ddTHH:MM:SSZ`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub commit: String,
    pub built: String,
}

impl Stamp {
    /// Exactly two LF-terminated lines, `commit=<40 lowercase hex>` then `built=<UTC time>`.
    pub fn parse(text: &str) -> Result<Stamp, String> {
        Stamp::parse_at(text, now_secs())
    }

    /// [`Stamp::parse`] against a given clock (seconds since 1970): the time must be a real one
    /// between [`MIN_BUILT`] and one day after `now` (clock skew tolerance).
    pub fn parse_at(text: &str, now: u64) -> Result<Stamp, String> {
        let bad = || format!("{STAMP} is malformed");
        let mut lines = text.strip_suffix('\n').ok_or_else(bad)?.split('\n');
        let commit = lines
            .next()
            .and_then(|l| l.strip_prefix("commit="))
            .ok_or_else(bad)?;
        let built = lines
            .next()
            .and_then(|l| l.strip_prefix("built="))
            .ok_or_else(bad)?;
        let commit_ok = commit.len() == 40
            && commit
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        // One character class per position: D digit, anything else literal.
        let time_ok = built.len() == 20
            && built
                .bytes()
                .zip("DDDD-DD-DDTDD:DD:DDZ".bytes())
                .all(|(b, p)| {
                    if p == b'D' {
                        b.is_ascii_digit()
                    } else {
                        b == p
                    }
                });
        if lines.next().is_some() || !commit_ok || !time_ok {
            return Err(bad());
        }
        let secs = epoch_secs(built).ok_or_else(bad)?;
        if secs > now.saturating_add(86_400) {
            return Err(format!("{STAMP} is dated in the future"));
        }
        if built < MIN_BUILT {
            return Err(format!("{STAMP} is older than the first stamped release"));
        }
        Ok(Stamp {
            commit: commit.into(),
            built: built.into(),
        })
    }

    /// The build's identity, from the signed stamp: `nightly-<date built>-<first 7 of the commit>`.
    pub fn version(&self) -> String {
        format!("nightly-{}-{}", &self.built[..10], &self.commit[..7])
    }

    fn text(&self) -> String {
        format!("commit={}\nbuilt={}\n", self.commit, self.built)
    }
}

/// The stamp, once its digest is found in the signed sums (before anything is parsed).
fn verified_stamp(
    stamp: &[u8],
    sums: &[u8],
    signature: &str,
    keys: &[PublicKey],
) -> Result<Stamp, String> {
    let expected = nightly_digest(STAMP, Some(sums), Some(signature), keys)?;
    check(STAMP, &expected, &sha256_hex(stamp))?;
    Stamp::parse(std::str::from_utf8(stamp).map_err(|_| format!("{STAMP} is not text"))?)
}

fn downgrade(new: &Stamp, have: &Stamp) -> String {
    format!(
        "downgrade refused: the release was built {} but the installed build {}",
        new.built, have.built
    )
}

/// A name that is safe on any system: relative, no `..`, no `:` (drive letters, alternate data
/// streams), no Windows device names, no trailing dot or space, no control characters.
fn safe_name(name: &str) -> bool {
    !name.starts_with(['/', '\\'])
        && name.split(['/', '\\']).all(|part| {
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .trim_end()
                .to_ascii_uppercase();
            let numbered = stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.ends_with(|c: char| c.is_ascii_digit());
            part != ".."
                && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !numbered
                && !part.contains(':')
                && !part.ends_with(['.', ' '])
                && !part.chars().any(char::is_control)
        })
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
        if !safe_name(f.name()) {
            return Err(format!("unsafe path in the archive: {}", f.name()));
        }
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
    let mut last = String::new();
    for attempt in 0..2 {
        if attempt > 0 {
            std::thread::sleep(retry_delay);
        }
        let try_once = || -> Result<PathBuf, String> {
            let sums = get(asset_url(SUMS), SUMS)?;
            let sig = get(asset_url(SIGNATURE), SIGNATURE)?;
            let stamp = get(asset_url(STAMP), STAMP)?;
            let sig = String::from_utf8(sig).map_err(|_| format!("{SIGNATURE} is not text"))?;
            // The identity comes from the signed stamp, never from the release title.
            let new = verified_stamp(&stamp, &sums, &sig, keys)?;
            if let (Some(have), Some(current)) = (store.installed_stamp(), store.current()) {
                // The same commit rebuilt later is the same game: nothing to download.
                if new.commit == have.commit {
                    if new.built > have.built {
                        store.refresh_stamp(&current, &new);
                    }
                    return Ok(current);
                }
                if new.built <= have.built {
                    if new.built < have.built {
                        store.log(&downgrade(&new, &have));
                    }
                    return Ok(current);
                }
            }
            let archive = get(asset_url(asset_name()), asset_name())?;
            store.install(&archive, &stamp, &sums, &sig, keys)
        };
        match try_once() {
            Ok(dir) => return Ok(dir),
            Err(e) if e.starts_with(OFFLINE) => return Err(e), // offline: a retry would only wait
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// The production fetch: a GET with the timeouts and User-Agent the bridge uses. 404 is `None`.
pub fn http_fetch(url: &str) -> Result<Option<Vec<u8>>, String> {
    let agent = ureq::AgentBuilder::new()
        .https_only(true) // a redirect to plain http fails too
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .timeout(Duration::from_secs(900)) // overall, so a trickling download cannot hang the launch
        .user_agent(concat!("zenith-bridge/", env!("CARGO_PKG_VERSION")))
        .build();
    let req = agent.get(url);
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
        Err(e @ ureq::Error::Transport(_)) => {
            Err(format!("{OFFLINE} or GitHub is not answering ({e})"))
        }
        Err(e) => Err(e.to_string()),
    }
}
