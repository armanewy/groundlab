use serde::{Deserialize, Serialize};

use crate::color::Rgba8;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpriteStyle {
    pub id: String,
    pub display_scale: u32,
    pub palette: CozyTerrainPalette,
    pub pixel: PixelRules,
    pub grass: GrassRules,
    pub dirt: DirtRules,
    pub transition: TransitionRules,
}

impl Default for TerrainSpriteStyle {
    fn default() -> Self {
        Self {
            id: "cozy_upland_pixel".to_string(),
            display_scale: 6,
            palette: CozyTerrainPalette::default(),
            pixel: PixelRules::default(),
            grass: GrassRules::default(),
            dirt: DirtRules::default(),
            transition: TransitionRules::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CozyTerrainPalette {
    pub grass_shadow: Rgba8,
    pub grass_dark: Rgba8,
    pub grass_mid: Rgba8,
    pub grass_light: Rgba8,
    pub grass_flower: Rgba8,
    pub dirt_shadow: Rgba8,
    pub dirt_dark: Rgba8,
    pub dirt_mid: Rgba8,
    pub dirt_light: Rgba8,
    pub pebble: Rgba8,
}

impl Default for CozyTerrainPalette {
    fn default() -> Self {
        Self {
            grass_shadow: Rgba8::from_rgb_hex(0x3a522b),
            grass_dark: Rgba8::from_rgb_hex(0x4f6b36),
            grass_mid: Rgba8::from_rgb_hex(0x627d40),
            grass_light: Rgba8::from_rgb_hex(0x809950),
            grass_flower: Rgba8::from_rgb_hex(0xd5ca70),
            dirt_shadow: Rgba8::from_rgb_hex(0x674329),
            dirt_dark: Rgba8::from_rgb_hex(0x7b5132),
            dirt_mid: Rgba8::from_rgb_hex(0xa16a40),
            dirt_light: Rgba8::from_rgb_hex(0xbc8352),
            pebble: Rgba8::from_rgb_hex(0x8f8b72),
        }
    }
}

impl CozyTerrainPalette {
    pub fn grass_ramp(&self) -> [Rgba8; 4] {
        [
            self.grass_shadow,
            self.grass_dark,
            self.grass_mid,
            self.grass_light,
        ]
    }

    pub fn dirt_ramp(&self) -> [Rgba8; 4] {
        [
            self.dirt_shadow,
            self.dirt_dark,
            self.dirt_mid,
            self.dirt_light,
        ]
    }

    pub fn all_colors(&self) -> Vec<Rgba8> {
        vec![
            self.grass_shadow,
            self.grass_dark,
            self.grass_mid,
            self.grass_light,
            self.grass_flower,
            self.dirt_shadow,
            self.dirt_dark,
            self.dirt_mid,
            self.dirt_light,
            self.pebble,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PixelRules {
    pub avoid_single_pixel_noise: bool,
    pub min_cluster_size: u32,
    pub max_cluster_size: u32,
    pub highlight_density: f32,
    pub shadow_density: f32,
    pub detail_density: f32,
}

impl Default for PixelRules {
    fn default() -> Self {
        Self {
            avoid_single_pixel_noise: true,
            min_cluster_size: 2,
            max_cluster_size: 3,
            highlight_density: 0.08,
            shadow_density: 0.12,
            detail_density: 0.30,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GrassRules {
    pub blade_cluster_density: f32,
    pub dark_cluster_density: f32,
    pub highlight_cluster_density: f32,
    pub flower_density: f32,
}

impl Default for GrassRules {
    fn default() -> Self {
        Self {
            blade_cluster_density: 0.30,
            dark_cluster_density: 0.085,
            highlight_cluster_density: 0.070,
            flower_density: 0.005,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DirtRules {
    pub pebble_density: f32,
    pub rut_density: f32,
    pub dust_patch_density: f32,
    pub compact_shadow_density: f32,
}

impl Default for DirtRules {
    fn default() -> Self {
        Self {
            pebble_density: 0.018,
            rut_density: 0.026,
            dust_patch_density: 0.115,
            compact_shadow_density: 0.060,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TransitionRules {
    pub edge_jitter_px: u32,
    pub grass_intrusion_density: f32,
    pub dirt_speckle_density: f32,
    pub edge_softness: f32,
}

impl Default for TransitionRules {
    fn default() -> Self {
        Self {
            edge_jitter_px: 3,
            grass_intrusion_density: 0.32,
            dirt_speckle_density: 0.08,
            edge_softness: 0.48,
        }
    }
}
