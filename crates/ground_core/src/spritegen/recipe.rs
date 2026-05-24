use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::pixel_image::PixelImage;
use crate::spritegen::{ObliqueProjectionProfile, TerrainMotifLibrary, TerrainSpriteStyle};

pub const DEFAULT_SPRITEGEN_EXPORT_DIR: &str = "exports/artgen_03_3";
pub const DEFAULT_SPRITE_STYLE_PATH: &str = "assets/sprite_styles/cozy_upland/style.ron";

pub const BUILTIN_SPRITE_STYLE_PROFILES: [(&str, &str); 3] = [
    ("cozy_upland", "assets/sprite_styles/cozy_upland/style.ron"),
    (
        "cozy_upland_lush",
        "assets/sprite_styles/cozy_upland_lush/style.ron",
    ),
    (
        "cozy_upland_sparse",
        "assets/sprite_styles/cozy_upland_sparse/style.ron",
    ),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpriteRecipe {
    pub id: String,
    pub tile_size: u32,
    pub seed: u64,
    pub variant_count: u32,
    pub override_dir: String,
    pub style: TerrainSpriteStyle,
    pub motifs: TerrainMotifLibrary,
}

impl Default for TerrainSpriteRecipe {
    fn default() -> Self {
        Self {
            id: "cozy_grass_dirt_artgen_01".to_string(),
            tile_size: 16,
            seed: 0x5eed_7101,
            variant_count: 4,
            override_dir: String::new(),
            style: TerrainSpriteStyle::default(),
            motifs: TerrainMotifLibrary::default(),
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
        self.style.projection.cell_width_px = self
            .style
            .projection
            .cell_width_px
            .clamp(self.tile_size, 256);
        self.style.projection.cell_height_px = self
            .style
            .projection
            .cell_height_px
            .clamp(self.tile_size, 192);
        self.style.projection.face_height_px = self.style.projection.face_height_px.clamp(1, 96);
        self.style.projection.shadow_offset_px.0 =
            self.style.projection.shadow_offset_px.0.clamp(-64, 64);
        self.style.projection.shadow_offset_px.1 =
            self.style.projection.shadow_offset_px.1.clamp(-64, 96);
        self.style.path.width_px = self.style.path.width_px.clamp(2.0, self.tile_size as f32);
        self.style.path.core_width_px = self
            .style
            .path
            .core_width_px
            .max(self.style.path.width_px)
            .clamp(2.0, self.tile_size as f32);
        self.style.path.corner_rounding = self.style.path.corner_rounding.clamp(0.0, 1.0);
        self.style.path.edge_noise = self.style.path.edge_noise.clamp(0.0, 3.0);
        self.style.trench.floor_darkness = self.style.trench.floor_darkness.clamp(0.0, 1.0);
        self.style.trench.floor_detail_density =
            self.style.trench.floor_detail_density.clamp(0.0, 1.0);
        self.style.trench.wall_shadow_strength =
            self.style.trench.wall_shadow_strength.clamp(0.0, 1.0);
        self.style.trench.wall_detail_density =
            self.style.trench.wall_detail_density.clamp(0.0, 1.0);
        self.style.trench.lip_highlight_strength =
            self.style.trench.lip_highlight_strength.clamp(0.0, 1.0);
        self.style.trench.lip_irregularity_px = self.style.trench.lip_irregularity_px.clamp(0, 12);
        self.style.trench.wood_plank_density = self.style.trench.wood_plank_density.clamp(0.0, 1.0);
        self.style.trench.wood_knot_density = self.style.trench.wood_knot_density.clamp(0.0, 1.0);
        self.style.trench.spoil_density = self.style.trench.spoil_density.clamp(0.0, 1.0);
        self.style.trench.grass_intrusion_density =
            self.style.trench.grass_intrusion_density.clamp(0.0, 1.0);
        self.style.trench.inner_shadow_strength =
            self.style.trench.inner_shadow_strength.clamp(0.0, 1.0);
        self.style.trench.contact_shadow_strength =
            self.style.trench.contact_shadow_strength.clamp(0.0, 1.0);
        self.style.berm.mound_height_strength =
            self.style.berm.mound_height_strength.clamp(0.0, 1.0);
        self.style.berm.face_shadow_strength = self.style.berm.face_shadow_strength.clamp(0.0, 1.0);
        self.style.berm.top_grass_blend = self.style.berm.top_grass_blend.clamp(0.0, 1.0);
        self.style.berm.lip_highlight_strength =
            self.style.berm.lip_highlight_strength.clamp(0.0, 1.0);
        self.style.berm.edge_irregularity_px = self.style.berm.edge_irregularity_px.clamp(0, 12);
        self.style.berm.spoil_density = self.style.berm.spoil_density.clamp(0.0, 1.0);
        self.style.berm.grass_intrusion_density =
            self.style.berm.grass_intrusion_density.clamp(0.0, 1.0);
        self.style.berm.contact_shadow_strength =
            self.style.berm.contact_shadow_strength.clamp(0.0, 1.0);
        self.motifs.sanitize();
    }

    pub fn from_style_profile_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)?;
        let profile: TerrainSpriteStyleProfile = ron::from_str(&text)?;
        profile.into_recipe(path.parent().unwrap_or_else(|| Path::new(".")))
    }

    pub fn from_default_style_profile() -> Self {
        Self::from_style_profile_path(DEFAULT_SPRITE_STYLE_PATH).unwrap_or_default()
    }

    pub fn save_style_profile_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(base_dir)?;
        let profile = TerrainSpriteStyleProfile {
            id: self.id.clone(),
            tile_size: self.tile_size,
            seed: self.seed,
            variant_count: self.variant_count,
            overrides: profile_relative_path(base_dir, &self.override_dir, "overrides"),
            style: self.style.clone(),
            motifs: "motifs.ron".to_string(),
        };
        let profile_text = ron::ser::to_string_pretty(&profile, PrettyConfig::new())?;
        fs::write(path, profile_text)?;

        let motif_path = resolve_profile_path(base_dir, &profile.motifs);
        let motif_text = ron::ser::to_string_pretty(&self.motifs, PrettyConfig::new())?;
        fs::write(motif_path, motif_text)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpriteStyleProfile {
    pub id: String,
    pub tile_size: u32,
    pub seed: u64,
    pub variant_count: u32,
    pub overrides: String,
    pub style: TerrainSpriteStyle,
    pub motifs: String,
}

impl Default for TerrainSpriteStyleProfile {
    fn default() -> Self {
        Self {
            id: "cozy_upland".to_string(),
            tile_size: 16,
            seed: 0x5eed_7101,
            variant_count: 4,
            overrides: "overrides".to_string(),
            style: TerrainSpriteStyle::default(),
            motifs: "motifs.ron".to_string(),
        }
    }
}

impl TerrainSpriteStyleProfile {
    pub fn into_recipe(self, base_dir: &Path) -> Result<TerrainSpriteRecipe> {
        let motif_path = resolve_profile_path(base_dir, &self.motifs);
        let motif_text = fs::read_to_string(motif_path)?;
        let motifs: TerrainMotifLibrary = ron::from_str(&motif_text)?;
        let override_dir = resolve_profile_path(base_dir, &self.overrides)
            .to_string_lossy()
            .to_string();
        let mut recipe = TerrainSpriteRecipe {
            id: self.id,
            tile_size: self.tile_size,
            seed: self.seed,
            variant_count: self.variant_count,
            override_dir,
            style: self.style,
            motifs,
        };
        recipe.sanitize();
        Ok(recipe)
    }
}

impl TerrainMotifLibrary {
    pub fn sanitize(&mut self) {
        self.grass_dark.retain(|motif| !motif.pixels.is_empty());
        self.grass_light.retain(|motif| !motif.pixels.is_empty());
        self.grass_blades.retain(|motif| !motif.pixels.is_empty());
        self.grass_flowers.retain(|motif| !motif.pixels.is_empty());
        self.dirt_dust.retain(|motif| !motif.pixels.is_empty());
        self.dirt_dents.retain(|motif| !motif.pixels.is_empty());
        self.dirt_ruts.retain(|motif| !motif.pixels.is_empty());
        self.transition_intrusion
            .retain(|motif| !motif.pixels.is_empty());
        self.trench_wood.retain(|motif| !motif.pixels.is_empty());
        self.trench_wall_shadow
            .retain(|motif| !motif.pixels.is_empty());
        self.trench_lip.retain(|motif| !motif.pixels.is_empty());
        self.trench_spoil.retain(|motif| !motif.pixels.is_empty());
        self.trench_grass_overhang
            .retain(|motif| !motif.pixels.is_empty());
        self.berm_soil_clump
            .retain(|motif| !motif.pixels.is_empty());
        self.berm_grass_overhang
            .retain(|motif| !motif.pixels.is_empty());
        self.berm_face_shadow
            .retain(|motif| !motif.pixels.is_empty());
        self.berm_edge_highlight
            .retain(|motif| !motif.pixels.is_empty());
        self.berm_spoil.retain(|motif| !motif.pixels.is_empty());
        let fallback = TerrainMotifLibrary::default();
        self.grass_dark = with_fallback(std::mem::take(&mut self.grass_dark), fallback.grass_dark);
        self.grass_light =
            with_fallback(std::mem::take(&mut self.grass_light), fallback.grass_light);
        self.grass_blades = with_fallback(
            std::mem::take(&mut self.grass_blades),
            fallback.grass_blades,
        );
        self.grass_flowers = with_fallback(
            std::mem::take(&mut self.grass_flowers),
            fallback.grass_flowers,
        );
        self.dirt_dust = with_fallback(std::mem::take(&mut self.dirt_dust), fallback.dirt_dust);
        self.dirt_dents = with_fallback(std::mem::take(&mut self.dirt_dents), fallback.dirt_dents);
        self.dirt_ruts = with_fallback(std::mem::take(&mut self.dirt_ruts), fallback.dirt_ruts);
        self.transition_intrusion = with_fallback(
            std::mem::take(&mut self.transition_intrusion),
            fallback.transition_intrusion,
        );
        self.trench_wood =
            with_fallback(std::mem::take(&mut self.trench_wood), fallback.trench_wood);
        self.trench_wall_shadow = with_fallback(
            std::mem::take(&mut self.trench_wall_shadow),
            fallback.trench_wall_shadow,
        );
        self.trench_lip = with_fallback(std::mem::take(&mut self.trench_lip), fallback.trench_lip);
        self.trench_spoil = with_fallback(
            std::mem::take(&mut self.trench_spoil),
            fallback.trench_spoil,
        );
        self.trench_grass_overhang = with_fallback(
            std::mem::take(&mut self.trench_grass_overhang),
            fallback.trench_grass_overhang,
        );
        self.berm_soil_clump = with_fallback(
            std::mem::take(&mut self.berm_soil_clump),
            fallback.berm_soil_clump,
        );
        self.berm_grass_overhang = with_fallback(
            std::mem::take(&mut self.berm_grass_overhang),
            fallback.berm_grass_overhang,
        );
        self.berm_face_shadow = with_fallback(
            std::mem::take(&mut self.berm_face_shadow),
            fallback.berm_face_shadow,
        );
        self.berm_edge_highlight = with_fallback(
            std::mem::take(&mut self.berm_edge_highlight),
            fallback.berm_edge_highlight,
        );
        self.berm_spoil = with_fallback(std::mem::take(&mut self.berm_spoil), fallback.berm_spoil);
    }
}

fn with_fallback<T>(value: Vec<T>, fallback: Vec<T>) -> Vec<T> {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

fn resolve_profile_path(base_dir: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn profile_relative_path(base_dir: &Path, value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        return fallback.to_string();
    }

    let value_path = PathBuf::from(value);
    if let Ok(path) = value_path.strip_prefix(base_dir) {
        return normalize_profile_path(path);
    }

    if let (Ok(base), Ok(path)) = (
        fs::canonicalize(base_dir),
        fs::canonicalize(if value_path.is_absolute() {
            value_path.clone()
        } else {
            PathBuf::from(value)
        }),
    ) {
        if let Ok(path) = path.strip_prefix(base) {
            return normalize_profile_path(path);
        }
    }

    if value_path.is_absolute() {
        normalize_profile_path(&value_path)
    } else {
        normalize_profile_path(Path::new(value))
    }
}

fn normalize_profile_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_style_profile_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "groundlab_sprite_profile_save_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("style.ron");

        let recipe = TerrainSpriteRecipe::default();
        recipe.save_style_profile_path(&path).unwrap();

        let loaded = TerrainSpriteRecipe::from_style_profile_path(&path).unwrap();
        assert_eq!(loaded.id, recipe.id);
        assert_eq!(loaded.tile_size, recipe.tile_size);
        assert_eq!(loaded.variant_count, recipe.variant_count);
        assert_eq!(loaded.style.display_scale, recipe.style.display_scale);
        assert!(dir.join("motifs.ron").exists());

        fs::remove_dir_all(&dir).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteBundleManifest {
    pub id: String,
    pub projection: ObliqueProjectionProfile,
    pub pieces: Vec<TerrainSpritePieceManifest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpritePieceManifest {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub role: SpriteRole,
    pub source: TerrainSpriteSource,
    pub file: String,
    pub width_px: u32,
    pub height_px: u32,
    pub anchor_px: (i32, i32),
    pub footprint_cells: (u32, u32),
    pub z_bias: i32,
    pub occludes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainSpriteSource {
    Generated,
    Override,
}

impl TerrainSpriteSource {
    pub fn id(self) -> &'static str {
        match self {
            TerrainSpriteSource::Generated => "generated",
            TerrainSpriteSource::Override => "override",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpriteRole {
    TopSurface,
    FrontFace,
    SideFace,
    Lip,
    CornerCap,
    ContactShadow,
    Prop,
    Decal,
}

impl SpriteRole {
    pub fn id(self) -> &'static str {
        match self {
            SpriteRole::TopSurface => "top_surface",
            SpriteRole::FrontFace => "front_face",
            SpriteRole::SideFace => "side_face",
            SpriteRole::Lip => "lip",
            SpriteRole::CornerCap => "corner_cap",
            SpriteRole::ContactShadow => "contact_shadow",
            SpriteRole::Prop => "prop",
            SpriteRole::Decal => "decal",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpritePieceMetadata {
    pub role: SpriteRole,
    pub anchor_px: (i32, i32),
    pub footprint_cells: (u32, u32),
    pub z_bias: i32,
    pub occludes: bool,
}

impl SpritePieceMetadata {
    pub fn new(role: SpriteRole) -> Self {
        Self {
            role,
            anchor_px: (0, 0),
            footprint_cells: (1, 1),
            z_bias: 0,
            occludes: false,
        }
    }

    pub fn z_bias(mut self, z_bias: i32) -> Self {
        self.z_bias = z_bias;
        self
    }

    pub fn anchor(mut self, anchor_px: (i32, i32)) -> Self {
        self.anchor_px = anchor_px;
        self
    }

    pub fn footprint(mut self, footprint_cells: (u32, u32)) -> Self {
        self.footprint_cells = footprint_cells;
        self
    }

    pub fn occludes(mut self, occludes: bool) -> Self {
        self.occludes = occludes;
        self
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
    TrenchFloorTop,
    TrenchWallFront,
    TrenchLipFront,
    TrenchLipBack,
    TrenchEndCapLeft,
    TrenchEndCapRight,
    TrenchCornerInner,
    TrenchCornerOuter,
    TrenchContactShadow,
    TrenchSpoilPile,
    BermTop,
    BermFaceFront,
    BermLipFront,
    BermLipBack,
    BermEndCapLeft,
    BermEndCapRight,
    BermCornerInner,
    BermCornerOuter,
    BermContactShadow,
    BermSpoilPile,
    BermGrassFringe,
    TrenchMask00,
    TrenchMask01,
    TrenchMask02,
    TrenchMask03,
    TrenchMask04,
    TrenchMask05,
    TrenchMask06,
    TrenchMask07,
    TrenchMask08,
    TrenchMask09,
    TrenchMask10,
    TrenchMask11,
    TrenchMask12,
    TrenchMask13,
    TrenchMask14,
    TrenchMask15,
    BermMask00,
    BermMask01,
    BermMask02,
    BermMask03,
    BermMask04,
    BermMask05,
    BermMask06,
    BermMask07,
    BermMask08,
    BermMask09,
    BermMask10,
    BermMask11,
    BermMask12,
    BermMask13,
    BermMask14,
    BermMask15,
}

impl TerrainSpriteKind {
    pub const ALL: [TerrainSpriteKind; 75] = [
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
        TerrainSpriteKind::TrenchFloorTop,
        TerrainSpriteKind::TrenchWallFront,
        TerrainSpriteKind::TrenchLipFront,
        TerrainSpriteKind::TrenchLipBack,
        TerrainSpriteKind::TrenchEndCapLeft,
        TerrainSpriteKind::TrenchEndCapRight,
        TerrainSpriteKind::TrenchCornerInner,
        TerrainSpriteKind::TrenchCornerOuter,
        TerrainSpriteKind::TrenchContactShadow,
        TerrainSpriteKind::TrenchSpoilPile,
        TerrainSpriteKind::BermTop,
        TerrainSpriteKind::BermFaceFront,
        TerrainSpriteKind::BermLipFront,
        TerrainSpriteKind::BermLipBack,
        TerrainSpriteKind::BermEndCapLeft,
        TerrainSpriteKind::BermEndCapRight,
        TerrainSpriteKind::BermCornerInner,
        TerrainSpriteKind::BermCornerOuter,
        TerrainSpriteKind::BermContactShadow,
        TerrainSpriteKind::BermSpoilPile,
        TerrainSpriteKind::BermGrassFringe,
        TerrainSpriteKind::TrenchMask00,
        TerrainSpriteKind::TrenchMask01,
        TerrainSpriteKind::TrenchMask02,
        TerrainSpriteKind::TrenchMask03,
        TerrainSpriteKind::TrenchMask04,
        TerrainSpriteKind::TrenchMask05,
        TerrainSpriteKind::TrenchMask06,
        TerrainSpriteKind::TrenchMask07,
        TerrainSpriteKind::TrenchMask08,
        TerrainSpriteKind::TrenchMask09,
        TerrainSpriteKind::TrenchMask10,
        TerrainSpriteKind::TrenchMask11,
        TerrainSpriteKind::TrenchMask12,
        TerrainSpriteKind::TrenchMask13,
        TerrainSpriteKind::TrenchMask14,
        TerrainSpriteKind::TrenchMask15,
        TerrainSpriteKind::BermMask00,
        TerrainSpriteKind::BermMask01,
        TerrainSpriteKind::BermMask02,
        TerrainSpriteKind::BermMask03,
        TerrainSpriteKind::BermMask04,
        TerrainSpriteKind::BermMask05,
        TerrainSpriteKind::BermMask06,
        TerrainSpriteKind::BermMask07,
        TerrainSpriteKind::BermMask08,
        TerrainSpriteKind::BermMask09,
        TerrainSpriteKind::BermMask10,
        TerrainSpriteKind::BermMask11,
        TerrainSpriteKind::BermMask12,
        TerrainSpriteKind::BermMask13,
        TerrainSpriteKind::BermMask14,
        TerrainSpriteKind::BermMask15,
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
            TerrainSpriteKind::TrenchFloorTop => "trench_floor_top",
            TerrainSpriteKind::TrenchWallFront => "trench_wall_front",
            TerrainSpriteKind::TrenchLipFront => "trench_lip_front",
            TerrainSpriteKind::TrenchLipBack => "trench_lip_back",
            TerrainSpriteKind::TrenchEndCapLeft => "trench_end_cap_left",
            TerrainSpriteKind::TrenchEndCapRight => "trench_end_cap_right",
            TerrainSpriteKind::TrenchCornerInner => "trench_corner_inner",
            TerrainSpriteKind::TrenchCornerOuter => "trench_corner_outer",
            TerrainSpriteKind::TrenchContactShadow => "trench_contact_shadow",
            TerrainSpriteKind::TrenchSpoilPile => "trench_spoil_pile",
            TerrainSpriteKind::BermTop => "berm_top",
            TerrainSpriteKind::BermFaceFront => "berm_face_front",
            TerrainSpriteKind::BermLipFront => "berm_lip_front",
            TerrainSpriteKind::BermLipBack => "berm_lip_back",
            TerrainSpriteKind::BermEndCapLeft => "berm_end_cap_left",
            TerrainSpriteKind::BermEndCapRight => "berm_end_cap_right",
            TerrainSpriteKind::BermCornerInner => "berm_corner_inner",
            TerrainSpriteKind::BermCornerOuter => "berm_corner_outer",
            TerrainSpriteKind::BermContactShadow => "berm_contact_shadow",
            TerrainSpriteKind::BermSpoilPile => "berm_spoil_pile",
            TerrainSpriteKind::BermGrassFringe => "berm_grass_fringe",
            TerrainSpriteKind::TrenchMask00 => "trench_mask_00",
            TerrainSpriteKind::TrenchMask01 => "trench_mask_01",
            TerrainSpriteKind::TrenchMask02 => "trench_mask_02",
            TerrainSpriteKind::TrenchMask03 => "trench_mask_03",
            TerrainSpriteKind::TrenchMask04 => "trench_mask_04",
            TerrainSpriteKind::TrenchMask05 => "trench_mask_05",
            TerrainSpriteKind::TrenchMask06 => "trench_mask_06",
            TerrainSpriteKind::TrenchMask07 => "trench_mask_07",
            TerrainSpriteKind::TrenchMask08 => "trench_mask_08",
            TerrainSpriteKind::TrenchMask09 => "trench_mask_09",
            TerrainSpriteKind::TrenchMask10 => "trench_mask_10",
            TerrainSpriteKind::TrenchMask11 => "trench_mask_11",
            TerrainSpriteKind::TrenchMask12 => "trench_mask_12",
            TerrainSpriteKind::TrenchMask13 => "trench_mask_13",
            TerrainSpriteKind::TrenchMask14 => "trench_mask_14",
            TerrainSpriteKind::TrenchMask15 => "trench_mask_15",
            TerrainSpriteKind::BermMask00 => "berm_mask_00",
            TerrainSpriteKind::BermMask01 => "berm_mask_01",
            TerrainSpriteKind::BermMask02 => "berm_mask_02",
            TerrainSpriteKind::BermMask03 => "berm_mask_03",
            TerrainSpriteKind::BermMask04 => "berm_mask_04",
            TerrainSpriteKind::BermMask05 => "berm_mask_05",
            TerrainSpriteKind::BermMask06 => "berm_mask_06",
            TerrainSpriteKind::BermMask07 => "berm_mask_07",
            TerrainSpriteKind::BermMask08 => "berm_mask_08",
            TerrainSpriteKind::BermMask09 => "berm_mask_09",
            TerrainSpriteKind::BermMask10 => "berm_mask_10",
            TerrainSpriteKind::BermMask11 => "berm_mask_11",
            TerrainSpriteKind::BermMask12 => "berm_mask_12",
            TerrainSpriteKind::BermMask13 => "berm_mask_13",
            TerrainSpriteKind::BermMask14 => "berm_mask_14",
            TerrainSpriteKind::BermMask15 => "berm_mask_15",
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
            TerrainSpriteKind::TrenchFloorTop => "Trench floor top",
            TerrainSpriteKind::TrenchWallFront => "Trench wall front",
            TerrainSpriteKind::TrenchLipFront => "Trench lip front",
            TerrainSpriteKind::TrenchLipBack => "Trench lip back",
            TerrainSpriteKind::TrenchEndCapLeft => "Trench end cap left",
            TerrainSpriteKind::TrenchEndCapRight => "Trench end cap right",
            TerrainSpriteKind::TrenchCornerInner => "Trench inner corner",
            TerrainSpriteKind::TrenchCornerOuter => "Trench outer corner",
            TerrainSpriteKind::TrenchContactShadow => "Trench contact shadow",
            TerrainSpriteKind::TrenchSpoilPile => "Trench spoil pile",
            TerrainSpriteKind::BermTop => "Berm top",
            TerrainSpriteKind::BermFaceFront => "Berm front face",
            TerrainSpriteKind::BermLipFront => "Berm lip front",
            TerrainSpriteKind::BermLipBack => "Berm lip back",
            TerrainSpriteKind::BermEndCapLeft => "Berm end cap left",
            TerrainSpriteKind::BermEndCapRight => "Berm end cap right",
            TerrainSpriteKind::BermCornerInner => "Berm inner corner",
            TerrainSpriteKind::BermCornerOuter => "Berm outer corner",
            TerrainSpriteKind::BermContactShadow => "Berm contact shadow",
            TerrainSpriteKind::BermSpoilPile => "Berm spoil pile",
            TerrainSpriteKind::BermGrassFringe => "Berm grass fringe",
            TerrainSpriteKind::TrenchMask00 => "Trench mask 00",
            TerrainSpriteKind::TrenchMask01 => "Trench mask 01",
            TerrainSpriteKind::TrenchMask02 => "Trench mask 02",
            TerrainSpriteKind::TrenchMask03 => "Trench mask 03",
            TerrainSpriteKind::TrenchMask04 => "Trench mask 04",
            TerrainSpriteKind::TrenchMask05 => "Trench mask 05",
            TerrainSpriteKind::TrenchMask06 => "Trench mask 06",
            TerrainSpriteKind::TrenchMask07 => "Trench mask 07",
            TerrainSpriteKind::TrenchMask08 => "Trench mask 08",
            TerrainSpriteKind::TrenchMask09 => "Trench mask 09",
            TerrainSpriteKind::TrenchMask10 => "Trench mask 10",
            TerrainSpriteKind::TrenchMask11 => "Trench mask 11",
            TerrainSpriteKind::TrenchMask12 => "Trench mask 12",
            TerrainSpriteKind::TrenchMask13 => "Trench mask 13",
            TerrainSpriteKind::TrenchMask14 => "Trench mask 14",
            TerrainSpriteKind::TrenchMask15 => "Trench mask 15",
            TerrainSpriteKind::BermMask00 => "Berm mask 00",
            TerrainSpriteKind::BermMask01 => "Berm mask 01",
            TerrainSpriteKind::BermMask02 => "Berm mask 02",
            TerrainSpriteKind::BermMask03 => "Berm mask 03",
            TerrainSpriteKind::BermMask04 => "Berm mask 04",
            TerrainSpriteKind::BermMask05 => "Berm mask 05",
            TerrainSpriteKind::BermMask06 => "Berm mask 06",
            TerrainSpriteKind::BermMask07 => "Berm mask 07",
            TerrainSpriteKind::BermMask08 => "Berm mask 08",
            TerrainSpriteKind::BermMask09 => "Berm mask 09",
            TerrainSpriteKind::BermMask10 => "Berm mask 10",
            TerrainSpriteKind::BermMask11 => "Berm mask 11",
            TerrainSpriteKind::BermMask12 => "Berm mask 12",
            TerrainSpriteKind::BermMask13 => "Berm mask 13",
            TerrainSpriteKind::BermMask14 => "Berm mask 14",
            TerrainSpriteKind::BermMask15 => "Berm mask 15",
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

    pub fn trench_mask(self) -> Option<u8> {
        match self {
            TerrainSpriteKind::TrenchMask00 => Some(0),
            TerrainSpriteKind::TrenchMask01 => Some(1),
            TerrainSpriteKind::TrenchMask02 => Some(2),
            TerrainSpriteKind::TrenchMask03 => Some(3),
            TerrainSpriteKind::TrenchMask04 => Some(4),
            TerrainSpriteKind::TrenchMask05 => Some(5),
            TerrainSpriteKind::TrenchMask06 => Some(6),
            TerrainSpriteKind::TrenchMask07 => Some(7),
            TerrainSpriteKind::TrenchMask08 => Some(8),
            TerrainSpriteKind::TrenchMask09 => Some(9),
            TerrainSpriteKind::TrenchMask10 => Some(10),
            TerrainSpriteKind::TrenchMask11 => Some(11),
            TerrainSpriteKind::TrenchMask12 => Some(12),
            TerrainSpriteKind::TrenchMask13 => Some(13),
            TerrainSpriteKind::TrenchMask14 => Some(14),
            TerrainSpriteKind::TrenchMask15 => Some(15),
            _ => None,
        }
    }

    pub fn from_trench_mask(mask: u8) -> Option<Self> {
        match mask {
            0 => Some(TerrainSpriteKind::TrenchMask00),
            1 => Some(TerrainSpriteKind::TrenchMask01),
            2 => Some(TerrainSpriteKind::TrenchMask02),
            3 => Some(TerrainSpriteKind::TrenchMask03),
            4 => Some(TerrainSpriteKind::TrenchMask04),
            5 => Some(TerrainSpriteKind::TrenchMask05),
            6 => Some(TerrainSpriteKind::TrenchMask06),
            7 => Some(TerrainSpriteKind::TrenchMask07),
            8 => Some(TerrainSpriteKind::TrenchMask08),
            9 => Some(TerrainSpriteKind::TrenchMask09),
            10 => Some(TerrainSpriteKind::TrenchMask10),
            11 => Some(TerrainSpriteKind::TrenchMask11),
            12 => Some(TerrainSpriteKind::TrenchMask12),
            13 => Some(TerrainSpriteKind::TrenchMask13),
            14 => Some(TerrainSpriteKind::TrenchMask14),
            15 => Some(TerrainSpriteKind::TrenchMask15),
            _ => None,
        }
    }

    pub fn is_trench_mask(self) -> bool {
        self.trench_mask().is_some()
    }

    pub fn berm_mask(self) -> Option<u8> {
        match self {
            TerrainSpriteKind::BermMask00 => Some(0),
            TerrainSpriteKind::BermMask01 => Some(1),
            TerrainSpriteKind::BermMask02 => Some(2),
            TerrainSpriteKind::BermMask03 => Some(3),
            TerrainSpriteKind::BermMask04 => Some(4),
            TerrainSpriteKind::BermMask05 => Some(5),
            TerrainSpriteKind::BermMask06 => Some(6),
            TerrainSpriteKind::BermMask07 => Some(7),
            TerrainSpriteKind::BermMask08 => Some(8),
            TerrainSpriteKind::BermMask09 => Some(9),
            TerrainSpriteKind::BermMask10 => Some(10),
            TerrainSpriteKind::BermMask11 => Some(11),
            TerrainSpriteKind::BermMask12 => Some(12),
            TerrainSpriteKind::BermMask13 => Some(13),
            TerrainSpriteKind::BermMask14 => Some(14),
            TerrainSpriteKind::BermMask15 => Some(15),
            _ => None,
        }
    }

    pub fn from_berm_mask(mask: u8) -> Option<Self> {
        match mask {
            0 => Some(TerrainSpriteKind::BermMask00),
            1 => Some(TerrainSpriteKind::BermMask01),
            2 => Some(TerrainSpriteKind::BermMask02),
            3 => Some(TerrainSpriteKind::BermMask03),
            4 => Some(TerrainSpriteKind::BermMask04),
            5 => Some(TerrainSpriteKind::BermMask05),
            6 => Some(TerrainSpriteKind::BermMask06),
            7 => Some(TerrainSpriteKind::BermMask07),
            8 => Some(TerrainSpriteKind::BermMask08),
            9 => Some(TerrainSpriteKind::BermMask09),
            10 => Some(TerrainSpriteKind::BermMask10),
            11 => Some(TerrainSpriteKind::BermMask11),
            12 => Some(TerrainSpriteKind::BermMask12),
            13 => Some(TerrainSpriteKind::BermMask13),
            14 => Some(TerrainSpriteKind::BermMask14),
            15 => Some(TerrainSpriteKind::BermMask15),
            _ => None,
        }
    }

    pub fn is_berm_mask(self) -> bool {
        self.berm_mask().is_some()
    }

    pub fn is_trench(self) -> bool {
        matches!(
            self,
            TerrainSpriteKind::TrenchFloorTop
                | TerrainSpriteKind::TrenchWallFront
                | TerrainSpriteKind::TrenchLipFront
                | TerrainSpriteKind::TrenchLipBack
                | TerrainSpriteKind::TrenchEndCapLeft
                | TerrainSpriteKind::TrenchEndCapRight
                | TerrainSpriteKind::TrenchCornerInner
                | TerrainSpriteKind::TrenchCornerOuter
                | TerrainSpriteKind::TrenchContactShadow
                | TerrainSpriteKind::TrenchSpoilPile
        )
    }

    pub fn is_berm(self) -> bool {
        matches!(
            self,
            TerrainSpriteKind::BermTop
                | TerrainSpriteKind::BermFaceFront
                | TerrainSpriteKind::BermLipFront
                | TerrainSpriteKind::BermLipBack
                | TerrainSpriteKind::BermEndCapLeft
                | TerrainSpriteKind::BermEndCapRight
                | TerrainSpriteKind::BermCornerInner
                | TerrainSpriteKind::BermCornerOuter
                | TerrainSpriteKind::BermContactShadow
                | TerrainSpriteKind::BermSpoilPile
                | TerrainSpriteKind::BermGrassFringe
        )
    }

    pub fn default_piece_metadata(self) -> SpritePieceMetadata {
        match self {
            TerrainSpriteKind::GrassTile
            | TerrainSpriteKind::DirtTile
            | TerrainSpriteKind::PathMask00
            | TerrainSpriteKind::PathMask01
            | TerrainSpriteKind::PathMask02
            | TerrainSpriteKind::PathMask03
            | TerrainSpriteKind::PathMask04
            | TerrainSpriteKind::PathMask05
            | TerrainSpriteKind::PathMask06
            | TerrainSpriteKind::PathMask07
            | TerrainSpriteKind::PathMask08
            | TerrainSpriteKind::PathMask09
            | TerrainSpriteKind::PathMask10
            | TerrainSpriteKind::PathMask11
            | TerrainSpriteKind::PathMask12
            | TerrainSpriteKind::PathMask13
            | TerrainSpriteKind::PathMask14
            | TerrainSpriteKind::PathMask15 => SpritePieceMetadata::new(SpriteRole::TopSurface),
            TerrainSpriteKind::GrassToDirtEdgeNorth
            | TerrainSpriteKind::GrassToDirtEdgeSouth
            | TerrainSpriteKind::GrassToDirtEdgeEast
            | TerrainSpriteKind::GrassToDirtEdgeWest => {
                SpritePieceMetadata::new(SpriteRole::Decal).z_bias(2)
            }
            TerrainSpriteKind::TrenchFloorTop => SpritePieceMetadata::new(SpriteRole::TopSurface)
                .footprint((2, 1))
                .z_bias(8),
            TerrainSpriteKind::TrenchWallFront => SpritePieceMetadata::new(SpriteRole::FrontFace)
                .anchor((0, -6))
                .footprint((2, 1))
                .z_bias(24)
                .occludes(true),
            TerrainSpriteKind::TrenchLipFront | TerrainSpriteKind::TrenchLipBack => {
                SpritePieceMetadata::new(SpriteRole::Lip)
                    .footprint((2, 1))
                    .z_bias(28)
            }
            TerrainSpriteKind::TrenchEndCapLeft
            | TerrainSpriteKind::TrenchEndCapRight
            | TerrainSpriteKind::TrenchCornerInner
            | TerrainSpriteKind::TrenchCornerOuter => {
                SpritePieceMetadata::new(SpriteRole::CornerCap)
                    .footprint((1, 1))
                    .z_bias(30)
                    .occludes(true)
            }
            TerrainSpriteKind::TrenchContactShadow => {
                SpritePieceMetadata::new(SpriteRole::ContactShadow)
                    .footprint((2, 1))
                    .z_bias(-4)
            }
            TerrainSpriteKind::TrenchSpoilPile => SpritePieceMetadata::new(SpriteRole::Decal)
                .footprint((1, 1))
                .z_bias(18),
            TerrainSpriteKind::BermTop => SpritePieceMetadata::new(SpriteRole::TopSurface)
                .footprint((2, 1))
                .z_bias(18),
            TerrainSpriteKind::BermFaceFront => SpritePieceMetadata::new(SpriteRole::FrontFace)
                .anchor((0, -8))
                .footprint((2, 1))
                .z_bias(32)
                .occludes(true),
            TerrainSpriteKind::BermLipFront | TerrainSpriteKind::BermLipBack => {
                SpritePieceMetadata::new(SpriteRole::Lip)
                    .footprint((2, 1))
                    .z_bias(36)
            }
            TerrainSpriteKind::BermEndCapLeft
            | TerrainSpriteKind::BermEndCapRight
            | TerrainSpriteKind::BermCornerInner
            | TerrainSpriteKind::BermCornerOuter => SpritePieceMetadata::new(SpriteRole::CornerCap)
                .footprint((1, 1))
                .z_bias(38)
                .occludes(true),
            TerrainSpriteKind::BermContactShadow => {
                SpritePieceMetadata::new(SpriteRole::ContactShadow)
                    .footprint((2, 1))
                    .z_bias(-2)
            }
            TerrainSpriteKind::BermSpoilPile | TerrainSpriteKind::BermGrassFringe => {
                SpritePieceMetadata::new(SpriteRole::Decal)
                    .footprint((1, 1))
                    .z_bias(22)
            }
            TerrainSpriteKind::TrenchMask00
            | TerrainSpriteKind::TrenchMask01
            | TerrainSpriteKind::TrenchMask02
            | TerrainSpriteKind::TrenchMask03
            | TerrainSpriteKind::TrenchMask04
            | TerrainSpriteKind::TrenchMask05
            | TerrainSpriteKind::TrenchMask06
            | TerrainSpriteKind::TrenchMask07
            | TerrainSpriteKind::TrenchMask08
            | TerrainSpriteKind::TrenchMask09
            | TerrainSpriteKind::TrenchMask10
            | TerrainSpriteKind::TrenchMask11
            | TerrainSpriteKind::TrenchMask12
            | TerrainSpriteKind::TrenchMask13
            | TerrainSpriteKind::TrenchMask14
            | TerrainSpriteKind::TrenchMask15
            | TerrainSpriteKind::BermMask00
            | TerrainSpriteKind::BermMask01
            | TerrainSpriteKind::BermMask02
            | TerrainSpriteKind::BermMask03
            | TerrainSpriteKind::BermMask04
            | TerrainSpriteKind::BermMask05
            | TerrainSpriteKind::BermMask06
            | TerrainSpriteKind::BermMask07
            | TerrainSpriteKind::BermMask08
            | TerrainSpriteKind::BermMask09
            | TerrainSpriteKind::BermMask10
            | TerrainSpriteKind::BermMask11
            | TerrainSpriteKind::BermMask12
            | TerrainSpriteKind::BermMask13
            | TerrainSpriteKind::BermMask14
            | TerrainSpriteKind::BermMask15 => SpritePieceMetadata::new(SpriteRole::TopSurface)
                .footprint((1, 1))
                .z_bias(20)
                .occludes(true),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedTerrainSprite {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub variant: u32,
    pub source: TerrainSpriteSource,
    pub metadata: SpritePieceMetadata,
    pub image: PixelImage,
}
