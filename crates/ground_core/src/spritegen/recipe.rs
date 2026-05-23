use serde::{Deserialize, Serialize};

use crate::pixel_image::PixelImage;
use crate::spritegen::TerrainSpriteStyle;

pub const DEFAULT_SPRITEGEN_EXPORT_DIR: &str = "exports/artgen_01_2";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpriteRecipe {
    pub id: String,
    pub tile_size: u32,
    pub seed: u64,
    pub variant_count: u32,
    pub style: TerrainSpriteStyle,
}

impl Default for TerrainSpriteRecipe {
    fn default() -> Self {
        Self {
            id: "cozy_grass_dirt_artgen_01".to_string(),
            tile_size: 16,
            seed: 0x5eed_7101,
            variant_count: 4,
            style: TerrainSpriteStyle::default(),
        }
    }
}

impl TerrainSpriteRecipe {
    pub fn sanitize(&mut self) {
        self.tile_size = self.tile_size.clamp(8, 64);
        self.variant_count = self.variant_count.clamp(1, 12);
        self.style.display_scale = self.style.display_scale.clamp(1, 16);
        self.style.pixel.min_cluster_size = self.style.pixel.min_cluster_size.clamp(1, 12);
        self.style.pixel.max_cluster_size = self
            .style
            .pixel
            .max_cluster_size
            .max(self.style.pixel.min_cluster_size)
            .clamp(1, 16);
        self.style.pixel.highlight_density = self.style.pixel.highlight_density.clamp(0.0, 1.0);
        self.style.pixel.shadow_density = self.style.pixel.shadow_density.clamp(0.0, 1.0);
        self.style.pixel.detail_density = self.style.pixel.detail_density.clamp(0.0, 1.0);
        self.style.grass.blade_cluster_density =
            self.style.grass.blade_cluster_density.clamp(0.0, 1.0);
        self.style.grass.dark_cluster_density =
            self.style.grass.dark_cluster_density.clamp(0.0, 1.0);
        self.style.grass.highlight_cluster_density =
            self.style.grass.highlight_cluster_density.clamp(0.0, 1.0);
        self.style.grass.flower_density = self.style.grass.flower_density.clamp(0.0, 1.0);
        self.style.dirt.pebble_density = self.style.dirt.pebble_density.clamp(0.0, 1.0);
        self.style.dirt.rut_density = self.style.dirt.rut_density.clamp(0.0, 1.0);
        self.style.dirt.dust_patch_density = self.style.dirt.dust_patch_density.clamp(0.0, 1.0);
        self.style.dirt.compact_shadow_density =
            self.style.dirt.compact_shadow_density.clamp(0.0, 1.0);
        self.style.transition.edge_jitter_px = self.style.transition.edge_jitter_px.clamp(0, 16);
        self.style.transition.grass_intrusion_density = self
            .style
            .transition
            .grass_intrusion_density
            .clamp(0.0, 1.0);
        self.style.transition.dirt_speckle_density =
            self.style.transition.dirt_speckle_density.clamp(0.0, 1.0);
        self.style.transition.edge_softness = self.style.transition.edge_softness.clamp(0.0, 1.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainSpriteKind {
    GrassTile,
    DirtTile,
    GrassToDirtEdgeNorth,
    GrassToDirtEdgeSouth,
    GrassToDirtEdgeEast,
    GrassToDirtEdgeWest,
    PathMask00,
    PathMask01,
    PathMask02,
    PathMask03,
    PathMask04,
    PathMask05,
    PathMask06,
    PathMask07,
    PathMask08,
    PathMask09,
    PathMask10,
    PathMask11,
    PathMask12,
    PathMask13,
    PathMask14,
    PathMask15,
}

impl TerrainSpriteKind {
    pub const ALL: [TerrainSpriteKind; 22] = [
        TerrainSpriteKind::GrassTile,
        TerrainSpriteKind::DirtTile,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        TerrainSpriteKind::GrassToDirtEdgeWest,
        TerrainSpriteKind::PathMask00,
        TerrainSpriteKind::PathMask01,
        TerrainSpriteKind::PathMask02,
        TerrainSpriteKind::PathMask03,
        TerrainSpriteKind::PathMask04,
        TerrainSpriteKind::PathMask05,
        TerrainSpriteKind::PathMask06,
        TerrainSpriteKind::PathMask07,
        TerrainSpriteKind::PathMask08,
        TerrainSpriteKind::PathMask09,
        TerrainSpriteKind::PathMask10,
        TerrainSpriteKind::PathMask11,
        TerrainSpriteKind::PathMask12,
        TerrainSpriteKind::PathMask13,
        TerrainSpriteKind::PathMask14,
        TerrainSpriteKind::PathMask15,
    ];

    pub fn id(self) -> &'static str {
        match self {
            TerrainSpriteKind::GrassTile => "grass_tile",
            TerrainSpriteKind::DirtTile => "dirt_tile",
            TerrainSpriteKind::GrassToDirtEdgeNorth => "grass_dirt_edge_north",
            TerrainSpriteKind::GrassToDirtEdgeSouth => "grass_dirt_edge_south",
            TerrainSpriteKind::GrassToDirtEdgeEast => "grass_dirt_edge_east",
            TerrainSpriteKind::GrassToDirtEdgeWest => "grass_dirt_edge_west",
            TerrainSpriteKind::PathMask00 => "path_mask_00",
            TerrainSpriteKind::PathMask01 => "path_mask_01",
            TerrainSpriteKind::PathMask02 => "path_mask_02",
            TerrainSpriteKind::PathMask03 => "path_mask_03",
            TerrainSpriteKind::PathMask04 => "path_mask_04",
            TerrainSpriteKind::PathMask05 => "path_mask_05",
            TerrainSpriteKind::PathMask06 => "path_mask_06",
            TerrainSpriteKind::PathMask07 => "path_mask_07",
            TerrainSpriteKind::PathMask08 => "path_mask_08",
            TerrainSpriteKind::PathMask09 => "path_mask_09",
            TerrainSpriteKind::PathMask10 => "path_mask_10",
            TerrainSpriteKind::PathMask11 => "path_mask_11",
            TerrainSpriteKind::PathMask12 => "path_mask_12",
            TerrainSpriteKind::PathMask13 => "path_mask_13",
            TerrainSpriteKind::PathMask14 => "path_mask_14",
            TerrainSpriteKind::PathMask15 => "path_mask_15",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TerrainSpriteKind::GrassTile => "Grass tile",
            TerrainSpriteKind::DirtTile => "Dirt tile",
            TerrainSpriteKind::GrassToDirtEdgeNorth => "Grass/dirt north edge",
            TerrainSpriteKind::GrassToDirtEdgeSouth => "Grass/dirt south edge",
            TerrainSpriteKind::GrassToDirtEdgeEast => "Grass/dirt east edge",
            TerrainSpriteKind::GrassToDirtEdgeWest => "Grass/dirt west edge",
            TerrainSpriteKind::PathMask00 => "Path mask 00",
            TerrainSpriteKind::PathMask01 => "Path mask 01",
            TerrainSpriteKind::PathMask02 => "Path mask 02",
            TerrainSpriteKind::PathMask03 => "Path mask 03",
            TerrainSpriteKind::PathMask04 => "Path mask 04",
            TerrainSpriteKind::PathMask05 => "Path mask 05",
            TerrainSpriteKind::PathMask06 => "Path mask 06",
            TerrainSpriteKind::PathMask07 => "Path mask 07",
            TerrainSpriteKind::PathMask08 => "Path mask 08",
            TerrainSpriteKind::PathMask09 => "Path mask 09",
            TerrainSpriteKind::PathMask10 => "Path mask 10",
            TerrainSpriteKind::PathMask11 => "Path mask 11",
            TerrainSpriteKind::PathMask12 => "Path mask 12",
            TerrainSpriteKind::PathMask13 => "Path mask 13",
            TerrainSpriteKind::PathMask14 => "Path mask 14",
            TerrainSpriteKind::PathMask15 => "Path mask 15",
        }
    }

    pub fn is_transition(self) -> bool {
        matches!(
            self,
            TerrainSpriteKind::GrassToDirtEdgeNorth
                | TerrainSpriteKind::GrassToDirtEdgeSouth
                | TerrainSpriteKind::GrassToDirtEdgeEast
                | TerrainSpriteKind::GrassToDirtEdgeWest
        )
    }

    pub fn path_mask(self) -> Option<u8> {
        match self {
            TerrainSpriteKind::PathMask00 => Some(0),
            TerrainSpriteKind::PathMask01 => Some(1),
            TerrainSpriteKind::PathMask02 => Some(2),
            TerrainSpriteKind::PathMask03 => Some(3),
            TerrainSpriteKind::PathMask04 => Some(4),
            TerrainSpriteKind::PathMask05 => Some(5),
            TerrainSpriteKind::PathMask06 => Some(6),
            TerrainSpriteKind::PathMask07 => Some(7),
            TerrainSpriteKind::PathMask08 => Some(8),
            TerrainSpriteKind::PathMask09 => Some(9),
            TerrainSpriteKind::PathMask10 => Some(10),
            TerrainSpriteKind::PathMask11 => Some(11),
            TerrainSpriteKind::PathMask12 => Some(12),
            TerrainSpriteKind::PathMask13 => Some(13),
            TerrainSpriteKind::PathMask14 => Some(14),
            TerrainSpriteKind::PathMask15 => Some(15),
            _ => None,
        }
    }

    pub fn from_path_mask(mask: u8) -> Option<Self> {
        match mask {
            0 => Some(TerrainSpriteKind::PathMask00),
            1 => Some(TerrainSpriteKind::PathMask01),
            2 => Some(TerrainSpriteKind::PathMask02),
            3 => Some(TerrainSpriteKind::PathMask03),
            4 => Some(TerrainSpriteKind::PathMask04),
            5 => Some(TerrainSpriteKind::PathMask05),
            6 => Some(TerrainSpriteKind::PathMask06),
            7 => Some(TerrainSpriteKind::PathMask07),
            8 => Some(TerrainSpriteKind::PathMask08),
            9 => Some(TerrainSpriteKind::PathMask09),
            10 => Some(TerrainSpriteKind::PathMask10),
            11 => Some(TerrainSpriteKind::PathMask11),
            12 => Some(TerrainSpriteKind::PathMask12),
            13 => Some(TerrainSpriteKind::PathMask13),
            14 => Some(TerrainSpriteKind::PathMask14),
            15 => Some(TerrainSpriteKind::PathMask15),
            _ => None,
        }
    }

    pub fn is_path_mask(self) -> bool {
        self.path_mask().is_some()
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedTerrainSprite {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub variant: u32,
    pub image: PixelImage,
}
