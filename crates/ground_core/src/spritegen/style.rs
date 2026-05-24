use serde::{Deserialize, Serialize};

use crate::color::Rgba8;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpriteStyle {
    pub id: String,
    pub display_scale: u32,
    pub projection: ObliqueProjectionProfile,
    pub palette: CozyTerrainPalette,
    pub pixel: PixelRules,
    pub grass: GrassRules,
    pub dirt: DirtRules,
    pub transition: TransitionRules,
    pub path: PathRules,
}

impl Default for TerrainSpriteStyle {
    fn default() -> Self {
        Self {
            id: "cozy_upland_pixel".to_string(),
            display_scale: 6,
            projection: ObliqueProjectionProfile::default(),
            palette: CozyTerrainPalette::default(),
            pixel: PixelRules::default(),
            grass: GrassRules::default(),
            dirt: DirtRules::default(),
            transition: TransitionRules::default(),
            path: PathRules::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SpriteProjectionKind {
    HighOblique2D,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SpriteLightDirection {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ObliqueProjectionProfile {
    pub kind: SpriteProjectionKind,
    pub cell_width_px: u32,
    pub cell_height_px: u32,
    pub face_height_px: u32,
    pub light_direction: SpriteLightDirection,
    pub shadow_offset_px: (i32, i32),
}

impl Default for ObliqueProjectionProfile {
    fn default() -> Self {
        Self {
            kind: SpriteProjectionKind::HighOblique2D,
            cell_width_px: 96,
            cell_height_px: 72,
            face_height_px: 28,
            light_direction: SpriteLightDirection::Northwest,
            shadow_offset_px: (10, 18),
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
            rut_density: 0.014,
            dust_patch_density: 0.090,
            compact_shadow_density: 0.045,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PathRules {
    pub width_px: f32,
    pub core_width_px: f32,
    pub corner_rounding: f32,
    pub edge_noise: f32,
}

impl Default for PathRules {
    fn default() -> Self {
        Self {
            width_px: 5.8,
            core_width_px: 6.7,
            corner_rounding: 0.55,
            edge_noise: 0.85,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainMotifLibrary {
    pub id: String,
    pub grass_dark: Vec<TerrainMotif>,
    pub grass_light: Vec<TerrainMotif>,
    pub grass_blades: Vec<TerrainMotif>,
    pub grass_flowers: Vec<TerrainMotif>,
    pub dirt_dust: Vec<TerrainMotif>,
    pub dirt_dents: Vec<TerrainMotif>,
    pub dirt_ruts: Vec<TerrainMotif>,
    pub transition_intrusion: Vec<TerrainMotif>,
}

impl Default for TerrainMotifLibrary {
    fn default() -> Self {
        Self {
            id: "cozy_upland_motifs".to_string(),
            grass_dark: vec![
                TerrainMotif::new("dark_clump_a", &[(-1, 0, -1), (0, 0, -1), (0, 1, -2)]),
                TerrainMotif::new("dark_clump_b", &[(0, 0, -2), (-1, 0, -1), (0, -1, -1)]),
                TerrainMotif::new("dark_clump_c", &[(0, 0, -1), (-1, 1, -2), (1, 1, -1)]),
                TerrainMotif::new("soft_dark_a", &[(0, 0, -1), (1, 1, -1)]),
                TerrainMotif::new("shadow_pocket_a", &[(0, 0, -2), (1, 0, -1)]),
                TerrainMotif::new("shadow_pocket_b", &[(0, 0, -1), (0, 1, -2)]),
            ],
            grass_light: vec![
                TerrainMotif::new("light_leaf_a", &[(0, 0, 1), (1, 0, 1)]),
                TerrainMotif::new("light_leaf_b", &[(0, 0, 1), (0, -1, 1)]),
                TerrainMotif::new("light_leaf_c", &[(0, 0, 1), (-1, 0, 1), (-1, 1, 0)]),
                TerrainMotif::new("light_speck_a", &[(0, 0, 1)]),
                TerrainMotif::new("light_speck_b", &[(0, 0, 1), (1, 1, 0)]),
                TerrainMotif::new("seed_fleck_a", &[(0, 0, 1), (1, 0, 0)]),
            ],
            grass_blades: vec![
                TerrainMotif::new("blade_a", &[(0, 0, 1), (1, -1, 1), (1, 0, -1)]),
                TerrainMotif::new("blade_b", &[(0, 0, 1), (-1, -1, 1), (0, 1, -1)]),
                TerrainMotif::new("blade_c", &[(0, 0, -1), (1, 0, 1), (2, -1, 1)]),
                TerrainMotif::new("blade_d", &[(0, 0, -1), (-1, 1, 1)]),
                TerrainMotif::new("blade_e", &[(0, 0, 1), (0, -1, 1), (-1, 0, -1)]),
                TerrainMotif::new("blade_f", &[(0, 0, -1), (1, -1, 1), (2, -1, 1)]),
            ],
            grass_flowers: vec![TerrainMotif::new("flower_fleck_a", &[(0, 0, 2), (1, 0, 1)])],
            dirt_dust: vec![
                TerrainMotif::new("dust_smear_a", &[(0, 0, 1), (1, 0, 1), (0, 1, 0)]),
                TerrainMotif::new("dust_smear_b", &[(0, 0, 1), (-1, 0, 1), (1, 1, 0)]),
                TerrainMotif::new("dust_smear_c", &[(0, 0, 1), (-1, 1, 0)]),
                TerrainMotif::new("dust_smear_d", &[(-1, 0, 1), (0, 0, 1), (1, 1, 0)]),
                TerrainMotif::new("dust_smear_e", &[(0, 0, 1), (1, -1, 1)]),
            ],
            dirt_dents: vec![
                TerrainMotif::new("dirt_dent_a", &[(0, 0, -1), (1, 0, -1)]),
                TerrainMotif::new("dirt_dent_b", &[(0, 0, -1), (0, 1, -1)]),
                TerrainMotif::new("dirt_dent_c", &[(0, 0, -1)]),
                TerrainMotif::new("dirt_dent_d", &[(0, 0, -1), (1, 1, 0)]),
            ],
            dirt_ruts: vec![
                TerrainMotif::new("rut_a", &[(0, 0, -1), (1, 1, -1), (1, -1, 0)]),
                TerrainMotif::new("rut_b", &[(0, 0, -1), (1, 1, -1)]),
                TerrainMotif::new("rut_c", &[(0, 0, 1), (-1, 1, -1)]),
                TerrainMotif::new("rut_d", &[(-1, 0, -1), (0, 0, 0), (1, 1, -1)]),
                TerrainMotif::new("rut_e", &[(0, 0, -1), (1, -1, 0)]),
            ],
            transition_intrusion: vec![
                TerrainMotif::new("edge_blade_a", &[(0, 0, 1), (1, -1, 1), (1, 0, -1)]),
                TerrainMotif::new("edge_blade_d", &[(0, 0, -1), (-1, 1, 1)]),
                TerrainMotif::new("edge_leaf_c", &[(0, 0, 1), (-1, 0, 1), (-1, 1, 0)]),
                TerrainMotif::new("edge_shadow_a", &[(0, 0, -1), (1, 1, -1)]),
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainMotif {
    pub id: String,
    pub weight: f32,
    pub allow_flip_x: bool,
    pub allow_flip_y: bool,
    pub pixels: Vec<TerrainMotifPixel>,
}

impl TerrainMotif {
    fn new(id: &str, pixels: &[(i32, i32, i8)]) -> Self {
        Self {
            id: id.to_string(),
            weight: 1.0,
            allow_flip_x: true,
            allow_flip_y: false,
            pixels: pixels
                .iter()
                .map(|(dx, dy, shade)| TerrainMotifPixel {
                    dx: *dx,
                    dy: *dy,
                    shade: *shade,
                })
                .collect(),
        }
    }
}

impl Default for TerrainMotif {
    fn default() -> Self {
        Self {
            id: "motif".to_string(),
            weight: 1.0,
            allow_flip_x: true,
            allow_flip_y: false,
            pixels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainMotifPixel {
    pub dx: i32,
    pub dy: i32,
    pub shade: i8,
}
