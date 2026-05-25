use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{clamp_u8, PixelImage, Rgba8};

pub const ART_VARIANT_MAX_COUNT: u32 = 64;
pub const ART_VARIANT_MIN_SIZE: u32 = 32;
pub const ART_VARIANT_MAX_SIZE: u32 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtSpriteFamily {
    TerrainBase,
    Path,
    Trench,
    Berm,
    Tree,
    Log,
    Rock,
    Wall,
    Stakes,
    Wire,
    ObjectiveMarker,
    SpawnMarker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtLabOverrideRole {
    PathDirtSurface,
    TrenchRecessedTerrain,
    BermRaisedTerrain,
    Tree,
    Log,
    Rock,
    Wall,
    Stakes,
    Wire,
    ObjectiveMarker,
    SpawnMarker,
}

impl ArtLabOverrideRole {
    pub const ALL: [ArtLabOverrideRole; 11] = [
        ArtLabOverrideRole::PathDirtSurface,
        ArtLabOverrideRole::TrenchRecessedTerrain,
        ArtLabOverrideRole::BermRaisedTerrain,
        ArtLabOverrideRole::Tree,
        ArtLabOverrideRole::Log,
        ArtLabOverrideRole::Rock,
        ArtLabOverrideRole::Wall,
        ArtLabOverrideRole::Stakes,
        ArtLabOverrideRole::Wire,
        ArtLabOverrideRole::ObjectiveMarker,
        ArtLabOverrideRole::SpawnMarker,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ArtLabOverrideRole::PathDirtSurface => "path / dirt surface",
            ArtLabOverrideRole::TrenchRecessedTerrain => "trench / recessed terrain",
            ArtLabOverrideRole::BermRaisedTerrain => "berm / raised terrain",
            ArtLabOverrideRole::Tree => "tree",
            ArtLabOverrideRole::Log => "log",
            ArtLabOverrideRole::Rock => "rock",
            ArtLabOverrideRole::Wall => "wall",
            ArtLabOverrideRole::Stakes => "stakes",
            ArtLabOverrideRole::Wire => "wire",
            ArtLabOverrideRole::ObjectiveMarker => "objective marker",
            ArtLabOverrideRole::SpawnMarker => "spawn marker",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            ArtLabOverrideRole::PathDirtSurface => "path_dirt_surface",
            ArtLabOverrideRole::TrenchRecessedTerrain => "trench_recessed_terrain",
            ArtLabOverrideRole::BermRaisedTerrain => "berm_raised_terrain",
            ArtLabOverrideRole::Tree => "tree",
            ArtLabOverrideRole::Log => "log",
            ArtLabOverrideRole::Rock => "rock",
            ArtLabOverrideRole::Wall => "wall",
            ArtLabOverrideRole::Stakes => "stakes",
            ArtLabOverrideRole::Wire => "wire",
            ArtLabOverrideRole::ObjectiveMarker => "objective_marker",
            ArtLabOverrideRole::SpawnMarker => "spawn_marker",
        }
    }

    pub fn suggested_family(self) -> ArtSpriteFamily {
        match self {
            ArtLabOverrideRole::PathDirtSurface => ArtSpriteFamily::Path,
            ArtLabOverrideRole::TrenchRecessedTerrain => ArtSpriteFamily::Trench,
            ArtLabOverrideRole::BermRaisedTerrain => ArtSpriteFamily::Berm,
            ArtLabOverrideRole::Tree => ArtSpriteFamily::Tree,
            ArtLabOverrideRole::Log => ArtSpriteFamily::Log,
            ArtLabOverrideRole::Rock => ArtSpriteFamily::Rock,
            ArtLabOverrideRole::Wall => ArtSpriteFamily::Wall,
            ArtLabOverrideRole::Stakes => ArtSpriteFamily::Stakes,
            ArtLabOverrideRole::Wire => ArtSpriteFamily::Wire,
            ArtLabOverrideRole::ObjectiveMarker => ArtSpriteFamily::ObjectiveMarker,
            ArtLabOverrideRole::SpawnMarker => ArtSpriteFamily::SpawnMarker,
        }
    }
}

impl fmt::Display for ArtLabOverrideRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtLabOverrideAssignment {
    pub role: ArtLabOverrideRole,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtLabOverrideProfile {
    pub assignments: Vec<ArtLabOverrideAssignment>,
}

impl ArtLabOverrideProfile {
    pub fn set_assignment(
        &mut self,
        role: ArtLabOverrideRole,
        path: PathBuf,
        variant_id: Option<String>,
    ) {
        if let Some(existing) = self
            .assignments
            .iter_mut()
            .find(|assignment| assignment.role == role)
        {
            existing.path = path;
            existing.variant_id = variant_id;
        } else {
            self.assignments.push(ArtLabOverrideAssignment {
                role,
                path,
                variant_id,
            });
        }
    }

    pub fn assignment_path(&self, role: ArtLabOverrideRole) -> Option<&Path> {
        self.assignments
            .iter()
            .find(|assignment| assignment.role == role)
            .map(|assignment| assignment.path.as_path())
    }
}

impl ArtSpriteFamily {
    pub const ALL: [ArtSpriteFamily; 12] = [
        ArtSpriteFamily::TerrainBase,
        ArtSpriteFamily::Path,
        ArtSpriteFamily::Trench,
        ArtSpriteFamily::Berm,
        ArtSpriteFamily::Tree,
        ArtSpriteFamily::Log,
        ArtSpriteFamily::Rock,
        ArtSpriteFamily::Wall,
        ArtSpriteFamily::Stakes,
        ArtSpriteFamily::Wire,
        ArtSpriteFamily::ObjectiveMarker,
        ArtSpriteFamily::SpawnMarker,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ArtSpriteFamily::TerrainBase => "Terrain base",
            ArtSpriteFamily::Path => "Path",
            ArtSpriteFamily::Trench => "Trench",
            ArtSpriteFamily::Berm => "Berm",
            ArtSpriteFamily::Tree => "Tree",
            ArtSpriteFamily::Log => "Log",
            ArtSpriteFamily::Rock => "Rock",
            ArtSpriteFamily::Wall => "Wall",
            ArtSpriteFamily::Stakes => "Stakes",
            ArtSpriteFamily::Wire => "Wire",
            ArtSpriteFamily::ObjectiveMarker => "Objective marker",
            ArtSpriteFamily::SpawnMarker => "Spawn marker",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            ArtSpriteFamily::TerrainBase => "terrain_base",
            ArtSpriteFamily::Path => "path",
            ArtSpriteFamily::Trench => "trench",
            ArtSpriteFamily::Berm => "berm",
            ArtSpriteFamily::Tree => "tree",
            ArtSpriteFamily::Log => "log",
            ArtSpriteFamily::Rock => "rock",
            ArtSpriteFamily::Wall => "wall",
            ArtSpriteFamily::Stakes => "stakes",
            ArtSpriteFamily::Wire => "wire",
            ArtSpriteFamily::ObjectiveMarker => "objective_marker",
            ArtSpriteFamily::SpawnMarker => "spawn_marker",
        }
    }
}

impl fmt::Display for ArtSpriteFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for ArtSpriteFamily {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        for family in Self::ALL {
            if normalized == family.slug() || normalized == family.label().to_ascii_lowercase() {
                return Ok(family);
            }
        }
        Err(format!("unknown art sprite family '{value}'"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantRequest {
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub count: u32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub style: ArtStyleControls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

impl ArtVariantRequest {
    pub fn sanitized(&self) -> Self {
        Self {
            family: self.family,
            seed: self.seed,
            count: self.count.clamp(1, ART_VARIANT_MAX_COUNT),
            width: self.width.clamp(ART_VARIANT_MIN_SIZE, ART_VARIANT_MAX_SIZE),
            height: self
                .height
                .clamp(ART_VARIANT_MIN_SIZE, ART_VARIANT_MAX_SIZE),
            style: self.style.sanitized(),
            parent_id: self.parent_id.clone(),
        }
    }
}

impl Default for ArtVariantRequest {
    fn default() -> Self {
        Self {
            family: ArtSpriteFamily::Trench,
            seed: 99_418_113,
            count: 12,
            width: 32,
            height: 32,
            style: ArtStyleControls::default(),
            parent_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtStyleControls {
    pub roughness: f32,
    pub contrast: f32,
    pub edge_emphasis: f32,
    pub noise: f32,
    pub warmth: f32,
}

impl ArtStyleControls {
    pub fn sanitized(self) -> Self {
        Self {
            roughness: self.roughness.clamp(0.0, 1.0),
            contrast: self.contrast.clamp(0.0, 1.0),
            edge_emphasis: self.edge_emphasis.clamp(0.0, 1.0),
            noise: self.noise.clamp(0.0, 1.0),
            warmth: self.warmth.clamp(0.0, 1.0),
        }
    }
}

impl Default for ArtStyleControls {
    fn default() -> Self {
        Self {
            roughness: 0.5,
            contrast: 0.5,
            edge_emphasis: 0.5,
            noise: 0.5,
            warmth: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariant {
    pub id: String,
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub variant_index: u32,
    pub style: ArtStyleControls,
    pub parent_id: Option<String>,
    pub image: PixelImage,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantBatch {
    pub request: ArtVariantRequest,
    pub variants: Vec<ArtVariant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantMetadata {
    pub id: String,
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub variant_index: u32,
    pub width: u32,
    pub height: u32,
    pub style: ArtStyleControls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub notes: Vec<String>,
}

impl From<&ArtVariant> for ArtVariantMetadata {
    fn from(variant: &ArtVariant) -> Self {
        Self {
            id: variant.id.clone(),
            family: variant.family,
            seed: variant.seed,
            variant_index: variant.variant_index,
            width: variant.image.width,
            height: variant.image.height,
            style: variant.style,
            parent_id: variant.parent_id.clone(),
            notes: variant.notes.clone(),
        }
    }
}

pub fn generate_art_variants(request: &ArtVariantRequest) -> ArtVariantBatch {
    let request = request.sanitized();
    let variants = (0..request.count)
        .map(|variant_index| {
            let seed = derive_variant_seed(request.seed, request.family, variant_index);
            let mut rng = TinyRng::new(seed);
            let image = generate_family_image(&request, variant_index, &mut rng);
            let mut notes = vec![
                format!("family: {}", request.family.label()),
                format!("deterministic seed: {seed}"),
                format!(
                    "style: roughness {:.2}, contrast {:.2}, edge {:.2}, noise {:.2}, warmth {:.2}",
                    request.style.roughness,
                    request.style.contrast,
                    request.style.edge_emphasis,
                    request.style.noise,
                    request.style.warmth
                ),
                "Art Lab procedural sprite".to_string(),
            ];
            if let Some(parent_id) = &request.parent_id {
                notes.push(format!("mutated from {parent_id}"));
            }
            ArtVariant {
                id: format!(
                    "{}_seed_{}_variant_{:02}",
                    request.family.slug(),
                    request.seed,
                    variant_index
                ),
                family: request.family,
                seed: request.seed,
                variant_index,
                style: request.style,
                parent_id: request.parent_id.clone(),
                image,
                notes,
            }
        })
        .collect();
    ArtVariantBatch { request, variants }
}

pub fn derive_mutated_art_seed(parent: &ArtVariant) -> u64 {
    let mut hash = parent.seed ^ 0xd1b5_4a32_d192_ed03;
    for b in parent.family.slug().bytes().chain(parent.id.bytes()) {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x9e37_79b1_85eb_ca87);
        hash ^= hash >> 29;
    }
    hash ^ (parent.variant_index as u64 + 1).wrapping_mul(0x94d0_49bb_1331_11eb)
}

pub use export::{
    art_contact_sheet_path, art_override_preview_path, art_override_profile_path,
    art_variant_approved_paths, build_art_variant_contact_sheet, export_art_contact_sheet,
    export_art_lab_override_preview, export_art_variant_approved, export_art_variant_batch,
    load_art_lab_override_profile, render_art_lab_override_preview, save_art_lab_override_profile,
};

pub mod export {
    use super::*;

    pub fn art_variant_approved_paths(
        variant: &ArtVariant,
        root_dir: impl AsRef<Path>,
    ) -> (PathBuf, PathBuf) {
        let dir = root_dir
            .as_ref()
            .join("approved")
            .join(variant.family.slug());
        (
            dir.join(format!("{}.png", variant.id)),
            dir.join(format!("{}.json", variant.id)),
        )
    }

    pub fn art_contact_sheet_path(batch: &ArtVariantBatch, root_dir: impl AsRef<Path>) -> PathBuf {
        root_dir.as_ref().join("contact_sheets").join(format!(
            "{}_{}_{}.png",
            batch.request.family.slug(),
            batch.request.seed,
            batch.variants.len()
        ))
    }

    pub fn art_override_profile_path(root_dir: impl AsRef<Path>) -> PathBuf {
        root_dir
            .as_ref()
            .join("approved")
            .join("art_lab_overrides.json")
    }

    pub fn art_override_preview_path(root_dir: impl AsRef<Path>) -> PathBuf {
        root_dir
            .as_ref()
            .join("previews")
            .join("art_lab_preview.png")
    }

    pub fn export_art_variant_approved(
        variant: &ArtVariant,
        root_dir: impl AsRef<Path>,
    ) -> Result<(PathBuf, PathBuf)> {
        let (png_path, json_path) = art_variant_approved_paths(variant, root_dir);
        let dir = png_path
            .parent()
            .context("approved art variant path has no parent directory")?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        variant
            .image
            .save_png(&png_path)
            .with_context(|| format!("failed to save {}", png_path.display()))?;
        let metadata = ArtVariantMetadata::from(variant);
        std::fs::write(&json_path, serde_json::to_string_pretty(&metadata)?)
            .with_context(|| format!("failed to write {}", json_path.display()))?;
        Ok((png_path, json_path))
    }

    pub fn export_art_variant_batch(
        batch: &ArtVariantBatch,
        out_dir: impl AsRef<Path>,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let out_dir = out_dir.as_ref();
        std::fs::create_dir_all(out_dir)
            .with_context(|| format!("failed to create {}", out_dir.display()))?;
        let mut exported = Vec::new();
        for variant in &batch.variants {
            let png_path = out_dir.join(format!("{}.png", variant.id));
            let json_path = out_dir.join(format!("{}.json", variant.id));
            variant
                .image
                .save_png(&png_path)
                .with_context(|| format!("failed to save {}", png_path.display()))?;
            let metadata = ArtVariantMetadata::from(variant);
            std::fs::write(&json_path, serde_json::to_string_pretty(&metadata)?)
                .with_context(|| format!("failed to write {}", json_path.display()))?;
            exported.push((png_path, json_path));
        }
        Ok(exported)
    }

    pub fn build_art_variant_contact_sheet(batch: &ArtVariantBatch) -> PixelImage {
        let scale = 3;
        let gap = 4;
        let border = 1;
        let count = batch.variants.len() as u32;
        let columns = (count as f32).sqrt().ceil().max(1.0) as u32;
        let rows = count.div_ceil(columns).max(1);
        let cell_w = batch.request.width * scale + border * 2;
        let cell_h = batch.request.height * scale + border * 2;
        let width = columns * cell_w + (columns + 1) * gap;
        let height = rows * cell_h + (rows + 1) * gap;
        let mut sheet = PixelImage::new(width, height, Rgba8::opaque(18, 21, 19));
        for (i, variant) in batch.variants.iter().enumerate() {
            let col = i as u32 % columns;
            let row = i as u32 / columns;
            let x0 = gap + col * (cell_w + gap);
            let y0 = gap + row * (cell_h + gap);
            sheet.fill_rect(x0, y0, cell_w, cell_h, Rgba8::opaque(34, 38, 34));
            sheet.outline_rect(x0, y0, cell_w, cell_h, family_color(batch.request.family));
            blit_scaled_nearest(&mut sheet, &variant.image, x0 + border, y0 + border, scale);
        }
        sheet
    }

    pub fn export_art_contact_sheet(
        batch: &ArtVariantBatch,
        root_dir: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let path = art_contact_sheet_path(batch, root_dir);
        let dir = path
            .parent()
            .context("art contact sheet path has no parent directory")?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        build_art_variant_contact_sheet(batch)
            .save_png(&path)
            .with_context(|| format!("failed to save {}", path.display()))?;
        Ok(path)
    }

    pub fn save_art_lab_override_profile(
        profile: &ArtLabOverrideProfile,
        root_dir: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let path = art_override_profile_path(root_dir);
        let dir = path
            .parent()
            .context("Art Lab override profile path has no parent directory")?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        std::fs::write(&path, serde_json::to_string_pretty(profile)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    pub fn load_art_lab_override_profile(path: impl AsRef<Path>) -> Result<ArtLabOverrideProfile> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn render_art_lab_override_preview(profile: &ArtLabOverrideProfile) -> PixelImage {
        let mut image = PixelImage::new(320, 208, Rgba8::opaque(38, 52, 38));
        fill_art_preview_background(&mut image);

        let path = art_role_image(profile, ArtLabOverrideRole::PathDirtSurface);
        let trench = art_role_image(profile, ArtLabOverrideRole::TrenchRecessedTerrain);
        let berm = art_role_image(profile, ArtLabOverrideRole::BermRaisedTerrain);
        let tree = art_role_image(profile, ArtLabOverrideRole::Tree);
        let log = art_role_image(profile, ArtLabOverrideRole::Log);
        let rock = art_role_image(profile, ArtLabOverrideRole::Rock);
        let wall = art_role_image(profile, ArtLabOverrideRole::Wall);
        let stakes = art_role_image(profile, ArtLabOverrideRole::Stakes);
        let wire = art_role_image(profile, ArtLabOverrideRole::Wire);
        let objective = art_role_image(profile, ArtLabOverrideRole::ObjectiveMarker);
        let spawn = art_role_image(profile, ArtLabOverrideRole::SpawnMarker);

        for (x, y) in [
            (32, 112),
            (64, 104),
            (96, 96),
            (128, 88),
            (160, 80),
            (192, 72),
            (224, 64),
        ] {
            blit_scaled_nearest_alpha(&mut image, &path, x, y, 2);
        }
        for (x, y) in [(72, 132), (104, 128), (136, 124)] {
            blit_scaled_nearest_alpha(&mut image, &trench, x, y, 2);
        }
        for (x, y) in [(158, 112), (190, 108), (222, 104)] {
            blit_scaled_nearest_alpha(&mut image, &berm, x, y, 2);
        }

        blit_scaled_nearest_alpha(&mut image, &tree, 48, 46, 2);
        blit_scaled_nearest_alpha(&mut image, &tree, 78, 38, 2);
        blit_scaled_nearest_alpha(&mut image, &tree, 248, 56, 2);
        blit_scaled_nearest_alpha(&mut image, &log, 114, 56, 2);
        blit_scaled_nearest_alpha(&mut image, &rock, 236, 120, 2);
        blit_scaled_nearest_alpha(&mut image, &wall, 30, 146, 2);
        blit_scaled_nearest_alpha(&mut image, &stakes, 176, 146, 2);
        blit_scaled_nearest_alpha(&mut image, &wire, 210, 146, 2);
        blit_scaled_nearest_alpha(&mut image, &spawn, 34, 86, 2);
        blit_scaled_nearest_alpha(&mut image, &objective, 254, 82, 2);

        image.outline_rect(8, 8, 304, 192, Rgba8::opaque(77, 88, 70));
        image
    }

    pub fn export_art_lab_override_preview(
        profile: &ArtLabOverrideProfile,
        root_dir: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let path = art_override_preview_path(root_dir);
        let dir = path
            .parent()
            .context("Art Lab override preview path has no parent directory")?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
        render_art_lab_override_preview(profile)
            .save_png(&path)
            .with_context(|| format!("failed to save {}", path.display()))?;
        Ok(path)
    }
}

fn generate_family_image(
    request: &ArtVariantRequest,
    variant_index: u32,
    rng: &mut TinyRng,
) -> PixelImage {
    let mut image = PixelImage::transparent(request.width, request.height);
    match request.family {
        ArtSpriteFamily::TerrainBase => draw_terrain_base(&mut image, rng),
        ArtSpriteFamily::Path => draw_path(&mut image, variant_index, request.style, rng),
        ArtSpriteFamily::Trench => draw_trench(&mut image, variant_index, request.style, rng),
        ArtSpriteFamily::Berm => draw_berm(&mut image, variant_index, request.style, rng),
        ArtSpriteFamily::Tree => draw_tree(&mut image, rng),
        ArtSpriteFamily::Log => draw_log(&mut image, rng),
        ArtSpriteFamily::Rock => draw_rock(&mut image, rng),
        ArtSpriteFamily::Wall => draw_wall(&mut image, rng),
        ArtSpriteFamily::Stakes => draw_stakes(&mut image, rng),
        ArtSpriteFamily::Wire => draw_wire(&mut image, rng),
        ArtSpriteFamily::ObjectiveMarker => draw_marker(&mut image, true, rng),
        ArtSpriteFamily::SpawnMarker => draw_marker(&mut image, false, rng),
    }
    image
}

fn art_role_image(profile: &ArtLabOverrideProfile, role: ArtLabOverrideRole) -> PixelImage {
    if let Some(path) = profile.assignment_path(role) {
        if let Ok(image) = PixelImage::load_png(path) {
            return image;
        }
    }
    generate_art_variants(&ArtVariantRequest {
        family: role.suggested_family(),
        seed: 7_301 + role as u64 * 97,
        count: 1,
        width: 32,
        height: 32,
        style: ArtStyleControls::default(),
        parent_id: None,
    })
    .variants
    .remove(0)
    .image
}

fn fill_art_preview_background(image: &mut PixelImage) {
    for y in 0..image.height {
        for x in 0..image.width {
            let vignette = ((x as f32 - 160.0).abs() / 190.0 + (y as f32 - 104.0).abs() / 140.0)
                .clamp(0.0, 1.0);
            let base = Rgba8::opaque(73, 103, 55).blend(Rgba8::opaque(28, 39, 31), vignette * 0.55);
            image.set(x, y, base);
        }
    }
    for y in 0..image.height {
        for x in 0..image.width {
            if (x + y * 3) % 17 == 0 {
                image.blend_pixel(x, y, Rgba8::opaque(113, 138, 76), 0.12);
            }
            if (x * 5 + y) % 29 == 0 {
                image.blend_pixel(x, y, Rgba8::opaque(42, 70, 39), 0.10);
            }
        }
    }
}

fn draw_terrain_base(image: &mut PixelImage, rng: &mut TinyRng) {
    let base = Rgba8::opaque(91, 126, 61);
    fill(image, base);
    speckles(image, rng, 92, Rgba8::opaque(119, 151, 76), 0.10);
    speckles(image, rng, 53, Rgba8::opaque(65, 96, 48), 0.07);
}

fn draw_path(
    image: &mut PixelImage,
    variant_index: u32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let style = style.sanitized();
    fill(image, Rgba8::opaque(88, 125, 62));
    let axis = art_band_axis(variant_index);
    let dirt = art_style_color(Rgba8::opaque(166, 107, 63), style);
    let compact = art_style_color(Rgba8::opaque(128, 78, 47), style).darken(style.contrast * 0.10);
    let dust = art_style_color(Rgba8::opaque(198, 145, 85), style).lighten(style.warmth * 0.04);
    let grass_intrusion = Rgba8::opaque(80, 125, 64);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.48 + (rng.next_f32() - 0.5) * 0.14);
    let half_width =
        (max_lane * (0.16 + rng.next_f32() * (0.04 + style.roughness * 0.05))).max(5.0);
    let phase = rng.next_f32() * 9.0;
    let wave_amp = 0.8 + style.roughness * 4.4 + rng.next_f32() * 1.4;
    let edge_noise_scale = 1.2 + style.roughness * 4.8;
    let edge_blend = 2.3 + style.edge_emphasis * 4.4;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let edge_noise = (rng.hash_xy(x, y) - 0.5) * edge_noise_scale;
            let dist = (lane - centerline + edge_noise).abs();
            if dist < half_width {
                let t = dist / half_width;
                let noise = rng.hash_xy(x.wrapping_add(19), y.wrapping_add(31));
                let color = dirt
                    .blend(compact, (0.08 + style.contrast * 0.26) * (1.0 - t))
                    .blend(dust, noise * style.noise * 0.22);
                image.set(x, y, color);
            } else if dist < half_width + edge_blend
                && rng.hash_xy(x, y) > 0.40 - style.roughness * 0.34
            {
                let t = ((dist - half_width) / edge_blend).clamp(0.0, 1.0);
                image.set(x, y, dirt.blend(grass_intrusion, 0.38 + t * 0.42));
            }
        }
    }
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(35, 130, style.noise)),
        dust,
        0.08 + style.noise * 0.10,
    );
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(12, 54, style.noise)),
        compact,
        0.06 + style.contrast * 0.10,
    );
    draw_path_ruts(image, axis, center, half_width, phase, style, rng);
}

fn draw_trench(
    image: &mut PixelImage,
    variant_index: u32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let style = style.sanitized();
    fill(image, Rgba8::opaque(83, 120, 61));
    let axis = art_band_axis(variant_index);
    let floor_dark =
        art_style_color(Rgba8::opaque(38, 29, 24), style).darken(style.contrast * 0.16);
    let floor_warm = art_style_color(Rgba8::opaque(67, 45, 31), style);
    let wall_lit = art_style_color(Rgba8::opaque(126, 79, 44), style).lighten(style.warmth * 0.05);
    let wall_shadow =
        art_style_color(Rgba8::opaque(84, 53, 35), style).darken(style.contrast * 0.08);
    let lip = art_style_color(Rgba8::opaque(178, 117, 66), style);
    let spoil = art_style_color(Rgba8::opaque(138, 88, 51), style);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.50 + (rng.next_f32() - 0.5) * 0.10);
    let half = (max_lane * (0.17 + rng.next_f32() * (0.03 + style.edge_emphasis * 0.05))).max(5.5);
    let floor_half = half * (0.42 + rng.next_f32() * 0.10);
    let phase = rng.next_f32() * 11.0;
    let wave_amp = 0.7 + style.roughness * 3.4 + rng.next_f32() * 1.1;
    let edge_noise_scale = 1.2 + style.roughness * 4.4;
    let lip_width = 2.2 + style.edge_emphasis * 3.6;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let signed = lane - centerline + (rng.hash_xy(x, y) - 0.5) * edge_noise_scale;
            let dist = signed.abs();
            if dist < floor_half {
                let center_t = 1.0 - (dist / floor_half).clamp(0.0, 1.0);
                let grain = rng.hash_xy(x.wrapping_add(71), y.wrapping_add(13));
                image.set(
                    x,
                    y,
                    floor_warm
                        .blend(floor_dark, 0.56 * center_t)
                        .blend(Rgba8::opaque(83, 56, 38), grain * 0.16),
                );
            } else if dist < half {
                let wall_t = ((dist - floor_half) / (half - floor_half)).clamp(0.0, 1.0);
                let side_light = if signed < 0.0 {
                    0.12 + style.contrast * 0.20
                } else {
                    0.0
                };
                image.set(
                    x,
                    y,
                    wall_shadow.blend(wall_lit, side_light + wall_t * 0.34),
                );
            } else if dist < half + lip_width {
                let edge_t = ((dist - half) / lip_width).clamp(0.0, 1.0);
                let dirt = lip.blend(spoil, rng.hash_xy(x, y) * 0.18);
                image.set(x, y, dirt.blend(Rgba8::opaque(91, 124, 65), edge_t * 0.36));
            } else if dist < half + lip_width + 2.8
                && rng.hash_xy(x, y) > 0.76 - style.roughness * 0.30
            {
                image.blend_pixel(x, y, spoil, 0.18 + style.edge_emphasis * 0.24);
            }
        }
    }
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(22, 78, style.noise)),
        Rgba8::opaque(204, 146, 83),
        0.06 + style.noise * 0.10,
    );
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(15, 62, style.noise)),
        Rgba8::opaque(27, 23, 22),
        0.05 + style.contrast * 0.10,
    );
    draw_trench_cross_details(image, axis, center, floor_half, phase, style, rng);
}

fn draw_berm(
    image: &mut PixelImage,
    variant_index: u32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let style = style.sanitized();
    fill(image, Rgba8::opaque(82, 119, 61));
    let axis = art_band_axis(variant_index);
    let top = art_style_color(Rgba8::opaque(149, 101, 56), style);
    let crest = art_style_color(Rgba8::opaque(187, 133, 75), style).lighten(style.warmth * 0.04);
    let face = art_style_color(Rgba8::opaque(101, 65, 40), style).darken(style.contrast * 0.05);
    let base_shadow = Rgba8::opaque(49, 47, 34).darken(style.contrast * 0.12);
    let grass = Rgba8::opaque(82, 122, 64);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.50 + (rng.next_f32() - 0.5) * 0.12);
    let half = (max_lane * (0.15 + rng.next_f32() * (0.04 + style.edge_emphasis * 0.05))).max(5.0);
    let crest_half = half * (0.34 + rng.next_f32() * 0.12);
    let phase = rng.next_f32() * 13.0;
    let wave_amp = 0.8 + style.roughness * 4.0 + rng.next_f32() * 1.2;
    let edge_noise_scale = 1.2 + style.roughness * 4.2;
    let transition_width = 2.0 + style.edge_emphasis * 3.4;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let signed = lane - centerline + (rng.hash_xy(x, y) - 0.5) * edge_noise_scale;
            let dist = signed.abs();
            if dist < crest_half {
                let crown = 1.0 - (dist / crest_half).clamp(0.0, 1.0);
                image.set(
                    x,
                    y,
                    top.blend(crest, 0.22 + crown * 0.28)
                        .blend(grass, rng.hash_xy(x, y) * 0.12),
                );
            } else if dist < half {
                let face_t = ((dist - crest_half) / (half - crest_half)).clamp(0.0, 1.0);
                let color = face.blend(top, (1.0 - face_t) * 0.22);
                image.set(x, y, color);
            } else if dist < half + transition_width
                && rng.hash_xy(x, y) > 0.42 - style.roughness * 0.32
            {
                let t = ((dist - half) / transition_width).clamp(0.0, 1.0);
                image.set(x, y, top.blend(grass, 0.35 + t * 0.42));
            } else if signed > 0.0 && dist < half + transition_width + 1.8 {
                image.blend_pixel(x, y, base_shadow, 0.08 + rng.hash_xy(x, y) * 0.06);
            }
        }
    }
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(18, 82, style.noise)),
        crest,
        0.05 + style.noise * 0.10,
    );
    speckles(
        image,
        rng,
        scaled_count(image, scaled_style_count(14, 58, style.noise)),
        Rgba8::opaque(67, 47, 34),
        0.04 + style.contrast * 0.09,
    );
    draw_mound_strata(image, axis, center, (crest_half, half), phase, style, rng);
}

#[derive(Clone, Copy, Debug)]
enum ArtBandAxis {
    Horizontal,
    Vertical,
    DiagonalDown,
    DiagonalUp,
}

fn art_band_axis(variant_index: u32) -> ArtBandAxis {
    match variant_index % 6 {
        1 => ArtBandAxis::Vertical,
        2 => ArtBandAxis::DiagonalDown,
        3 => ArtBandAxis::Horizontal,
        4 => ArtBandAxis::DiagonalUp,
        _ => ArtBandAxis::Horizontal,
    }
}

fn art_band_max_lane(axis: ArtBandAxis, image: &PixelImage) -> f32 {
    match axis {
        ArtBandAxis::Horizontal | ArtBandAxis::DiagonalDown | ArtBandAxis::DiagonalUp => {
            image.height as f32
        }
        ArtBandAxis::Vertical => image.width as f32,
    }
}

fn art_band_coords(axis: ArtBandAxis, x: u32, y: u32, image: &PixelImage) -> (f32, f32) {
    let xf = x as f32;
    let yf = y as f32;
    let w = image.width.max(1) as f32;
    let h = image.height.max(1) as f32;
    match axis {
        ArtBandAxis::Horizontal => (yf, xf),
        ArtBandAxis::Vertical => (xf, yf),
        ArtBandAxis::DiagonalDown => (yf - xf * (h / w) + h * 0.50, xf),
        ArtBandAxis::DiagonalUp => (yf + xf * (h / w) - h * 0.50, xf),
    }
}

fn art_band_wave(along: f32, phase: f32, amplitude: f32) -> f32 {
    (along * 0.34 + phase).sin() * amplitude + (along * 0.17 + phase * 1.7).sin() * amplitude * 0.42
}

fn scaled_count(image: &PixelImage, base_count: u32) -> u32 {
    let area = image.width.max(1) * image.height.max(1);
    let scale = area as f32 / (32.0 * 32.0);
    (base_count as f32 * scale).round().max(1.0) as u32
}

fn scaled_style_count(min: u32, max: u32, value: f32) -> u32 {
    let t = value.clamp(0.0, 1.0);
    (min as f32 + (max as f32 - min as f32) * t).round() as u32
}

fn art_style_color(color: Rgba8, style: ArtStyleControls) -> Rgba8 {
    let warm = Rgba8::opaque(214, 124, 58);
    let cooled = Rgba8::opaque(92, 121, 91);
    let warmed = color
        .blend(cooled, (1.0 - style.warmth) * 0.08)
        .blend(warm, style.warmth * 0.12);
    if style.contrast >= 0.5 {
        let t = (style.contrast - 0.5) * 2.0;
        if warmed.luma() > 120 {
            warmed.lighten(t * 0.08)
        } else {
            warmed.darken(t * 0.10)
        }
    } else {
        let t = (0.5 - style.contrast) * 2.0;
        warmed.blend(Rgba8::opaque(128, 112, 82), t * 0.12)
    }
}

fn draw_path_ruts(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    half_width: f32,
    phase: f32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let color = art_style_color(Rgba8::opaque(101, 67, 43), style)
        .darken(style.contrast * 0.08)
        .with_alpha(clamp_u8(150.0 + style.contrast * 90.0));
    let marks = scaled_count(image, scaled_style_count(2, 12, style.edge_emphasis));
    for _ in 0..marks {
        let along = rng.next_f32() * image.width.max(image.height) as f32;
        let side = if rng.next_f32() > 0.5 { 1.0 } else { -1.0 };
        let lane = center + art_band_wave(along, phase, 1.0) + side * half_width * 0.34;
        let (x, y) = art_band_point(axis, along, lane, image);
        match axis {
            ArtBandAxis::Horizontal => {
                image.draw_line(x - 4, y, x + 5, y + rng.range_i32(-1, 2), color)
            }
            ArtBandAxis::Vertical => {
                image.draw_line(x, y - 4, x + rng.range_i32(-1, 2), y + 5, color)
            }
            ArtBandAxis::DiagonalDown => image.draw_line(x - 3, y - 3, x + 4, y + 4, color),
            ArtBandAxis::DiagonalUp => image.draw_line(x - 3, y + 3, x + 4, y - 4, color),
        }
    }
}

fn draw_trench_cross_details(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    floor_half: f32,
    phase: f32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let detail = art_style_color(Rgba8::opaque(104, 66, 39), style)
        .darken(style.contrast * 0.08)
        .with_alpha(clamp_u8(135.0 + style.edge_emphasis * 96.0));
    let marks = scaled_count(image, scaled_style_count(1, 7, style.edge_emphasis));
    for _ in 0..marks {
        let along = rng.next_f32() * image.width.max(image.height) as f32;
        let lane = center + art_band_wave(along, phase, 1.0);
        let (x, y) = art_band_point(axis, along, lane, image);
        let half = floor_half.max(3.0) as i32;
        match axis {
            ArtBandAxis::Horizontal => image.draw_line(x, y - half / 2, x, y + half / 2, detail),
            ArtBandAxis::Vertical => image.draw_line(x - half / 2, y, x + half / 2, y, detail),
            ArtBandAxis::DiagonalDown => image.draw_line(x - 2, y + 2, x + 2, y - 2, detail),
            ArtBandAxis::DiagonalUp => image.draw_line(x - 2, y - 2, x + 2, y + 2, detail),
        }
    }
}

fn draw_mound_strata(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    mound_widths: (f32, f32),
    phase: f32,
    style: ArtStyleControls,
    rng: &mut TinyRng,
) {
    let color = art_style_color(Rgba8::opaque(78, 52, 36), style)
        .darken(style.contrast * 0.08)
        .with_alpha(clamp_u8(110.0 + style.edge_emphasis * 95.0));
    let lines = 1 + (style.edge_emphasis * 3.0).round() as u32 + (rng.next_u32() % 2);
    for i in 0..lines {
        let (crest_half, half) = mound_widths;
        let lane_offset = crest_half + (half - crest_half) * (i as f32 + 0.45) / lines as f32;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let mut prev = None;
        for step in 0..image.width.max(image.height) {
            let along = step as f32;
            let lane = center + art_band_wave(along, phase, 0.9) + side * lane_offset;
            let point = art_band_point(axis, along, lane, image);
            if let Some((px, py)) = prev {
                if rng.hash_xy(step, i) > 0.32 {
                    image.draw_line(px, py, point.0, point.1, color);
                }
            }
            prev = Some(point);
        }
    }
}

fn art_band_point(axis: ArtBandAxis, along: f32, lane: f32, image: &PixelImage) -> (i32, i32) {
    let w = image.width.max(1) as f32;
    let h = image.height.max(1) as f32;
    match axis {
        ArtBandAxis::Horizontal => (along.round() as i32, lane.round() as i32),
        ArtBandAxis::Vertical => (lane.round() as i32, along.round() as i32),
        ArtBandAxis::DiagonalDown => {
            let x = along;
            let y = lane + x * (h / w) - h * 0.50;
            (x.round() as i32, y.round() as i32)
        }
        ArtBandAxis::DiagonalUp => {
            let x = along;
            let y = lane - x * (h / w) + h * 0.50;
            (x.round() as i32, y.round() as i32)
        }
    }
}

fn draw_tree(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.58);
    let trunk = Rgba8::opaque(99, 63, 35);
    let dark = Rgba8::opaque(39, 83, 44);
    let mid = Rgba8::opaque(54, 119, 55);
    let light = Rgba8::opaque(82, 146, 66);
    let cx = image.width as i32 / 2 + rng.range_i32(-2, 3);
    let cy = image.height as i32 / 2 + 5;
    rect_i32(image, cx - 2, cy, 4, 10, trunk);
    for layer in 0..3 {
        let radius = 10 - layer * 2;
        let y = cy - 7 - layer * 6;
        ellipse(image, cx, y, radius, 6, dark);
        ellipse(image, cx - 2, y - 1, radius - 2, 4, mid);
        if layer == 0 {
            ellipse(image, cx - 4, y - 3, 4, 2, light);
        }
    }
}

fn draw_log(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.45);
    let y = image.height as i32 / 2 + rng.range_i32(1, 4);
    let x0 = image.width as i32 / 2 - 12;
    for i in 0..25 {
        let x = x0 + i;
        rect_i32(image, x, y - 2 + i / 10, 1, 6, Rgba8::opaque(126, 72, 37));
        if i % 6 == 0 {
            rect_i32(image, x, y - 3 + i / 10, 1, 7, Rgba8::opaque(78, 48, 31));
        }
    }
    ellipse(image, x0, y, 3, 4, Rgba8::opaque(88, 54, 32));
    ellipse(image, x0 + 24, y + 2, 3, 4, Rgba8::opaque(169, 103, 55));
}

fn draw_rock(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.40);
    let cx = image.width as i32 / 2 + rng.range_i32(-2, 2);
    let cy = image.height as i32 / 2 + rng.range_i32(1, 4);
    ellipse(image, cx, cy, 10, 7, Rgba8::opaque(99, 104, 96));
    ellipse(image, cx - 2, cy - 2, 7, 4, Rgba8::opaque(139, 143, 131));
    image.draw_line(cx - 8, cy + 1, cx + 2, cy + 5, Rgba8::opaque(70, 74, 70));
}

fn draw_wall(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.40);
    let y = image.height / 2;
    for i in 0..4 {
        let x = 5 + i * 6;
        let color = if i % 2 == 0 {
            Rgba8::opaque(119, 117, 105)
        } else {
            Rgba8::opaque(92, 92, 84)
        };
        image.fill_rect(x, y - 5 + (rng.next_u32() % 3), 7, 10, color);
        image.outline_rect(x, y - 5, 7, 10, Rgba8::opaque(53, 54, 51));
    }
}

fn draw_stakes(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.34);
    for i in 0..5 {
        let x = 8 + i * 4 + rng.range_i32(-1, 2);
        let y = 19 + rng.range_i32(-2, 2);
        image.draw_line(x, y, x, y - 12, Rgba8::opaque(117, 73, 38));
        image.draw_line(x - 2, y - 7, x + 2, y - 9, Rgba8::opaque(184, 126, 63));
    }
}

fn draw_wire(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.20);
    for strand in 0..3 {
        let y = 13 + strand * 4;
        for x in 4..image.width.saturating_sub(4) {
            let wobble = if (x + strand) % 4 < 2 { 1 } else { -1 };
            image.set_i32(x as i32, y as i32 + wobble, Rgba8::opaque(143, 144, 128));
            if x % 7 == (rng.next_u32() % 7) {
                image.set_i32(x as i32, y as i32 - 2, Rgba8::opaque(219, 208, 126));
            }
        }
    }
}

fn draw_marker(image: &mut PixelImage, objective: bool, _rng: &mut TinyRng) {
    draw_shadow(image, 0.32);
    let pole = if objective {
        Rgba8::opaque(222, 194, 91)
    } else {
        Rgba8::opaque(92, 143, 207)
    };
    let flag = if objective {
        Rgba8::opaque(210, 63, 50)
    } else {
        Rgba8::opaque(233, 186, 73)
    };
    let cx = image.width as i32 / 2;
    let base_y = image.height as i32 - 9;
    rect_i32(image, cx - 7, base_y, 14, 4, Rgba8::opaque(119, 91, 45));
    image.draw_line(cx, base_y, cx, base_y - 17, pole);
    rect_i32(image, cx + 1, base_y - 17, 10, 7, flag);
    rect_i32(image, cx + 1, base_y - 10, 6, 3, flag.darken(0.20));
}

fn fill(image: &mut PixelImage, color: Rgba8) {
    image.fill_rect(0, 0, image.width, image.height, color);
}

fn draw_shadow(image: &mut PixelImage, alpha: f32) {
    let cx = image.width as i32 / 2;
    let cy = image.height as i32 - 8;
    ellipse(
        image,
        cx,
        cy,
        (image.width as i32 / 3).max(4),
        4,
        Rgba8::BLACK.with_alpha(clamp_u8(alpha * 150.0)),
    );
}

fn speckles(image: &mut PixelImage, rng: &mut TinyRng, count: u32, color: Rgba8, alpha: f32) {
    for _ in 0..count {
        let x = rng.next_u32() % image.width.max(1);
        let y = rng.next_u32() % image.height.max(1);
        image.blend_pixel(x, y, color, alpha);
    }
}

fn rect_i32(image: &mut PixelImage, x: i32, y: i32, width: i32, height: i32, color: Rgba8) {
    for yy in y..y + height {
        for xx in x..x + width {
            image.set_i32(xx, yy, color);
        }
    }
}

fn ellipse(image: &mut PixelImage, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba8) {
    let rx = rx.max(1);
    let ry = ry.max(1);
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let dx = (x - cx) as f32 / rx as f32;
            let dy = (y - cy) as f32 / ry as f32;
            if dx * dx + dy * dy <= 1.0 {
                if color.a == 255 {
                    image.set_i32(x, y, color);
                } else if image.in_bounds(x, y) {
                    let current = image.get(x as u32, y as u32);
                    image.set_i32(x, y, current.blend(color, color.a as f32 / 255.0));
                }
            }
        }
    }
}

fn blit_scaled_nearest(target: &mut PixelImage, source: &PixelImage, x0: u32, y0: u32, scale: u32) {
    for y in 0..source.height {
        for x in 0..source.width {
            let color = source.get(x, y);
            for sy in 0..scale {
                for sx in 0..scale {
                    target.set(x0 + x * scale + sx, y0 + y * scale + sy, color);
                }
            }
        }
    }
}

fn blit_scaled_nearest_alpha(
    target: &mut PixelImage,
    source: &PixelImage,
    x0: u32,
    y0: u32,
    scale: u32,
) {
    for y in 0..source.height {
        for x in 0..source.width {
            let color = source.get(x, y);
            if color.a == 0 {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    let tx = x0 + x * scale + sx;
                    let ty = y0 + y * scale + sy;
                    if tx >= target.width || ty >= target.height {
                        continue;
                    }
                    if color.a == 255 {
                        target.set(tx, ty, color);
                    } else {
                        target.blend_pixel(tx, ty, color, color.a as f32 / 255.0);
                    }
                }
            }
        }
    }
}

fn family_color(family: ArtSpriteFamily) -> Rgba8 {
    match family {
        ArtSpriteFamily::TerrainBase => Rgba8::opaque(104, 151, 82),
        ArtSpriteFamily::Path => Rgba8::opaque(190, 134, 80),
        ArtSpriteFamily::Trench => Rgba8::opaque(78, 128, 164),
        ArtSpriteFamily::Berm => Rgba8::opaque(190, 139, 76),
        ArtSpriteFamily::Tree => Rgba8::opaque(82, 162, 86),
        ArtSpriteFamily::Log => Rgba8::opaque(175, 101, 52),
        ArtSpriteFamily::Rock | ArtSpriteFamily::Wall => Rgba8::opaque(156, 156, 144),
        ArtSpriteFamily::Stakes | ArtSpriteFamily::Wire => Rgba8::opaque(217, 181, 91),
        ArtSpriteFamily::ObjectiveMarker => Rgba8::opaque(218, 80, 62),
        ArtSpriteFamily::SpawnMarker => Rgba8::opaque(90, 146, 220),
    }
}

fn derive_variant_seed(seed: u64, family: ArtSpriteFamily, variant_index: u32) -> u64 {
    let mut hash = seed ^ 0x9e37_79b9_7f4a_7c15;
    for b in family.slug().bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash ^ (variant_index as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
}

#[derive(Clone, Debug)]
struct TinyRng {
    state: u64,
}

impl TinyRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xa076_1d64_78bd_642f,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(0x5851_f42d_4c95_7f2d)
            .wrapping_add(0x1405_7b7e_f767_814f);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_u32() % (max - min) as u32) as i32
    }

    fn hash_xy(&self, x: u32, y: u32) -> f32 {
        let mut h = self.state ^ ((x as u64) << 32) ^ y as u64;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        ((h >> 40) as f32) / ((1_u64 << 24) as f32)
    }
}

pub fn parse_art_variant_cli(family: &str, seed: &str, count: &str) -> Result<ArtVariantRequest> {
    let family = family
        .parse::<ArtSpriteFamily>()
        .map_err(|err| anyhow::anyhow!(err))?;
    let seed = seed.parse::<u64>().context("invalid art variant seed")?;
    let count = count.parse::<u32>().context("invalid art variant count")?;
    Ok(ArtVariantRequest {
        family,
        seed,
        count,
        width: 32,
        height: 32,
        style: ArtStyleControls::default(),
        parent_id: None,
    }
    .sanitized())
}

pub fn ensure_art_batch_not_empty(batch: &ArtVariantBatch) -> Result<()> {
    if batch.variants.is_empty() {
        bail!("art variant batch is empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_variants_are_deterministic() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Trench,
            seed: 123,
            count: 4,
            width: 32,
            height: 32,
            style: ArtStyleControls::default(),
            parent_id: None,
        };
        let a = generate_art_variants(&request);
        let b = generate_art_variants(&request);
        assert_eq!(
            a.variants[2].image.to_rgba_bytes(),
            b.variants[2].image.to_rgba_bytes()
        );
        assert_eq!(a.variants[2].id, "trench_seed_123_variant_02");
    }

    #[test]
    fn art_variant_count_is_clamped() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Path,
            seed: 55,
            count: 128,
            width: 8,
            height: 512,
            style: ArtStyleControls {
                roughness: -1.0,
                contrast: 2.0,
                edge_emphasis: 0.5,
                noise: 0.5,
                warmth: 0.5,
            },
            parent_id: None,
        };
        let batch = generate_art_variants(&request);
        assert_eq!(batch.variants.len(), ART_VARIANT_MAX_COUNT as usize);
        assert_eq!(batch.request.width, ART_VARIANT_MIN_SIZE);
        assert_eq!(batch.request.height, ART_VARIANT_MAX_SIZE);
        assert_eq!(batch.request.style.roughness, 0.0);
        assert_eq!(batch.request.style.contrast, 1.0);
    }

    #[test]
    fn art_variant_metadata_serializes_and_contact_sheet_has_size() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Berm,
            seed: 77,
            count: 3,
            width: 32,
            height: 32,
            style: ArtStyleControls {
                roughness: 0.7,
                contrast: 0.6,
                edge_emphasis: 0.8,
                noise: 0.4,
                warmth: 0.9,
            },
            parent_id: Some("parent_sprite".to_string()),
        };
        let batch = generate_art_variants(&request);
        let metadata = ArtVariantMetadata::from(&batch.variants[0]);
        let json = serde_json::to_string(&metadata).expect("metadata should serialize");
        assert!(json.contains("berm_seed_77_variant_00"));
        assert!(json.contains("\"warmth\":0.9"));
        assert!(json.contains("parent_sprite"));

        let sheet = build_art_variant_contact_sheet(&batch);
        assert!(sheet.width > batch.request.width);
        assert!(sheet.height > batch.request.height);
    }

    #[test]
    fn terrain_art_families_produce_distinct_variants() {
        for family in [
            ArtSpriteFamily::Path,
            ArtSpriteFamily::Trench,
            ArtSpriteFamily::Berm,
        ] {
            let request = ArtVariantRequest {
                family,
                seed: 99_418_113,
                count: 2,
                width: 32,
                height: 32,
                style: ArtStyleControls::default(),
                parent_id: None,
            };
            let batch = generate_art_variants(&request);
            assert_ne!(
                batch.variants[0].image.to_rgba_bytes(),
                batch.variants[1].image.to_rgba_bytes(),
                "{family:?} variants should differ"
            );
        }
    }

    #[test]
    fn terrain_style_controls_affect_output() {
        let muted = ArtVariantRequest {
            family: ArtSpriteFamily::Trench,
            seed: 44,
            count: 1,
            width: 32,
            height: 32,
            style: ArtStyleControls {
                roughness: 0.0,
                contrast: 0.0,
                edge_emphasis: 0.0,
                noise: 0.0,
                warmth: 0.0,
            },
            parent_id: None,
        };
        let sharp = ArtVariantRequest {
            style: ArtStyleControls {
                roughness: 1.0,
                contrast: 1.0,
                edge_emphasis: 1.0,
                noise: 1.0,
                warmth: 1.0,
            },
            ..muted.clone()
        };
        let muted_batch = generate_art_variants(&muted);
        let sharp_batch = generate_art_variants(&sharp);
        assert_ne!(
            muted_batch.variants[0].image.to_rgba_bytes(),
            sharp_batch.variants[0].image.to_rgba_bytes()
        );
        assert_eq!(sharp_batch.variants[0].style.warmth, 1.0);
    }

    #[test]
    fn mutated_seed_is_deterministic_and_records_parent() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Path,
            seed: 12,
            count: 1,
            width: 32,
            height: 32,
            style: ArtStyleControls::default(),
            parent_id: None,
        };
        let mut parent_batch = generate_art_variants(&request);
        let parent = parent_batch.variants.remove(0);
        let seed_a = derive_mutated_art_seed(&parent);
        let seed_b = derive_mutated_art_seed(&parent);
        assert_eq!(seed_a, seed_b);

        let mutated = ArtVariantRequest {
            seed: seed_a,
            parent_id: Some(parent.id.clone()),
            ..request
        };
        let batch = generate_art_variants(&mutated);
        assert_eq!(
            batch.variants[0].parent_id.as_deref(),
            Some(parent.id.as_str())
        );
        assert!(batch.variants[0]
            .notes
            .iter()
            .any(|note| note.contains("mutated from")));
    }

    #[test]
    fn art_lab_override_profile_replaces_roles_and_renders_preview() {
        let mut profile = ArtLabOverrideProfile::default();
        profile.set_assignment(
            ArtLabOverrideRole::PathDirtSurface,
            PathBuf::from("missing/path_a.png"),
            Some("path_a".to_string()),
        );
        profile.set_assignment(
            ArtLabOverrideRole::PathDirtSurface,
            PathBuf::from("missing/path_b.png"),
            Some("path_b".to_string()),
        );
        assert_eq!(profile.assignments.len(), 1);
        assert_eq!(
            profile
                .assignment_path(ArtLabOverrideRole::PathDirtSurface)
                .and_then(Path::file_name)
                .and_then(|name| name.to_str()),
            Some("path_b.png")
        );

        let preview = render_art_lab_override_preview(&profile);
        assert_eq!(preview.width, 320);
        assert_eq!(preview.height, 208);
    }

    #[test]
    fn art_lab_override_profile_round_trips() {
        let root =
            std::env::temp_dir().join(format!("groundlab_art_profile_test_{}", std::process::id()));
        let mut profile = ArtLabOverrideProfile::default();
        profile.set_assignment(
            ArtLabOverrideRole::Tree,
            PathBuf::from("exports/art_lab/approved/tree/test.png"),
            Some("tree_test".to_string()),
        );
        let path = save_art_lab_override_profile(&profile, &root).expect("profile should save");
        let loaded = load_art_lab_override_profile(&path).expect("profile should load");
        assert_eq!(loaded.assignments.len(), 1);
        assert_eq!(loaded.assignments[0].role, ArtLabOverrideRole::Tree);
        let _ = std::fs::remove_dir_all(root);
    }
}
