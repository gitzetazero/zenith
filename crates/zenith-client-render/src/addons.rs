//! The Addons Zenith ships with (`docs/superpowers/specs/2026-11-07-zenith-ui-design.md`):
//! their registry, the on/off switch kept in `horizon.txt` and the search filter. No loader:
//! the list is compiled in, and a player only turns entries on or off.
use crate::lang;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Hud,
    Appearance,
}

impl Category {
    pub const ALL: [Category; 2] = [Category::Hud, Category::Appearance];
    pub fn key(self) -> &'static str {
        match self {
            Category::Hud => "zenith.addons.category.hud",
            Category::Appearance => "zenith.addons.category.appearance",
        }
    }
    pub fn label(self) -> String {
        let fallback = match self {
            Category::Hud => "HUD",
            Category::Appearance => "Appearance",
        };
        lang::get_or(self.key(), fallback)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AddonDef {
    pub id: &'static str,
    pub name_key: &'static str,
    pub desc_key: &'static str,
    pub category: Category,
    pub has_settings: bool,
}

pub const ADDONS: &[AddonDef] = &[
    AddonDef {
        id: "appearance",
        name_key: "zenith.appearance.name",
        desc_key: "zenith.appearance.desc",
        category: Category::Appearance,
        has_settings: true,
    },
    AddonDef {
        id: "fps",
        name_key: "zenith.fps.name",
        desc_key: "zenith.fps.desc",
        category: Category::Hud,
        has_settings: true,
    },
    AddonDef {
        id: "keystrokes",
        name_key: "zenith.keystrokes.name",
        desc_key: "zenith.keystrokes.desc",
        category: Category::Hud,
        has_settings: true,
    },
];

/// The key `horizon.txt` keeps an Addon's switch under.
pub fn key(id: &str) -> String {
    format!("addon.{id}")
}

fn preference(dir: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("horizon.txt")).ok()?;
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}

fn set_preference(dir: &Path, key: &str, value: &str) {
    let text = std::fs::read_to_string(dir.join("horizon.txt")).unwrap_or_default();
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| l.split_once(':').is_none_or(|(k, _)| k.trim() != key))
        .map(str::to_owned)
        .collect();
    lines.push(format!("{key}:{value}"));
    let mut out = lines.join("\n");
    out.push('\n');
    let _ = std::fs::write(dir.join("horizon.txt"), out);
}

/// Whether the Addon is on. Absent means off, so the HUD stays as vanilla until chosen.
pub fn enabled(dir: &Path, id: &str) -> bool {
    preference(dir, &key(id)).is_some_and(|v| v == "true")
}

pub fn set_enabled(dir: &Path, id: &str, on: bool) {
    set_preference(dir, &key(id), if on { "true" } else { "false" });
}

/// Fold a letter to ASCII so "Teclas" matches "teclas" and "Música" matches "musica".
fn fold(ch: char) -> char {
    match ch {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => ch.to_ascii_lowercase(),
    }
}

fn normalise(text: &str) -> String {
    text.chars().map(fold).collect()
}

/// The Addons matching `query` (name or description, case and accents ignored) and
/// `category`, in registry order.
pub fn filter(query: &str, category: Option<Category>) -> Vec<&'static AddonDef> {
    let query = normalise(query.trim());
    ADDONS
        .iter()
        .filter(|a| category.is_none_or(|c| a.category == c))
        .filter(|a| {
            if query.is_empty() {
                return true;
            }
            let name = normalise(&lang::get_or(a.name_key, a.id));
            let desc = normalise(&lang::get_or(a.desc_key, ""));
            name.contains(&query) || desc.contains(&query)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("horizon-addons-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_search_ignores_case_accents_and_finds_nothing_for_nonsense() {
        assert_eq!(
            filter("FPS", None).iter().map(|a| a.id).collect::<Vec<_>>(),
            filter("fps", None).iter().map(|a| a.id).collect::<Vec<_>>()
        );
        assert_eq!(filter("fps", None).len(), 1);
        assert!(filter("zzz", None).is_empty());
        assert_eq!(filter("", None).len(), 3);
        assert!(filter("key", None).iter().any(|a| a.id == "keystrokes"));
        assert_eq!(normalise("Música"), "musica");
    }

    #[test]
    fn the_category_narrows_the_list() {
        let hud = filter("", Some(Category::Hud));
        assert!(hud.iter().all(|a| a.category == Category::Hud));
        assert_eq!(hud.len(), 2);
        assert_eq!(filter("", Some(Category::Appearance)).len(), 1);
    }

    #[test]
    fn switching_an_addon_keeps_one_line_in_horizon_txt() {
        let d = dir("switch");
        assert!(!enabled(&d, "fps"));
        set_enabled(&d, "fps", true);
        assert!(enabled(&d, "fps"));
        for i in 0..10 {
            set_enabled(&d, "fps", i % 2 == 0);
        }
        let text = std::fs::read_to_string(d.join("horizon.txt")).unwrap();
        assert_eq!(
            text.lines().filter(|l| l.starts_with("addon.fps:")).count(),
            1
        );
        assert!(text.contains("addon.fps:false"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
