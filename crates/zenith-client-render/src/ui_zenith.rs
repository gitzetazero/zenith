//! The Zenith look for [`crate::ui`]'s shared widgets: dark rounded plates, one blue accent
//! and white text (`docs/superpowers/specs/2026-11-07-zenith-ui-design.md`). Every function
//! here is reached only while [`crate::theme::current`] is `Zenith`; the `Vanilla` branch in
//! `ui.rs` is untouched, so a player who picks `Vanilla` gets today's screens.
use crate::{
    hud::{HudAtlas, HudBatch},
    text::Text,
    theme::palette,
};

/// A filled rectangle with rounded corners, in GUI pixels. The corners are a one-pixel
/// staircase (no new geometry in `HudBatch`), which at a GUI scale of 2 or more reads round.
pub fn rounded(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: u32,
) {
    if w <= 0.0 || h <= 0.0 || color >> 24 == 0 {
        return;
    }
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    if r < 1.0 {
        batch.fill(atlas, x, y, w, h, color);
        return;
    }
    let ri = r.round();
    batch.fill(atlas, x + ri, y, w - 2.0 * ri, h, color);
    batch.fill(atlas, x, y + ri, ri, h - 2.0 * ri, color);
    batch.fill(atlas, x + w - ri, y + ri, ri, h - 2.0 * ri, color);
    let mut row = 0.0;
    while row < ri {
        let dy = row + 0.5;
        let dx = (ri * ri - (ri - dy) * (ri - dy)).max(0.0).sqrt();
        // Top left and top right.
        batch.fill(atlas, x + ri - dx, y + row, dx, 1.0, color);
        batch.fill(atlas, x + w - ri, y + row, dx, 1.0, color);
        // Bottom left and bottom right.
        let yy = y + h - 1.0 - row;
        batch.fill(atlas, x + ri - dx, yy, dx, 1.0, color);
        batch.fill(atlas, x + w - ri, yy, dx, 1.0, color);
        row += 1.0;
    }
}

/// A 1 px rounded outline.
fn outline(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32, color: u32) {
    batch.fill(atlas, x + 2.0, y, w - 4.0, 1.0, color);
    batch.fill(atlas, x + 2.0, y + h - 1.0, w - 4.0, 1.0, color);
    batch.fill(atlas, x, y + 2.0, 1.0, h - 4.0, color);
    batch.fill(atlas, x + w - 1.0, y + 2.0, 1.0, h - 4.0, color);
}

/// `ui::button_with_room` with the same hit box and room for an action glyph, drawn as a
/// rounded plate with a centred label.
#[allow(clippy::too_many_arguments)]
pub fn button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [x, y, w, h]: [f32; 4],
    label: &Text,
    active: bool,
    hover: bool,
    room: f32,
) {
    let p = palette();
    let fill = if !active {
        p.button_disabled
    } else if hover {
        p.button_hover
    } else {
        p.button
    };
    let radius = (h / 2.0).min(6.0);
    rounded(batch, atlas, x, y, w, h, radius, fill);
    outline(batch, atlas, x, y, w, h, p.outline);
    let color = if active { p.text } else { p.text_dim };
    let size = 12.0;
    crate::ui_text::text_fitted(
        batch,
        atlas,
        crate::ui_text::Face::Medium,
        size,
        x + 2.0 + room,
        w - 4.0 - room,
        y + ((h - size * 1.45) / 2.0).floor(),
        color,
        &label.plain(),
    );
}

/// `ui::slider_ext`: a thin track, the filled part in the accent, a round knob and the
/// label on the left, keeping the vanilla value placement.
#[allow(clippy::too_many_arguments)]
pub fn slider(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    label: &Text,
    value: f32,
    hover: bool,
    active: bool,
    dragging: bool,
) {
    let p = palette();
    let track_y = y + 7.0;
    rounded(batch, atlas, x + 2.0, track_y, w - 4.0, 6.0, 3.0, p.outline);
    let filled = (value.clamp(0.0, 1.0) * (w - 8.0)).max(0.0);
    if filled > 0.0 {
        rounded(batch, atlas, x + 4.0, track_y, filled, 6.0, 3.0, p.accent);
    }
    let knob = if active && (hover || dragging) {
        p.text
    } else {
        p.text_dim
    };
    let kx = x + 4.0 + (value.clamp(0.0, 1.0) * (w - 8.0)) - 4.0;
    rounded(batch, atlas, kx, y + 5.0, 8.0, 10.0, 5.0, knob);
    crate::ui_text::text(
        batch,
        atlas,
        crate::ui_text::Face::Medium,
        11.0,
        x + 3.0,
        y + 5.0,
        if active { p.text } else { p.text_dim },
        &label.plain(),
    );
}

/// The dark veil over the title's panorama, replacing the vanilla menu texture.
pub fn background(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32) {
    batch.fill(atlas, x, y, w, h, 0xE014_161D);
}

/// Header and footer rules: 1 px lines in the outline colour.
pub fn separators(batch: &mut HudBatch, atlas: &HudAtlas, header_bottom: f32, footer_top: f32) {
    let p = palette();
    batch.fill(atlas, 0.0, header_bottom - 1.0, batch.width, 1.0, p.outline);
    batch.fill(atlas, 0.0, footer_top, batch.width, 1.0, p.outline);
}

/// A list's plate between the separators.
pub fn list_background(batch: &mut HudBatch, atlas: &HudAtlas, top: f32, bottom: f32) {
    batch.fill(atlas, 0.0, top, batch.width, bottom - top, 0xC01B_1E26);
}

/// The scrollbar's track and thumb, as rounded plates.
pub fn scrollbar(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    top: f32,
    bottom: f32,
    content: f32,
    scroll: f32,
) {
    let Some((y, bar)) = crate::ui::scrollbar_thumb(top, bottom, content, scroll) else {
        return;
    };
    let p = palette();
    rounded(batch, atlas, x, top, 6.0, bottom - top, 3.0, 0x802A_2D38);
    rounded(batch, atlas, x, y, 6.0, bar, 3.0, p.text_dim);
}

/// A tooltip's plate, with the same text positions as `ui::tooltip_box`.
pub fn tooltip_box(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    lines: &[Text],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let p = palette();
    rounded(
        batch,
        atlas,
        x - 8.0,
        y - 8.0,
        w + 16.0,
        h + 16.0,
        6.0,
        0xF01B_1E26,
    );
    outline(
        batch,
        atlas,
        x - 8.0,
        y - 8.0,
        w + 16.0,
        h + 16.0,
        p.outline,
    );
    let mut ly = y;
    for (i, line) in lines.iter().enumerate() {
        batch.text_styled(atlas, line, x, ly, p.text, true);
        ly += if i == 0 { 12.0 } else { 10.0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{self, UiTheme};

    fn atlas() -> HudAtlas {
        HudAtlas::boot()
    }

    #[test]
    fn a_zero_radius_plate_is_one_rectangle() {
        let atlas = atlas();
        let mut batch = HudBatch::new([320, 240]);
        rounded(
            &mut batch,
            &atlas,
            10.0,
            10.0,
            100.0,
            20.0,
            0.0,
            0xFFFF_FFFF,
        );
        assert_eq!(batch.vertices.len(), 6);
    }

    #[test]
    fn rounded_plates_add_corner_steps() {
        let atlas = atlas();
        let mut batch = HudBatch::new([320, 240]);
        rounded(
            &mut batch,
            &atlas,
            10.0,
            10.0,
            100.0,
            20.0,
            6.0,
            0xFFFF_FFFF,
        );
        assert!(batch.vertices.len() > 6);
    }

    /// The one test that changes the process-wide theme, so it is serialized with itself.
    #[test]
    fn the_zenith_button_differs_from_vanillas() {
        let atlas = atlas();
        let label = Text::literal("Options...");
        theme::set(UiTheme::Vanilla);
        let mut vanilla = HudBatch::new([320, 240]);
        crate::ui::button(
            &mut vanilla,
            &atlas,
            10.0,
            10.0,
            150.0,
            20.0,
            &label,
            true,
            false,
        );

        theme::set(UiTheme::Zenith);
        let mut zenith = HudBatch::new([320, 240]);
        crate::ui::button(
            &mut zenith,
            &atlas,
            10.0,
            10.0,
            150.0,
            20.0,
            &label,
            true,
            false,
        );
        theme::set(UiTheme::Vanilla);

        assert!(!vanilla.vertices.is_empty());
        assert_ne!(vanilla.vertices.len(), zenith.vertices.len());
        assert!(
            zenith
                .vertices
                .iter()
                .any(|v| v.color == theme::palette().accent),
            "the accent fills part of the plate"
        );
    }
}
