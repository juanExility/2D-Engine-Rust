/// 2D floating-point vector with common math operations.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE:  Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const UP:   Vec2 = Vec2 { x: 0.0, y: -1.0 };
    pub const DOWN: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const LEFT: Vec2 = Vec2 { x: -1.0, y: 0.0 };
    pub const RIGHT:Vec2 = Vec2 { x: 1.0, y: 0.0 };

    #[inline] pub fn new(x: f32, y: f32) -> Self { Self { x, y } }

    #[inline] pub fn length_sq(self) -> f32 { self.x * self.x + self.y * self.y }
    #[inline] pub fn length(self) -> f32 { self.length_sq().sqrt() }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 1e-10 { Self::new(self.x / len, self.y / len) } else { Self::ZERO }
    }

    #[inline] pub fn dot(self, other: Vec2) -> f32 { self.x * other.x + self.y * other.y }
    #[inline] pub fn distance(self, other: Vec2) -> f32 { (self - other).length() }
    #[inline] pub fn lerp(self, target: Vec2, t: f32) -> Vec2 { self + (target - self) * t }

    /// Rotate vector by `angle` radians.
    pub fn rotate(self, angle: f32) -> Vec2 {
        let (sin, cos) = angle.sin_cos();
        Vec2::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
}

impl std::ops::Add  for Vec2 { type Output=Vec2; fn add(self,o:Vec2)->Vec2{Vec2::new(self.x+o.x,self.y+o.y)} }
impl std::ops::Sub  for Vec2 { type Output=Vec2; fn sub(self,o:Vec2)->Vec2{Vec2::new(self.x-o.x,self.y-o.y)} }
impl std::ops::Mul<f32> for Vec2 { type Output=Vec2; fn mul(self,s:f32)->Vec2{Vec2::new(self.x*s,self.y*s)} }
impl std::ops::Div<f32> for Vec2 { type Output=Vec2; fn div(self,s:f32)->Vec2{Vec2::new(self.x/s,self.y/s)} }
impl std::ops::Neg  for Vec2 { type Output=Vec2; fn neg(self)->Vec2{Vec2::new(-self.x,-self.y)} }
impl std::ops::AddAssign for Vec2 { fn add_assign(&mut self,o:Vec2){self.x+=o.x;self.y+=o.y;} }
impl std::ops::SubAssign for Vec2 { fn sub_assign(&mut self,o:Vec2){self.x-=o.x;self.y-=o.y;} }
impl std::ops::MulAssign<f32> for Vec2 { fn mul_assign(&mut self,s:f32){self.x*=s;self.y*=s;} }

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn from_center(center: Vec2, w: f32, h: f32) -> Self {
        Self::new(center.x - w * 0.5, center.y - h * 0.5, w, h)
    }

    #[inline] pub fn center(self) -> Vec2 { Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5) }
    #[inline] pub fn right(self) -> f32  { self.x + self.w }
    #[inline] pub fn bottom(self) -> f32 { self.y + self.h }

    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.right() && self.right() > other.x &&
        self.y < other.bottom() && self.bottom() > other.y
    }

    /// Translate by a vector.
    pub fn offset(self, v: Vec2) -> Self { Self::new(self.x + v.x, self.y + v.y, self.w, self.h) }
}
