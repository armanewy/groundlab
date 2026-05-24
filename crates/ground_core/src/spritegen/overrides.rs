use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pixel_image::PixelImage;
use crate::spritegen::{
    generate_terrain_sprites, GeneratedTerrainSprite, TerrainSpriteKind, TerrainSpriteRecipe,
    TerrainSpriteSource,
};

#[derive(Clone, Debug)]
pub struct EffectiveTerrainSprites {
    pub generated: Vec<GeneratedTerrainSprite>,
    pub effective: Vec<GeneratedTerrainSprite>,
    pub report: TerrainSpriteOverrideReport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteOverrideReport {
    pub override_dir: Option<String>,
    pub generated_count: usize,
    pub overridden_count: usize,
    pub invalid_count: usize,
    pub warning_count: usize,
    pub unused_override_files: Vec<String>,
    pub entries: Vec<TerrainSpriteOverrideEntry>,
}

impl TerrainSpriteOverrideReport {
    pub fn issue_count(&self) -> usize {
        self.invalid_count + self.warning_count
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteOverrideEntry {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub source: TerrainSpriteSource,
    pub status: TerrainSpriteOverrideStatus,
    pub file: Option<String>,
    pub expected_size: (u32, u32),
    pub actual_size: Option<(u32, u32)>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainSpriteOverrideStatus {
    Generated,
    MissingOverride,
    Overridden,
    InvalidSize,
    LoadError,
}

impl TerrainSpriteOverrideStatus {
    pub fn label(self) -> &'static str {
        match self {
            TerrainSpriteOverrideStatus::Generated => "Generated",
            TerrainSpriteOverrideStatus::MissingOverride => "Missing override",
            TerrainSpriteOverrideStatus::Overridden => "Overridden",
            TerrainSpriteOverrideStatus::InvalidSize => "Invalid size",
            TerrainSpriteOverrideStatus::LoadError => "Load error",
        }
    }
}

pub fn generate_effective_terrain_sprites(recipe: &TerrainSpriteRecipe) -> EffectiveTerrainSprites {
    let generated = generate_terrain_sprites(recipe);
    let (effective, report) = apply_sprite_overrides(&generated, recipe);
    EffectiveTerrainSprites {
        generated,
        effective,
        report,
    }
}

pub fn apply_sprite_overrides(
    generated: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> (Vec<GeneratedTerrainSprite>, TerrainSpriteOverrideReport) {
    let override_dir = override_dir(recipe);
    let mut effective = Vec::with_capacity(generated.len());
    let mut entries = Vec::with_capacity(generated.len());
    let mut overridden_count = 0;
    let mut invalid_count = 0;
    let mut warning_count = 0;
    let mut consumed = HashSet::new();

    for sprite in generated {
        let expected_size = (sprite.image.width, sprite.image.height);
        let Some(dir) = &override_dir else {
            effective.push(sprite.clone());
            entries.push(generated_entry(
                sprite,
                None,
                expected_size,
                TerrainSpriteOverrideStatus::Generated,
            ));
            continue;
        };
        let file = dir.join(format!("{}.png", sprite.id));
        if !file.exists() {
            effective.push(sprite.clone());
            entries.push(generated_entry(
                sprite,
                Some(&file),
                expected_size,
                TerrainSpriteOverrideStatus::MissingOverride,
            ));
            continue;
        }

        consumed.insert(normalize_path(&file));
        match PixelImage::load_png(&file) {
            Ok(image)
                if image.width == sprite.image.width && image.height == sprite.image.height =>
            {
                let mut replaced = sprite.clone();
                replaced.image = image;
                replaced.source = TerrainSpriteSource::Override;
                let warnings = override_warnings(sprite, &replaced);
                if !warnings.is_empty() {
                    warning_count += warnings.len();
                }
                overridden_count += 1;
                entries.push(TerrainSpriteOverrideEntry {
                    id: sprite.id.clone(),
                    kind: sprite.kind,
                    source: TerrainSpriteSource::Override,
                    status: TerrainSpriteOverrideStatus::Overridden,
                    file: Some(file.to_string_lossy().to_string()),
                    expected_size,
                    actual_size: Some((replaced.image.width, replaced.image.height)),
                    warnings,
                });
                effective.push(replaced);
            }
            Ok(image) => {
                invalid_count += 1;
                entries.push(TerrainSpriteOverrideEntry {
                    id: sprite.id.clone(),
                    kind: sprite.kind,
                    source: TerrainSpriteSource::Generated,
                    status: TerrainSpriteOverrideStatus::InvalidSize,
                    file: Some(file.to_string_lossy().to_string()),
                    expected_size,
                    actual_size: Some((image.width, image.height)),
                    warnings: vec!["override dimensions do not match generated sprite".to_string()],
                });
                effective.push(sprite.clone());
            }
            Err(err) => {
                invalid_count += 1;
                entries.push(TerrainSpriteOverrideEntry {
                    id: sprite.id.clone(),
                    kind: sprite.kind,
                    source: TerrainSpriteSource::Generated,
                    status: TerrainSpriteOverrideStatus::LoadError,
                    file: Some(file.to_string_lossy().to_string()),
                    expected_size,
                    actual_size: None,
                    warnings: vec![err.to_string()],
                });
                effective.push(sprite.clone());
            }
        }
    }

    let unused_override_files = override_dir
        .as_ref()
        .map(|dir| unused_png_files(dir, &consumed))
        .unwrap_or_default();
    warning_count += unused_override_files.len();

    (
        effective,
        TerrainSpriteOverrideReport {
            override_dir: override_dir.map(|path| path.to_string_lossy().to_string()),
            generated_count: generated.len(),
            overridden_count,
            invalid_count,
            warning_count,
            unused_override_files,
            entries,
        },
    )
}

pub fn promote_generated_sprites_to_overrides(
    profile_path: impl AsRef<Path>,
) -> Result<TerrainSpriteOverrideReport> {
    let recipe = TerrainSpriteRecipe::from_style_profile_path(profile_path)?;
    let generated = generate_terrain_sprites(&recipe);
    let dir = override_dir(&recipe).unwrap_or_else(|| PathBuf::from("overrides"));
    fs::create_dir_all(&dir)?;
    for sprite in &generated {
        sprite
            .image
            .save_png(dir.join(format!("{}.png", sprite.id)))?;
    }
    let (_effective, report) = apply_sprite_overrides(&generated, &recipe);
    Ok(report)
}

pub fn override_dir(recipe: &TerrainSpriteRecipe) -> Option<PathBuf> {
    let trimmed = recipe.override_dir.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn generated_entry(
    sprite: &GeneratedTerrainSprite,
    file: Option<&Path>,
    expected_size: (u32, u32),
    status: TerrainSpriteOverrideStatus,
) -> TerrainSpriteOverrideEntry {
    TerrainSpriteOverrideEntry {
        id: sprite.id.clone(),
        kind: sprite.kind,
        source: TerrainSpriteSource::Generated,
        status,
        file: file.map(|path| path.to_string_lossy().to_string()),
        expected_size,
        actual_size: None,
        warnings: Vec::new(),
    }
}

fn override_warnings(
    generated: &GeneratedTerrainSprite,
    effective: &GeneratedTerrainSprite,
) -> Vec<String> {
    let generated_alpha = alpha_coverage(&generated.image);
    let override_alpha = alpha_coverage(&effective.image);
    let mut warnings = Vec::new();
    if generated_alpha < 0.98 && override_alpha > 0.995 {
        warnings.push(
            "override is effectively opaque but generated sprite uses transparency".to_string(),
        );
    }
    if generated_alpha > 0.995 && override_alpha < 0.80 {
        warnings
            .push("override introduces substantial transparency into an opaque sprite".to_string());
    }
    warnings
}

fn alpha_coverage(image: &PixelImage) -> f32 {
    let total = image.pixels.len().max(1) as f32;
    image.pixels.iter().filter(|pixel| pixel.a > 8).count() as f32 / total
}

fn unused_png_files(dir: &Path, consumed: &HashSet<String>) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        })
        .filter(|path| !consumed.contains(&normalize_path(path)))
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}
