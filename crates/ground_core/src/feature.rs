use serde::{Deserialize, Serialize};

use crate::recipe::GroundMaterial;
use crate::terrain::{TerrainCell, TerrainMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainFeatureKind {
    Open,
    Plateau,
    Ledge,
    Ditch,
    Trench,
    Berm,
}

impl TerrainFeatureKind {
    pub fn label(self) -> &'static str {
        match self {
            TerrainFeatureKind::Open => "open",
            TerrainFeatureKind::Plateau => "plateau",
            TerrainFeatureKind::Ledge => "ledge",
            TerrainFeatureKind::Ditch => "ditch",
            TerrainFeatureKind::Trench => "trench",
            TerrainFeatureKind::Berm => "berm",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardinalDir {
    North,
    South,
    East,
    West,
}

impl CardinalDir {
    pub const ALL: [CardinalDir; 4] = [
        CardinalDir::North,
        CardinalDir::South,
        CardinalDir::East,
        CardinalDir::West,
    ];

    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(CardinalDir::North),
            (0, 1) => Some(CardinalDir::South),
            (1, 0) => Some(CardinalDir::East),
            (-1, 0) => Some(CardinalDir::West),
            _ => None,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            CardinalDir::North => (0, -1),
            CardinalDir::South => (0, 1),
            CardinalDir::East => (1, 0),
            CardinalDir::West => (-1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeMask {
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}

impl EdgeMask {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(self, dir: CardinalDir) -> bool {
        match dir {
            CardinalDir::North => self.north,
            CardinalDir::South => self.south,
            CardinalDir::East => self.east,
            CardinalDir::West => self.west,
        }
    }

    pub fn set(&mut self, dir: CardinalDir, value: bool) {
        match dir {
            CardinalDir::North => self.north = value,
            CardinalDir::South => self.south = value,
            CardinalDir::East => self.east = value,
            CardinalDir::West => self.west = value,
        }
    }

    pub fn any(self) -> bool {
        self.north || self.south || self.east || self.west
    }

    pub fn count(self) -> u8 {
        self.north as u8 + self.south as u8 + self.east as u8 + self.west as u8
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TerrainFeatureCell {
    pub kind: TerrainFeatureKind,
    pub material: GroundMaterial,
    pub visual_material: GroundMaterial,
    pub effective_height: f32,
    pub material_edges: EdgeMask,
    pub ledge_edges: EdgeMask,
    pub trench_edges: EdgeMask,
    pub berm_edges: EdgeMask,
}

impl TerrainFeatureCell {
    pub fn has_structural_edge(self) -> bool {
        self.ledge_edges.any() || self.trench_edges.any() || self.berm_edges.any()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainFeatureMap {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<TerrainFeatureCell>,
}

impl TerrainFeatureMap {
    pub fn from_terrain(map: &TerrainMap) -> Self {
        let mut cells = Vec::with_capacity(map.cells.len());
        for y in 0..map.height {
            for x in 0..map.width {
                let Some(cell) = map.cell(x, y) else {
                    continue;
                };
                cells.push(derive_feature_cell(map, x, y, cell));
            }
        }
        Self {
            width: map.width,
            height: map.height,
            cells,
        }
    }

    pub fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    pub fn cell(&self, x: u32, y: u32) -> Option<&TerrainFeatureCell> {
        self.index(x, y).map(|idx| &self.cells[idx])
    }

    pub fn structural_edge_count(&self) -> usize {
        self.cells
            .iter()
            .map(|cell| {
                cell.ledge_edges.count() as usize
                    + cell.trench_edges.count() as usize
                    + cell.berm_edges.count() as usize
            })
            .sum()
    }

    pub fn material_edge_count(&self) -> usize {
        self.cells
            .iter()
            .map(|cell| cell.material_edges.count() as usize)
            .sum()
    }
}

fn derive_feature_cell(map: &TerrainMap, x: u32, y: u32, cell: &TerrainCell) -> TerrainFeatureCell {
    let visual_material = feature_visual_material(cell.ground);
    let effective_height = cell.effective_height();
    let kind = if cell.trench_depth >= 2 || matches!(cell.ground, GroundMaterial::TrenchFloor) {
        TerrainFeatureKind::Trench
    } else if cell.berm_height > 0 || matches!(cell.ground, GroundMaterial::BermTop) {
        TerrainFeatureKind::Berm
    } else if cell.trench_depth > 0 {
        TerrainFeatureKind::Ditch
    } else {
        TerrainFeatureKind::Open
    };

    let mut material_edges = EdgeMask::empty();
    let mut ledge_edges = EdgeMask::empty();
    let mut trench_edges = EdgeMask::empty();
    let mut berm_edges = EdgeMask::empty();

    for dir in CardinalDir::ALL {
        let (dx, dy) = dir.delta();
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        let neighbor = if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
            None
        } else {
            map.cell(nx as u32, ny as u32)
        };

        let neighbor_height = neighbor.map(TerrainCell::effective_height).unwrap_or(0.0);
        if effective_height - neighbor_height > 0.15 {
            ledge_edges.set(dir, true);
        }

        let neighbor_visual = neighbor.map(|n| feature_visual_material(n.ground));
        if neighbor_visual != Some(visual_material) {
            material_edges.set(dir, true);
        }

        let neighbor_trench = neighbor
            .map(|n| n.trench_depth > 0 || matches!(n.ground, GroundMaterial::TrenchFloor))
            .unwrap_or(false);
        if matches!(kind, TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch)
            && (!neighbor_trench || neighbor_height > effective_height + 0.15)
        {
            trench_edges.set(dir, true);
        }

        let neighbor_berm = neighbor
            .map(|n| n.berm_height > 0 || matches!(n.ground, GroundMaterial::BermTop))
            .unwrap_or(false);
        if matches!(kind, TerrainFeatureKind::Berm)
            && (!neighbor_berm || effective_height > neighbor_height + 0.15)
        {
            berm_edges.set(dir, true);
        }
    }

    let kind = if matches!(kind, TerrainFeatureKind::Open) && ledge_edges.any() {
        TerrainFeatureKind::Ledge
    } else if matches!(kind, TerrainFeatureKind::Open) && material_edges.count() <= 1 {
        TerrainFeatureKind::Plateau
    } else {
        kind
    };

    TerrainFeatureCell {
        kind,
        material: cell.ground,
        visual_material,
        effective_height,
        material_edges,
        ledge_edges,
        trench_edges,
        berm_edges,
    }
}

pub fn feature_visual_material(material: GroundMaterial) -> GroundMaterial {
    match material {
        GroundMaterial::TrenchWall => GroundMaterial::TrenchFloor,
        GroundMaterial::BermFace => GroundMaterial::BermTop,
        other => other,
    }
}
