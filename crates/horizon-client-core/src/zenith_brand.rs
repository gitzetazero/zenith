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

pub fn apply(map: &mut HashMap<String, String>, _code: &str) {
    for v in map.values_mut() {
        if v.contains("Horizon") {
            *v = rebrand(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn replaces_whole_word() { assert_eq!(rebrand("Horizon Graphics"), "Zenith Graphics"); }
    #[test] fn keeps_spanish_word() { assert_eq!(rebrand("Horizonte Horizon."), "Horizonte Zenith."); }
    #[test] fn keeps_lowercase() { assert_eq!(rebrand("horizon-client"), "horizon-client"); }
}
