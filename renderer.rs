use crate::engine::{
    color::Color,
    math::{Vec2, Rect},
};

// ── Texture ───────────────────────────────────────────────────────────────────

/// An RGBA pixel image that can be drawn onto the canvas.
#[derive(Debug, Clone)]
pub struct Texture {
    pub width:  usize,
    pub height: usize,
    pixels: Vec<Color>, // row-major
}

impl Texture {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![Color::TRANSPARENT; width * height] }
    }

    pub fn from_fn(width: usize, height: usize, f: impl Fn(usize, usize) -> Color) -> Self {
        let pixels = (0..height).flat_map(|y| (0..width).map(move |x| f(x, y))).collect();
        Self { width, height, pixels }
    }

    #[inline] pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.width + x]
    }
    #[inline] pub fn set(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y * self.width + x] = color;
    }
}

// ── Canvas ────────────────────────────────────────────────────────────────────

/// Software-rendered pixel canvas. Backed by a `Vec<u32>` compatible with minifb.
pub struct Canvas {
    pub width:  usize,
    pub height: usize,
    buffer: Vec<u32>,
    /// World-space offset applied to every draw call (camera translation).
    pub camera: Vec2,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0u32; width * height],
            camera: Vec2::ZERO,
        }
    }

    /// Raw buffer slice for minifb.
    #[inline] pub fn buffer(&self) -> &[u32] { &self.buffer }

    // ── Low-level pixel ───────────────────────────────────────────────────────

    /// Write a pixel directly (no clipping, no camera).
    #[inline] pub fn put_pixel_raw(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
        let idx = y as usize * self.width + x as usize;
        // Alpha-blend over existing pixel
        if color.a == 255 {
            self.buffer[idx] = color.to_u32();
        } else if color.a > 0 {
            let bg = Color::rgb(
                ((self.buffer[idx] >> 16) & 0xFF) as u8,
                ((self.buffer[idx] >>  8) & 0xFF) as u8,
                ( self.buffer[idx]        & 0xFF) as u8,
            );
            self.buffer[idx] = bg.blend(color).to_u32();
        }
    }

    /// Write a pixel with camera transform applied.
    #[inline] pub fn put_pixel(&mut self, x: f32, y: f32, color: Color) {
        self.put_pixel_raw(
            (x - self.camera.x) as i32,
            (y - self.camera.y) as i32,
            color,
        );
    }

    // ── Fill / clear ──────────────────────────────────────────────────────────

    pub fn clear(&mut self, color: Color) {
        let v = color.to_u32();
        self.buffer.fill(v);
    }

    // ── Rectangles ────────────────────────────────────────────────────────────

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = (rect.x - self.camera.x) as i32;
        let y0 = (rect.y - self.camera.y) as i32;
        let x1 = x0 + rect.w as i32;
        let y1 = y0 + rect.h as i32;
        for y in y0.max(0)..y1.min(self.height as i32) {
            for x in x0.max(0)..x1.min(self.width as i32) {
                self.put_pixel_raw(x, y, color);
            }
        }
    }

    pub fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: i32) {
        for t in 0..thickness {
            let r = Rect::new(
                rect.x + t as f32, rect.y + t as f32,
                rect.w - t as f32 * 2.0, rect.h - t as f32 * 2.0,
            );
            self.draw_line(Vec2::new(r.x,          r.y),           Vec2::new(r.right(),     r.y),           color);
            self.draw_line(Vec2::new(r.right(),     r.y),           Vec2::new(r.right(),     r.bottom()),    color);
            self.draw_line(Vec2::new(r.right(),     r.bottom()),    Vec2::new(r.x,           r.bottom()),    color);
            self.draw_line(Vec2::new(r.x,           r.bottom()),    Vec2::new(r.x,           r.y),           color);
        }
    }

    // ── Circles ───────────────────────────────────────────────────────────────

    pub fn fill_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        let r = radius as i32;
        let cx = (center.x - self.camera.x) as i32;
        let cy = (center.y - self.camera.y) as i32;
        let r2 = (radius * radius) as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    self.put_pixel_raw(cx + dx, cy + dy, color);
                }
            }
        }
    }

    pub fn stroke_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        // Midpoint circle algorithm
        let cx = (center.x - self.camera.x) as i32;
        let cy = (center.y - self.camera.y) as i32;
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut err = 0i32;
        while x >= y {
            for (px, py) in [
                (cx+x,cy+y),(cx-x,cy+y),(cx+x,cy-y),(cx-x,cy-y),
                (cx+y,cy+x),(cx-y,cy+x),(cx+y,cy-x),(cx-y,cy-x),
            ] { self.put_pixel_raw(px, py, color); }
            y += 1;
            err += 2 * y - 1;
            if 2 * err + 1 - 2 * x > 0 { x -= 1; err += 1 - 2 * x; }
        }
    }

    // ── Lines ─────────────────────────────────────────────────────────────────

    /// Bresenham line.
    pub fn draw_line(&mut self, from: Vec2, to: Vec2, color: Color) {
        let (mut x0, mut y0) = (
            (from.x - self.camera.x) as i32,
            (from.y - self.camera.y) as i32,
        );
        let (x1, y1) = (
            (to.x - self.camera.x) as i32,
            (to.y - self.camera.y) as i32,
        );
        let dx =  (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put_pixel_raw(x0, y0, color);
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x0 += sx; }
            if e2 <= dx { err += dx; y0 += sy; }
        }
    }

    // ── Textures ──────────────────────────────────────────────────────────────

    /// Draw a texture at `pos`, optionally scaled to `size`.
    pub fn draw_texture(&mut self, texture: &Texture, pos: Vec2, size: Option<Vec2>) {
        let (tw, th) = (texture.width as f32, texture.height as f32);
        let (dw, dh) = size.map_or((tw, th), |s| (s.x, s.y));
        let sx = tw / dw;
        let sy = th / dh;
        for dy in 0..dh as i32 {
            for dx in 0..dw as i32 {
                let tx = ((dx as f32 * sx) as usize).min(texture.width  - 1);
                let ty = ((dy as f32 * sy) as usize).min(texture.height - 1);
                let color = texture.get(tx, ty);
                self.put_pixel(pos.x + dx as f32, pos.y + dy as f32, color);
            }
        }
    }

    // ── Bitmap text ───────────────────────────────────────────────────────────

    /// Render ASCII text using a built-in 6×8 bitmap font.
    pub fn draw_text(&mut self, text: &str, pos: Vec2, scale: u32, color: Color) {
        let scale = scale.max(1) as f32;
        let mut cx = pos.x;
        for ch in text.chars() {
            let glyph = glyph_for(ch);
            for row in 0..8usize {
                for col in 0..6usize {
                    if glyph[row] & (1 << (5 - col)) != 0 {
                        self.fill_rect(
                            Rect::new(
                                cx + col as f32 * scale,
                                pos.y + row as f32 * scale,
                                scale, scale,
                            ),
                            color,
                        );
                    }
                }
            }
            cx += 7.0 * scale;
        }
    }
}

// ── Embedded 6×8 bitmap font ──────────────────────────────────────────────────
// Each [u8;8] is 8 pixel rows, each byte has 6 significant bits (columns MSB).

fn glyph_for(ch: char) -> [u8; 8] {
    match ch {
        ' '  => [0,0,0,0,0,0,0,0],
        '!'  => [0o10,0o10,0o10,0o10,0o10,0,0o10,0],
        '"'  => [0o24,0o24,0,0,0,0,0,0],
        '-'  => [0,0,0,0o76,0,0,0,0],
        '.'  => [0,0,0,0,0,0,0o10,0],
        ','  => [0,0,0,0,0,0o10,0o10,0o20],
        '/'  => [0o02,0o04,0o10,0o20,0o40,0,0,0],
        '+'  => [0,0o10,0o10,0o76,0o10,0o10,0,0],
        '='  => [0,0,0o76,0,0o76,0,0,0],
        ':'  => [0,0o10,0o10,0,0o10,0o10,0,0],
        '0'  => [0o34,0o42,0o46,0o52,0o62,0o42,0o34,0],
        '1'  => [0o10,0o30,0o10,0o10,0o10,0o10,0o34,0],
        '2'  => [0o34,0o42,0o02,0o04,0o10,0o20,0o76,0],
        '3'  => [0o76,0o04,0o10,0o04,0o02,0o42,0o34,0],
        '4'  => [0o04,0o14,0o24,0o44,0o76,0o04,0o04,0],
        '5'  => [0o76,0o40,0o74,0o02,0o02,0o42,0o34,0],
        '6'  => [0o14,0o20,0o40,0o74,0o42,0o42,0o34,0],
        '7'  => [0o76,0o02,0o04,0o10,0o20,0o20,0o20,0],
        '8'  => [0o34,0o42,0o42,0o34,0o42,0o42,0o34,0],
        '9'  => [0o34,0o42,0o42,0o36,0o02,0o04,0o30,0],
        'A'|'a' => [0o34,0o42,0o42,0o76,0o42,0o42,0o42,0],
        'B'|'b' => [0o74,0o42,0o42,0o74,0o42,0o42,0o74,0],
        'C'|'c' => [0o34,0o42,0o40,0o40,0o40,0o42,0o34,0],
        'D'|'d' => [0o74,0o42,0o42,0o42,0o42,0o42,0o74,0],
        'E'|'e' => [0o76,0o40,0o40,0o74,0o40,0o40,0o76,0],
        'F'|'f' => [0o76,0o40,0o40,0o74,0o40,0o40,0o40,0],
        'G'|'g' => [0o34,0o42,0o40,0o46,0o42,0o42,0o34,0],
        'H'|'h' => [0o42,0o42,0o42,0o76,0o42,0o42,0o42,0],
        'I'|'i' => [0o34,0o10,0o10,0o10,0o10,0o10,0o34,0],
        'J'|'j' => [0o16,0o02,0o02,0o02,0o02,0o42,0o34,0],
        'K'|'k' => [0o42,0o44,0o50,0o60,0o50,0o44,0o42,0],
        'L'|'l' => [0o40,0o40,0o40,0o40,0o40,0o40,0o76,0],
        'M'|'m' => [0o42,0o66,0o52,0o42,0o42,0o42,0o42,0],
        'N'|'n' => [0o42,0o62,0o52,0o46,0o42,0o42,0o42,0],
        'O'|'o' => [0o34,0o42,0o42,0o42,0o42,0o42,0o34,0],
        'P'|'p' => [0o74,0o42,0o42,0o74,0o40,0o40,0o40,0],
        'Q'|'q' => [0o34,0o42,0o42,0o42,0o52,0o44,0o32,0],
        'R'|'r' => [0o74,0o42,0o42,0o74,0o50,0o44,0o42,0],
        'S'|'s' => [0o34,0o42,0o40,0o34,0o02,0o42,0o34,0],
        'T'|'t' => [0o76,0o10,0o10,0o10,0o10,0o10,0o10,0],
        'U'|'u' => [0o42,0o42,0o42,0o42,0o42,0o42,0o34,0],
        'V'|'v' => [0o42,0o42,0o42,0o42,0o24,0o10,0,0],
        'W'|'w' => [0o42,0o42,0o42,0o52,0o52,0o66,0o42,0],
        'X'|'x' => [0o42,0o24,0o10,0o10,0o24,0o42,0,0],
        'Y'|'y' => [0o42,0o24,0o10,0o10,0o10,0o10,0o10,0],
        'Z'|'z' => [0o76,0o04,0o10,0o20,0o40,0o76,0,0],
        _    => [0; 8],
    }
}
