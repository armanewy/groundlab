use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::terrain_artkit::TerrainArtPieceKind;

pub const DEFAULT_HERO_SCENE_PATH: &str = "assets/heroscenes/dry_upland_outpost_hero_01.ron";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroScene {
    pub id: String,
    pub placements: Vec<HeroPlacement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroPlacement {
    pub id: String,
    pub piece_kind: TerrainArtPieceKind,
    pub cell: (u32, u32),
    pub offset_px: (i32, i32),
    pub size_px: (u32, u32),
    pub z_bias: i32,
    pub opacity: f32,
    pub seed: u32,
    pub note: String,
}

impl HeroScene {
    pub fn default_dry_upland_outpost() -> Self {
        Self {
            id: "dry_upland_outpost_hero_01".to_string(),
            placements: vec![
                placement(
                    "road_worn_edge_center",
                    TerrainArtPieceKind::RoadWornEdgePatch,
                    (5, 6),
                    (-52, -10),
                    (150, 42),
                    14,
                    0.82,
                    0x4011,
                    "breaks the road rectangle through the scene center",
                ),
                placement(
                    "trench_spoil_left",
                    TerrainArtPieceKind::TrenchSpoilPile,
                    (4, 4),
                    (-50, -38),
                    (142, 52),
                    28,
                    0.95,
                    0x5031,
                    "loose dug earth beside the trench lip",
                ),
                placement(
                    "trench_end_cap_left",
                    TerrainArtPieceKind::TrenchEndCapLeft,
                    (3, 4),
                    (-58, -30),
                    (76, 58),
                    30,
                    0.94,
                    0x5111,
                    "non-rectangular trench end cap",
                ),
                placement(
                    "trench_end_cap_right",
                    TerrainArtPieceKind::TrenchEndCapRight,
                    (8, 4),
                    (8, -26),
                    (76, 58),
                    30,
                    0.94,
                    0x5221,
                    "non-rectangular trench end cap",
                ),
                placement(
                    "fallen_log_road_block",
                    TerrainArtPieceKind::FallenLog,
                    (6, 6),
                    (-82, -42),
                    (172, 52),
                    44,
                    0.96,
                    0x6111,
                    "large foreground obstacle silhouette",
                ),
                placement(
                    "stake_cluster_objective",
                    TerrainArtPieceKind::StakeCluster,
                    (10, 3),
                    (-32, -58),
                    (96, 92),
                    46,
                    0.88,
                    0x7011,
                    "vertical field-engineering silhouette near objective",
                ),
                placement(
                    "sandbags_objective_front",
                    TerrainArtPieceKind::SandbagShort,
                    (11, 4),
                    (-74, -20),
                    (158, 44),
                    48,
                    0.90,
                    0x7221,
                    "short sandbag run on the raised pad",
                ),
                placement(
                    "berm_corner_left",
                    TerrainArtPieceKind::BermCornerLeft,
                    (7, 7),
                    (-62, -36),
                    (78, 66),
                    34,
                    0.88,
                    0x8111,
                    "berm cap to avoid a flat strip end",
                ),
                placement(
                    "berm_corner_right",
                    TerrainArtPieceKind::BermCornerRight,
                    (11, 7),
                    (14, -36),
                    (78, 66),
                    34,
                    0.88,
                    0x8221,
                    "berm cap to avoid a flat strip end",
                ),
                placement(
                    "broken_stone_corner",
                    TerrainArtPieceKind::LedgeBrokenCorner,
                    (10, 2),
                    (-44, -38),
                    (94, 78),
                    38,
                    0.84,
                    0x8331,
                    "broken ledge silhouette on the objective platform",
                ),
                placement(
                    "large_cast_shadow_foreground",
                    TerrainArtPieceKind::CastShadowLarge,
                    (6, 7),
                    (-92, 0),
                    (220, 72),
                    8,
                    0.58,
                    0x9011,
                    "soft staging shadow under foreground dressing",
                ),
                placement(
                    "grass_tuft_cluster_west",
                    TerrainArtPieceKind::GrassTuftCluster,
                    (2, 5),
                    (-26, -18),
                    (82, 70),
                    36,
                    0.80,
                    0xa011,
                    "grass breaks up the flat floor region",
                ),
                placement(
                    "loose_rocks_near_outcrop",
                    TerrainArtPieceKind::LooseRocks,
                    (3, 2),
                    (-28, -18),
                    (82, 58),
                    40,
                    0.82,
                    0xa221,
                    "small stone debris around rock outcrop",
                ),
                placement(
                    "dirt_scrape_spawn",
                    TerrainArtPieceKind::DirtScrape,
                    (1, 7),
                    (-30, -18),
                    (104, 54),
                    36,
                    0.72,
                    0xa431,
                    "scrape marks near the spawn pad",
                ),
                placement(
                    "tool_marks_objective",
                    TerrainArtPieceKind::ToolMark,
                    (9, 5),
                    (-34, -18),
                    (96, 48),
                    42,
                    0.70,
                    0xa641,
                    "small field-engineering marks beside the road",
                ),
                placement(
                    "broken_berm_edge_center",
                    TerrainArtPieceKind::BrokenBermEdge,
                    (8, 7),
                    (-44, -30),
                    (140, 46),
                    36,
                    0.86,
                    0xa851,
                    "irregular berm edge overlay",
                ),
            ],
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .with_context(|| format!("reading hero scene {}", path.display()))?;
        ron::de::from_str(&text).with_context(|| format!("parsing hero scene {}", path.display()))
    }

    pub fn load_default_or_builtin() -> Self {
        Self::load(DEFAULT_HERO_SCENE_PATH).unwrap_or_else(|_| Self::default_dry_upland_outpost())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = ron::ser::to_string_pretty(self, PrettyConfig::new())?;
        fs::write(path, text)?;
        Ok(())
    }

    pub fn ensure_default_file(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            return Ok(());
        }
        Self::default_dry_upland_outpost().save(path)
    }
}

#[allow(clippy::too_many_arguments)]
fn placement(
    id: &str,
    piece_kind: TerrainArtPieceKind,
    cell: (u32, u32),
    offset_px: (i32, i32),
    size_px: (u32, u32),
    z_bias: i32,
    opacity: f32,
    seed: u32,
    note: &str,
) -> HeroPlacement {
    HeroPlacement {
        id: id.to_string(),
        piece_kind,
        cell,
        offset_px,
        size_px,
        z_bias,
        opacity,
        seed,
        note: note.to_string(),
    }
}
