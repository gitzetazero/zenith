#![allow(dead_code)]
//! Laby's interactive menu background: little branded critters pace the ground
//! and a press on open ground picks one up; holding it dangles it under the
//! cursor and letting go throws it (the cursor's own speed becomes its fling).
//! Drawn in the Zenith palette, under the buttons, on the main menu only.

use crate::hud::{HudAtlas, HudBatch};

/// The ground the critters stand on: the window's bottom edge.
fn ground(h: f32) -> f32 {
    h - 2.0
}

/// Gravity and friends, in GUI pixels per second.
const GRAVITY: f32 = 640.0;
const WALK: f32 = 16.0;
const AIR_DRAG: f32 = 0.4;
const GROUND_FRICTION: f32 = 8.0;
const HOLD_SPRING: f32 = 320.0;
const HOLD_DAMPING: f32 = 10.0;

/// One critter: a pill-shaped body with a face, four stub legs and two eyes.
#[derive(Clone, Debug)]
pub struct Critter {
    /// The body's centre, in window pixels.
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    /// The face's side: -1 left or 1 right, from the walk direction.
    dir: f32,
    /// The body's half-height; the body runs 1.2 sizes wide.
    size: f32,
    /// The accent critter wears the theme's blue.
    accent: bool,
    /// Walk animation clock, in seconds.
    phase: f32,
    /// Landing squash, 0 = none, 1 = a hard hit.
    squash: f32,
    /// Time to the next walk decision (a pause, a turn or a hop), in seconds.
    wander: f32,
    held: bool,
    rng: u32,
}

impl Critter {
    fn spawn(x: f32, h: f32, accent: bool, size: f32) -> Self {
        Self {
            pos: (x, ground(h) - size),
            vel: (0.0, 0.0),
            dir: 1.0,
            size,
            accent,
            phase: x,
            squash: 0.0,
            wander: 0.0,
            held: false,
            rng: x as u32 ^ 0x9E37_79B9,
        }
    }

    fn on_ground(&self, h: f32) -> bool {
        self.pos.1 >= ground(h) - self.size - 0.5 && !self.held
    }

    fn held(&self) -> bool {
        self.held
    }

    fn next(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }
}

/// The menu's critter troupe and the cursor's own motion, for the fling.
#[derive(Default)]
pub struct State {
    critters: Vec<Critter>,
    hold: Option<usize>,
    cursor: (f32, f32),
    /// The cursor's speed, smoothed and in GUI pixels per second.
    cursor_vel: (f32, f32),
    last_ms: Option<i64>,
}

impl State {
    /// A troupe of `n` critters for a window of `w x h`, the middle one in the
    /// theme's accent colour.
    pub fn new(n: usize, w: f32, h: f32) -> Self {
        let mut state = Self::default();
        for i in 0..n {
            let spread = w / (n as f32 + 1.0);
            let size = match i % 3 {
                0 => 10.0,
                1 => 12.0,
                _ => 11.0,
            };
            let mut critter = Critter::spawn(spread * (i as f32 + 1.0), h, i == 1, size);
            critter.squash = 0.0;
            state.critters.push(critter);
        }
        state
    }

    /// Advance the physics by the time between `now_ms` and the last call,
    /// keeping the cursor in the picture.
    pub fn tick(&mut self, cursor: (f32, f32), w: f32, h: f32, now_ms: i64) {
        if self.critters.is_empty() {
            *self = State::new(3, w, h);
            self.last_ms = Some(now_ms);
        }
        let ms = self
            .last_ms
            .replace(now_ms)
            .map_or(16, |last| now_ms - last);
        let dt = ms.clamp(1, 50) as f32 / 1000.0;
        self.track_cursor(cursor, dt);
        if self.hold.is_some_and(|i| i >= self.critters.len()) {
            self.hold = None;
        }
        for (i, critter) in self.critters.iter_mut().enumerate() {
            let held = self.hold == Some(i);
            critter.held = held;
            critter.squash = (critter.squash - 6.0 * dt).max(0.0);
            if held {
                critter.follow(self.cursor, self.cursor_vel, dt, w);
                continue;
            }
            critter.fall(dt);
            critter.bounce(w, h);
            if critter.on_ground(h) {
                critter.wander(dt, self.hold.is_none());
                critter.pace(dt);
                critter.hop();
            }
        }
    }

    fn track_cursor(&mut self, cursor: (f32, f32), dt: f32) {
        let (px, py) = self.cursor;
        let step = ((cursor.0 - px) / dt, (cursor.1 - py) / dt);
        if px.is_finite() && py.is_finite() {
            self.cursor_vel.0 = 0.4 * self.cursor_vel.0 + 0.6 * step.0;
            self.cursor_vel.1 = 0.4 * self.cursor_vel.1 + 0.6 * step.1;
        }
        self.cursor = cursor;
    }

    /// The critters near a press on open ground jump in surprise.
    pub fn startle(&mut self, cursor: (f32, f32)) {
        for critter in &mut self.critters {
            let (dx, dy) = (cursor.0 - critter.pos.0, cursor.1 - critter.pos.1);
            if dy.hypot(dx) < 48.0 {
                critter.hop_up(-110.0);
            }
        }
    }

    /// A press that could hold a critter: the nearest one within reach. Returns
    /// whether the press is now busy holding one.
    pub fn grab(&mut self, cursor: (f32, f32)) -> bool {
        let reach = self
            .critters
            .iter()
            .enumerate()
            .filter(|(_, c)| Self::in_reach(c, cursor))
            .min_by_key(|(_, c)| {
                let (dx, dy) = (cursor.0 - c.pos.0, cursor.1 - c.pos.1);
                (dx.hypot(dy) * 8.0) as u32
            });
        let (index, _) = match reach {
            Some(found) => found,
            None => return false,
        };
        self.hold = Some(index);
        self.cursor_vel = (0.0, 0.0);
        true
    }

    fn in_reach(c: &Critter, cursor: (f32, f32)) -> bool {
        let (dx, dy) = (cursor.0 - c.pos.0, cursor.1 - c.pos.1);
        dx.hypot(dy) < c.size * 2.4
    }

    /// Letting go of a grab: the critter flies with the cursor's own speed,
    /// so a drag-and-flick throws it. Returns whether it was holding one.
    pub fn drop(&mut self) -> bool {
        let Some(index) = self.hold.take() else {
            return false;
        };
        if let Some(critter) = self.critters.get_mut(index) {
            critter.end_hold(self.cursor_vel);
            critter.squash = 0.0;
        }
        true
    }

    /// Whether the troupe is holding something right now.
    pub fn holding(&self) -> bool {
        self.hold.is_some()
    }

    pub fn draw(&self, batch: &mut HudBatch, atlas: &HudAtlas) {
        let h = batch.height;
        for critter in &self.critters {
            critter.draw(batch, atlas, h);
        }
    }

    pub fn count(&self) -> usize {
        self.critters.len()
    }
}

impl Critter {
    /// A held critter springs under the cursor and inherits its pace.
    fn follow(&mut self, cursor: (f32, f32), _cursor_vel: (f32, f32), dt: f32, w: f32) {
        let hang = self.size * 1.4;
        let target = (cursor.0, cursor.1 - hang);
        let stiffness = HOLD_SPRING * dt;
        self.vel.0 += (target.0 - self.pos.0) * stiffness;
        self.vel.1 += (target.1 - self.pos.1) * stiffness;
        self.vel.0 *= 1.0 - HOLD_DAMPING * dt;
        self.vel.1 *= 1.0 - HOLD_DAMPING * dt;
        self.pos.0 += self.vel.0 * dt;
        self.pos.1 += self.vel.1 * dt;
        self.pos.0 = self
            .pos
            .0
            .clamp(self.size * 1.4, (w - self.size * 1.4).max(0.0));
        if self.vel.0.abs() > 4.0 {
            self.dir = self.vel.0.signum();
        }
    }

    fn fall(&mut self, dt: f32) {
        self.vel.1 += GRAVITY * dt;
        let drag = 1.0 - AIR_DRAG * dt;
        self.vel.0 *= drag;
        self.pos.0 += self.vel.0 * dt;
        self.pos.1 += self.vel.1 * dt;
    }

    fn bounce(&mut self, w: f32, h: f32) {
        let floor = ground(h);
        if self.pos.1 + self.size >= floor {
            self.pos.1 = floor - self.size;
            if self.vel.1 > 90.0 {
                self.squash = clamp01(self.vel.1 / 480.0).max(0.25);
                self.vel.1 *= -0.4;
            } else {
                self.vel.1 = 0.0;
            }
            self.vel.0 *= 1.0 - GROUND_FRICTION * 0.016;
        }
        let margin = self.size * 1.6;
        if self.pos.0 < margin {
            self.pos.0 = margin;
            self.vel.0 = self.vel.0.abs() * 0.5;
            self.dir = 1.0;
        }
        if self.pos.0 > w - margin {
            self.pos.0 = w - margin;
            self.vel.0 = -self.vel.0.abs() * 0.5;
            self.dir = -1.0;
        }
    }

    /// Every few seconds the critter chooses a new pace: walk on, turn
    /// around or stand still looking at things.
    fn wander(&mut self, dt: f32, shy: bool) {
        self.wander = (self.wander - dt).max(0.0);
        if self.wander > 0.0 {
            return;
        }
        let roll = self.next() % 100;
        if roll < 25 && shy {
            self.wander = 0.9 + (self.next() % 70) as f32 / 100.0;
            self.dir = 0.0;
        } else {
            if roll < 55 && self.dir != 0.0 {
                self.dir = -self.dir;
            }
            if self.dir == 0.0 {
                self.dir = if self.next().is_multiple_of(2) {
                    -1.0
                } else {
                    1.0
                };
            }
            self.wander = 1.6 + (self.next() % 160) as f32 / 100.0;
        }
    }

    fn pace(&mut self, dt: f32) {
        let speed = self.dir * WALK;
        self.vel.0 += (speed - self.vel.0) * (2.0 * dt).min(1.0);
        self.pos.0 += self.vel.0 * dt;
        if self.dir != 0.0 {
            self.phase += dt * 6.0;
        }
    }

    fn hop(&mut self) {
        if self.wander <= 0.0 && self.next() % 100 < 30 {
            let impulse = -70.0 - (self.next() % 60) as f32;
            self.hop_up(impulse);
        }
    }

    fn hop_up(&mut self, impulse: f32) {
        self.vel.1 = impulse;
        self.vel.0 += self.dir * 10.0;
    }

    fn end_hold(&mut self, fling: (f32, f32)) {
        self.vel.0 = fling.0.clamp(-600.0, 600.0) * 0.9;
        self.vel.1 = fling.1.clamp(-600.0, 600.0) * 0.9 - 40.0;
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

impl Critter {
    /// The body, face and legs, in the Zenith palette: whites and panels,
    /// the accent critter rings blue.
    fn draw(&self, batch: &mut HudBatch, atlas: &HudAtlas, h: f32) {
        let p = crate::theme::palette();
        let (x, y) = self.pos;
        let s = 1.0 + self.squash * 0.3;
        let v = 1.0 - self.squash * 0.35;
        let r = self.size;
        let on_ground = self.on_ground(h);
        // A shadow on the ground, lifted with the critter.
        let air = (ground(h) - y).max(0.0);
        let shadow = (0.5 - (air / 240.0)).clamp(0.05, 0.5);
        crate::ui_zenith::rounded(
            batch,
            atlas,
            x - r * 1.1,
            ground(h) - 2.5,
            r * 2.2,
            3.0,
            1.5,
            fill(p.background, (shadow * 255.0) as u8),
        );
        // Legs first, behind the body, then the body and the face.
        for (i, leg) in [-1.0, -0.35, 0.35, 1.0].iter().enumerate() {
            let step = if i.is_multiple_of(2) {
                self.phase.sin()
            } else {
                -self.phase.sin()
            };
            let sway = if on_ground { step * r * 0.25 } else { 0.0 };
            let lx = x + leg * r * 0.9 + sway;
            let ly = y + r * v - r * 0.55;
            crate::ui_zenith::rounded(
                batch,
                atlas,
                lx - 1.0,
                ly,
                2.0,
                r * 0.75,
                1.0,
                shade(p.panel, -18),
            );
        }
        crate::ui_zenith::rounded(
            batch,
            atlas,
            x - r * 1.2 * s,
            y - r * v,
            r * 2.4 * s,
            r * 2.0 * v,
            r * 1.0,
            body(p, self.accent),
        );
        // The face on the front side, with an eye looking along the walk.
        let face = 1.5 * r;
        let fx = if self.dir >= 0.0 {
            x + r * 0.75
        } else {
            x - r * 0.75 - face
        };
        crate::ui_zenith::rounded(
            batch,
            atlas,
            fx,
            y - r * 0.9,
            face,
            face,
            r * 0.75,
            shade(p.panel, -18),
        );
        let eye = 1.6;
        let ex = if self.dir >= 0.0 {
            fx + face - r * 0.55
        } else {
            fx + r * 0.55
        };
        crate::ui_zenith::rounded(
            batch,
            atlas,
            ex - eye / 2.0,
            y - r * 0.62,
            eye,
            eye,
            0.8,
            p.background,
        );
    }
}

fn fill(color: u32, alpha: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((alpha as u32) << 24)
}

fn shade(color: u32, amount: i32) -> u32 {
    let darken = |c: u32| -> u32 { (c as i32 + amount).clamp(0, 255) as u32 };
    let (r, g, b) = (
        darken(color & 0xFF),
        darken((color >> 8) & 0xFF),
        darken((color >> 16) & 0xFF),
    );
    r | (g << 8) | (b << 16) | (color & 0xFF00_0000)
}

/// The critter's body: white for the troupe, the accent blue for the leader.
fn body(p: crate::theme::Palette, accent: bool) -> u32 {
    if accent { p.accent } else { p.text }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 854.0;
    const H: f32 = 480.0;
    const STEP: i64 = 16;

    fn settled(state: &mut State, frames: usize, cursor: (f32, f32)) -> i64 {
        let mut ms = 0;
        for _ in 0..frames {
            ms += STEP;
            state.tick(cursor, W, H, ms);
        }
        ms
    }

    #[test]
    fn critters_rest_on_the_ground() {
        let mut state = State::new(3, W, H);
        settled(&mut state, 900, (W / 2.0, H * 0.6));
        for critter in &state.critters {
            assert!((critter.pos.1 - (ground(H) - critter.size)).abs() < 12.0);
        }
    }

    #[test]
    fn a_hold_springs_under_the_cursor_and_a_flick_throws_it() {
        let mut state = State::new(3, W, H);
        let mut ms = settled(&mut state, 250, (W / 2.0, H * 0.6));
        // Grab the middle critter, then sweep the cursor fast to the right.
        let anchor = state.critters[1].pos;
        assert!(state.grab(anchor), "a press on the critter holds it");
        let frames_ms = 20 * STEP;
        for i in 1..=20 {
            ms += STEP;
            let k = i as f32;
            state.tick((anchor.0 + k * 20.0, anchor.1 - k * 3.0), W, H, ms);
        }
        assert!(state.holding());
        let vel_x = state.critters[1].vel.0;
        if vel_x <= 0.0 {
            panic!("the held critter should track a rightward sweep");
        }
        state.drop();
        assert!(!state.holding());
        let _ = frames_ms;
    }

    #[test]
    fn a_press_far_from_every_critter_is_free_and_startles_the_near_ones() {
        let mut state = State::new(3, W, H);
        settled(&mut state, 250, (W / 2.0, H * 0.6));
        let miss = !state.grab((8.0, H / 4.0));
        assert!(miss);
        state.startle((W / 2.0, ground(H) - 12.0));
        assert!(!state.holding(), "a miss never holds anything");
    }

    #[test]
    fn a_startle_hops_the_critters_under_the_cursor() {
        let mut state = State::new(3, W, H);
        settled(&mut state, 250, (W / 2.0, H * 0.6));
        state.startle((W / 2.0, ground(H) - 12.0));
        assert!(state.critters.iter().any(|c| c.vel.1 < -20.0));
    }

    #[test]
    fn a_flying_critter_comes_back_down() {
        let mut state = State::new(3, W, H);
        let mut ms = settled(&mut state, 250, (W / 2.0, H * 0.6));
        let index = 1;
        state.critters[index].vel.1 = -560.0;
        state.hold = Some(index);
        state.drop();
        for _ in 0..400 {
            ms += STEP;
            state.tick((W / 2.0, H * 0.6), W, H, ms);
        }
        for critter in &state.critters {
            assert!((critter.pos.1 - (ground(H) - critter.size)).abs() < 12.0);
        }
    }
}
