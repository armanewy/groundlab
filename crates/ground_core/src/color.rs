use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_rgb_hex(hex: u32) -> Self {
        Self::opaque(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    pub fn blend(self, top: Self, alpha: f32) -> Self {
        let t = clamp01(alpha);
        Self::new(
            lerp_u8(self.r, top.r, t),
            lerp_u8(self.g, top.g, t),
            lerp_u8(self.b, top.b, t),
            lerp_u8(self.a, top.a, t),
        )
    }

    pub fn scale_rgb(self, factor: f32) -> Self {
        Self::new(
            clamp_u8(self.r as f32 * factor),
            clamp_u8(self.g as f32 * factor),
            clamp_u8(self.b as f32 * factor),
            self.a,
        )
    }

    pub fn lighten(self, amount: f32) -> Self {
        self.blend(Self::WHITE, amount)
    }

    pub fn darken(self, amount: f32) -> Self {
        self.blend(Self::BLACK, amount)
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn luma(self) -> u8 {
        clamp_u8(self.r as f32 * 0.299 + self.g as f32 * 0.587 + self.b as f32 * 0.114)
    }

    pub fn rgb_distance(self, other: Self) -> f32 {
        let dr = self.r as f32 - other.r as f32;
        let dg = self.g as f32 - other.g as f32;
        let db = self.b as f32 - other.b as f32;
        ((dr * dr + dg * dg + db * db) / 3.0).sqrt()
    }
}

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

pub fn clamp_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * clamp01(t)
}

pub fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    clamp_u8(lerp_f32(a as f32, b as f32, t))
}
