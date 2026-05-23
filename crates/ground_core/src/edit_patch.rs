use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::recipe::GroundMaterial;
use crate::terrain::{CoverKind, TerrainCell, TerrainMap};
use crate::visual_target::{ImageCellRect, VisualTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainEditPatchKind {
    Grass,
    Road,
    Mud,
    Stone,
    Trench,
    Berm,
    Mixed,
}

impl TerrainEditPatchKind {
    pub fn from_cell(cell: &TerrainCell) -> Self {
        if cell.trench_depth > 0
            || matches!(
                cell.ground,
                GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall
            )
        {
            Self::Trench
        } else if cell.berm_height > 0
            || matches!(
                cell.ground,
                GroundMaterial::BermTop | GroundMaterial::BermFace
            )
        {
            Self::Berm
        } else {
            match cell.ground {
                GroundMaterial::Grass => Self::Grass,
                GroundMaterial::Dirt => Self::Road,
                GroundMaterial::Mud => Self::Mud,
                GroundMaterial::Rock => Self::Stone,
                GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall => Self::Trench,
                GroundMaterial::BermTop | GroundMaterial::BermFace => Self::Berm,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainSignature {
    pub height: i8,
    pub ground: GroundMaterial,
    pub trench_depth: u8,
    pub berm_height: u8,
    pub cover: CoverKind,
    pub blocks_sight: bool,
    pub effective_height_centis: i16,
}

impl TerrainSignature {
    pub fn from_cell(cell: &TerrainCell) -> Self {
        Self {
            height: cell.height,
            ground: cell.ground,
            trench_depth: cell.trench_depth,
            berm_height: cell.berm_height,
            cover: cell.cover,
            blocks_sight: cell.blocks_sight,
            effective_height_centis: (cell.effective_height() * 100.0).round() as i16,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainCellChange {
    pub cell: (u32, u32),
    pub old: Option<TerrainSignature>,
    pub new: Option<TerrainSignature>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PatchRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PatchRect {
    pub fn expanded(self, amount: i32, max_width: u32, max_height: u32) -> Self {
        let x0 = (self.x - amount).max(0);
        let y0 = (self.y - amount).max(0);
        let x1 = (self.x + self.width as i32 + amount).clamp(0, max_width as i32);
        let y1 = (self.y + self.height as i32 + amount).clamp(0, max_height as i32);
        Self {
            x: x0,
            y: y0,
            width: (x1 - x0).max(1) as u32,
            height: (y1 - y0).max(1) as u32,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainEditPatch {
    pub id: String,
    pub kind: TerrainEditPatchKind,
    pub cells: Vec<(u32, u32)>,
    pub neighbor_cells: Vec<(u32, u32)>,
    pub bounds_px: PatchRect,
    pub old_signature: Option<TerrainSignature>,
    pub new_signature: Option<TerrainSignature>,
    pub changes: Vec<TerrainCellChange>,
}

pub fn build_edit_patches(
    map: &TerrainMap,
    baseline: &TerrainMap,
    target: &VisualTarget,
) -> Vec<TerrainEditPatch> {
    let max_width = map.width.min(target.spec.map_size_cells.0);
    let max_height = map.height.min(target.spec.map_size_cells.1);
    let mut dirty = HashSet::new();

    for y in 0..max_height {
        for x in 0..max_width {
            let current = map.cell(x, y).map(TerrainSignature::from_cell);
            let base = baseline.cell(x, y).map(TerrainSignature::from_cell);
            if current != base {
                dirty.insert((x, y));
            }
        }
    }

    let mut remaining = dirty.clone();
    let mut patches = Vec::new();

    while let Some(&seed_cell) = remaining.iter().next() {
        remaining.remove(&seed_cell);
        let seed_kind = map
            .cell(seed_cell.0, seed_cell.1)
            .map(TerrainEditPatchKind::from_cell)
            .unwrap_or(TerrainEditPatchKind::Mixed);

        let mut queue = VecDeque::from([seed_cell]);
        let mut cells = Vec::new();
        cells.push(seed_cell);

        while let Some((x, y)) = queue.pop_front() {
            for (nx, ny) in neighbors4_in_bounds(x, y, max_width, max_height) {
                if !remaining.contains(&(nx, ny)) {
                    continue;
                }
                let next_kind = map
                    .cell(nx, ny)
                    .map(TerrainEditPatchKind::from_cell)
                    .unwrap_or(TerrainEditPatchKind::Mixed);
                if next_kind != seed_kind {
                    continue;
                }
                remaining.remove(&(nx, ny));
                cells.push((nx, ny));
                queue.push_back((nx, ny));
            }
        }

        cells.sort_unstable();
        let cell_set: HashSet<(u32, u32)> = cells.iter().copied().collect();
        let mut neighbor_cells = HashSet::new();
        for &(x, y) in &cells {
            for neighbor in neighbors4_in_bounds(x, y, max_width, max_height) {
                if !cell_set.contains(&neighbor) {
                    neighbor_cells.insert(neighbor);
                }
            }
        }
        let mut neighbor_cells: Vec<_> = neighbor_cells.into_iter().collect();
        neighbor_cells.sort_unstable();

        let bounds_px = patch_bounds(&cells, target)
            .unwrap_or(PatchRect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            })
            .expanded(18, target.image.width, target.image.height);

        let changes: Vec<_> = cells
            .iter()
            .map(|&(x, y)| TerrainCellChange {
                cell: (x, y),
                old: baseline.cell(x, y).map(TerrainSignature::from_cell),
                new: map.cell(x, y).map(TerrainSignature::from_cell),
            })
            .collect();

        let old_signature = changes.first().and_then(|change| change.old);
        let new_signature = changes.first().and_then(|change| change.new);
        patches.push(TerrainEditPatch {
            id: format!(
                "edit_patch_{:02}_{:02}_{}",
                seed_cell.0,
                seed_cell.1,
                patches.len()
            ),
            kind: seed_kind,
            cells,
            neighbor_cells,
            bounds_px,
            old_signature,
            new_signature,
            changes,
        });
    }

    patches.sort_by_key(|patch| {
        (
            patch
                .cells
                .iter()
                .map(|cell| cell.1)
                .min()
                .unwrap_or_default(),
            patch
                .cells
                .iter()
                .map(|cell| cell.0)
                .min()
                .unwrap_or_default(),
        )
    });
    patches
}

fn neighbors4_in_bounds(x: u32, y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    if x + 1 < width {
        out.push((x + 1, y));
    }
    if y + 1 < height {
        out.push((x, y + 1));
    }
    out
}

fn patch_bounds(cells: &[(u32, u32)], target: &VisualTarget) -> Option<PatchRect> {
    let mut x0 = i32::MAX;
    let mut y0 = i32::MAX;
    let mut x1 = i32::MIN;
    let mut y1 = i32::MIN;

    for &cell in cells {
        let ImageCellRect {
            x,
            y,
            width,
            height,
        } = target.cell_rect(cell)?;
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + width as i32);
        y1 = y1.max(y + height as i32);
    }

    Some(PatchRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(1) as u32,
        height: (y1 - y0).max(1) as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba8;
    use crate::terrain::{Brush, BrushKind};
    use crate::visual_target::VisualTargetSpec;

    fn test_target() -> VisualTarget {
        VisualTarget {
            spec: VisualTargetSpec {
                id: "test".to_string(),
                image: "test.png".to_string(),
                image_size_px: (320, 240),
                map_size_cells: (16, 12),
                grid_origin_px: (0, 0),
                cell_size_px: (20, 20),
                spawn_cell: (1, 7),
                objective_cell: (12, 3),
                light_direction: "northwest".to_string(),
                notes: "test target".to_string(),
            },
            image: crate::pixel_image::PixelImage::new(320, 240, Rgba8::BLACK),
        }
    }

    #[test]
    fn baseline_has_no_edit_patches() {
        let target = test_target();
        let map = TerrainMap::target_derived(16, 12, 0);
        let baseline = TerrainMap::target_derived(16, 12, 0);
        assert!(build_edit_patches(&map, &baseline, &target).is_empty());
    }

    #[test]
    fn brush_change_builds_patch_with_neighbor_context() {
        let target = test_target();
        let baseline = TerrainMap::target_derived(16, 12, 0);
        let mut map = baseline.clone();
        map.apply_brush(0, 0, Brush::new(BrushKind::DigTrench, 1, 1));

        let patches = build_edit_patches(&map, &baseline, &target);
        assert!(!patches.is_empty());
        assert!(patches
            .iter()
            .any(|patch| patch.kind == TerrainEditPatchKind::Trench));
        assert!(patches.iter().any(|patch| !patch.neighbor_cells.is_empty()));
        assert!(patches
            .iter()
            .flat_map(|patch| patch.changes.iter())
            .any(|change| change.old != change.new));
    }
}
