use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, StructureFaceKind};
use crate::tileset::Tileset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

impl ValidationSeverity {
    pub fn label(self) -> &'static str {
        match self {
            ValidationSeverity::Info => "info",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub category: String,
    pub message: String,
    pub tile_a: Option<String>,
    pub tile_b: Option<String>,
    pub metric: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub score: f32,
    pub surface_tiles: usize,
    pub transition_tiles: usize,
    pub structure_face_tiles: usize,
    pub palette_ramps: usize,
    pub missing_ramps: Vec<String>,
    pub max_seam_delta: f32,
    pub avg_seam_delta: f32,
    pub avg_palette_drift: f32,
    pub issues: Vec<ValidationIssue>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            score: 0.0,
            surface_tiles: 0,
            transition_tiles: 0,
            structure_face_tiles: 0,
            palette_ramps: 0,
            missing_ramps: Vec::new(),
            max_seam_delta: 0.0,
            avg_seam_delta: 0.0,
            avg_palette_drift: 0.0,
            issues: Vec::new(),
        }
    }
}

impl ValidationReport {
    pub fn summary_line(&self) -> String {
        format!(
            "score {:.0}/100 · {} surface · {} transition · {} faces · max seam {:.1} · palette drift {:.1}",
            self.score,
            self.surface_tiles,
            self.transition_tiles,
            self.structure_face_tiles,
            self.max_seam_delta,
            self.avg_palette_drift
        )
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == ValidationSeverity::Warning)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == ValidationSeverity::Error)
            .count()
    }
}

pub fn validate_tileset(tileset: &Tileset) -> ValidationReport {
    let mut report = ValidationReport {
        surface_tiles: tileset.surface_tile_count(),
        transition_tiles: tileset.transition_tile_count(),
        structure_face_tiles: tileset.structure_face_tile_count(),
        palette_ramps: tileset.palette.ramps.len(),
        ..ValidationReport::default()
    };

    validate_palette(tileset, &mut report);
    validate_surface_counts(tileset, &mut report);
    validate_structure_face_counts(tileset, &mut report);
    validate_same_material_seams(tileset, &mut report);
    validate_palette_drift(tileset, &mut report);
    compute_score(&mut report);
    report
}

pub fn build_seam_test_sheet(tileset: &Tileset) -> PixelImage {
    let tile = tileset.recipe.tile_size;
    let padding = 2;
    let cols = 6_u32;
    let material_block_rows = GroundMaterial::ALL.len() as u32 * 2;
    let transition_rows = if tileset.transition_tile_count() > 0 {
        6_u32
    } else {
        0
    };
    let face_rows = if tileset.structure_face_tile_count() > 0 {
        8_u32
    } else {
        0
    };
    let rows = material_block_rows + transition_rows + face_rows;
    let width = cols * tile + (cols + 1) * padding;
    let height = rows * tile + (rows + 1) * padding;
    let mut image = PixelImage::new(width, height, Rgba8::opaque(15, 15, 18));

    let mut row = 0_u32;
    for material in GroundMaterial::ALL {
        for local_y in 0..2 {
            for col in 0..cols {
                let variant = (col + local_y * cols + material as u32)
                    % tileset.recipe.variants_per_material.max(1);
                let asset = tileset.tile(material, variant);
                let x = padding + col * (tile + padding);
                let y = padding + row * (tile + padding);
                image.blit(&asset.image, x, y);
            }
            row += 1;
        }
    }

    let transitions = tileset.transition_tiles();
    if !transitions.is_empty() {
        for transition_row in 0..transition_rows {
            for col in 0..cols {
                let idx = (transition_row * cols + col) as usize % transitions.len();
                let asset = transitions[idx];
                let x = padding + col * (tile + padding);
                let y = padding + row * (tile + padding);
                image.blit(&asset.image, x, y);
                image.outline_rect(x, y, tile, tile, Rgba8::opaque(67, 88, 78));
            }
            row += 1;
        }
    }

    let faces = tileset.structure_face_tiles();
    if !faces.is_empty() {
        for face_row in 0..face_rows {
            for col in 0..cols {
                let idx = (face_row * cols + col) as usize % faces.len();
                let asset = faces[idx];
                let x = padding + col * (tile + padding);
                let y = padding + row * (tile + padding);
                image.blit(&asset.image, x, y);
                image.outline_rect(x, y, tile, tile, Rgba8::opaque(104, 70, 46));
            }
            row += 1;
        }
    }

    image
}

fn validate_palette(tileset: &Tileset, report: &mut ValidationReport) {
    for material in GroundMaterial::ALL {
        if tileset.palette.ramp(material.ramp()).is_none() {
            report.missing_ramps.push(material.ramp().to_string());
            report.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                category: "palette".to_string(),
                message: format!(
                    "missing palette ramp '{}' required by {}",
                    material.ramp(),
                    material.display_name()
                ),
                tile_a: None,
                tile_b: None,
                metric: None,
            });
        }
    }
    if tileset.palette.all_colors().len() < 16 {
        report.issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            category: "palette".to_string(),
            message: "palette has fewer than 16 total colors; generated assets may lack enough ramp detail".to_string(),
            tile_a: None,
            tile_b: None,
            metric: Some(tileset.palette.all_colors().len() as f32),
        });
    }
}

fn validate_surface_counts(tileset: &Tileset, report: &mut ValidationReport) {
    for material in GroundMaterial::ALL {
        let count = tileset.surface_tiles_for(material).len();
        if count == 0 {
            report.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                category: "tiles".to_string(),
                message: format!("no surface tiles generated for {}", material.display_name()),
                tile_a: None,
                tile_b: None,
                metric: None,
            });
        }
    }
    if tileset.recipe.generate_transitions && tileset.transition_tile_count() == 0 {
        report.issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            category: "tiles".to_string(),
            message: "transition generation is enabled, but no transition tiles were produced"
                .to_string(),
            tile_a: None,
            tile_b: None,
            metric: None,
        });
    }
}

fn validate_structure_face_counts(tileset: &Tileset, report: &mut ValidationReport) {
    if !tileset.recipe.generate_structure_faces {
        return;
    }

    for material in [
        GroundMaterial::Dirt,
        GroundMaterial::Mud,
        GroundMaterial::Rock,
        GroundMaterial::TrenchWall,
        GroundMaterial::BermFace,
    ] {
        for face in StructureFaceKind::ALL {
            let count = tileset.structure_face_tiles_for(material, face).len();
            if count == 0 {
                report.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    category: "tiles".to_string(),
                    message: format!(
                        "no structure-face tiles generated for {} {}",
                        material.display_name(),
                        face.label()
                    ),
                    tile_a: None,
                    tile_b: None,
                    metric: None,
                });
            }
        }
    }
}

fn validate_same_material_seams(tileset: &Tileset, report: &mut ValidationReport) {
    let mut total = 0.0;
    let mut count = 0_u32;
    let threshold = tileset.recipe.seam_warning_threshold;

    for material in GroundMaterial::ALL {
        let variants = tileset.surface_tiles_for(material);
        for a in &variants {
            for b in &variants {
                if a.meta.variant == b.meta.variant {
                    continue;
                }
                let horizontal = edge_delta(&a.image, &b.image, EdgeSide::East, EdgeSide::West);
                let vertical = edge_delta(&a.image, &b.image, EdgeSide::South, EdgeSide::North);
                for value in [horizontal, vertical] {
                    report.max_seam_delta = report.max_seam_delta.max(value);
                    total += value;
                    count += 1;
                }
                if horizontal > threshold {
                    report.issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        category: "seams".to_string(),
                        message: format!(
                            "high horizontal seam delta between {} variants",
                            material.display_name()
                        ),
                        tile_a: Some(a.meta.id.clone()),
                        tile_b: Some(b.meta.id.clone()),
                        metric: Some(horizontal),
                    });
                }
                if vertical > threshold {
                    report.issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        category: "seams".to_string(),
                        message: format!(
                            "high vertical seam delta between {} variants",
                            material.display_name()
                        ),
                        tile_a: Some(a.meta.id.clone()),
                        tile_b: Some(b.meta.id.clone()),
                        metric: Some(vertical),
                    });
                }
            }
        }
    }

    if count > 0 {
        report.avg_seam_delta = total / count as f32;
    }
}

fn validate_palette_drift(tileset: &Tileset, report: &mut ValidationReport) {
    let mut total = 0.0;
    let mut count = 0_u32;
    for asset in &tileset.tiles {
        let step = (tileset.recipe.tile_size / 8).max(1);
        let mut y = 0;
        while y < asset.image.height {
            let mut x = 0;
            while x < asset.image.width {
                total += tileset.palette.nearest_distance(asset.image.get(x, y));
                count += 1;
                x += step;
            }
            y += step;
        }
    }
    if count > 0 {
        report.avg_palette_drift = total / count as f32;
    }
    if report.avg_palette_drift > 42.0 {
        report.issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            category: "palette".to_string(),
            message: "average generated color is drifting far from the configured palette anchors"
                .to_string(),
            tile_a: None,
            tile_b: None,
            metric: Some(report.avg_palette_drift),
        });
    }
}

fn compute_score(report: &mut ValidationReport) {
    let mut score = 100.0;
    score -= report.error_count() as f32 * 22.0;
    score -= report.warning_count() as f32 * 1.8;
    score -= (report.max_seam_delta / 4.0).min(24.0);
    score -= (report.avg_palette_drift / 5.0).min(16.0);
    report.score = score.clamp(0.0, 100.0);
}

#[derive(Clone, Copy)]
enum EdgeSide {
    North,
    South,
    East,
    West,
}

fn edge_delta(a: &PixelImage, b: &PixelImage, a_side: EdgeSide, b_side: EdgeSide) -> f32 {
    let samples = match a_side {
        EdgeSide::North | EdgeSide::South => a.width.min(b.width),
        EdgeSide::East | EdgeSide::West => a.height.min(b.height),
    };
    if samples == 0 {
        return 0.0;
    }

    let mut total = 0.0;
    for i in 0..samples {
        let ac = sample_edge(a, a_side, i);
        let bc = sample_edge(b, b_side, i);
        total += ac.rgb_distance(bc);
    }
    total / samples as f32
}

fn sample_edge(image: &PixelImage, side: EdgeSide, i: u32) -> Rgba8 {
    match side {
        EdgeSide::North => image.get(i.min(image.width - 1), 0),
        EdgeSide::South => image.get(i.min(image.width - 1), image.height - 1),
        EdgeSide::East => image.get(image.width - 1, i.min(image.height - 1)),
        EdgeSide::West => image.get(0, i.min(image.height - 1)),
    }
}

pub fn validation_score_color(score: f32) -> Rgba8 {
    let t = clamp01(score / 100.0);
    if t > 0.75 {
        Rgba8::opaque(118, 190, 124)
    } else if t > 0.45 {
        Rgba8::opaque(224, 190, 86)
    } else {
        Rgba8::opaque(214, 88, 78)
    }
}
