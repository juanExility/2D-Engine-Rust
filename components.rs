use crate::engine::{color::Color, math::{Vec2, Rect}};

// ── Transform ─────────────────────────────────────────────────────────────────

/// World-space position, rotation, and scale of an entity.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,   // radians
    pub scale:    Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }
    pub fn at(x: f32, y: f32) -> Self {
        Self { position: Vec2::new(x, y), rotation: 0.0, scale: Vec2::ONE }
    }
    pub fn with_rotation(mut self, r: f32) -> Self { self.rotation = r; self }
    pub fn with_scale(mut self, s: Vec2) -> Self { self.scale = s; self }
}

impl Default for Transform {
    fn default() -> Self { Self::at(0.0, 0.0) }
}

// ── Velocity ──────────────────────────────────────────────────────────────────

/// Linear + angular velocity.
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear:  Vec2,  // pixels / second
    pub angular: f32,   // radians / second
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self { Self { linear: Vec2::new(x, y), angular: 0.0 } }
    pub fn zero() -> Self { Self::default() }
    pub fn from_angle(angle: f32, speed: f32) -> Self {
        Self::new(angle.cos() * speed, angle.sin() * speed)
    }
}

// ── Sprite ────────────────────────────────────────────────────────────────────

/// How an entity should be drawn.
#[derive(Debug, Clone, Copy)]
pub enum SpriteShape {
    /// Filled rectangle of the given size.
    Rect { w: f32, h: f32 },
    /// Filled circle of the given radius.
    Circle { radius: f32 },
    /// A diamond (square rotated 45°).
    Diamond { size: f32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub shape:  SpriteShape,
    pub color:  Color,
    /// Drawn on top of the fill if Some.
    pub outline: Option<Color>,
    /// Pixel offset from the entity's transform position.
    pub offset: Vec2,
    pub visible: bool,
    /// Higher values are drawn last (on top).
    pub z_order: i32,
}

impl Sprite {
    pub fn rect(w: f32, h: f32, color: Color) -> Self {
        Self {
            shape: SpriteShape::Rect { w, h },
            color,
            outline: None,
            offset: Vec2::new(-w * 0.5, -h * 0.5), // centered by default
            visible: true,
            z_order: 0,
        }
    }
    pub fn circle(radius: f32, color: Color) -> Self {
        Self {
            shape: SpriteShape::Circle { radius },
            color,
            outline: None,
            offset: Vec2::ZERO,
            visible: true,
            z_order: 0,
        }
    }
    pub fn diamond(size: f32, color: Color) -> Self {
        Self {
            shape: SpriteShape::Diamond { size },
            color,
            outline: None,
            offset: Vec2::ZERO,
            visible: true,
            z_order: 0,
        }
    }
    pub fn with_outline(mut self, color: Color) -> Self { self.outline = Some(color); self }
    pub fn with_z(mut self, z: i32) -> Self { self.z_order = z; self }
    pub fn with_offset(mut self, offset: Vec2) -> Self { self.offset = offset; self }
}

// ── Collider ──────────────────────────────────────────────────────────────────

/// Axis-aligned bounding box used for collision detection.
#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub half_size: Vec2,   // half-extents centered on the transform position
    pub layer:     u32,    // bitmask – entities on the same layer can collide
    pub is_trigger: bool,  // if true, overlaps are reported but no response applied
}

impl Collider {
    pub fn new(w: f32, h: f32) -> Self {
        Self { half_size: Vec2::new(w * 0.5, h * 0.5), layer: 1, is_trigger: false }
    }
    pub fn circle_approx(radius: f32) -> Self { Self::new(radius * 2.0, radius * 2.0) }
    pub fn with_layer(mut self, l: u32) -> Self { self.layer = l; self }
    pub fn trigger(mut self) -> Self { self.is_trigger = true; self }

    pub fn rect_for(&self, pos: Vec2) -> Rect {
        Rect::new(
            pos.x - self.half_size.x,
            pos.y - self.half_size.y,
            self.half_size.x * 2.0,
            self.half_size.y * 2.0,
        )
    }
}

// ── Tag ───────────────────────────────────────────────────────────────────────

/// A string label attached to an entity; useful for identification in systems.
#[derive(Debug, Clone)]
pub struct Tag(pub String);

impl Tag {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn is(&self, s: &str) -> bool { self.0 == s }
}

// ── Health ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    /// Seconds of invincibility after being hit.
    pub invincible_timer: f32,
}

impl Health {
    pub fn new(max: f32) -> Self { Self { current: max, max, invincible_timer: 0.0 } }
    pub fn is_alive(&self) -> bool { self.current > 0.0 }
    pub fn is_invincible(&self) -> bool { self.invincible_timer > 0.0 }

    pub fn take_damage(&mut self, amount: f32, invincibility_secs: f32) {
        if self.is_invincible() { return; }
        self.current = (self.current - amount).max(0.0);
        self.invincible_timer = invincibility_secs;
    }

    pub fn tick(&mut self, dt: f32) {
        self.invincible_timer = (self.invincible_timer - dt).max(0.0);
    }

    pub fn fraction(&self) -> f32 { self.current / self.max }
}

// ── Lifetime ──────────────────────────────────────────────────────────────────

/// Automatically despawn the entity after `seconds`.
#[derive(Debug, Clone, Copy)]
pub struct Lifetime(pub f32);
