//! The FPS and Keystrokes Addons drawn in the corners: the frame counter
//! (`extras::FpsCounter`) and the held keys and clicks, switched on from the Addons
//! screen. The lines and cells are pure functions so the layout is tested without a GPU.
use zenith_client::{
    addons,
    extras::{Corner, Extras},
    extras_hud::Corners,
    hud::{HudAtlas, HudBatch, rgba},
};

/// The keys the keystroke Addon draws and whether each is held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
    pub attack: bool,
    pub use_item: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub label: String,
    pub pressed: bool,
}

/// The FPS box's lines: the rate, and with `lows`, the minimum and 1% low.
pub fn fps_lines(fps: f32, lows: Option<(f32, f32)>) -> Vec<String> {
    let mut lines = vec![format!("{} FPS", fps.round() as i64)];
    if let Some((min, low)) = lows {
        lines.push(format!(
            "min {} · 1% {}",
            min.round() as i64,
            low.round() as i64
        ));
    }
    lines
}

/// The keys in three rows (W; A S D; Space Shift Ctrl) and, with `cps`, the mouse row.
pub fn keystroke_cells(pressed: &KeyState, cps: Option<(u32, u32)>) -> Vec<Cell> {
    let mut cells = vec![
        Cell {
            label: "W".into(),
            pressed: pressed.forward,
        },
        Cell {
            label: "A".into(),
            pressed: pressed.left,
        },
        Cell {
            label: "S".into(),
            pressed: pressed.back,
        },
        Cell {
            label: "D".into(),
            pressed: pressed.right,
        },
        Cell {
            label: "Space".into(),
            pressed: pressed.jump,
        },
        Cell {
            label: "Shift".into(),
            pressed: pressed.sneak,
        },
        Cell {
            label: "Ctrl".into(),
            pressed: pressed.sprint,
        },
    ];
    if let Some((left, right)) = cps {
        cells.push(Cell {
            label: format!("LMB {left}"),
            pressed: pressed.attack,
        });
        cells.push(Cell {
            label: format!("RMB {right}"),
            pressed: pressed.use_item,
        });
    }
    cells
}

/// Draw the Addons that are on, in the corners `horizon-extras.txt` names.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    extras: &Extras,
    dir: &std::path::Path,
    fps: &[String],
    cps: Option<(u32, u32)>,
    pressed: &KeyState,
) {
    if addons::enabled(dir, "fps") && !fps.is_empty() {
        let lines: Vec<(&str, u32)> = fps.iter().map(|l| (l.as_str(), 0xFFFFFF)).collect();
        zenith_client::extras_hud::draw_lines(batch, atlas, corners, extras.fps_corner, &lines);
    }
    if addons::enabled(dir, "keystrokes") {
        let cells = keystroke_cells(pressed, cps);
        let cell_w = cells
            .iter()
            .map(|c| HudBatch::text_width(atlas, &c.label) + 8.0)
            .fold(20.0_f32, f32::max);
        let cell_h = 14.0;
        let rows = 3 + u32::from(cps.is_some());
        let (w, h) = (cell_w * 3.0 + 4.0, cell_h * rows as f32 + 2.0);
        let (x, y) = corners.place(extras.keystrokes_corner, w, h);
        batch.fill(atlas, x, y, w, h, 0x7800_0000);
        for (i, cell) in cells.iter().enumerate() {
            let (row, col) = match i {
                0 => (0, 1),
                1..=3 => (1, i - 1),
                4..=6 => (2, i - 4),
                _ => (3, i - 7),
            };
            let cx = x + 2.0 + col as f32 * cell_w;
            let cy = y + 1.0 + row as f32 * cell_h;
            if cell.pressed {
                batch.fill(
                    atlas,
                    cx,
                    cy,
                    cell_w - 2.0,
                    cell_h - 2.0,
                    rgba(74, 108, 247, 255),
                );
            }
            let tw = HudBatch::text_width(atlas, &cell.label);
            batch.text(
                atlas,
                &cell.label,
                cx + (cell_w - 2.0 - tw) / 2.0,
                cy + 2.0,
                u32::MAX,
                true,
            );
        }
    }
}

/// The corner a settings row cycles.
pub fn next_corner(corner: Corner) -> Corner {
    match corner {
        Corner::TopLeft => Corner::TopRight,
        Corner::TopRight => Corner::BottomRight,
        Corner::BottomRight => Corner::BottomLeft,
        Corner::BottomLeft => Corner::TopLeft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fps_box_shows_the_rate_and_its_lows() {
        let one = fps_lines(144.0, None);
        assert_eq!(one.len(), 1);
        assert!(one[0].contains("144"));
        let two = fps_lines(144.0, Some((120.0, 90.0)));
        assert_eq!(two.len(), 2);
        assert!(two[1].contains("120") && two[1].contains("90"));
    }

    #[test]
    fn only_held_keys_are_marked_and_clicks_appear_only_with_cps() {
        let pressed = KeyState {
            forward: true,
            jump: true,
            ..KeyState::default()
        };
        let cells = keystroke_cells(&pressed, None);
        assert_eq!(cells.len(), 7);
        assert!(cells.iter().filter(|c| c.pressed).count() == 2);
        assert_eq!(cells[0].label, "W");
        assert!(cells[0].pressed && !cells[1].pressed);
        assert_eq!(keystroke_cells(&pressed, Some((3, 1))).len(), 9);
    }

    #[test]
    fn the_corner_cycles_through_all_four() {
        let mut c = Corner::TopLeft;
        for _ in 0..4 {
            c = next_corner(c);
        }
        assert_eq!(c, Corner::TopLeft);
    }
}
