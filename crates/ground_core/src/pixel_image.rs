use std::path::Path;

use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};

use crate::color::Rgba8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PixelImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba8>,
}

impl PixelImage {
    pub fn new(width: u32, height: u32, fill: Rgba8) -> Self {
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            pixels: vec![fill; len],
        }
    }

    pub fn transparent(width: u32, height: u32) -> Self {
        Self::new(width, height, Rgba8::TRANSPARENT)
    }

    pub fn size(&self) -> [usize; 2] {
        [self.width as usize, self.height as usize]
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub fn index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u32, y: u32) -> Rgba8 {
        self.pixels[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, color: Rgba8) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.pixels[idx] = color;
        }
    }

    pub fn set_i32(&mut self, x: i32, y: i32, color: Rgba8) {
        if self.in_bounds(x, y) {
            self.set(x as u32, y as u32, color);
        }
    }

    pub fn blend_pixel(&mut self, x: u32, y: u32, color: Rgba8, alpha: f32) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.pixels[idx] = self.pixels[idx].blend(color, alpha);
        }
    }

    pub fn blit(&mut self, src: &PixelImage, dst_x: u32, dst_y: u32) {
        for y in 0..src.height {
            for x in 0..src.width {
                let tx = dst_x + x;
                let ty = dst_y + y;
                if tx < self.width && ty < self.height {
                    let s = src.get(x, y);
                    let idx = self.index(tx, ty);
                    if s.a == 255 {
                        self.pixels[idx] = s;
                    } else if s.a > 0 {
                        self.pixels[idx] = self.pixels[idx].blend(s, s.a as f32 / 255.0);
                    }
                }
            }
        }
    }

    pub fn blit_tinted(
        &mut self,
        src: &PixelImage,
        dst_x: u32,
        dst_y: u32,
        tint: Rgba8,
        alpha: f32,
    ) {
        for y in 0..src.height {
            for x in 0..src.width {
                let tx = dst_x + x;
                let ty = dst_y + y;
                if tx < self.width && ty < self.height {
                    let base = src.get(x, y);
                    let tinted = base.blend(tint, alpha);
                    self.set(tx, ty, tinted);
                }
            }
        }
    }

    pub fn fill_rect(&mut self, x0: u32, y0: u32, width: u32, height: u32, color: Rgba8) {
        let x1 = (x0 + width).min(self.width);
        let y1 = (y0 + height).min(self.height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.set(x, y, color);
            }
        }
    }

    pub fn outline_rect(&mut self, x0: u32, y0: u32, width: u32, height: u32, color: Rgba8) {
        if width == 0 || height == 0 {
            return;
        }
        let x1 = (x0 + width - 1).min(self.width.saturating_sub(1));
        let y1 = (y0 + height - 1).min(self.height.saturating_sub(1));
        for x in x0.min(self.width)..=x1 {
            self.set(x, y0.min(self.height.saturating_sub(1)), color);
            self.set(x, y1, color);
        }
        for y in y0.min(self.height)..=y1 {
            self.set(x0.min(self.width.saturating_sub(1)), y, color);
            self.set(x1, y, color);
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba8) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_i32(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixels.len() * 4);
        for px in &self.pixels {
            out.extend_from_slice(&px.to_array());
        }
        out
    }

    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_rgba_bytes();
        let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_vec(self.width, self.height, bytes)
                .ok_or_else(|| anyhow!("invalid image buffer dimensions"))?;
        buffer.save(path)?;
        Ok(())
    }
}
