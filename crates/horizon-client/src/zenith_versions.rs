//! Zenith: which version to join an older server as, and the one retry when the server names a range.
use horizon_protocol::adapter::ServerVersion;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type Version = (u32, u32, u32);

/// The overrides file (`zenith-servers.json`, beside servers.dat) and the address -> version name map.
static STATE: Mutex<(Option<PathBuf>, BTreeMap<String, String>)> =
    Mutex::new((None, BTreeMap::new()));

fn key(address: &str) -> String {
    address.trim().to_lowercase()
}

fn state() -> std::sync::MutexGuard<'static, (Option<PathBuf>, BTreeMap<String, String>)> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Reads `zenith-servers.json` from the player folder; a missing or corrupt file is an empty list.
pub fn init(player_dir: &Path) {
    let file = player_dir.join("zenith-servers.json");
    let map = std::fs::read_to_string(&file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    *state() = (Some(file), map);
}

/// "26.2" as (26, 2, 0): at most three parts, the rest dropped.
fn tuple_of(parts: &str) -> Version {
    let mut n = parts.split('.').map(|p| p.parse().unwrap_or(0));
    (
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
        n.next().unwrap_or(0),
    )
}

/// The version starting at `i` (digits and dots, at least one dot, no empty part) and where it ends.
fn version_at(c: &[char], i: usize) -> Option<(Version, usize)> {
    let mut end = i;
    while end < c.len() && (c[end].is_ascii_digit() || c[end] == '.') {
        end += 1;
    }
    while end > i && c[end - 1] == '.' {
        end -= 1; // a sentence's full stop
    }
    let s: String = c[i..end].iter().collect();
    (s.contains('.') && s.split('.').all(|p| !p.is_empty())).then(|| (tuple_of(&s), end))
}

/// The upper bound of "version - version" ("1.7.2-26.2", "1.8 – 1.21.8"). A date has no dot and a
/// lone decimal or version has no second one, so neither is a range.
pub fn range_upper(text: &str) -> Option<Version> {
    let c: Vec<char> = text.chars().collect();
    let skip = |mut j: usize| {
        while c.get(j) == Some(&' ') {
            j += 1;
        }
        j
    };
    let mut i = 0;
    while i < c.len() {
        if !c[i].is_ascii_digit() || (i > 0 && (c[i - 1].is_ascii_digit() || c[i - 1] == '.')) {
            i += 1;
            continue;
        }
        if let Some((_, after)) = version_at(&c, i) {
            let j = skip(after);
            if matches!(c.get(j), Some('-' | '\u{2013}'))
                && let Some((upper, _)) = version_at(&c, skip(j + 1))
            {
                return Some(upper);
            }
        }
        while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
            i += 1;
        }
    }
    None
}

/// The newest translatable version older than the pinned one and inside the named range.
pub fn retry_version(text: &str) -> Option<ServerVersion> {
    let upper = range_upper(text)?;
    let current = tuple_of(ServerVersion::Current.name());
    ServerVersion::ALL.into_iter().find(|v| {
        let t = tuple_of(v.name());
        *v != ServerVersion::Current && t < current && t <= upper
    })
}

pub fn override_for(address: &str) -> Option<ServerVersion> {
    let name = state().1.get(&key(address))?.clone();
    ServerVersion::ALL.into_iter().find(|v| v.name() == name)
}

/// Remembers (or with `None` forgets) the version for an address, and saves the file.
pub fn set_override(address: &str, version: Option<ServerVersion>) -> std::io::Result<()> {
    let mut s = state();
    match version {
        Some(v) => s.1.insert(key(address), v.name().to_owned()),
        None => s.1.remove(&key(address)),
    };
    match &s.0 {
        Some(file) => std::fs::write(file, serde_json::to_string_pretty(&s.1)?),
        None => Ok(()),
    }
}

/// `forced`, else the saved override, else the version the status ping's protocol names. `Current` is
/// `None`: nothing to translate. No ping when `forced` or an override decides.
pub fn choose(
    address: &str,
    forced: Option<ServerVersion>,
    ping: impl FnOnce() -> Option<i32>,
) -> Option<ServerVersion> {
    forced
        .or_else(|| override_for(address))
        .or_else(|| ServerVersion::from_protocol(ping()?))
        .filter(|v| *v != ServerVersion::Current)
}

/// The text with the version the connection was translated as: "... (26.2)".
pub fn with_version(text: String, version: Option<ServerVersion>) -> String {
    match version {
        Some(v) => format!("{text} ({})", v.name()),
        None => text,
    }
}

/// Edit-server button cycle: Automatic (`None`), newest to oldest, Automatic again.
pub fn next_version(v: Option<ServerVersion>) -> Option<ServerVersion> {
    let all = ServerVersion::ALL;
    match v {
        None => all.first().copied(),
        Some(v) => all
            .iter()
            .position(|a| *a == v)
            .and_then(|i| all.get(i + 1).copied()),
    }
}

/// A version's name, marked experimental below 1.21.8.
fn name_label(name: &str) -> String {
    if tuple_of(name) < (1, 21, 8) {
        crate::lang::get_or(
            "zenith.multiplayer.version.experimental",
            "%s (experimental)",
        )
        .replacen("%s", name, 1)
    } else {
        name.to_owned()
    }
}

/// The Edit-server button's label: "Versión: Automática" or "Versión: 26.2".
pub fn version_label(v: Option<ServerVersion>) -> String {
    let value = match v {
        Some(v) => name_label(v.name()),
        None => crate::lang::get_or("zenith.multiplayer.version.auto", "Automática"),
    };
    crate::lang::get_or("zenith.multiplayer.version", "Versión: %s").replacen("%s", &value, 1)
}

/// The overrides are process-wide: tests that touch them take turns.
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::TEST_LOCK as LOCK;
    use super::*;

    fn no_ping() -> Option<i32> {
        panic!("pinged")
    }
    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("zenith-versions-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn screenshot_message() {
        assert_eq!(
            range_upper("¡Cliente no compatible! Por favor, usa 1.7.2-26.2"),
            Some((26, 2, 0))
        );
    }
    #[test]
    fn english_and_en_dash() {
        assert_eq!(range_upper("Please use 1.8 – 1.21.8"), Some((1, 21, 8)));
    }
    #[test]
    fn date_is_not_a_range() {
        assert_eq!(range_upper("Baneado hasta 2026-11-05"), None);
    }
    #[test]
    fn lone_decimal_is_not_a_range() {
        assert_eq!(range_upper("Reinicio en 1.5 minutos"), None);
    }
    #[test]
    fn lone_version_is_not_a_range() {
        assert_eq!(range_upper("Usa 1.21.4"), None);
    }
    #[test]
    fn retries_with_newest_inside_range() {
        assert_eq!(retry_version("usa 1.7.2-26.2"), Some(ServerVersion::V26_2));
    }
    #[test]
    fn current_in_range_gives_next_older() {
        assert_eq!(
            retry_version("Velocity 1.7.2-26.3"),
            Some(ServerVersion::V26_2)
        );
    }
    #[test]
    fn range_too_old_gives_none() {
        assert_eq!(retry_version("usa 1.7.2-1.12.2"), None);
    }
    #[test]
    fn forced_skips_ping() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(
            choose("a:1", Some(ServerVersion::V26_1), no_ping),
            Some(ServerVersion::V26_1)
        );
        assert_eq!(choose("a:1", Some(ServerVersion::Current), no_ping), None);
    }
    #[test]
    fn override_skips_ping() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init(&temp("skip"));
        set_override("Skip.Example", Some(ServerVersion::V1_21_11)).unwrap();
        assert_eq!(
            choose("skip.example", None, no_ping),
            Some(ServerVersion::V1_21_11)
        );
    }
    #[test]
    fn ping_of_current_is_none() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(
            choose("nope:1", None, || Some(ServerVersion::Current.protocol())),
            None
        );
    }
    #[test]
    fn ping_of_older_is_that_version() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(
            choose("nope:1", None, || Some(ServerVersion::V26_2.protocol())),
            Some(ServerVersion::V26_2)
        );
    }
    #[test]
    fn unknown_ping_is_none() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(choose("nope:1", None, || Some(-5)), None);
        assert_eq!(choose("nope:1", None, || None), None);
    }
    #[test]
    fn override_survives_restart() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp("restart");
        init(&dir);
        set_override("mc.example:25565", Some(ServerVersion::V26_1)).unwrap();
        assert!(dir.join("zenith-servers.json").is_file());
        state().1.clear();
        init(&dir);
        assert_eq!(override_for("mc.example:25565"), Some(ServerVersion::V26_1));
        set_override("mc.example:25565", None).unwrap();
        init(&dir);
        assert_eq!(override_for("mc.example:25565"), None);
    }
    #[test]
    fn corrupt_overrides_file_is_ignored() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp("corrupt");
        std::fs::write(dir.join("zenith-servers.json"), "{ not json").unwrap();
        init(&dir);
        assert_eq!(override_for("x"), None);
    }
    #[test]
    fn cycle_goes_auto_then_newest_to_oldest_then_auto() {
        let mut seen = vec![];
        let mut v = next_version(None);
        while v.is_some() {
            seen.push(v.unwrap());
            v = next_version(v);
        }
        assert_eq!(seen, ServerVersion::ALL);
    }
    #[test]
    fn label_marks_below_1_21_8_experimental() {
        assert_eq!(name_label("1.21.4"), "1.21.4 (experimental)");
        assert_eq!(name_label("1.21.8"), "1.21.8");
        assert_eq!(version_label(None), "Versión: Automática");
    }
    #[test]
    fn with_version_appends_name() {
        assert_eq!(
            with_version("Lost".into(), Some(ServerVersion::V26_2)),
            "Lost (26.2)"
        );
        assert_eq!(with_version("Lost".into(), None), "Lost");
    }
}
