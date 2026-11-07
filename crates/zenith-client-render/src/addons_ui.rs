//! The Addons screen: the compiled-in registry behind a search field and a category
//! filter, each entry with an on/off switch, and the Appearance Addon's theme picker.
//! It draws over the blurred world or the title backdrop; the spec leaves the Discord,
//! events, shop and skin slots empty, so no such control exists here.
use crate::{
    addons::{self, AddonDef, Category},
    hud::{HudAtlas, HudBatch, ZENITH_MARK_SPRITE},
    lang,
    text::Text,
    theme::{self, UiTheme},
};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub search: String,
    pub category: Option<Category>,
    /// Grid instead of a single list.
    pub grid: bool,
    /// The settings sub-screen of an Addon, by id.
    pub open: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    /// Back to the list (a sub-screen), or leave (handled by the caller).
    Back,
    Changed,
}

fn tr(key: &str, fallback: &str) -> String {
    lang::get_or(key, fallback)
}

/// The panel in GUI pixels, below the top bar so its title is never covered.
pub fn panel(screen: (f32, f32)) -> [f32; 4] {
    let w = (screen.0 - 20.0).clamp(200.0, 360.0);
    let h = (screen.1 - crate::top_bar::HEIGHT - 8.0).clamp(140.0, 240.0);
    let y = ((screen.1 - h) / 2.0)
        .floor()
        .max(crate::top_bar::HEIGHT + 4.0)
        .min((screen.1 - h - 4.0).max(crate::top_bar::HEIGHT + 4.0));
    [((screen.0 - w) / 2.0).floor(), y, w, h]
}

fn rows(panel: [f32; 4], list: &[&AddonDef]) -> Vec<[f32; 4]> {
    let [x, y, w, _] = panel;
    list.iter()
        .enumerate()
        .map(|(i, _)| [x + 8.0, y + 58.0 + i as f32 * 30.0, w - 16.0, 26.0])
        .collect()
}

fn next_corner(corner: crate::extras::Corner) -> crate::extras::Corner {
    use crate::extras::Corner;
    match corner {
        Corner::TopLeft => Corner::TopRight,
        Corner::TopRight => Corner::BottomRight,
        Corner::BottomRight => Corner::BottomLeft,
        Corner::BottomLeft => Corner::TopLeft,
    }
}

fn switch_rect(row: [f32; 4]) -> [f32; 4] {
    [row[0] + row[2] - 32.0, row[1] + 7.0, 28.0, 12.0]
}

/// The two preview cards of the Appearance Addon.
fn theme_cards(screen: (f32, f32)) -> [([f32; 4], UiTheme); 2] {
    let w = (screen.0 - 40.0).min(160.0);
    let h = 84.0;
    let y = screen.1 / 2.0 - h / 2.0;
    let left = screen.0 / 2.0 - w - 8.0;
    let right = screen.0 / 2.0 + 8.0;
    [
        ([left, y, w, h], UiTheme::Zenith),
        ([right, y, w, h], UiTheme::Vanilla),
    ]
}

/// Pick a theme at once and keep it in `horizon.txt`.
pub fn select_theme(dir: &std::path::Path, theme: UiTheme) {
    theme::set(theme);
    let path = dir.join("horizon.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| l.split_once(':').is_none_or(|(k, _)| k.trim() != "uiTheme"))
        .map(str::to_owned)
        .collect();
    lines.push(format!("uiTheme:{}", theme.key()));
    let mut out = lines.join("\n");
    out.push('\n');
    let _ = std::fs::write(&path, out);
}

/// Draw the screen. `dir` is the game folder that keeps the switches.
pub fn draw(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    state: &State,
    dir: &std::path::Path,
    cursor: (f32, f32),
) {
    let screen = (batch.width, batch.height);
    let p = theme::palette();
    let panel = panel(screen);
    // A dark plate over the blurred backdrop.
    batch.fill(atlas, 0.0, 0.0, screen.0, screen.1, 0x8000_0000);
    crate::ui_zenith::rounded(
        batch, atlas, panel[0], panel[1], panel[2], panel[3], 8.0, p.panel,
    );
    batch.fill(
        atlas,
        panel[0] + 8.0,
        panel[1] + 8.0,
        panel[2] - 16.0,
        1.0,
        p.outline,
    );
    if let Some(id) = state.open {
        draw_settings(batch, atlas, screen, id, state, dir, cursor);
        return;
    }
    let title = Text::literal(tr("zenith.addons.title", "Addons"));
    batch.text_styled(
        atlas,
        &title,
        panel[0] + 10.0,
        panel[1] + 10.0,
        p.text,
        true,
    );
    // The search field: a rounded well with the typed text or its hint.
    crate::ui_zenith::rounded(
        batch,
        atlas,
        panel[0] + 8.0,
        panel[1] + 28.0,
        panel[2] - 16.0,
        18.0,
        5.0,
        p.button,
    );
    let shown = if state.search.is_empty() {
        Text::literal(tr("zenith.addons.search", "Search addons"))
    } else {
        Text::literal(state.search.clone())
    };
    batch.text_styled(
        atlas,
        &shown,
        panel[0] + 14.0,
        panel[1] + 33.0,
        if state.search.is_empty() {
            p.text_dim
        } else {
            p.text
        },
        true,
    );
    let list = addons::filter(&state.search, state.category);
    if list.is_empty() {
        let empty = Text::literal(tr("zenith.addons.empty", "No addons match"));
        let tw = atlas.font.width_text(&empty);
        batch.text_styled(
            atlas,
            &empty,
            (screen.0 - tw) / 2.0,
            panel[1] + panel[3] / 2.0,
            p.text_dim,
            true,
        );
        return;
    }
    for (row, addon) in rows(panel, &list).into_iter().zip(&list) {
        let hover = cursor.0 >= row[0]
            && cursor.0 < row[0] + row[2]
            && cursor.1 >= row[1]
            && cursor.1 < row[1] + row[3];
        crate::ui_zenith::rounded(
            batch,
            atlas,
            row[0],
            row[1],
            row[2],
            row[3],
            5.0,
            if hover { p.button_hover } else { p.button },
        );
        let name = Text::literal(tr(addon.name_key, addon.id));
        batch.text_styled(atlas, &name, row[0] + 8.0, row[1] + 4.0, p.text, true);
        let desc = Text::literal(tr(addon.desc_key, ""));
        batch.text_styled(atlas, &desc, row[0] + 8.0, row[1] + 14.0, p.text_dim, true);
        if addon.id != "appearance" {
            let on = addons::enabled(dir, addon.id);
            let s = switch_rect(row);
            crate::ui_zenith::rounded(
                batch,
                atlas,
                s[0],
                s[1],
                s[2],
                s[3],
                s[3] / 2.0,
                if on { p.accent } else { p.outline },
            );
            let knob_x = if on { s[0] + s[2] - 12.0 } else { s[0] + 2.0 };
            crate::ui_zenith::rounded(batch, atlas, knob_x, s[1] + 2.0, 8.0, 8.0, 4.0, p.text);
        }
    }
}

fn draw_settings(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    screen: (f32, f32),
    id: &str,
    state: &State,
    dir: &std::path::Path,
    cursor: (f32, f32),
) {
    let p = theme::palette();
    let title = Text::literal(tr(
        match id {
            "fps" => "zenith.fps.name",
            "keystrokes" => "zenith.keystrokes.name",
            _ => "zenith.appearance.name",
        },
        id,
    ));
    // Below the top bar, which is drawn over this screen.
    batch.text_styled(atlas, &title, 12.0, 30.0, p.text, true);
    if id == "appearance" {
        let selected = theme::current();
        for (rect, option) in theme_cards(screen) {
            let hover = cursor.0 >= rect[0]
                && cursor.0 < rect[0] + rect[2]
                && cursor.1 >= rect[1]
                && cursor.1 < rect[1] + rect[3];
            crate::ui_zenith::rounded(
                batch,
                atlas,
                rect[0],
                rect[1],
                rect[2],
                rect[3],
                8.0,
                if hover { p.button_hover } else { p.button },
            );
            // A little preview of each palette: a plate, a bar and a line.
            let preview = match option {
                UiTheme::Zenith => theme::zenith_palette(),
                UiTheme::Vanilla => theme::Palette {
                    background: 0xFF3B3B3B,
                    panel: 0xFF5A5A5A,
                    button: 0xFF6E6E6E,
                    button_hover: 0xFF8A8A8A,
                    button_disabled: 0xFF4A4A4A,
                    accent: 0xFF808080,
                    text: 0xFFFF_FFFF,
                    text_dim: 0xFFC0C0C0,
                    outline: 0xFF2A2A2A,
                },
            };
            // A miniature of the menu with this theme: the bar, the mark, its buttons and
            // the version line. Drawn from the palette, no pictures to ship.
            let (px, py) = (rect[0] + 8.0, rect[1] + 8.0);
            let (pw, ph) = (rect[2] - 16.0, 50.0);
            crate::ui_zenith::rounded(batch, atlas, px, py, pw, ph, 4.0, preview.background);
            batch.fill(atlas, px, py, pw, 9.0, preview.panel);
            crate::ui_zenith::rounded(
                batch,
                atlas,
                px + 4.0,
                py + 3.0,
                20.0,
                4.0,
                2.0,
                preview.accent,
            );
            let mark = atlas.sprites.get(ZENITH_MARK_SPRITE).map(|rect| {
                let height = 9.0;
                (rect, rect.w as f32 * (height / rect.h as f32), height)
            });
            match (option, mark) {
                (UiTheme::Zenith, Some((rect, width, height))) => batch.rect_uv(
                    px + (pw - width) / 2.0,
                    py + 13.0,
                    width,
                    height,
                    *rect,
                    0xFFFF_FFFF,
                ),
                (option, _) => {
                    let label = Text::literal(match option {
                        UiTheme::Zenith => "ZENITH",
                        UiTheme::Vanilla => "MINECRAFT",
                    });
                    let lw = atlas.font.width_text(&label);
                    batch.text_styled(
                        atlas,
                        &label,
                        px + (pw - lw) / 2.0,
                        py + 13.0,
                        preview.text,
                        true,
                    );
                }
            }
            for i in 0..3 {
                let (bx, by) = (px + pw / 2.0 - 30.0, py + 26.0 + i as f32 * 7.0);
                match option {
                    UiTheme::Zenith => crate::ui_zenith::rounded(
                        batch,
                        atlas,
                        bx,
                        by,
                        60.0,
                        5.0,
                        2.5,
                        if i == 0 {
                            preview.button_hover
                        } else {
                            preview.button
                        },
                    ),
                    UiTheme::Vanilla => {
                        batch.fill(atlas, bx, by, 60.0, 5.0, 0xFFA0_9A90);
                        batch.fill(atlas, bx, by, 60.0, 1.0, 0xFF6A_5A50);
                        batch.fill(atlas, bx, by + 4.0, 60.0, 1.0, 0xFF4A_3A30);
                    }
                }
            }
            batch.fill(atlas, px + 4.0, py + ph - 5.0, 40.0, 2.0, preview.text_dim);
            if option == selected {
                // The picked card gets an accent ring.
                for (x, y, w, hgt) in [
                    (rect[0], rect[1], rect[2], 2.0),
                    (rect[0], rect[1] + rect[3] - 2.0, rect[2], 2.0),
                    (rect[0], rect[1], 2.0, rect[3]),
                    (rect[0] + rect[2] - 2.0, rect[1], 2.0, rect[3]),
                ] {
                    batch.fill(atlas, x, y, w, hgt, p.accent);
                }
            }
            let label = Text::literal(match option {
                UiTheme::Zenith => tr("zenith.theme.zenith", "Zenith"),
                UiTheme::Vanilla => tr("zenith.theme.vanilla", "Vanilla"),
            });
            let tw = atlas.font.width_text(&label);
            batch.text_styled(
                atlas,
                &label,
                rect[0] + (rect[2] - tw) / 2.0,
                rect[1] + rect[3] - 20.0,
                if option == selected { p.accent } else { p.text },
                true,
            );
        }
    } else {
        // The corner, and the Addon's own extra, kept in `horizon-extras.txt`.
        let extras = crate::extras::Extras::load(dir);
        let (corner, toggle, toggle_label) = if id == "fps" {
            (
                extras.fps_corner,
                extras.fps_lows,
                tr("zenith.fps.lows", "Minimum and 1% low"),
            )
        } else {
            (
                extras.keystrokes_corner,
                extras.keystrokes_cps,
                tr("zenith.keystrokes.cps", "Clicks per second"),
            )
        };
        for (i, line) in [
            format!(
                "{}: {}",
                tr("zenith.addons.corner", "Position"),
                corner.label()
            ),
            format!(
                "{}: {}",
                toggle_label,
                tr(
                    if toggle { "options.on" } else { "options.off" },
                    if toggle { "On" } else { "Off" },
                )
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 54.0 + i as f32 * 22.0;
            crate::ui_zenith::rounded(batch, atlas, 12.0, y, 200.0, 18.0, 5.0, p.button);

            batch.text_styled(atlas, &Text::literal(line), 18.0, y + 5.0, p.text, true);
        }
    }
    let _ = state;
}

/// Handle a press on the Addons screen.
pub fn click(
    state: &mut State,
    dir: &std::path::Path,
    (x, y): (f32, f32),
    screen: (f32, f32),
) -> Outcome {
    if let Some(id) = state.open.take() {
        if id == "appearance" {
            for (rect, option) in theme_cards(screen) {
                if x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3] {
                    select_theme(dir, option);
                    state.open = Some(id);
                    return Outcome::Changed;
                }
            }
        } else {
            for i in 0..2 {
                let row = [12.0, 54.0 + i as f32 * 22.0, 200.0, 18.0];
                if x >= row[0] && x < row[0] + row[2] && y >= row[1] && y < row[1] + row[3] {
                    let mut extras = crate::extras::Extras::load(dir);
                    if i == 0 {
                        if id == "fps" {
                            extras.fps_corner = next_corner(extras.fps_corner);
                        } else {
                            extras.keystrokes_corner = next_corner(extras.keystrokes_corner);
                        }
                    } else if id == "fps" {
                        extras.fps_lows = !extras.fps_lows;
                    } else {
                        extras.keystrokes_cps = !extras.keystrokes_cps;
                    }
                    extras.save(dir);
                    state.open = Some(id);
                    return Outcome::Changed;
                }
            }
        }
        state.open = None;
        return Outcome::Back;
    }
    let panel = panel(screen);
    let list = addons::filter(&state.search, state.category);
    for (row, addon) in rows(panel, &list).into_iter().zip(&list) {
        if x < row[0] || x >= row[0] + row[2] || y < row[1] || y >= row[1] + row[3] {
            continue;
        }
        if addon.id != "appearance" {
            let s = switch_rect(row);
            if x >= s[0] {
                let on = addons::enabled(dir, addon.id);
                addons::set_enabled(dir, addon.id, !on);
                return Outcome::Changed;
            }
        }
        if addon.has_settings {
            state.open = Some(addon.id);
            return Outcome::Changed;
        }
    }
    Outcome::None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let d =
            std::env::temp_dir().join(format!("horizon-addons-ui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_panel_stays_inside_small_screens_below_the_top_bar() {
        for screen in [(320.0, 240.0), (854.0, 480.0), (240.0, 240.0)] {
            let [x, y, w, h] = panel(screen);
            assert!(x >= 0.0 && y >= crate::top_bar::HEIGHT);
            assert!(x + w <= screen.0 && y + h <= screen.1);
        }
    }

    #[test]
    fn the_appearance_cards_pick_a_theme_and_stay_on_the_page() {
        let d = dir("cards");
        let before = theme::current();
        let mut state = State {
            open: Some("appearance"),
            ..State::default()
        };
        let screen = (480.0, 320.0);
        let (rect, option) = theme_cards(screen)[1];
        assert_eq!(
            click(&mut state, &d, (rect[0] + 1.0, rect[1] + 1.0), screen),
            Outcome::Changed
        );
        assert_eq!(option, UiTheme::Vanilla);
        assert_eq!(theme::current(), UiTheme::Vanilla);
        assert_eq!(state.open, Some("appearance"));
        theme::set(before);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn picking_a_theme_changes_it_and_is_kept() {
        let d = dir("theme");
        let before = theme::current();
        select_theme(&d, UiTheme::Vanilla);
        assert_eq!(theme::current(), UiTheme::Vanilla);
        let text = std::fs::read_to_string(d.join("horizon.txt")).unwrap();
        assert!(text.contains("uiTheme:vanilla"));
        select_theme(&d, UiTheme::Zenith);
        assert_eq!(theme::current(), UiTheme::Zenith);
        let text = std::fs::read_to_string(d.join("horizon.txt")).unwrap();
        assert_eq!(
            text.lines().filter(|l| l.starts_with("uiTheme:")).count(),
            1
        );
        theme::set(before);
        let _ = std::fs::remove_dir_all(&d);
    }
}
