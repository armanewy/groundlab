use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::pixel_image::PixelImage;
use crate::recipe::GroundMaterial;
use crate::tileset::Tileset;

pub const DEFAULT_ARTKIT_DIR: &str = "assets/artkits/dry_upland_outpost";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainArtPieceKind {
    GrassFloorLarge,
    GrassFloorEdge,
    DirtRoadLarge,
    DirtRoadEdge,
    TrenchFloor,
    TrenchWallFront,
    TrenchLip,
    BermTop,
    BermFaceFront,
    StoneFloor,
    StoneWallFront,
    MudFloor,
    SoftShadow,
    CornerCap,
    PropDebris,
}

impl TerrainArtPieceKind {
    pub fn id(self) -> &'static str {
        match self {
            TerrainArtPieceKind::GrassFloorLarge => "grass_floor_large",
            TerrainArtPieceKind::GrassFloorEdge => "grass_floor_edge",
            TerrainArtPieceKind::DirtRoadLarge => "dirt_road_large",
            TerrainArtPieceKind::DirtRoadEdge => "dirt_road_edge",
            TerrainArtPieceKind::TrenchFloor => "trench_floor",
            TerrainArtPieceKind::TrenchWallFront => "trench_wall_front",
            TerrainArtPieceKind::TrenchLip => "trench_lip",
            TerrainArtPieceKind::BermTop => "berm_top",
            TerrainArtPieceKind::BermFaceFront => "berm_face_front",
            TerrainArtPieceKind::StoneFloor => "stone_floor",
            TerrainArtPieceKind::StoneWallFront => "stone_wall_front",
            TerrainArtPieceKind::MudFloor => "mud_floor",
            TerrainArtPieceKind::SoftShadow => "soft_shadow",
            TerrainArtPieceKind::CornerCap => "corner_cap",
            TerrainArtPieceKind::PropDebris => "prop_debris",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainArtRepeatMode {
    Stretch,
    Tile,
    StretchMiddle,
    Stamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainArtOrientationSupport {
    SouthOnly,
    FourWay,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainArtOcclusion {
    None,
    Soft,
    Solid,
    Cutaway,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtPiece {
    pub id: String,
    pub kind: TerrainArtPieceKind,
    pub material: Option<GroundMaterial>,
    pub width_px: u32,
    pub height_px: u32,
    pub anchor_px: (i32, i32),
    pub footprint_cells: (u32, u32),
    pub repeat_mode: TerrainArtRepeatMode,
    pub orientation: TerrainArtOrientationSupport,
    pub z_bias: i32,
    pub opacity: f32,
    pub occlusion: TerrainArtOcclusion,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TerrainArtPieceAsset {
    pub definition: TerrainArtPiece,
    pub image: PixelImage,
}

#[derive(Clone, Debug)]
pub struct TerrainArtKit {
    pub id: String,
    pub pieces: Vec<TerrainArtPieceAsset>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtKitManifest {
    pub id: String,
    pub atlas_path: String,
    pub pieces: Vec<TerrainArtPieceManifest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtPieceManifest {
    pub piece: TerrainArtPiece,
    pub atlas_x: u32,
    pub atlas_y: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtKitFile {
    pub id: String,
    pub pieces: Vec<TerrainArtPieceFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtPieceFile {
    pub piece: TerrainArtPiece,
    pub file: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtKitValidation {
    pub kit_id: String,
    pub required_piece_count: usize,
    pub present_piece_count: usize,
    pub missing_required: Vec<TerrainArtPieceKind>,
    pub duplicate_ids: Vec<String>,
    pub issues: Vec<TerrainArtKitIssue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtKitIssue {
    pub severity: TerrainArtKitIssueSeverity,
    pub piece_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainArtKitIssueSeverity {
    Info,
    Warning,
    Error,
}

impl TerrainArtKit {
    pub fn generate(tileset: &Tileset) -> Self {
        let mut pieces = Vec::new();

        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::GrassFloorLarge,
                Some(GroundMaterial::Grass),
                (224, 160),
                TerrainArtRepeatMode::Tile,
                TerrainArtOrientationSupport::Any,
                &["floor", "grass", "irregular"],
            )
            .footprint((3, 2)),
            surface_piece(
                224,
                160,
                palette_sample(tileset, GroundMaterial::Grass, 0.45),
                0x1827,
                false,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::GrassFloorEdge,
                Some(GroundMaterial::Grass),
                (224, 36),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::FourWay,
                &["edge", "grass"],
            )
            .z_bias(2),
            edge_piece(
                224,
                36,
                palette_sample(tileset, GroundMaterial::Grass, 0.42),
                0x2147,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::DirtRoadLarge,
                Some(GroundMaterial::Dirt),
                (224, 128),
                TerrainArtRepeatMode::Tile,
                TerrainArtOrientationSupport::Any,
                &["floor", "road", "worn"],
            )
            .footprint((3, 2)),
            surface_piece(
                224,
                128,
                palette_sample(tileset, GroundMaterial::Dirt, 0.48),
                0x3d71,
                false,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::DirtRoadEdge,
                Some(GroundMaterial::Dirt),
                (224, 34),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::FourWay,
                &["edge", "road"],
            )
            .z_bias(3),
            edge_piece(
                224,
                34,
                palette_sample(tileset, GroundMaterial::Dirt, 0.46),
                0x4159,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::MudFloor,
                Some(GroundMaterial::Mud),
                (192, 112),
                TerrainArtRepeatMode::Tile,
                TerrainArtOrientationSupport::Any,
                &["floor", "mud", "wet"],
            )
            .footprint((3, 2)),
            surface_piece(
                192,
                112,
                palette_sample(tileset, GroundMaterial::Mud, 0.35),
                0x4aa1,
                false,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::StoneFloor,
                Some(GroundMaterial::Rock),
                (192, 112),
                TerrainArtRepeatMode::Tile,
                TerrainArtOrientationSupport::Any,
                &["floor", "stone"],
            )
            .footprint((3, 2)),
            stone_floor_piece(192, 112, tileset),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::StoneWallFront,
                Some(GroundMaterial::Rock),
                (224, 64),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::SouthOnly,
                &["face", "stone", "front"],
            )
            .z_bias(18)
            .occlusion(TerrainArtOcclusion::Solid),
            wall_piece(
                224,
                64,
                palette_sample(tileset, GroundMaterial::Rock, 0.44),
                0x65ce,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::TrenchFloor,
                Some(GroundMaterial::TrenchFloor),
                (224, 80),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::SouthOnly,
                &["trench", "floor", "shadow"],
            )
            .footprint((3, 1))
            .z_bias(-6),
            trench_floor_piece(224, 80, tileset),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::TrenchWallFront,
                Some(GroundMaterial::TrenchWall),
                (224, 56),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::SouthOnly,
                &["trench", "wall", "front"],
            )
            .z_bias(20)
            .occlusion(TerrainArtOcclusion::Cutaway),
            wall_piece(
                224,
                56,
                palette_sample(tileset, GroundMaterial::TrenchWall, 0.42),
                0x81a7,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::TrenchLip,
                Some(GroundMaterial::TrenchWall),
                (224, 22),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::FourWay,
                &["trench", "lip"],
            )
            .z_bias(24),
            edge_piece(
                224,
                22,
                palette_sample(tileset, GroundMaterial::TrenchWall, 0.58),
                0x92b1,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::BermTop,
                Some(GroundMaterial::BermTop),
                (224, 72),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::SouthOnly,
                &["berm", "top"],
            )
            .footprint((3, 1))
            .z_bias(10),
            surface_piece(
                224,
                72,
                palette_sample(tileset, GroundMaterial::BermTop, 0.58),
                0xa177,
                false,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::BermFaceFront,
                Some(GroundMaterial::BermFace),
                (224, 58),
                TerrainArtRepeatMode::StretchMiddle,
                TerrainArtOrientationSupport::SouthOnly,
                &["berm", "face", "front"],
            )
            .z_bias(22)
            .occlusion(TerrainArtOcclusion::Soft),
            wall_piece(
                224,
                58,
                palette_sample(tileset, GroundMaterial::BermFace, 0.44),
                0xb531,
            ),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::SoftShadow,
                None,
                (224, 64),
                TerrainArtRepeatMode::Stretch,
                TerrainArtOrientationSupport::Any,
                &["shadow"],
            )
            .opacity(0.75),
            soft_shadow_piece(224, 64),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::CornerCap,
                Some(GroundMaterial::Dirt),
                (64, 64),
                TerrainArtRepeatMode::Stamp,
                TerrainArtOrientationSupport::FourWay,
                &["corner", "cap"],
            )
            .z_bias(26)
            .occlusion(TerrainArtOcclusion::Soft),
            corner_cap_piece(64, 64, tileset),
        ));
        pieces.push(piece(
            PieceSpec::new(
                TerrainArtPieceKind::PropDebris,
                Some(GroundMaterial::Dirt),
                (96, 80),
                TerrainArtRepeatMode::Stamp,
                TerrainArtOrientationSupport::Any,
                &["prop", "debris", "tufts"],
            )
            .z_bias(32),
            debris_piece(96, 80, tileset),
        ));

        Self {
            id: format!("{}_terrain_artkit", tileset.recipe.id),
            pieces,
        }
    }

    pub fn load_external(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let manifest_path = dir.join("manifest.ron");
        let text = fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading art-kit manifest {}", manifest_path.display()))?;
        let file: TerrainArtKitFile = ron::de::from_str(&text)
            .with_context(|| format!("parsing art-kit manifest {}", manifest_path.display()))?;
        let mut pieces = Vec::with_capacity(file.pieces.len());
        for entry in file.pieces {
            let image_path = dir.join(&entry.file);
            let image = PixelImage::load_png(&image_path)
                .with_context(|| format!("loading art-kit piece {}", image_path.display()))?;
            pieces.push(TerrainArtPieceAsset {
                definition: entry.piece,
                image,
            });
        }
        Ok(Self {
            id: file.id,
            pieces,
        })
    }

    pub fn load_default_or_generate(tileset: &Tileset) -> Self {
        Self::load_external(DEFAULT_ARTKIT_DIR).unwrap_or_else(|_| Self::generate(tileset))
    }

    pub fn ensure_external_files(tileset: &Tileset, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        if dir.join("manifest.ron").exists() {
            return Ok(());
        }
        let kit = Self::generate(tileset);
        kit.save_external_files(dir)
    }

    pub fn save_external_files(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        let pieces_dir = dir.join("pieces");
        fs::create_dir_all(&pieces_dir)?;
        let mut manifest_pieces = Vec::with_capacity(self.pieces.len());
        for asset in &self.pieces {
            let file = format!("pieces/{}.png", asset.definition.id);
            asset.image.save_png(dir.join(&file))?;
            manifest_pieces.push(TerrainArtPieceFile {
                piece: asset.definition.clone(),
                file,
            });
        }
        let manifest = TerrainArtKitFile {
            id: self.id.clone(),
            pieces: manifest_pieces,
        };
        let text = ron::ser::to_string_pretty(&manifest, PrettyConfig::new())?;
        fs::write(dir.join("manifest.ron"), text)?;
        Ok(())
    }

    pub fn piece(&self, kind: TerrainArtPieceKind) -> Option<&TerrainArtPieceAsset> {
        self.pieces
            .iter()
            .find(|piece| piece.definition.kind == kind)
    }

    pub fn build_atlas(&self, padding: u32) -> PixelImage {
        let width = self
            .pieces
            .iter()
            .map(|piece| piece.image.width)
            .max()
            .unwrap_or(1)
            + padding * 2;
        let height = self
            .pieces
            .iter()
            .map(|piece| piece.image.height + padding)
            .sum::<u32>()
            + padding;
        let mut atlas = PixelImage::transparent(width, height.max(1));
        let mut y = padding;
        for piece in &self.pieces {
            atlas.blit(&piece.image, padding, y);
            y += piece.image.height + padding;
        }
        atlas
    }

    pub fn manifest(&self, atlas_path: impl Into<String>, padding: u32) -> TerrainArtKitManifest {
        let mut y = padding;
        let mut pieces = Vec::with_capacity(self.pieces.len());
        for asset in &self.pieces {
            pieces.push(TerrainArtPieceManifest {
                piece: asset.definition.clone(),
                atlas_x: padding,
                atlas_y: y,
            });
            y += asset.image.height + padding;
        }
        TerrainArtKitManifest {
            id: self.id.clone(),
            atlas_path: atlas_path.into(),
            pieces,
        }
    }

    pub fn validate(&self) -> TerrainArtKitValidation {
        let mut issues = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut duplicate_ids = Vec::new();
        let present_kinds: HashSet<_> = self
            .pieces
            .iter()
            .map(|piece| piece.definition.kind)
            .collect();

        for asset in &self.pieces {
            let def = &asset.definition;
            if !seen_ids.insert(def.id.clone()) {
                duplicate_ids.push(def.id.clone());
                issues.push(TerrainArtKitIssue {
                    severity: TerrainArtKitIssueSeverity::Error,
                    piece_id: Some(def.id.clone()),
                    message: "duplicate art-piece id".to_string(),
                });
            }
            if def.width_px != asset.image.width || def.height_px != asset.image.height {
                issues.push(TerrainArtKitIssue {
                    severity: TerrainArtKitIssueSeverity::Warning,
                    piece_id: Some(def.id.clone()),
                    message: format!(
                        "manifest size {}x{} differs from image size {}x{}",
                        def.width_px, def.height_px, asset.image.width, asset.image.height
                    ),
                });
            }
            if def.footprint_cells.0 == 0 || def.footprint_cells.1 == 0 {
                issues.push(TerrainArtKitIssue {
                    severity: TerrainArtKitIssueSeverity::Error,
                    piece_id: Some(def.id.clone()),
                    message: "footprint_cells must be at least 1x1".to_string(),
                });
            }
            if def.opacity <= 0.0 || def.opacity > 1.0 {
                issues.push(TerrainArtKitIssue {
                    severity: TerrainArtKitIssueSeverity::Error,
                    piece_id: Some(def.id.clone()),
                    message: "opacity must be in the range (0, 1]".to_string(),
                });
            }
            if matches!(
                def.repeat_mode,
                TerrainArtRepeatMode::Stretch | TerrainArtRepeatMode::StretchMiddle
            ) && asset.image.width < 32
            {
                issues.push(TerrainArtKitIssue {
                    severity: TerrainArtKitIssueSeverity::Warning,
                    piece_id: Some(def.id.clone()),
                    message: "stretchable piece is narrow enough to show repetition artifacts"
                        .to_string(),
                });
            }
        }

        let missing_required = required_piece_kinds()
            .iter()
            .copied()
            .filter(|kind| !present_kinds.contains(kind))
            .collect::<Vec<_>>();
        for kind in &missing_required {
            issues.push(TerrainArtKitIssue {
                severity: TerrainArtKitIssueSeverity::Error,
                piece_id: None,
                message: format!("missing required art piece kind {}", kind.id()),
            });
        }

        TerrainArtKitValidation {
            kit_id: self.id.clone(),
            required_piece_count: required_piece_kinds().len(),
            present_piece_count: self.pieces.len(),
            missing_required,
            duplicate_ids,
            issues,
        }
    }
}

struct PieceSpec<'a> {
    kind: TerrainArtPieceKind,
    material: Option<GroundMaterial>,
    size_px: (u32, u32),
    repeat_mode: TerrainArtRepeatMode,
    orientation: TerrainArtOrientationSupport,
    footprint_cells: (u32, u32),
    z_bias: i32,
    opacity: f32,
    occlusion: TerrainArtOcclusion,
    tags: &'a [&'a str],
}

impl<'a> PieceSpec<'a> {
    fn new(
        kind: TerrainArtPieceKind,
        material: Option<GroundMaterial>,
        size_px: (u32, u32),
        repeat_mode: TerrainArtRepeatMode,
        orientation: TerrainArtOrientationSupport,
        tags: &'a [&'a str],
    ) -> Self {
        Self {
            kind,
            material,
            size_px,
            repeat_mode,
            orientation,
            footprint_cells: (1, 1),
            z_bias: 0,
            opacity: 1.0,
            occlusion: TerrainArtOcclusion::None,
            tags,
        }
    }

    fn footprint(mut self, footprint_cells: (u32, u32)) -> Self {
        self.footprint_cells = footprint_cells;
        self
    }

    fn z_bias(mut self, z_bias: i32) -> Self {
        self.z_bias = z_bias;
        self
    }

    fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    fn occlusion(mut self, occlusion: TerrainArtOcclusion) -> Self {
        self.occlusion = occlusion;
        self
    }
}

fn piece(spec: PieceSpec<'_>, image: PixelImage) -> TerrainArtPieceAsset {
    let (width_px, height_px) = spec.size_px;
    TerrainArtPieceAsset {
        definition: TerrainArtPiece {
            id: format!("{}_01", spec.kind.id()),
            kind: spec.kind,
            material: spec.material,
            width_px,
            height_px,
            anchor_px: (0, 0),
            footprint_cells: spec.footprint_cells,
            repeat_mode: spec.repeat_mode,
            orientation: spec.orientation,
            z_bias: spec.z_bias,
            opacity: spec.opacity,
            occlusion: spec.occlusion,
            tags: spec.tags.iter().map(|tag| (*tag).to_string()).collect(),
        },
        image,
    }
}

fn required_piece_kinds() -> &'static [TerrainArtPieceKind] {
    &[
        TerrainArtPieceKind::GrassFloorLarge,
        TerrainArtPieceKind::GrassFloorEdge,
        TerrainArtPieceKind::DirtRoadLarge,
        TerrainArtPieceKind::DirtRoadEdge,
        TerrainArtPieceKind::TrenchFloor,
        TerrainArtPieceKind::TrenchWallFront,
        TerrainArtPieceKind::TrenchLip,
        TerrainArtPieceKind::BermTop,
        TerrainArtPieceKind::BermFaceFront,
        TerrainArtPieceKind::StoneFloor,
        TerrainArtPieceKind::StoneWallFront,
        TerrainArtPieceKind::MudFloor,
        TerrainArtPieceKind::SoftShadow,
        TerrainArtPieceKind::CornerCap,
        TerrainArtPieceKind::PropDebris,
    ]
}

fn palette_sample(tileset: &Tileset, material: GroundMaterial, t: f32) -> Rgba8 {
    tileset.palette.sample(material.ramp(), t)
}

fn surface_piece(
    width: u32,
    height: u32,
    base: Rgba8,
    seed: u32,
    irregular_edge: bool,
) -> PixelImage {
    let mut image = PixelImage::transparent(width, height);
    for y in 0..height {
        for x in 0..width {
            let noise = signed_noise(seed, x, y);
            let mut color = if noise > 0.35 {
                base.lighten(noise * 0.08)
            } else {
                base.darken(-noise * 0.08)
            };
            if x % 41 == 0 || y % 37 == 0 {
                color = color.darken(0.035);
            }
            let alpha = if irregular_edge {
                irregular_alpha(seed ^ 0x9e37, x, y, width, height, 11)
            } else {
                255
            };
            image.set(x, y, color.with_alpha(alpha));
        }
    }
    image
}

fn edge_piece(width: u32, height: u32, base: Rgba8, seed: u32) -> PixelImage {
    let mut image = PixelImage::transparent(width, height);
    for y in 0..height {
        let t = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let n = signed_noise(seed, x, y);
            let alpha = irregular_alpha(seed, x, y, width, height, 8);
            let color = base
                .lighten((1.0 - t) * 0.08)
                .darken(t * 0.20 + n.abs() * 0.04);
            image.set(x, y, color.with_alpha(alpha));
        }
    }
    image
}

fn wall_piece(width: u32, height: u32, base: Rgba8, seed: u32) -> PixelImage {
    let mut image = PixelImage::transparent(width, height);
    for y in 0..height {
        let t = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let n = signed_noise(seed, x, y);
            let mut color = base.darken(t * 0.30).lighten((1.0 - t) * 0.06);
            if (x + seed) % 43 < 2 || (y + seed / 3).is_multiple_of(19) {
                color = color.darken(0.12);
            }
            if n > 0.44 {
                color = color.lighten(0.08);
            }
            image.set(
                x,
                y,
                color.with_alpha(irregular_alpha(seed ^ 0x54ad, x, y, width, height, 6)),
            );
        }
    }
    image
}

fn trench_floor_piece(width: u32, height: u32, tileset: &Tileset) -> PixelImage {
    let base = palette_sample(tileset, GroundMaterial::TrenchFloor, 0.30);
    let mut image = surface_piece(width, height, base, 0xc031, true);
    let shadow = palette_sample(tileset, GroundMaterial::TrenchWall, 0.20).darken(0.20);
    for y in height / 4..height {
        let alpha = ((y - height / 4) as f32 / height.max(1) as f32 * 180.0).min(120.0) as u8;
        for x in 0..width {
            let px = image.get(x, y).blend(shadow, alpha as f32 / 255.0);
            image.set(x, y, px);
        }
    }
    image
}

fn stone_floor_piece(width: u32, height: u32, tileset: &Tileset) -> PixelImage {
    let base = palette_sample(tileset, GroundMaterial::Rock, 0.48);
    let mut image = surface_piece(width, height, base, 0xd611, false);
    let line = base.darken(0.22);
    let cell_w = 42;
    let cell_h = 34;
    for y in 0..height {
        for x in 0..width {
            if x % cell_w < 2 || y % cell_h < 2 {
                let px = image.get(x, y).blend(line, 0.30);
                image.set(x, y, px);
            }
        }
    }
    image
}

fn soft_shadow_piece(width: u32, height: u32) -> PixelImage {
    let mut image = PixelImage::transparent(width, height);
    let cx = width as f32 * 0.50;
    let cy = height as f32 * 0.35;
    for y in 0..height {
        for x in 0..width {
            let dx = ((x as f32 - cx) / width.max(1) as f32).abs();
            let dy = ((y as f32 - cy) / height.max(1) as f32).abs();
            let a = (1.0 - (dx * 2.1 + dy * 1.7)).clamp(0.0, 1.0);
            image.set(x, y, Rgba8::new(0, 0, 0, (a * 120.0) as u8));
        }
    }
    image
}

fn corner_cap_piece(width: u32, height: u32, tileset: &Tileset) -> PixelImage {
    let base = palette_sample(tileset, GroundMaterial::BermFace, 0.50);
    let mut image = PixelImage::transparent(width, height);
    for y in 0..height {
        for x in 0..width {
            if x + y < width / 2 || x > width - 7 || y > height - 7 {
                continue;
            }
            let color = base.darken((y as f32 / height.max(1) as f32) * 0.24);
            image.set(
                x,
                y,
                color.with_alpha(irregular_alpha(0xe351, x, y, width, height, 5)),
            );
        }
    }
    image
}

fn debris_piece(width: u32, height: u32, tileset: &Tileset) -> PixelImage {
    let mut image = PixelImage::transparent(width, height);
    let dirt = palette_sample(tileset, GroundMaterial::Dirt, 0.52);
    let grass = palette_sample(tileset, GroundMaterial::Grass, 0.62);
    for i in 0..26 {
        let x = hash2(0xf0ad, i * 17, i * 11) % width.max(1);
        let y = hash2(0xf0ad, i * 23, i * 7) % height.max(1);
        let color = if i % 3 == 0 { grass } else { dirt.darken(0.16) };
        let w = 2 + (hash2(0x9921, x, y) % 6);
        let h = 1 + (hash2(0x4221, x, y) % 4);
        for yy in y..(y + h).min(height) {
            for xx in x..(x + w).min(width) {
                image.set(xx, yy, color.with_alpha(180));
            }
        }
    }
    image
}

fn irregular_alpha(seed: u32, x: u32, y: u32, width: u32, height: u32, edge_px: u32) -> u8 {
    let edge = x
        .min(y)
        .min(width.saturating_sub(1).saturating_sub(x))
        .min(height.saturating_sub(1).saturating_sub(y));
    if edge >= edge_px {
        return 255;
    }
    let threshold = hash2(seed, x / 2, y / 2) % edge_px.max(1);
    if edge + 1 < threshold {
        0
    } else {
        ((edge as f32 / edge_px.max(1) as f32).sqrt() * 255.0).clamp(120.0, 255.0) as u8
    }
}

fn signed_noise(seed: u32, x: u32, y: u32) -> f32 {
    let n = hash2(seed, x / 3, y / 3) as f32 / u32::MAX as f32;
    clamp01(n) * 2.0 - 1.0
}

fn hash2(seed: u32, x: u32, y: u32) -> u32 {
    let mut v = seed ^ x.wrapping_mul(0x9e37_79b1) ^ y.wrapping_mul(0x85eb_ca6b);
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^ (v >> 16)
}
