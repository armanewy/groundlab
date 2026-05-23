use serde::{Deserialize, Serialize};

use crate::pixel_image::PixelImage;
use crate::spritegen::TerrainSpriteStyle;

pub const DEFAULT_SPRITEGEN_EXPORT_DIR: &str = "exports/artgen_01";

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
}

impl TerrainSpriteKind {
    pub const ALL: [TerrainSpriteKind; 6] = [
        TerrainSpriteKind::GrassTile,
        TerrainSpriteKind::DirtTile,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        TerrainSpriteKind::GrassToDirtEdgeWest,
    ];

    pub fn id(self) -> &'static str {
        match self {
            TerrainSpriteKind::GrassTile => "grass_tile",
            TerrainSpriteKind::DirtTile => "dirt_tile",
            TerrainSpriteKind::GrassToDirtEdgeNorth => "grass_dirt_edge_north",
            TerrainSpriteKind::GrassToDirtEdgeSouth => "grass_dirt_edge_south",
            TerrainSpriteKind::GrassToDirtEdgeEast => "grass_dirt_edge_east",
            TerrainSpriteKind::GrassToDirtEdgeWest => "grass_dirt_edge_west",
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
}

#[derive(Clone, Debug)]
pub struct GeneratedTerrainSprite {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub variant: u32,
    pub image: PixelImage,
}
