use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::pixel_image::PixelImage;
use crate::recipe::GroundMaterial;
use crate::tileset::Tileset;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainArtPiece {
    pub id: String,
    pub kind: TerrainArtPieceKind,
    pub material: Option<GroundMaterial>,
    pub width_px: u32,
    pub height_px: u32,
    pub anchor_px: (i32, i32),
    pub repeat_mode: TerrainArtRepeatMode,
    pub orientation: TerrainArtOrientationSupport,
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
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
            ),
            debris_piece(96, 80, tileset),
        ));

        Self {
            id: format!("{}_terrain_artkit", tileset.recipe.id),
            pieces,
        }
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
}

struct PieceSpec<'a> {
    kind: TerrainArtPieceKind,
    material: Option<GroundMaterial>,
    size_px: (u32, u32),
    repeat_mode: TerrainArtRepeatMode,
    orientation: TerrainArtOrientationSupport,
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
            tags,
        }
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
            repeat_mode: spec.repeat_mode,
            orientation: spec.orientation,
            tags: spec.tags.iter().map(|tag| (*tag).to_string()).collect(),
        },
        image,
    }
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
