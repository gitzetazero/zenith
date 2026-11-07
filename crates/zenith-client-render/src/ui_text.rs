//! The design system's bundled faces (Geist) drawn into the HUD atlas, for the Zenith
//! interface. Vanilla screens keep the game's bitmap font; the menus that Laby draws with
//! its own type use this module while the theme is `Zenith`.
use crate::{
    hud::{HudAtlas, HudBatch, Rect},
    theme::UiTheme,
};
use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont as _};
use std::{cell::RefCell, collections::HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Regular,
    Medium,
    SemiBold,
    Mono,
}

struct Fonts {
    faces: [FontRef<'static>; 4],
}
impl Fonts {
    fn load() -> Self {
        let f = |b: &'static [u8]| FontRef::try_from_slice(b).expect("bundled font");
        Self {
            faces: [
                f(zenith_theme::fonts::GEIST_REGULAR),
                f(zenith_theme::fonts::GEIST_MEDIUM),
                f(zenith_theme::fonts::GEIST_SEMIBOLD),
                f(zenith_theme::fonts::GEIST_MONO),
            ],
        }
    }
    fn face(&self, f: Face) -> &FontRef<'static> {
        &self.faces[f as usize]
    }
    fn scale(&self, f: Face, px: f32) -> PxScale {
        let font = self.face(f);
        let upem = font.units_per_em().unwrap_or(1000.0);
        PxScale::from(px * font.height_unscaled() / upem)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    face: Face,
    glyph: u16,
    size: u16,
}
#[derive(Clone, Copy)]
struct Cached {
    rect: Option<Rect>,
    left: f32,
    top: f32,
}
struct Cache {
    fonts: Fonts,
    entries: HashMap<Key, Cached>,
    shelf: (u32, u32, u32),
    pending: Vec<crate::font::Patch>,
    atlas: usize,
}
thread_local! {
    static CACHE: RefCell<Option<Cache>> = const { RefCell::new(None) };
}
pub fn reset() {
    CACHE.with(|cache| *cache.borrow_mut() = None);
}
fn with_cache<R>(atlas: &HudAtlas, f: impl FnOnce(&mut Cache) -> R) -> R {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let id = atlas as *const HudAtlas as usize;
        let cache = c.get_or_insert_with(|| Cache {
            fonts: Fonts::load(),
            entries: HashMap::new(),
            shelf: (0, 0, 0),
            pending: Vec::new(),
            atlas: id,
        });
        if cache.atlas != id {
            cache.atlas = id;
            cache.entries.clear();
            cache.shelf = (0, 0, 0);
            cache.pending.clear();
        }
        f(cache)
    })
}
impl Cache {
    fn region(atlas: &HudAtlas) -> Option<Rect> {
        atlas.sprites.get("horizon/ui-text").copied()
    }
    fn place(&mut self, atlas: &HudAtlas, w: u32, h: u32) -> Option<(u32, u32, Rect)> {
        let region = Self::region(atlas)?;
        let (mut x, mut y, mut row) = self.shelf;
        if x + w + 1 > region.w {
            x = 0;
            y += row + 1;
            row = 0;
        }
        if y + h > region.h || w > region.w {
            self.entries.clear();
            self.shelf = (0, 0, 0);
            return None;
        }
        self.shelf = (x + w + 1, y, row.max(h));
        Some((x, y, region))
    }
    fn upload(&mut self, atlas: &HudAtlas, w: u32, h: u32, alpha: &[f32]) -> Option<Rect> {
        let (x, y, region) = self.place(atlas, w, h)?;
        let (ax, ay) = (
            (region.u0 * atlas.width as f32).round() as u32 + x,
            (region.v0 * atlas.height as f32).round() as u32 + y,
        );
        let rgba = alpha
            .iter()
            .flat_map(|a| [255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8])
            .collect();
        self.pending.push(crate::font::Patch {
            x: ax,
            y: ay,
            width: w,
            height: h,
            rgba,
        });
        let (aw, ah) = (atlas.width as f32, atlas.height as f32);
        Some(Rect {
            u0: ax as f32 / aw,
            v0: ay as f32 / ah,
            u1: (ax + w) as f32 / aw,
            v1: (ay + h) as f32 / ah,
            w,
            h,
        })
    }
    fn glyph(&mut self, atlas: &HudAtlas, face: Face, id: ab_glyph::GlyphId, px: f32) -> Cached {
        let size = (px * 4.0).round() as u16;
        let key = Key {
            face,
            glyph: id.0,
            size,
        };
        if let Some(c) = self.entries.get(&key) {
            return *c;
        }
        let scale = self.fonts.scale(face, f32::from(size) / 4.0);
        let glyph = id.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0));
        let made = match self.fonts.face(face).outline_glyph(glyph) {
            Some(outline) => {
                let b = outline.px_bounds();
                let (w, h) = (b.width() as u32, b.height() as u32);
                let mut alpha = vec![0.0; (w * h) as usize];
                outline.draw(|x, y, c| {
                    if x < w && y < h {
                        alpha[(y * w + x) as usize] = c;
                    }
                });
                for a in &mut alpha {
                    *a = a.powf(0.85);
                }
                Cached {
                    rect: self.upload(atlas, w, h, &alpha),
                    left: b.min.x,
                    top: b.min.y,
                }
            }
            None => Cached {
                rect: None,
                left: 0.0,
                top: 0.0,
            },
        };
        if made.rect.is_some() || made.left == 0.0 {
            self.entries.insert(key, made);
        }
        made
    }
}

/// The advance of `text` in GUI pixels.
pub fn measure(atlas: &HudAtlas, face: Face, size: f32, text: &str) -> f32 {
    with_cache(atlas, |c| {
        let font = c.fonts.face(face);
        let scaled = font.as_scaled(c.fonts.scale(face, size));
        let mut w = 0.0;
        let mut prev = None;
        for ch in text.chars() {
            let id = font.glyph_id(ch);
            if let Some(p) = prev {
                w += scaled.kern(p, id);
            }
            w += scaled.h_advance(id);
            prev = Some(id);
        }
        w
    })
}

/// Draw one line in the bundled face at `size` GUI pixels, top-left at `(x, y)`.
/// Returns its advance.
pub fn text(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    face: Face,
    size: f32,
    x: f32,
    y: f32,
    color: u32,
    text: &str,
) -> f32 {
    let s = batch.scale;
    let px = (size * s * 4.0).round() / 4.0;
    let line = size * 1.45;
    let (x0, y0) = (x * s, y * s);
    let real = color >> 24 != 0;
    let mut quads = Vec::new();
    let width = with_cache(atlas, |c| {
        let scale = c.fonts.scale(face, px);
        let (ascent, descent) = {
            let f = c.fonts.face(face).as_scaled(scale);
            (f.ascent(), f.descent())
        };
        let baseline = y0 + ((line * s - (ascent - descent)) / 2.0 + ascent).round();
        let mut pen = x0;
        let mut prev = None;
        for ch in text.chars() {
            let (id, advance, kern) = {
                let font = c.fonts.face(face);
                let id = font.glyph_id(ch);
                let f = font.as_scaled(scale);
                (id, f.h_advance(id), prev.map_or(0.0, |p| f.kern(p, id)))
            };
            pen += kern;
            if real && !ch.is_whitespace() {
                let g = c.glyph(atlas, face, id, px);
                if let Some(rect) = g.rect {
                    quads.push(((pen + g.left).round(), baseline + g.top, rect));
                }
            }
            pen += advance;
            prev = Some(id);
        }
        (pen - x0) / s
    });
    for (gx, gy, rect) in quads {
        batch.rect_uv(
            gx / s,
            gy / s,
            rect.w as f32 / s,
            rect.h as f32 / s,
            rect,
            color,
        );
    }
    let pending = with_cache(atlas, |c| std::mem::take(&mut c.pending));
    batch.patches.extend(pending);
    width
}

/// Draw `text` centred in `[x, x + w)`, cut with an ellipsis when it does not fit.
pub fn text_fitted(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    face: Face,
    size: f32,
    x: f32,
    w: f32,
    y: f32,
    color: u32,
    label: &str,
) {
    let mut shown = label.to_owned();
    if measure(atlas, face, size, label) > w {
        let chars: Vec<char> = label.chars().collect();
        let mut end = chars.len();
        while end > 0 {
            end -= 1;
            let mut cut: String = chars[..end].iter().collect();
            cut.push('…');
            if measure(atlas, face, size, &cut) <= w {
                shown = cut;
                break;
            }
        }
    }
    let tw = measure(atlas, face, size, &shown);
    text(
        batch,
        atlas,
        face,
        size,
        x + (w - tw) / 2.0,
        y,
        color,
        &shown,
    );
}

/// Whether the menus should use the bundled face.
pub fn themed() -> bool {
    crate::theme::current() == UiTheme::Zenith
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measuring_grows_with_the_text() {
        let atlas = HudAtlas::boot();
        assert!(measure(&atlas, Face::SemiBold, 16.0, "ZENITH") > 0.0);
        assert!(
            measure(&atlas, Face::SemiBold, 16.0, "ZENITH")
                > measure(&atlas, Face::SemiBold, 16.0, "ZEN")
        );
    }

    #[test]
    fn a_fitted_label_stays_inside_its_width() {
        let atlas = HudAtlas::boot();
        let mut batch = HudBatch::new([320, 240]);
        text_fitted(
            &mut batch,
            &atlas,
            Face::Medium,
            12.0,
            10.0,
            40.0,
            20.0,
            u32::MAX,
            "Multijugador y algo mas largo",
        );
        // The drawn width fits; ellipsis is a single glyph.
        assert!(batch.patches.iter().all(|p| p.width <= 64));
    }
}
