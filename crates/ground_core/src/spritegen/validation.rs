use serde::{Deserialize, Serialize};

use crate::spritegen::{GeneratedTerrainSprite, TerrainSpriteKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteValidationReport {
    pub sprite_count: usize,
    pub path_mask_coverage: usize,
    pub missing_path_masks: Vec<u8>,
    pub average_seam_score: f32,
    pub average_single_pixel_noise: f32,
    pub average_motif_repetition_score: f32,
    pub average_diagonal_pattern_score: f32,
    pub variant_similarity_score: f32,
    pub edge_mask_continuity_score: f32,
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
    pub motif_repetition_score: f32,
    pub diagonal_pattern_score: f32,
    pub cluster_diversity_score: f32,
    pub edge_mask_continuity_score: Option<f32>,
    pub unique_colors: usize,
}

pub fn validate_terrain_sprites(
    sprites: &[GeneratedTerrainSprite],
) -> TerrainSpriteValidationReport {
    let mut issues = Vec::new();
    let mut summaries = Vec::new();
    let mut seam_total = 0.0;
    let mut noise_total = 0.0;
    let mut motif_total = 0.0;
    let mut diagonal_total = 0.0;
    let mut edge_total = 0.0;
    let mut edge_count = 0.0;

    for sprite in sprites {
        let summary = summarize(sprite);
        seam_total += summary.seam_score;
        noise_total += summary.single_pixel_noise_count as f32;
        motif_total += summary.motif_repetition_score;
        diagonal_total += summary.diagonal_pattern_score;
        if let Some(score) = summary.edge_mask_continuity_score {
            edge_total += score;
            edge_count += 1.0;
        }
        let surface = matches!(
            sprite.kind,
            TerrainSpriteKind::GrassTile | TerrainSpriteKind::DirtTile
        );
        if surface && summary.seam_score > 42.0 {
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
        if surface && summary.motif_repetition_score > 0.68 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "surface tile has a high motif repetition score".to_string(),
            });
        }
        if surface && summary.diagonal_pattern_score > 0.38 && detail_pixel_count(sprite) >= 12 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "surface tile has a visible diagonal-pattern risk".to_string(),
            });
        }
        if (sprite.kind.is_transition() || sprite.kind.is_path_mask())
            && summary
                .edge_mask_continuity_score
                .is_some_and(|score| score > 0.34)
        {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "transition edge has too many disconnected material changes".to_string(),
            });
        }
        summaries.push(summary);
    }

    for required in TerrainSpriteKind::ALL {
        if !sprites.iter().any(|sprite| sprite.kind == required) {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: None,
                message: format!("missing required sprite kind {}", required.id()),
            });
        }
    }
    let missing_path_masks = (0..16)
        .filter(|mask| {
            !sprites
                .iter()
                .any(|sprite| sprite.kind.path_mask() == Some(*mask))
        })
        .collect::<Vec<_>>();
    if !missing_path_masks.is_empty() {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: format!("missing path mask coverage for {:?}", missing_path_masks),
        });
    }

    let count = sprites.len().max(1) as f32;
    let variant_similarity_score = variant_similarity_score(sprites);
    if variant_similarity_score > 0.985 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "surface variants are very similar".to_string(),
        });
    }
    TerrainSpriteValidationReport {
        sprite_count: sprites.len(),
        path_mask_coverage: 16 - missing_path_masks.len(),
        missing_path_masks,
        average_seam_score: seam_total / count,
        average_single_pixel_noise: noise_total / count,
        average_motif_repetition_score: motif_total / count,
        average_diagonal_pattern_score: diagonal_total / count,
        variant_similarity_score,
        edge_mask_continuity_score: if edge_count > 0.0 {
            edge_total / edge_count
        } else {
            0.0
        },
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
        motif_repetition_score: motif_repetition_score(sprite),
        diagonal_pattern_score: diagonal_pattern_score(sprite),
        cluster_diversity_score: cluster_diversity_score(sprite),
        edge_mask_continuity_score: if sprite.kind.is_transition() || sprite.kind.is_path_mask() {
            Some(edge_mask_continuity_score(sprite))
        } else {
            None
        },
        unique_colors: colors.len(),
    }
}

fn motif_repetition_score(sprite: &GeneratedTerrainSprite) -> f32 {
    let image = &sprite.image;
    let base = dominant_color(sprite);
    let mut best = 0.0_f32;
    for (sx, sy) in [(1, 1), (2, 2), (3, 1), (1, 3), (4, 4)] {
        let mut both_detail = 0.0;
        let mut detail = 0.0;
        for y in 0..image.height {
            for x in 0..image.width {
                let a = image.get(x, y) != base;
                let b = image.get((x + sx) % image.width, (y + sy) % image.height) != base;
                if a {
                    detail += 1.0;
                    if b {
                        both_detail += 1.0;
                    }
                }
            }
        }
        if detail > 0.0 {
            best = best.max(both_detail / detail);
        }
    }
    best
}

fn diagonal_pattern_score(sprite: &GeneratedTerrainSprite) -> f32 {
    let image = &sprite.image;
    let base = dominant_color(sprite);
    let size = image.width.max(1) as usize;
    let mut bins = vec![0u32; size];
    let mut total = 0u32;
    for y in 0..image.height {
        for x in 0..image.width {
            if image.get(x, y) != base {
                let bin = (x as i32 - y as i32).rem_euclid(size as i32) as usize;
                bins[bin] += 1;
                total += 1;
            }
        }
    }
    if total == 0 {
        return 0.0;
    }
    bins.into_iter().max().unwrap_or(0) as f32 / total as f32
}

fn cluster_diversity_score(sprite: &GeneratedTerrainSprite) -> f32 {
    let image = &sprite.image;
    let base = dominant_color(sprite);
    let mut visited = vec![false; (image.width * image.height) as usize];
    let mut sizes = Vec::new();
    for y in 0..image.height {
        for x in 0..image.width {
            let idx = (y * image.width + x) as usize;
            if visited[idx] || image.get(x, y) == base {
                continue;
            }
            let size = flood_detail_cluster(image, base, x, y, &mut visited);
            sizes.push(size);
        }
    }
    if sizes.len() <= 1 {
        return sizes.len() as f32;
    }
    sizes.sort_unstable();
    sizes.dedup();
    (sizes.len() as f32 / 6.0).min(1.0)
}

fn edge_mask_continuity_score(sprite: &GeneratedTerrainSprite) -> f32 {
    let image = &sprite.image;
    let base = dominant_color(sprite);
    let mut transitions = 0;
    let mut lines = 0;
    for y in 0..image.height {
        let mut changes = 0;
        let mut previous = image.get(0, y) == base;
        for x in 1..image.width {
            let current = image.get(x, y) == base;
            if current != previous {
                changes += 1;
                previous = current;
            }
        }
        transitions += changes;
        lines += 1;
    }
    for x in 0..image.width {
        let mut changes = 0;
        let mut previous = image.get(x, 0) == base;
        for y in 1..image.height {
            let current = image.get(x, y) == base;
            if current != previous {
                changes += 1;
                previous = current;
            }
        }
        transitions += changes;
        lines += 1;
    }
    transitions as f32 / (lines as f32 * image.width.max(1) as f32)
}

fn variant_similarity_score(sprites: &[GeneratedTerrainSprite]) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for kind in [TerrainSpriteKind::GrassTile, TerrainSpriteKind::DirtTile] {
        let variants = sprites
            .iter()
            .filter(|sprite| sprite.kind == kind)
            .collect::<Vec<_>>();
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                total += image_similarity(variants[i], variants[j]);
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        0.0
    } else {
        total / count
    }
}

fn image_similarity(a: &GeneratedTerrainSprite, b: &GeneratedTerrainSprite) -> f32 {
    let width = a.image.width.min(b.image.width);
    let height = a.image.height.min(b.image.height);
    let mut same = 0.0;
    let mut total = 0.0;
    for y in 0..height {
        for x in 0..width {
            total += 1.0;
            if a.image.get(x, y) == b.image.get(x, y) {
                same += 1.0;
            }
        }
    }
    if total == 0.0 {
        0.0
    } else {
        same / total
    }
}

fn flood_detail_cluster(
    image: &crate::PixelImage,
    base: crate::Rgba8,
    x: u32,
    y: u32,
    visited: &mut [bool],
) -> u32 {
    let mut stack = vec![(x, y)];
    let mut count = 0;
    while let Some((x, y)) = stack.pop() {
        let idx = (y * image.width + x) as usize;
        if visited[idx] || image.get(x, y) == base {
            continue;
        }
        visited[idx] = true;
        count += 1;
        for (nx, ny) in neighbors(image, x, y) {
            let nidx = (ny * image.width + nx) as usize;
            if !visited[nidx] && image.get(nx, ny) != base {
                stack.push((nx, ny));
            }
        }
    }
    count
}

fn neighbors(image: &crate::PixelImage, x: u32, y: u32) -> [(u32, u32); 4] {
    [
        ((x + image.width - 1) % image.width, y),
        ((x + 1) % image.width, y),
        (x, (y + image.height - 1) % image.height),
        (x, (y + 1) % image.height),
    ]
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

fn detail_pixel_count(sprite: &GeneratedTerrainSprite) -> u32 {
    let base = dominant_color(sprite);
    sprite
        .image
        .pixels
        .iter()
        .filter(|pixel| **pixel != base)
        .count() as u32
}

fn dominant_color(sprite: &GeneratedTerrainSprite) -> crate::Rgba8 {
    let mut colors = sprite.image.pixels.clone();
    colors.sort_by_key(|color| color_key(*color));
    let mut best = colors[0];
    let mut best_count = 0;
    let mut current = colors[0];
    let mut current_count = 0;
    for color in colors {
        if color == current {
            current_count += 1;
        } else {
            if current_count > best_count {
                best = current;
                best_count = current_count;
            }
            current = color;
            current_count = 1;
        }
    }
    if current_count > best_count {
        best = current;
    }
    best
}

fn color_key(color: crate::Rgba8) -> u32 {
    ((color.r as u32) << 24) | ((color.g as u32) << 16) | ((color.b as u32) << 8) | color.a as u32
}
