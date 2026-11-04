//! Zenith's visible name over Horizon's strings.
use std::collections::HashMap;

pub const WINDOW_TITLE: &str = "Zenith";

/// Replaces the word "Horizon" (not "Horizonte", not lowercase "horizon").
pub fn rebrand(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("Horizon") {
        let (head, tail) = rest.split_at(i);
        let after = &tail["Horizon".len()..];
        out.push_str(head);
        if after.chars().next().is_some_and(char::is_alphanumeric) {
            out.push_str("Horizon");
        } else {
            out.push_str("Zenith");
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Rebrands the text and adds Zenith's own `zenith.*` strings (Spanish for es_*, English otherwise);
/// a key a resource pack already set stays.
pub fn apply(map: &mut HashMap<String, String>, code: &str) {
    for v in map.values_mut() {
        if v.contains("Horizon") {
            *v = rebrand(v);
        }
    }
    let json = if code.starts_with("es_") {
        include_str!("../../horizon-client/assets/zenith/lang/es_es.json")
    } else {
        include_str!("../../horizon-client/assets/zenith/lang/en_us.json")
    };
    for (k, v) in serde_json::from_str::<HashMap<String, String>>(json).unwrap_or_default() {
        map.entry(k).or_insert(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_whole_word() {
        assert_eq!(rebrand("Horizon Graphics"), "Zenith Graphics");
    }
    #[test]
    fn keeps_spanish_word() {
        assert_eq!(rebrand("Horizonte Horizon."), "Horizonte Zenith.");
    }
    #[test]
    fn apply_merges_keys_in_spanish_and_falls_back_to_english() {
        let get = |code: &str, k: &str| {
            let mut m = HashMap::new();
            apply(&mut m, code);
            m.get(k).cloned()
        };
        assert_eq!(
            get("es_mx", "zenith.multiplayer.version.auto").as_deref(),
            Some("Automática")
        );
        assert_eq!(
            get("fr_fr", "zenith.multiplayer.joining_as").as_deref(),
            Some("Joining as %s")
        );
        // A resource pack's own value stays.
        let mut m = HashMap::from([("zenith.multiplayer.version".to_owned(), "Pack".to_owned())]);
        apply(&mut m, "es_es");
        assert_eq!(m["zenith.multiplayer.version"], "Pack");
    }
    #[test]
    fn keeps_lowercase() {
        assert_eq!(rebrand("horizon-client"), "horizon-client");
    }
}
