use serde::{Deserialize, Serialize};

use crate::feature::{feature_visual_material, TerrainFeatureKind, TerrainFeatureMap};
use crate::recipe::GroundMaterial;
use crate::terrain::TerrainMap;
use crate::terrain_artkit::TerrainArtPieceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainStampKind {
    GrassFieldPatch,
    DirtRoadSegment,
    DirtRoadJunction,
    TrenchStraight,
    TrenchCorner,
    TrenchEndCap,
    BermStraight,
    BermCorner,
    StonePlatform,
    MudPatch,
    GrassTuftCluster,
    RockScatter,
    CastShadow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StampPiece {
    pub piece_kind: TerrainArtPieceKind,
    pub offset_px: (i32, i32),
    pub size_px: (u32, u32),
    pub opacity: f32,
    pub z_bias: i32,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainStampDefinition {
    pub id: String,
    pub kind: TerrainStampKind,
    pub material: Option<GroundMaterial>,
    pub footprint_cells: (u32, u32),
    pub cells: Vec<(u32, u32)>,
    pub pieces: Vec<StampPiece>,
    pub z_bias: i32,
    pub note: String,
}

pub struct TerrainStampResolver;

impl TerrainStampResolver {
    pub fn resolve(map: &TerrainMap) -> Vec<TerrainStampDefinition> {
        let features = TerrainFeatureMap::from_terrain(map);
        let mut visited = vec![false; map.width as usize * map.height as usize];
        let mut stamps = Vec::new();

        for y in 0..map.height {
            for x in 0..map.width {
                let idx = y as usize * map.width as usize + x as usize;
                if visited[idx] {
                    continue;
                }
                let Some(cell) = map.cell(x, y) else {
                    visited[idx] = true;
                    continue;
                };
                let Some(feature) = features.cell(x, y) else {
                    visited[idx] = true;
                    continue;
                };
                let key = stamp_key(feature.kind, feature_visual_material(cell.ground));
                let component = collect_component(map, &features, &mut visited, x, y, key);
                stamps.push(component_stamp(key, component, stamps.len()));
            }
        }

        append_decorative_stamps(map, &mut stamps);
        stamps.sort_by_key(|stamp| {
            let (x, y, _, _) = stamp_bounds(&stamp.cells);
            (stamp.z_bias, y, x, stamp.id.clone())
        });
        stamps
    }
}

fn collect_component(
    map: &TerrainMap,
    features: &TerrainFeatureMap,
    visited: &mut [bool],
    start_x: u32,
    start_y: u32,
    key: StampKey,
) -> Vec<(u32, u32)> {
    let mut stack = vec![(start_x, start_y)];
    let mut cells = Vec::new();
    while let Some((x, y)) = stack.pop() {
        if x >= map.width || y >= map.height {
            continue;
        }
        let idx = y as usize * map.width as usize + x as usize;
        if visited[idx] {
            continue;
        }
        let Some(cell) = map.cell(x, y) else {
            visited[idx] = true;
            continue;
        };
        let Some(feature) = features.cell(x, y) else {
            visited[idx] = true;
            continue;
        };
        if stamp_key(feature.kind, feature_visual_material(cell.ground)) != key {
            continue;
        }
        visited[idx] = true;
        cells.push((x, y));
        if x > 0 {
            stack.push((x - 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if x + 1 < map.width {
            stack.push((x + 1, y));
        }
        if y + 1 < map.height {
            stack.push((x, y + 1));
        }
    }
    cells
}

fn component_stamp(key: StampKey, cells: Vec<(u32, u32)>, index: usize) -> TerrainStampDefinition {
    let (x0, y0, width, height) = stamp_bounds(&cells);
    let seed = hash_stamp(x0, y0, index as u32);
    let (kind, material, piece_kind, z_bias, note) = match key {
        StampKey::Trench => (
            stamp_shape_kind(
                &cells,
                TerrainStampKind::TrenchStraight,
                TerrainStampKind::TrenchCorner,
            ),
            Some(GroundMaterial::TrenchFloor),
            TerrainArtPieceKind::TrenchFloor,
            30,
            "trench component resolved into floor, lip, caps, spoil, and shadow stamps",
        ),
        StampKey::Berm => (
            stamp_shape_kind(
                &cells,
                TerrainStampKind::BermStraight,
                TerrainStampKind::BermCorner,
            ),
            Some(GroundMaterial::BermTop),
            TerrainArtPieceKind::BermTop,
            34,
            "berm component resolved into mound, edge, corner, and base shadow stamps",
        ),
        StampKey::Dirt => (
            if junction_like(&cells) {
                TerrainStampKind::DirtRoadJunction
            } else {
                TerrainStampKind::DirtRoadSegment
            },
            Some(GroundMaterial::Dirt),
            TerrainArtPieceKind::DirtRoadLarge,
            12,
            "dirt road component resolved as an organic road surface",
        ),
        StampKey::Rock => (
            TerrainStampKind::StonePlatform,
            Some(GroundMaterial::Rock),
            TerrainArtPieceKind::StoneFloor,
            26,
            "stone component resolved as a platform/outcrop with block volume",
        ),
        StampKey::Mud => (
            TerrainStampKind::MudPatch,
            Some(GroundMaterial::Mud),
            TerrainArtPieceKind::MudFloor,
            11,
            "mud component resolved as a soft wet patch",
        ),
        StampKey::Grass => (
            TerrainStampKind::GrassFieldPatch,
            Some(GroundMaterial::Grass),
            TerrainArtPieceKind::GrassFloorLarge,
            0,
            "grass component resolved as painterly ground fill",
        ),
    };
    TerrainStampDefinition {
        id: format!("target_{kind:?}_{index:03}").to_lowercase(),
        kind,
        material,
        footprint_cells: (width, height),
        cells,
        pieces: vec![StampPiece {
            piece_kind,
            offset_px: (0, 0),
            size_px: (width.max(1) * 96, height.max(1) * 80),
            opacity: if matches!(key, StampKey::Grass) {
                0.55
            } else {
                1.0
            },
            z_bias,
            seed,
        }],
        z_bias,
        note: note.to_string(),
    }
}

fn append_decorative_stamps(map: &TerrainMap, stamps: &mut Vec<TerrainStampDefinition>) {
    let mut count = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let Some(cell) = map.cell(x, y) else {
                continue;
            };
            let hash = hash_stamp(x, y, 0x9e37);
            if matches!(cell.ground, GroundMaterial::Grass) && hash.is_multiple_of(7) {
                stamps.push(deco_stamp(
                    TerrainStampKind::GrassTuftCluster,
                    TerrainArtPieceKind::GrassTuftCluster,
                    (x, y),
                    36,
                    hash,
                    "deterministic grass dressing generated from editable terrain",
                ));
                count += 1;
            } else if matches!(cell.ground, GroundMaterial::Rock) && hash.is_multiple_of(3) {
                stamps.push(deco_stamp(
                    TerrainStampKind::RockScatter,
                    TerrainArtPieceKind::LooseRocks,
                    (x, y),
                    40,
                    hash,
                    "deterministic rock scatter generated from editable terrain",
                ));
                count += 1;
            }
            if count > 18 {
                return;
            }
        }
    }
}

fn deco_stamp(
    kind: TerrainStampKind,
    piece_kind: TerrainArtPieceKind,
    cell: (u32, u32),
    z_bias: i32,
    seed: u32,
    note: &str,
) -> TerrainStampDefinition {
    TerrainStampDefinition {
        id: format!(
            "target_deco_{}_{:02}_{:02}",
            piece_kind.id(),
            cell.0,
            cell.1
        ),
        kind,
        material: None,
        footprint_cells: (1, 1),
        cells: vec![cell],
        pieces: vec![StampPiece {
            piece_kind,
            offset_px: (((seed & 0xf) as i32) - 8, (((seed >> 4) & 0xf) as i32) - 8),
            size_px: (72, 56),
            opacity: 0.55,
            z_bias,
            seed,
        }],
        z_bias,
        note: note.to_string(),
    }
}

fn stamp_bounds(cells: &[(u32, u32)]) -> (u32, u32, u32, u32) {
    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_x = cells.iter().map(|(x, _)| *x).max().unwrap_or(min_x);
    let max_y = cells.iter().map(|(_, y)| *y).max().unwrap_or(min_y);
    (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

fn stamp_shape_kind(
    cells: &[(u32, u32)],
    straight: TerrainStampKind,
    corner: TerrainStampKind,
) -> TerrainStampKind {
    if junction_like(cells) {
        corner
    } else {
        straight
    }
}

fn junction_like(cells: &[(u32, u32)]) -> bool {
    let (_, _, width, height) = stamp_bounds(cells);
    width > 1 && height > 1
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StampKey {
    Grass,
    Dirt,
    Mud,
    Rock,
    Trench,
    Berm,
}

fn stamp_key(kind: TerrainFeatureKind, material: GroundMaterial) -> StampKey {
    match kind {
        TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch => StampKey::Trench,
        TerrainFeatureKind::Berm => StampKey::Berm,
        _ => match material {
            GroundMaterial::Dirt => StampKey::Dirt,
            GroundMaterial::Mud => StampKey::Mud,
            GroundMaterial::Rock => StampKey::Rock,
            GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall => StampKey::Trench,
            GroundMaterial::BermTop | GroundMaterial::BermFace => StampKey::Berm,
            GroundMaterial::Grass => StampKey::Grass,
        },
    }
}

fn hash_stamp(x: u32, y: u32, salt: u32) -> u32 {
    let mut v = salt ^ x.wrapping_mul(0x9e37_79b1) ^ y.wrapping_mul(0x85eb_ca6b);
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^ (v >> 16)
}
