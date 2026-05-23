use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroundMaterial {
    Grass,
    Dirt,
    Mud,
    Rock,
    TrenchFloor,
    TrenchWall,
    BermTop,
    BermFace,
}

impl GroundMaterial {
    pub const ALL: [GroundMaterial; 8] = [
        GroundMaterial::Grass,
        GroundMaterial::Dirt,
        GroundMaterial::Mud,
        GroundMaterial::Rock,
        GroundMaterial::TrenchFloor,
        GroundMaterial::TrenchWall,
        GroundMaterial::BermTop,
        GroundMaterial::BermFace,
    ];

    pub fn id(self) -> &'static str {
        match self {
            GroundMaterial::Grass => "grass",
            GroundMaterial::Dirt => "dirt",
            GroundMaterial::Mud => "mud",
            GroundMaterial::Rock => "rock",
            GroundMaterial::TrenchFloor => "trench_floor",
            GroundMaterial::TrenchWall => "trench_wall",
            GroundMaterial::BermTop => "berm_top",
            GroundMaterial::BermFace => "berm_face",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            GroundMaterial::Grass => "Grass",
            GroundMaterial::Dirt => "Packed dirt",
            GroundMaterial::Mud => "Mud",
            GroundMaterial::Rock => "Rock",
            GroundMaterial::TrenchFloor => "Trench floor",
            GroundMaterial::TrenchWall => "Trench wall",
            GroundMaterial::BermTop => "Berm top",
            GroundMaterial::BermFace => "Berm face",
        }
    }

    pub fn ramp(self) -> &'static str {
        match self {
            GroundMaterial::Grass => "grass",
            GroundMaterial::Dirt => "dirt",
            GroundMaterial::Mud => "mud",
            GroundMaterial::Rock => "rock",
            GroundMaterial::TrenchFloor => "trench_floor",
            GroundMaterial::TrenchWall => "trench_wall",
            GroundMaterial::BermTop => "berm_top",
            GroundMaterial::BermFace => "berm_face",
        }
    }

    pub fn base_movement_cost(self) -> f32 {
        match self {
            GroundMaterial::Grass => 1.1,
            GroundMaterial::Dirt => 1.0,
            GroundMaterial::Mud => 2.25,
            GroundMaterial::Rock => 1.45,
            GroundMaterial::TrenchFloor => 1.8,
            GroundMaterial::TrenchWall => 2.6,
            GroundMaterial::BermTop => 1.3,
            GroundMaterial::BermFace => 2.2,
        }
    }
}

impl fmt::Display for GroundMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileRole {
    Surface,
    Transition,
    StructureFace,
}

impl TileRole {
    pub fn label(self) -> &'static str {
        match self {
            TileRole::Surface => "surface",
            TileRole::Transition => "transition",
            TileRole::StructureFace => "structure_face",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionEdge {
    North,
    South,
    East,
    West,
}

impl TransitionEdge {
    pub const ALL: [TransitionEdge; 4] = [
        TransitionEdge::North,
        TransitionEdge::South,
        TransitionEdge::East,
        TransitionEdge::West,
    ];

    pub fn id(self) -> &'static str {
        match self {
            TransitionEdge::North => "north",
            TransitionEdge::South => "south",
            TransitionEdge::East => "east",
            TransitionEdge::West => "west",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TransitionEdge::North => "north edge",
            TransitionEdge::South => "south edge",
            TransitionEdge::East => "east edge",
            TransitionEdge::West => "west edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LightDirection {
    Northwest,
    North,
    Northeast,
    West,
}

impl LightDirection {
    pub const ALL: [LightDirection; 4] = [
        LightDirection::Northwest,
        LightDirection::North,
        LightDirection::Northeast,
        LightDirection::West,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LightDirection::Northwest => "Northwest",
            LightDirection::North => "North",
            LightDirection::Northeast => "Northeast",
            LightDirection::West => "West",
        }
    }

    pub fn vector(self) -> (f32, f32) {
        match self {
            LightDirection::Northwest => (-0.7, -0.7),
            LightDirection::North => (0.0, -1.0),
            LightDirection::Northeast => (0.7, -0.7),
            LightDirection::West => (-1.0, 0.0),
        }
    }
}

impl fmt::Display for LightDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TilesetRecipe {
    pub id: String,
    pub palette_id: String,
    pub tile_size: u32,
    pub seed: u64,
    pub variants_per_material: u32,
    pub detail_density: f32,
    pub shadow_strength: f32,
    pub highlight_strength: f32,
    pub outline_strength: f32,
    pub light_direction: LightDirection,
    pub generate_transitions: bool,
    pub transition_feather: f32,
    pub mask_strength: f32,
    pub seam_warning_threshold: f32,
}

impl Default for TilesetRecipe {
    fn default() -> Self {
        Self {
            id: "dry_upland_outpost".to_string(),
            palette_id: "muted_field_32".to_string(),
            tile_size: 32,
            seed: 1337,
            variants_per_material: 8,
            detail_density: 0.55,
            shadow_strength: 0.45,
            highlight_strength: 0.28,
            outline_strength: 0.45,
            light_direction: LightDirection::Northwest,
            generate_transitions: true,
            transition_feather: 0.24,
            mask_strength: 0.75,
            seam_warning_threshold: 38.0,
        }
    }
}

impl TilesetRecipe {
    pub fn sanitize(&mut self) {
        self.tile_size = match self.tile_size {
            0..=15 => 16,
            16..=23 => 16,
            24..=31 => 24,
            32..=47 => 32,
            _ => 48,
        };
        self.variants_per_material = self.variants_per_material.clamp(1, 16);
        self.detail_density = self.detail_density.clamp(0.0, 1.0);
        self.shadow_strength = self.shadow_strength.clamp(0.0, 1.0);
        self.highlight_strength = self.highlight_strength.clamp(0.0, 1.0);
        self.outline_strength = self.outline_strength.clamp(0.0, 1.0);
        self.transition_feather = self.transition_feather.clamp(0.05, 0.45);
        self.mask_strength = self.mask_strength.clamp(0.0, 2.0);
        self.seam_warning_threshold = self.seam_warning_threshold.clamp(8.0, 160.0);
        if self.palette_id.trim().is_empty() {
            self.palette_id = "muted_field_32".to_string();
        }
        if self.id.trim().is_empty() {
            self.id = "unnamed_tileset".to_string();
        }
    }
}
