//! The tabs above the inner menus (Singleplayer, Multiplayer, Addons, Options) and the
//! account at their right, from the Zenith UI spec. Both themes keep Laby's layout: Zenith
//! draws it with its own plates and bundled face, Vanilla with the game's button sprites and
//! bitmap font. The title screen has no bar at all.
use crate::{
    hud::{HudAtlas, HudBatch},
    lang,
    theme::{self, UiTheme},
};

/// The bar's height in GUI pixels; inner screens keep their content below it.
pub const HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Singleplayer,
    Multiplayer,
    Addons,
    Options,
}

impl Tab {
    pub const ALL: [Tab; 4] = [
        Tab::Singleplayer,
        Tab::Multiplayer,
        Tab::Addons,
        Tab::Options,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Tab::Singleplayer => "zenith.tab.singleplayer",
            Tab::Multiplayer => "zenith.tab.multiplayer",
            Tab::Addons => "zenith.tab.addons",
            Tab::Options => "zenith.tab.options",
        }
    }
    pub fn label(self) -> String {
        let fallback = match self {
            Tab::Singleplayer => "Singleplayer",
            Tab::Multiplayer => "Multiplayer",
            Tab::Addons => "Addons",
            Tab::Options => "Options",
        };
        lang::get_or(self.key(), fallback)
    }
}

/// The tabs' rectangles, left to right. They shrink on a narrow screen and never leave
/// `[0, w]`.
pub fn layout(w: f32, tabs: usize) -> Vec<[f32; 4]> {
    if tabs == 0 {
        return Vec::new();
    }
    let left = 8.0;
    let gap = 4.0;
    let avail = (w - left * 2.0 - gap * (tabs as f32 - 1.0)).max(1.0);
    let tw = (avail / tabs as f32).min(96.0);
    let total = tw * tabs as f32 + gap * (tabs as f32 - 1.0);
    let x0 = ((w - total) / 2.0).max(left);
    (0..tabs)
        .map(|i| [x0 + i as f32 * (tw + gap), 3.0, tw, HEIGHT - 6.0])
        .collect()
}

/// The tab under the pointer, if any.
pub fn hit(w: f32, x: f32, y: f32) -> Option<Tab> {
    if y < 0.0 || y >= HEIGHT {
        return None;
    }
    layout(w, Tab::ALL.len())
        .iter()
        .position(|r| x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3])
        .map(|i| Tab::ALL[i])
}

/// Cut `account` to `max_w` at about six pixels a character, with an ellipsis.
pub fn fit(account: &str, max_w: f32) -> String {
    let room = (max_w / 6.0).floor() as usize;
    let chars: Vec<char> = account.chars().collect();
    if chars.len() <= room {
        return account.to_owned();
    }
    if room == 0 {
        return String::new();
    }
    let mut out: String = chars.into_iter().take(room.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Draw the bar: a panel across the top, the active tab lit, the account at the right.
/// The icons the spec leaves empty are not drawn.
pub fn draw(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    active: Tab,
    account: &str,
    _cursor: (f32, f32),
) {
    let zenith = theme::current() == UiTheme::Zenith;
    let p = theme::palette();
    let w = batch.width;
    batch.fill(
        atlas,
        0.0,
        0.0,
        w,
        HEIGHT,
        if zenith { p.panel } else { 0xC010_1010 },
    );
    batch.fill(atlas, 0.0, HEIGHT - 1.0, w, 1.0, p.outline);
    for (i, r) in layout(w, Tab::ALL.len()).iter().enumerate() {
        let tab = Tab::ALL[i];
        if zenith {
            let color = if tab == active { p.accent } else { p.button };
            crate::ui_zenith::rounded(batch, atlas, r[0], r[1], r[2], r[3], 6.0, color);
        } else {
            let sprite = if tab == active {
                "gui/sprites/widget/button_highlighted"
            } else {
                "gui/sprites/widget/button"
            };
            crate::ui::nine_slice(batch, atlas, sprite, r[0], r[1], r[2], r[3], u32::MAX);
        }
        let label = tab.label();
        if zenith {
            crate::ui_text::text_fitted(
                batch,
                atlas,
                crate::ui_text::Face::Medium,
                12.0,
                r[0] + 2.0,
                r[2] - 4.0,
                r[1] + ((r[3] - 12.0 * 1.45) / 2.0).floor(),
                if tab == active { p.text } else { p.text_dim },
                &label,
            );
        } else {
            let text = crate::text::Text::literal(label);
            crate::ui::scrolling_text_around_at(
                batch,
                atlas,
                &text,
                r[0] + r[2] / 2.0,
                r[0] + 2.0,
                r[0] + r[2] - 2.0,
                r[1],
                r[1] + r[3],
                r[1] + ((r[3] - 9.0) / 2.0).floor(),
                crate::hud_overlays::now_ms(),
                if tab == active { u32::MAX } else { 0xFFA0A0A0 },
            );
        }
    }
    if !account.is_empty() {
        let name = fit(account, 120.0);
        if zenith {
            let tw = crate::ui_text::measure(atlas, crate::ui_text::Face::Regular, 12.0, &name);
            crate::ui_text::text(
                batch,
                atlas,
                crate::ui_text::Face::Regular,
                12.0,
                w - tw - 8.0,
                (HEIGHT - 12.0 * 1.45) / 2.0,
                p.text,
                &name,
            );
        } else {
            let text = crate::text::Text::literal(name);
            let tw = atlas.font.width_text(&text);
            batch.text_styled(
                atlas,
                &text,
                w - tw - 8.0,
                (HEIGHT - 9.0) / 2.0,
                p.text,
                true,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_fit_wide_and_narrow_screens_without_overlapping() {
        for w in [854.0, 300.0, 240.0] {
            let rows = layout(w, Tab::ALL.len());
            assert_eq!(rows.len(), 4);
            for (i, a) in rows.iter().enumerate() {
                assert!(a[0] >= 0.0 && a[0] + a[2] <= w, "{a:?} leaves {w}");
                for b in &rows[i + 1..] {
                    assert!(a[0] + a[2] <= b[0], "{a:?} overlaps {b:?}");
                }
            }
        }
    }

    #[test]
    fn a_click_selects_the_tab_under_it_and_nothing_outside() {
        let rows = layout(854.0, Tab::ALL.len());
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(
                hit(854.0, r[0] + r[2] / 2.0, r[1] + r[3] / 2.0),
                Some(Tab::ALL[i])
            );
        }
        assert_eq!(hit(854.0, 0.0, 0.0), None);
        assert_eq!(hit(854.0, 100.0, HEIGHT + 1.0), None);
    }

    #[test]
    fn long_and_empty_account_names_fit_the_right_corner() {
        assert_eq!(fit("", 120.0), "");
        let long = "A".repeat(40);
        let fitted = fit(&long, 120.0);
        assert!(fitted.chars().count() <= 20);
        assert!(fitted.ends_with('…'));
    }
}
