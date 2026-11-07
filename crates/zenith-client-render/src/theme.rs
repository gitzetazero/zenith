//! The game's own interface theme (`docs/superpowers/specs/2026-11-07-zenith-ui-design.md`):
//! `Zenith` draws the menus as Laby Client with Zenith's mark, `Vanilla` keeps every screen
//! exactly as it was. The drawing primitives in [`crate::ui`] ask [`current`] and hand over to
//! [`crate::ui_zenith`] only for `Zenith`, so a player who picks `Vanilla` sees the old game.
//!
//! The preference lives in `horizon.txt` (`uiTheme:zenith|vanilla`); `zenith-client`'s
//! `ui_theme` loads and saves it. The process-wide default is `Vanilla` so tests and the
//! parity harnesses that never load preferences draw today's screens; the game calls
//! [`set`] with [`crate::theme::UiTheme::Zenith`] when the file has no choice.
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTheme {
    Zenith,
    Vanilla,
}

impl UiTheme {
    /// Read a stored value. Anything unknown falls back to `Zenith`, the default look.
    pub fn parse(text: &str) -> UiTheme {
        match text.trim().to_ascii_lowercase().as_str() {
            "vanilla" => UiTheme::Vanilla,
            _ => UiTheme::Zenith,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            UiTheme::Zenith => "zenith",
            UiTheme::Vanilla => "vanilla",
        }
    }
    fn code(self) -> u8 {
        match self {
            UiTheme::Zenith => 1,
            UiTheme::Vanilla => 0,
        }
    }
    fn from_code(code: u8) -> UiTheme {
        match code {
            0 => UiTheme::Vanilla,
            _ => UiTheme::Zenith,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// The theme this process draws with.
pub fn current() -> UiTheme {
    UiTheme::from_code(CURRENT.load(Ordering::Relaxed))
}

/// Choose the theme for every screen drawn from now on.
pub fn set(theme: UiTheme) {
    CURRENT.store(theme.code(), Ordering::Relaxed);
}

/// The Zenith palette in the HUD's `0xAARRGGBB` encoding. The values are the spec's:
/// a blue accent on near-black plates, white text and a dim white for values.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub background: u32,
    pub panel: u32,
    pub button: u32,
    pub button_hover: u32,
    pub button_disabled: u32,
    pub accent: u32,
    pub text: u32,
    pub text_dim: u32,
    pub outline: u32,
}

const ZENITH: Palette = Palette {
    background: 0xFF14_161D,
    panel: 0xFF1B_1E26,
    button: 0xFF23_252E,
    button_hover: 0xFF2D_303B,
    button_disabled: 0xFF1B_1D24,
    accent: 0xFF4A_6CF7,
    text: 0xFFFF_FFFF,
    text_dim: 0xFF9B_A1AE,
    outline: 0xFF2A_2D38,
};

/// Vanilla's old widgets keep their sprites; only the text colour is shared.
const VANILLA: Palette = Palette {
    background: 0xFF00_0000,
    panel: 0xFF00_0000,
    button: 0xFF00_0000,
    button_hover: 0xFF00_0000,
    button_disabled: 0xFF00_0000,
    accent: 0xFFFF_FFFF,
    text: 0xFFFF_FFFF,
    text_dim: 0xFFA0_A0A0,
    outline: 0xFF00_0000,
};

/// The palette the current theme draws with.
pub fn palette() -> Palette {
    match current() {
        UiTheme::Zenith => ZENITH,
        UiTheme::Vanilla => VANILLA,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_values_round_trip_and_unknown_ones_are_zenith() {
        assert_eq!(UiTheme::parse("vanilla"), UiTheme::Vanilla);
        assert_eq!(UiTheme::parse("ZENITH"), UiTheme::Zenith);
        assert_eq!(UiTheme::parse(""), UiTheme::Zenith);
        assert_eq!(UiTheme::parse("glass"), UiTheme::Zenith);
        for theme in [UiTheme::Zenith, UiTheme::Vanilla] {
            assert_eq!(UiTheme::parse(theme.key()), theme);
        }
    }

    #[test]
    fn setting_the_theme_changes_what_the_screens_read() {
        let before = current();
        set(UiTheme::Vanilla);
        assert_eq!(current(), UiTheme::Vanilla);
        set(UiTheme::Zenith);
        assert_eq!(current(), UiTheme::Zenith);
        assert_eq!(palette().accent, 0xFF4A_6CF7);
        set(before);
    }
}
