use serde::{Deserialize, Serialize};

use crate::spritegen::{GeneratedTerrainSprite, SpriteRole, TerrainSpriteKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainSpriteValidationReport {
    pub sprite_count: usize,
    pub path_mask_coverage: usize,
    pub missing_path_masks: Vec<u8>,
    pub path_neighbor_seam_score: f32,
    pub average_seam_score: f32,
    pub average_single_pixel_noise: f32,
    pub average_motif_repetition_score: f32,
    pub average_diagonal_pattern_score: f32,
    pub variant_similarity_score: f32,
    pub edge_mask_continuity_score: f32,
    pub trench: TrenchSpriteValidationSummary,
    pub issues: Vec<TerrainSpriteValidationIssue>,
    pub sprites: Vec<TerrainSpriteValidationSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrenchSpriteValidationSummary {
    pub trench_piece_coverage: usize,
    pub trench_role_coverage: usize,
    pub trench_floor_darkness_score: f32,
    pub trench_wall_floor_contrast: f32,
    pub trench_lip_contrast: f32,
    pub trench_shadow_continuity: f32,
    pub trench_cap_presence: bool,
    pub trench_oblique_anchor_validity: f32,
    pub trench_mask_coverage: usize,
    pub missing_trench_masks: Vec<u8>,
    pub trench_neighbor_seam_score: f32,
    pub trench_floor_continuity_score: f32,
    pub trench_wall_alignment_score: f32,
    pub trench_lip_continuity_score: f32,
    pub trench_cap_coverage: usize,
    pub trench_corner_coverage: usize,
    pub trench_junction_coverage: usize,
    pub worst_trench_neighbor_pairs: Vec<TrenchNeighborPairScore>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrenchNeighborPairScore {
    pub mask_a: u8,
    pub edge: String,
    pub mask_b: u8,
    pub score: f32,
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
    let mut aggregate_count = 0.0;

    for sprite in sprites {
        let summary = summarize(sprite);
        if !sprite.kind.is_trench() && !sprite.kind.is_trench_mask() {
            seam_total += summary.seam_score;
            noise_total += summary.single_pixel_noise_count as f32;
            motif_total += summary.motif_repetition_score;
            diagonal_total += summary.diagonal_pattern_score;
            aggregate_count += 1.0;
        }
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
        if !sprite.kind.is_trench()
            && !sprite.kind.is_trench_mask()
            && summary.single_pixel_noise_count > 6
        {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Warning,
                sprite_id: Some(sprite.id.clone()),
                message: "sprite contains several isolated single-pixel color changes".to_string(),
            });
        }
        if !sprite.kind.is_trench() && !sprite.kind.is_trench_mask() && summary.unique_colors > 10 {
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
    let missing_trench_masks = (0..16)
        .filter(|mask| {
            !sprites
                .iter()
                .any(|sprite| sprite.kind.trench_mask() == Some(*mask))
        })
        .collect::<Vec<_>>();
    if !missing_trench_masks.is_empty() {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: format!(
                "missing trench mask coverage for {:?}",
                missing_trench_masks
            ),
        });
    }

    let count = f32::max(aggregate_count, 1.0);
    let variant_similarity_score = variant_similarity_score(sprites);
    let path_neighbor_seam_score = path_neighbor_seam_score(sprites);
    if path_neighbor_seam_score > 0.32 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "path masks have visible neighbor seam discontinuity".to_string(),
        });
    }
    let trench_neighbor_score = trench_neighbor_seam_score(sprites);
    if trench_neighbor_score > 0.20 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "trench masks have visible neighbor seam discontinuity".to_string(),
        });
    }
    if variant_similarity_score > 0.985 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "surface variants are very similar".to_string(),
        });
    }
    let trench = validate_trench_sprites(sprites, &mut issues);
    TerrainSpriteValidationReport {
        sprite_count: sprites.len(),
        path_mask_coverage: 16 - missing_path_masks.len(),
        missing_path_masks,
        path_neighbor_seam_score,
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
        trench,
        issues,
        sprites: summaries,
    }
}

fn validate_trench_sprites(
    sprites: &[GeneratedTerrainSprite],
    issues: &mut Vec<TerrainSpriteValidationIssue>,
) -> TrenchSpriteValidationSummary {
    let required = [
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
    ];
    let required_roles = [
        SpriteRole::TopSurface,
        SpriteRole::FrontFace,
        SpriteRole::Lip,
        SpriteRole::CornerCap,
        SpriteRole::ContactShadow,
        SpriteRole::Decal,
    ];
    let trench_sprites = sprites
        .iter()
        .filter(|sprite| sprite.kind.is_trench())
        .collect::<Vec<_>>();
    let trench_piece_coverage = required
        .iter()
        .filter(|kind| trench_sprites.iter().any(|sprite| sprite.kind == **kind))
        .count();
    let trench_role_coverage = required_roles
        .iter()
        .filter(|role| {
            trench_sprites
                .iter()
                .any(|sprite| sprite.metadata.role == **role)
        })
        .count();

    for kind in required {
        if !trench_sprites.iter().any(|sprite| sprite.kind == kind) {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: None,
                message: format!("missing trench sprite {}", kind.id()),
            });
        }
    }
    for role in required_roles {
        if !trench_sprites
            .iter()
            .any(|sprite| sprite.metadata.role == role)
        {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: None,
                message: format!("missing trench sprite role {}", role.id()),
            });
        }
    }

    for sprite in &trench_sprites {
        let expected = expected_trench_role(sprite.kind);
        if sprite.metadata.role != expected {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: Some(sprite.id.clone()),
                message: format!(
                    "trench sprite has role {}, expected {}",
                    sprite.metadata.role.id(),
                    expected.id()
                ),
            });
        }
        if sprite.metadata.footprint_cells.0 == 0 || sprite.metadata.footprint_cells.1 == 0 {
            issues.push(TerrainSpriteValidationIssue {
                severity: TerrainSpriteValidationSeverity::Error,
                sprite_id: Some(sprite.id.clone()),
                message: "trench sprite has invalid zero-cell footprint".to_string(),
            });
        }
    }

    let floor = first_kind(sprites, TerrainSpriteKind::TrenchFloorTop);
    let wall = first_kind(sprites, TerrainSpriteKind::TrenchWallFront);
    let lip = first_kind(sprites, TerrainSpriteKind::TrenchLipFront);
    let shadow = first_kind(sprites, TerrainSpriteKind::TrenchContactShadow);
    let floor_luma = floor.map(average_luma).unwrap_or(0.0);
    let wall_luma = wall.map(average_luma).unwrap_or(0.0);
    let lip_luma = lip.map(average_luma).unwrap_or(0.0);
    let trench_floor_darkness_score = 1.0 - floor_luma / 255.0;
    let trench_wall_floor_contrast = ((wall_luma - floor_luma).abs() / 255.0).min(1.0);
    let trench_lip_contrast = ((lip_luma - floor_luma).abs() / 255.0).min(1.0);
    let trench_shadow_continuity = shadow.map(alpha_coverage).unwrap_or(0.0);
    let trench_cap_presence = first_kind(sprites, TerrainSpriteKind::TrenchEndCapLeft).is_some()
        && first_kind(sprites, TerrainSpriteKind::TrenchEndCapRight).is_some()
        && first_kind(sprites, TerrainSpriteKind::TrenchCornerInner).is_some()
        && first_kind(sprites, TerrainSpriteKind::TrenchCornerOuter).is_some();
    let valid_anchors = trench_sprites
        .iter()
        .filter(|sprite| {
            sprite.metadata.footprint_cells.0 > 0 && sprite.metadata.footprint_cells.1 > 0
        })
        .count() as f32;
    let trench_oblique_anchor_validity = if trench_sprites.is_empty() {
        0.0
    } else {
        valid_anchors / trench_sprites.len() as f32
    };
    let trench_mask_sprites = sprites
        .iter()
        .filter(|sprite| sprite.kind.is_trench_mask())
        .collect::<Vec<_>>();
    let missing_trench_masks = (0..16)
        .filter(|mask| {
            !trench_mask_sprites
                .iter()
                .any(|sprite| sprite.kind.trench_mask() == Some(*mask))
        })
        .collect::<Vec<_>>();
    let trench_mask_coverage = 16 - missing_trench_masks.len();
    let trench_neighbor_seam_score = trench_neighbor_seam_score(sprites);
    let trench_floor_continuity_score = trench_mask_band_continuity_score(sprites, 0.50);
    let trench_lip_continuity_score = trench_mask_band_continuity_score(sprites, 0.18);
    let trench_wall_alignment_score = trench_mask_band_continuity_score(sprites, 0.78);
    let worst_trench_neighbor_pairs = worst_trench_neighbor_pairs(sprites);
    let trench_cap_coverage = [0_u8, 1, 2, 4, 8]
        .into_iter()
        .filter(|mask| {
            trench_mask_sprites
                .iter()
                .any(|sprite| sprite.kind.trench_mask() == Some(*mask))
        })
        .count();
    let trench_corner_coverage = [3_u8, 6, 9, 12]
        .into_iter()
        .filter(|mask| {
            trench_mask_sprites
                .iter()
                .any(|sprite| sprite.kind.trench_mask() == Some(*mask))
        })
        .count();
    let trench_junction_coverage = [7_u8, 11, 13, 14, 15]
        .into_iter()
        .filter(|mask| {
            trench_mask_sprites
                .iter()
                .any(|sprite| sprite.kind.trench_mask() == Some(*mask))
        })
        .count();

    if trench_floor_darkness_score < 0.42 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: first_kind(sprites, TerrainSpriteKind::TrenchFloorTop)
                .map(|sprite| sprite.id.clone()),
            message: "trench floor may be too bright to read as recessed".to_string(),
        });
    }
    if trench_wall_floor_contrast < 0.05 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "trench wall and floor have low contrast".to_string(),
        });
    }
    if trench_lip_contrast < 0.10 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "trench lip has low contrast against floor".to_string(),
        });
    }
    if trench_shadow_continuity < 0.12 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: first_kind(sprites, TerrainSpriteKind::TrenchContactShadow)
                .map(|sprite| sprite.id.clone()),
            message: "trench contact shadow has low alpha coverage".to_string(),
        });
    }
    if !trench_cap_presence {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: "trench end/corner cap coverage is incomplete".to_string(),
        });
    }
    if trench_mask_coverage < 16 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: format!("missing trench topology masks {:?}", missing_trench_masks),
        });
    }
    if trench_cap_coverage < 5 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: "trench cap mask coverage is incomplete".to_string(),
        });
    }
    if trench_corner_coverage < 4 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: "trench corner mask coverage is incomplete".to_string(),
        });
    }
    if trench_junction_coverage < 5 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Error,
            sprite_id: None,
            message: "trench junction mask coverage is incomplete".to_string(),
        });
    }
    if trench_floor_continuity_score > 0.12 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "trench floor continuity score is high".to_string(),
        });
    }
    if trench_lip_continuity_score > 0.12 {
        issues.push(TerrainSpriteValidationIssue {
            severity: TerrainSpriteValidationSeverity::Warning,
            sprite_id: None,
            message: "trench lip continuity score is high".to_string(),
        });
    }

    TrenchSpriteValidationSummary {
        trench_piece_coverage,
        trench_role_coverage,
        trench_floor_darkness_score,
        trench_wall_floor_contrast,
        trench_lip_contrast,
        trench_shadow_continuity,
        trench_cap_presence,
        trench_oblique_anchor_validity,
        trench_mask_coverage,
        missing_trench_masks,
        trench_neighbor_seam_score,
        trench_floor_continuity_score,
        trench_wall_alignment_score,
        trench_lip_continuity_score,
        trench_cap_coverage,
        trench_corner_coverage,
        trench_junction_coverage,
        worst_trench_neighbor_pairs,
    }
}

fn expected_trench_role(kind: TerrainSpriteKind) -> SpriteRole {
    match kind {
        TerrainSpriteKind::TrenchFloorTop => SpriteRole::TopSurface,
        TerrainSpriteKind::TrenchWallFront => SpriteRole::FrontFace,
        TerrainSpriteKind::TrenchLipFront | TerrainSpriteKind::TrenchLipBack => SpriteRole::Lip,
        TerrainSpriteKind::TrenchEndCapLeft
        | TerrainSpriteKind::TrenchEndCapRight
        | TerrainSpriteKind::TrenchCornerInner
        | TerrainSpriteKind::TrenchCornerOuter => SpriteRole::CornerCap,
        TerrainSpriteKind::TrenchContactShadow => SpriteRole::ContactShadow,
        TerrainSpriteKind::TrenchSpoilPile => SpriteRole::Decal,
        _ => SpriteRole::TopSurface,
    }
}

fn first_kind(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
) -> Option<&GeneratedTerrainSprite> {
    sprites.iter().find(|sprite| sprite.kind == kind)
}

fn average_luma(sprite: &GeneratedTerrainSprite) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for pixel in &sprite.image.pixels {
        if pixel.a > 0 {
            total += pixel.luma() as f32;
            count += 1.0;
        }
    }
    if count == 0.0 {
        0.0
    } else {
        total / count
    }
}

fn alpha_coverage(sprite: &GeneratedTerrainSprite) -> f32 {
    let total = sprite.image.pixels.len().max(1) as f32;
    sprite
        .image
        .pixels
        .iter()
        .filter(|pixel| pixel.a > 8)
        .count() as f32
        / total
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

fn path_neighbor_seam_score(sprites: &[GeneratedTerrainSprite]) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for sprite in sprites.iter().filter(|sprite| sprite.kind.is_path_mask()) {
        let Some(mask) = sprite.kind.path_mask() else {
            continue;
        };
        for (direction, opposite) in [(1, 4), (2, 8), (4, 1), (8, 2)] {
            if mask & direction == 0 {
                continue;
            }
            let Some(neighbor) = sprites
                .iter()
                .find(|candidate| candidate.kind.path_mask() == Some(opposite))
            else {
                continue;
            };
            total += path_edge_difference(sprite, neighbor, direction);
            count += 1.0;
        }
    }
    if count == 0.0 {
        0.0
    } else {
        total / count
    }
}

fn trench_neighbor_seam_score(sprites: &[GeneratedTerrainSprite]) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for sprite in sprites.iter().filter(|sprite| sprite.kind.is_trench_mask()) {
        let Some(mask) = sprite.kind.trench_mask() else {
            continue;
        };
        for (direction, opposite) in [(1, 4), (2, 8), (4, 1), (8, 2)] {
            if mask & direction == 0 {
                continue;
            }
            let Some(neighbor) = sprites
                .iter()
                .find(|candidate| candidate.kind.trench_mask() == Some(opposite))
            else {
                continue;
            };
            total += trench_edge_difference(sprite, neighbor, direction, 0.0);
            count += 1.0;
        }
    }
    if count == 0.0 {
        0.0
    } else {
        total / count
    }
}

fn trench_mask_band_continuity_score(sprites: &[GeneratedTerrainSprite], band: f32) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    for sprite in sprites.iter().filter(|sprite| sprite.kind.is_trench_mask()) {
        let Some(mask) = sprite.kind.trench_mask() else {
            continue;
        };
        for (direction, opposite) in [(1, 4), (2, 8), (4, 1), (8, 2)] {
            if mask & direction == 0 {
                continue;
            }
            let Some(neighbor) = sprites
                .iter()
                .find(|candidate| candidate.kind.trench_mask() == Some(opposite))
            else {
                continue;
            };
            total += trench_edge_difference(sprite, neighbor, direction, band);
            count += 1.0;
        }
    }
    if count == 0.0 {
        0.0
    } else {
        total / count
    }
}

fn worst_trench_neighbor_pairs(sprites: &[GeneratedTerrainSprite]) -> Vec<TrenchNeighborPairScore> {
    let mut pairs = Vec::new();
    for sprite in sprites.iter().filter(|sprite| sprite.kind.is_trench_mask()) {
        let Some(mask) = sprite.kind.trench_mask() else {
            continue;
        };
        for (direction, opposite, edge) in [
            (1_u8, 4_u8, "north"),
            (2_u8, 8_u8, "east"),
            (4_u8, 1_u8, "south"),
            (8_u8, 2_u8, "west"),
        ] {
            if mask & direction == 0 {
                continue;
            }
            let Some(neighbor) = sprites
                .iter()
                .find(|candidate| candidate.kind.trench_mask() == Some(opposite))
            else {
                continue;
            };
            pairs.push(TrenchNeighborPairScore {
                mask_a: mask,
                edge: edge.to_string(),
                mask_b: opposite,
                score: trench_edge_difference(sprite, neighbor, direction, 0.0),
            });
        }
    }
    pairs.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pairs.truncate(8);
    pairs
}

fn path_edge_difference(
    sprite: &GeneratedTerrainSprite,
    neighbor: &GeneratedTerrainSprite,
    direction: u8,
) -> f32 {
    let tile = sprite.image.width.min(sprite.image.height);
    let mut total = 0.0;
    for i in 0..tile {
        let (a, b) = match direction {
            1 => (
                sprite.image.get(i, 0),
                neighbor.image.get(i, neighbor.image.height - 1),
            ),
            2 => (
                sprite.image.get(sprite.image.width - 1, i),
                neighbor.image.get(0, i),
            ),
            4 => (
                sprite.image.get(i, sprite.image.height - 1),
                neighbor.image.get(i, 0),
            ),
            8 => (
                sprite.image.get(0, i),
                neighbor.image.get(neighbor.image.width - 1, i),
            ),
            _ => (sprite.image.get(i, 0), sprite.image.get(i, 0)),
        };
        total += a.rgb_distance(b).min(120.0) / 120.0;
    }
    total / tile.max(1) as f32
}

fn trench_edge_difference(
    sprite: &GeneratedTerrainSprite,
    neighbor: &GeneratedTerrainSprite,
    direction: u8,
    band: f32,
) -> f32 {
    let width = sprite.image.width.min(neighbor.image.width).max(1);
    let height = sprite.image.height.min(neighbor.image.height).max(1);
    let surface_h = ((height as f32 * 0.68).round() as u32).clamp(1, height);
    let mut total = 0.0;
    let mut count = 0.0;
    match direction {
        1 | 4 => {
            let center = width / 2;
            let open = (width as f32 * 0.34).round() as u32;
            let start = center.saturating_sub(open / 2);
            let end = (center + open / 2).min(width - 1);
            let y_a = if direction == 1 { 0 } else { surface_h - 1 };
            let y_b = if direction == 1 { surface_h - 1 } else { 0 };
            let offset = (surface_h as f32 * band).round() as u32;
            let ya = if direction == 1 {
                (y_a + offset).min(surface_h - 1)
            } else {
                y_a.saturating_sub(offset)
            };
            let yb = if direction == 1 {
                y_b.saturating_sub(offset)
            } else {
                (y_b + offset).min(surface_h - 1)
            };
            for x in start..=end {
                total += sprite
                    .image
                    .get(x, ya)
                    .rgb_distance(neighbor.image.get(x, yb));
                count += 1.0;
            }
        }
        2 | 8 => {
            let center = surface_h / 2;
            let open = (surface_h as f32 * 0.34).round() as u32;
            let start = center.saturating_sub(open / 2);
            let end = (center + open / 2).min(surface_h - 1);
            let x_a = if direction == 8 { 0 } else { width - 1 };
            let x_b = if direction == 8 { width - 1 } else { 0 };
            let offset = (width as f32 * band * 0.5).round() as u32;
            let xa = if direction == 8 {
                (x_a + offset).min(width - 1)
            } else {
                x_a.saturating_sub(offset)
            };
            let xb = if direction == 8 {
                x_b.saturating_sub(offset)
            } else {
                (x_b + offset).min(width - 1)
            };
            for y in start..=end {
                total += sprite
                    .image
                    .get(xa, y)
                    .rgb_distance(neighbor.image.get(xb, y));
                count += 1.0;
            }
        }
        _ => {}
    }
    if count == 0.0 {
        0.0
    } else {
        (total / count).min(140.0) / 140.0
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
