use std::collections::HashSet;
use crate::engine::math::Vec2;

// ── Key ───────────────────────────────────────────────────────────────────────

/// Logical keys the engine understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Digits
    Key0, Key1, Key2, Key3, Key4,
    Key5, Key6, Key7, Key8, Key9,
    // Arrows
    Up, Down, Left, Right,
    // Special
    Space, Enter, Escape, Backspace, Tab,
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt,
    // Function
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton { Left, Right, Middle }

// ── InputState ────────────────────────────────────────────────────────────────

/// Snapshot of keyboard + mouse state for one frame.
pub struct Input {
    current:  HashSet<Key>,
    previous: HashSet<Key>,
    pub mouse_pos:   Vec2,
    pub mouse_delta: Vec2,
    mouse_current:  [bool; 3],
    mouse_previous: [bool; 3],
    pub scroll_delta: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            current:  HashSet::new(),
            previous: HashSet::new(),
            mouse_pos:   Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            mouse_current:  [false; 3],
            mouse_previous: [false; 3],
            scroll_delta: 0.0,
        }
    }

    /// Pull the latest state from minifb.
    pub fn update(&mut self, window: &minifb::Window) {
        // ── Keyboard ──────────────────────────────────────────────────────────
        self.previous = self.current.clone();
        self.current.clear();
        for mk in window.get_keys() {
            if let Some(k) = from_minifb(mk) {
                self.current.insert(k);
            }
        }

        // ── Mouse ─────────────────────────────────────────────────────────────
        let prev_mouse = self.mouse_pos;
        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            self.mouse_pos = Vec2::new(mx, my);
        }
        self.mouse_delta = self.mouse_pos - prev_mouse;

        self.mouse_previous = self.mouse_current;
        self.mouse_current[0] = window.get_mouse_down(minifb::MouseButton::Left);
        self.mouse_current[1] = window.get_mouse_down(minifb::MouseButton::Right);
        self.mouse_current[2] = window.get_mouse_down(minifb::MouseButton::Middle);

        self.scroll_delta = window.get_scroll_wheel().map_or(0.0, |(_, y)| y);
    }

    // ── Key queries ───────────────────────────────────────────────────────────

    /// Key is currently held.
    #[inline] pub fn down(&self, key: Key) -> bool { self.current.contains(&key) }
    /// Key was pressed this frame (not last frame).
    #[inline] pub fn pressed(&self, key: Key) -> bool {
        self.current.contains(&key) && !self.previous.contains(&key)
    }
    /// Key was released this frame.
    #[inline] pub fn released(&self, key: Key) -> bool {
        !self.current.contains(&key) && self.previous.contains(&key)
    }

    /// Returns a direction vector from two axis keys (e.g. A/D → horizontal).
    pub fn axis(
        &self,
        neg_x: Key, pos_x: Key,
        neg_y: Key, pos_y: Key,
    ) -> Vec2 {
        let x = if self.down(pos_x) { 1.0 } else { 0.0 }
              - if self.down(neg_x) { 1.0 } else { 0.0 };
        let y = if self.down(pos_y) { 1.0 } else { 0.0 }
              - if self.down(neg_y) { 1.0 } else { 0.0 };
        Vec2::new(x, y)
    }

    // ── Mouse queries ─────────────────────────────────────────────────────────

    #[inline] fn btn_idx(b: MouseButton) -> usize {
        match b { MouseButton::Left=>0, MouseButton::Right=>1, MouseButton::Middle=>2 }
    }

    #[inline] pub fn mouse_down(&self, b: MouseButton) -> bool {
        self.mouse_current[Self::btn_idx(b)]
    }
    #[inline] pub fn mouse_pressed(&self, b: MouseButton) -> bool {
        let i = Self::btn_idx(b);
        self.mouse_current[i] && !self.mouse_previous[i]
    }
    #[inline] pub fn mouse_released(&self, b: MouseButton) -> bool {
        let i = Self::btn_idx(b);
        !self.mouse_current[i] && self.mouse_previous[i]
    }
}

impl Default for Input { fn default() -> Self { Self::new() } }

// ── Key mapping ───────────────────────────────────────────────────────────────

fn from_minifb(k: minifb::Key) -> Option<Key> {
    use minifb::Key as M;
    Some(match k {
        M::A=>Key::A, M::B=>Key::B, M::C=>Key::C, M::D=>Key::D,
        M::E=>Key::E, M::F=>Key::F, M::G=>Key::G, M::H=>Key::H,
        M::I=>Key::I, M::J=>Key::J, M::K=>Key::K, M::L=>Key::L,
        M::M=>Key::M, M::N=>Key::N, M::O=>Key::O, M::P=>Key::P,
        M::Q=>Key::Q, M::R=>Key::R, M::S=>Key::S, M::T=>Key::T,
        M::U=>Key::U, M::V=>Key::V, M::W=>Key::W, M::X=>Key::X,
        M::Y=>Key::Y, M::Z=>Key::Z,
        M::Key0=>Key::Key0, M::Key1=>Key::Key1, M::Key2=>Key::Key2,
        M::Key3=>Key::Key3, M::Key4=>Key::Key4, M::Key5=>Key::Key5,
        M::Key6=>Key::Key6, M::Key7=>Key::Key7, M::Key8=>Key::Key8,
        M::Key9=>Key::Key9,
        M::Up=>Key::Up, M::Down=>Key::Down,
        M::Left=>Key::Left, M::Right=>Key::Right,
        M::Space=>Key::Space, M::Enter=>Key::Enter,
        M::Escape=>Key::Escape, M::Backspace=>Key::Backspace,
        M::Tab=>Key::Tab,
        M::LeftShift=>Key::LShift, M::RightShift=>Key::RShift,
        M::LeftCtrl=>Key::LCtrl,  M::RightCtrl=>Key::RCtrl,
        M::LeftAlt=>Key::LAlt,    M::RightAlt=>Key::RAlt,
        M::F1=>Key::F1,   M::F2=>Key::F2,   M::F3=>Key::F3,
        M::F4=>Key::F4,   M::F5=>Key::F5,   M::F6=>Key::F6,
        M::F7=>Key::F7,   M::F8=>Key::F8,   M::F9=>Key::F9,
        M::F10=>Key::F10, M::F11=>Key::F11, M::F12=>Key::F12,
        _ => return None,
    })
}
