use serde::{Deserialize, Serialize};

use crate::edit_patch::{
    build_edit_patches, summarize_edit_patches, TerrainEditPatch, TerrainEditPatchMetrics,
};
use crate::recipe::GroundMaterial;
use crate::terrain::{CoverKind, TerrainMap};
use crate::visual_target::VisualTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditScenarioId {
    NewTrench,
    NewBerm,
    NewRoad,
    RemoveTrench,
    RemoveRoad,
    FlattenTrench,
    PaintStone,
}

impl EditScenarioId {
    pub fn file_id(self) -> &'static str {
        match self {
            Self::NewTrench => "new_trench",
            Self::NewBerm => "new_berm",
            Self::NewRoad => "new_road",
            Self::RemoveTrench => "remove_trench",
            Self::RemoveRoad => "remove_road",
            Self::FlattenTrench => "flatten_trench",
            Self::PaintStone => "paint_stone",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NewTrench => "Dig a new trench through grass",
            Self::NewBerm => "Raise a berm across the road",
            Self::NewRoad => "Paint a new dirt path through grass",
            Self::RemoveTrench => "Cover an existing trench with grass",
            Self::RemoveRoad => "Convert baked road cells back to grass",
            Self::FlattenTrench => "Flatten part of an existing trench",
            Self::PaintStone => "Paint stone over grass",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditScenarioReport {
    pub id: EditScenarioId,
    pub file_id: String,
    pub label: String,
    pub patch_metrics: TerrainEditPatchMetrics,
    pub patches: Vec<TerrainEditPatch>,
}

pub const EDIT_SCENARIOS: [EditScenarioId; 7] = [
    EditScenarioId::NewTrench,
    EditScenarioId::NewBerm,
    EditScenarioId::NewRoad,
    EditScenarioId::RemoveTrench,
    EditScenarioId::RemoveRoad,
    EditScenarioId::FlattenTrench,
    EditScenarioId::PaintStone,
];

pub fn apply_edit_scenario(base: &TerrainMap, id: EditScenarioId) -> TerrainMap {
    let mut map = base.clone();
    match id {
        EditScenarioId::NewTrench => {
            for cell in [(7, 2), (8, 2), (9, 2), (10, 2)] {
                dig_trench_cell(&mut map, cell.0, cell.1, 2);
            }
        }
        EditScenarioId::NewBerm => {
            for cell in [(8, 5), (9, 5), (10, 5), (11, 5)] {
                raise_berm_cell(&mut map, cell.0, cell.1, 2);
            }
        }
        EditScenarioId::NewRoad => {
            for cell in [(1, 9), (2, 9), (3, 9), (4, 9), (5, 9)] {
                paint_cell(&mut map, cell.0, cell.1, GroundMaterial::Dirt);
            }
        }
        EditScenarioId::RemoveTrench => {
            for cell in [(4, 7), (5, 7), (6, 7), (7, 7)] {
                paint_cell(&mut map, cell.0, cell.1, GroundMaterial::Grass);
            }
        }
        EditScenarioId::RemoveRoad => {
            for cell in [(5, 5), (6, 5), (7, 5), (8, 5), (9, 5)] {
                paint_cell(&mut map, cell.0, cell.1, GroundMaterial::Grass);
            }
        }
        EditScenarioId::FlattenTrench => {
            for cell in [(8, 8), (8, 9), (7, 9)] {
                flatten_cell(&mut map, cell.0, cell.1, GroundMaterial::Dirt);
            }
        }
        EditScenarioId::PaintStone => {
            for cell in [(2, 8), (3, 8), (2, 9), (3, 9)] {
                paint_cell(&mut map, cell.0, cell.1, GroundMaterial::Rock);
            }
        }
    }
    map
}

pub fn build_edit_scenario_report(
    id: EditScenarioId,
    edited: &TerrainMap,
    baseline: &TerrainMap,
    target: &VisualTarget,
) -> EditScenarioReport {
    let patches = build_edit_patches(edited, baseline, target);
    let patch_metrics = summarize_edit_patches(&patches);
    EditScenarioReport {
        id,
        file_id: id.file_id().to_string(),
        label: id.label().to_string(),
        patch_metrics,
        patches,
    }
}

fn paint_cell(map: &mut TerrainMap, x: u32, y: u32, material: GroundMaterial) {
    let Some(cell) = map.cell_mut(x, y) else {
        return;
    };
    cell.ground = material;
    cell.trench_depth = 0;
    cell.berm_height = 0;
    match material {
        GroundMaterial::TrenchFloor => {
            cell.cover = CoverKind::Strong;
            cell.blocks_sight = false;
        }
        GroundMaterial::TrenchWall | GroundMaterial::BermFace => {
            cell.cover = CoverKind::Partial;
            cell.blocks_sight = true;
        }
        GroundMaterial::BermTop => {
            cell.cover = CoverKind::Partial;
            cell.blocks_sight = false;
        }
        _ => {
            cell.cover = CoverKind::None;
            cell.blocks_sight = false;
        }
    }
}

fn dig_trench_cell(map: &mut TerrainMap, x: u32, y: u32, depth: u8) {
    let Some(cell) = map.cell_mut(x, y) else {
        return;
    };
    let depth = depth.clamp(1, 4);
    cell.height = (cell.height - depth as i8).clamp(0, 9);
    cell.ground = GroundMaterial::TrenchFloor;
    cell.trench_depth = depth;
    cell.berm_height = 0;
    cell.cover = CoverKind::Strong;
    cell.blocks_sight = false;
}

fn raise_berm_cell(map: &mut TerrainMap, x: u32, y: u32, lift: u8) {
    let Some(cell) = map.cell_mut(x, y) else {
        return;
    };
    let lift = lift.clamp(1, 4);
    cell.height = (cell.height + lift as i8).clamp(0, 9);
    cell.ground = GroundMaterial::BermTop;
    cell.trench_depth = 0;
    cell.berm_height = lift;
    cell.cover = CoverKind::Partial;
    cell.blocks_sight = lift >= 2;
}

fn flatten_cell(map: &mut TerrainMap, x: u32, y: u32, material: GroundMaterial) {
    let new_height = {
        let mut total = 0_i32;
        let mut count = 0_i32;
        for (nx, ny) in map.neighbors4(x, y) {
            total += map.height_at(nx, ny) as i32;
            count += 1;
        }
        if count == 0 {
            map.height_at(x, y)
        } else {
            (total as f32 / count as f32).round().clamp(0.0, 9.0) as i8
        }
    };
    let Some(cell) = map.cell_mut(x, y) else {
        return;
    };
    cell.height = new_height;
    paint_cell(map, x, y, material);
}
