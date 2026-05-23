use serde::{Deserialize, Serialize};

use crate::spritegen::{GeneratedTerrainSprite, TerrainSpriteKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteValidationReport {
    pub sprite_count: usize,
    pub average_seam_score: f32,
    pub average_single_pixel_noise: f32,
    pub issues: Vec<TerrainSpriteValidationIssue>,
    pub sprites: Vec<TerrainSpriteValidationSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteValidationIssue {
    pub severity: TerrainSpriteValidationSeverity,
    pub sprite_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TerrainSpriteValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteValidationSummary {
    pub id: String,
    pub kind: TerrainSpriteKind,
    pub seam_score: f32,
    pub single_pixel_noise_count: u32,
    pub unique_colors: usize,
}

pub fn validate_terrain_sprites(
    sprites: &[GeneratedTerrainSprite],
) -> TerrainSpriteValidationReport {
    let mut issues = Vec::new();
    let mut summaries = Vec::new();
    let mut seam_total = 0.0;
    let mut noise_total = 0.0;

    for sprite in sprites {
        let summary = summarize(sprite);
        seam_total += summary.seam_score;
        noise_total += summary.single_pixel_noise_count as f32;
        if !sprite.kind.is_transition() && summary.seam_score > 42.0 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "surface tile has a visible opposing-edge seam risk".to_string(),
            });
        }
        if summary.single_pixel_noise_count > 6 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "sprite contains several isolated single-pixel color changes".to_string(),
            });
        }
        if summary.unique_colors > 10 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Info,
                sprite_id: Some(sprite.id.clone()),
                message: "sprite uses more colors than the cozy base palette".to_string(),
            });
        }
        summaries.push(summary);
    }

    for required in [
        TerrainSpriteKind::GrassTile,
        TerrainSpriteKind::DirtTile,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        TerrainSpriteKind::GrassToDirtEdgeWest,
    ] {
        if !sprites.iter().any(|sprite| sprite.kind == required) {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: None,
                message: format!("missing required sprite kind {}", required.id()),
            });
        }
    }

    let count = sprites.len().max(1) as f32;
    TerrainSpriteValidationReport {
        sprite_count: sprites.len(),
        average_seam_score: seam_total / count,
        average_single_pixel_noise: noise_total / count,
        issues,
        sprites: summaries,
    }
}

fn summarize(sprite: &GeneratedTerrainSprite) -> TerrainSpriteValidationSummary {
    let image = &sprite.image;
    let mut colors = Vec::new();
    for pixel in &image.pixels {
        colors.push(color_key(*pixel));
    }
    colors.sort_unstable();
    colors.dedup();

    TerrainSpriteValidationSummary {
        id: sprite.id.clone(),
        kind: sprite.kind,
        seam_score: seam_score(sprite),
        single_pixel_noise_count: single_pixel_noise_count(sprite),
        unique_colors: colors.len(),
    }
}

fn seam_score(sprite: &GeneratedTerrainSprite) -> f32 {
    let image = &sprite.image;
    let mut total = 0.0;
    let mut count = 0.0;
    for y in 0..image.height {
        total += image.get(0, y).rgb_distance(image.get(image.width - 1, y));
        count += 1.0;
    }
    for x in 0..image.width {
        total += image.get(x, 0).rgb_distance(image.get(x, image.height - 1));
        count += 1.0;
    }
    total / count
}

fn single_pixel_noise_count(sprite: &GeneratedTerrainSprite) -> u32 {
    let image = &sprite.image;
    let mut count = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            let center = image.get(x, y);
            let mut same = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + dx).rem_euclid(image.width as i32) as u32;
                    let ny = (y as i32 + dy).rem_euclid(image.height as i32) as u32;
                    if image.get(nx, ny) == center {
                        same += 1;
                    }
                }
            }
            if same == 0 {
                count += 1;
            }
        }
    }
    count
}

fn color_key(color: crate::Rgba8) -> u32 {
    ((color.r as u32) << 24) | ((color.g as u32) << 16) | ((color.b as u32) << 8) | color.a as u32
}
