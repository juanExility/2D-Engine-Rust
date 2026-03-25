/// RGBA color with helpers for blending and conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline] pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
    #[inline] pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self::rgba(r, g, b, 255) }

    /// Pack into `0x00RRGGBB` for minifb pixel buffer.
    #[inline] pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Alpha-blend `self` (background) with `over` (foreground).
    pub fn blend(self, over: Color) -> Color {
        if over.a == 255 { return over; }
        if over.a == 0   { return self; }
        let a  = over.a as u32;
        let ia = 255 - a;
        Color::rgb(
            ((self.r as u32 * ia + over.r as u32 * a) / 255) as u8,
            ((self.g as u32 * ia + over.g as u32 * a) / 255) as u8,
            ((self.b as u32 * ia + over.b as u32 * a) / 255) as u8,
        )
    }

    /// Linearly interpolate between two colors.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let it = 1.0 - t;
        Color::rgba(
            (self.r as f32 * it + other.r as f32 * t) as u8,
            (self.g as f32 * it + other.g as f32 * t) as u8,
            (self.b as f32 * it + other.b as f32 * t) as u8,
            (self.a as f32 * it + other.a as f32 * t) as u8,
        )
    }

    /// Multiply RGB components (tint).
    pub fn tint(self, t: f32) -> Color {
        Color::rgba(
            (self.r as f32 * t).min(255.0) as u8,
            (self.g as f32 * t).min(255.0) as u8,
            (self.b as f32 * t).min(255.0) as u8,
            self.a,
        )
    }

    // ── Palette ──────────────────────────────────────────────────────────
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const BLACK:   Color = Color::rgb(0,   0,   0  );
    pub const WHITE:   Color = Color::rgb(255, 255, 255);
    pub const RED:     Color = Color::rgb(220, 50,  50 );
    pub const GREEN:   Color = Color::rgb(50,  200, 80 );
    pub const BLUE:    Color = Color::rgb(60,  120, 220);
    pub const YELLOW:  Color = Color::rgb(255, 220, 0  );
    pub const ORANGE:  Color = Color::rgb(255, 140, 0  );
    pub const CYAN:    Color = Color::rgb(0,   230, 230);
    pub const MAGENTA: Color = Color::rgb(200, 50,  200);
    pub const PURPLE:  Color = Color::rgb(130, 60,  200);
    pub const PINK:    Color = Color::rgb(255, 150, 180);
    pub const GRAY:    Color = Color::rgb(128, 128, 128);
    pub const DARK_GRAY: Color = Color::rgb(50,  50,  50 );
    pub const LIGHT_GRAY:Color = Color::rgb(200, 200, 200);
    pub const SKY_BLUE:  Color = Color::rgb(30,  144, 255);
    pub const DARK_GREEN:Color = Color::rgb(0,   100, 50 );
    pub const GOLD:    Color = Color::rgb(255, 195, 0  );
}
