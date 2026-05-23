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
    /// Erected/extruded terrain body tile used for visible vertical faces,
    /// trench walls, berm sides, cliffs, and terrain cutaway/cross-section art.
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
pub enum StructureFaceKind {
    Front,
    Left,
    Right,
    Lip,
}

impl StructureFaceKind {
    pub const ALL: [StructureFaceKind; 4] = [
        StructureFaceKind::Front,
        StructureFaceKind::Left,
        StructureFaceKind::Right,
        StructureFaceKind::Lip,
    ];

    pub fn id(self) -> &'static str {
        match self {
            StructureFaceKind::Front => "front",
            StructureFaceKind::Left => "left",
            StructureFaceKind::Right => "right",
            StructureFaceKind::Lip => "lip",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StructureFaceKind::Front => "front / south face",
            StructureFaceKind::Left => "left side face",
            StructureFaceKind::Right => "right side face",
            StructureFaceKind::Lip => "top lip / cut edge",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectionKind {
    /// Legacy square-map/debug view. Kept for command maps, masks, and schematic overlays.
    SquareTopDown,
    /// Screen-aligned 2D terrain whose sprite stack implies depth, height, and perspective.
    FauxPerspective2D,
    /// Angled tactical 2.5D projection with diamond top footprints and visible terrain body.
    Dimetric,
}

impl ProjectionKind {
    pub fn label(self) -> &'static str {
        match self {
            ProjectionKind::SquareTopDown => "square top-down",
            ProjectionKind::FauxPerspective2D => "faux-perspective 2D",
            ProjectionKind::Dimetric => "angled dimetric",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewOrientation {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

impl ViewOrientation {
    pub const ALL: [ViewOrientation; 4] = [
        ViewOrientation::NorthEast,
        ViewOrientation::SouthEast,
        ViewOrientation::SouthWest,
        ViewOrientation::NorthWest,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ViewOrientation::NorthEast => "NE view",
            ViewOrientation::SouthEast => "SE view",
            ViewOrientation::SouthWest => "SW view",
            ViewOrientation::NorthWest => "NW view",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            ViewOrientation::NorthEast => "ne",
            ViewOrientation::SouthEast => "se",
            ViewOrientation::SouthWest => "sw",
            ViewOrientation::NorthWest => "nw",
        }
    }

    pub fn rotate_cw(self) -> Self {
        match self {
            ViewOrientation::NorthEast => ViewOrientation::SouthEast,
            ViewOrientation::SouthEast => ViewOrientation::SouthWest,
            ViewOrientation::SouthWest => ViewOrientation::NorthWest,
            ViewOrientation::NorthWest => ViewOrientation::NorthEast,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            ViewOrientation::NorthEast => ViewOrientation::NorthWest,
            ViewOrientation::SouthEast => ViewOrientation::NorthEast,
            ViewOrientation::SouthWest => ViewOrientation::SouthEast,
            ViewOrientation::NorthWest => ViewOrientation::SouthWest,
        }
    }
}

impl fmt::Display for ViewOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectionSpec {
    pub kind: ProjectionKind,
    /// Source art size in pixels. For now this is kept in sync with tile_size so
    /// generated material tiles can be projected into larger screen footprints.
    pub source_tile_px: u32,
    /// Diamond footprint width on screen in angled previews.
    pub tile_screen_width_px: u32,
    /// Diamond footprint height on screen in angled previews.
    pub tile_screen_height_px: u32,
    /// Screen-aligned faux-perspective cell width. This is the main-view footprint.
    pub faux_cell_width_px: u32,
    /// Screen-aligned faux-perspective cell height. This keeps the camera directly overhead.
    pub faux_cell_height_px: u32,
    /// Visual lift per effective terrain-height unit in faux-perspective previews.
    pub faux_height_step_px: u32,
    /// Side-face strip width for visible left/right height hints.
    pub faux_side_face_width_px: u32,
    /// Visual lift per effective terrain-height unit in angled previews.
    pub height_step_px: u32,
    pub default_orientation: ViewOrientation,
    pub supports_four_way_rotation: bool,
}

impl Default for ProjectionSpec {
    fn default() -> Self {
        Self {
            kind: ProjectionKind::FauxPerspective2D,
            source_tile_px: 64,
            tile_screen_width_px: 96,
            tile_screen_height_px: 48,
            faux_cell_width_px: 96,
            faux_cell_height_px: 80,
            faux_height_step_px: 32,
            faux_side_face_width_px: 20,
            height_step_px: 24,
            default_orientation: ViewOrientation::SouthEast,
            supports_four_way_rotation: true,
        }
    }
}

impl ProjectionSpec {
    pub fn sanitize(&mut self, tile_size: u32) {
        self.source_tile_px = tile_size.max(16);
        self.tile_screen_width_px = sanitize_screen_dim(self.tile_screen_width_px, 48, 192);
        self.tile_screen_height_px = sanitize_screen_dim(self.tile_screen_height_px, 24, 128);
        if self.tile_screen_width_px < self.tile_screen_height_px {
            self.tile_screen_width_px = (self.tile_screen_height_px * 2).min(192);
        }
        self.faux_cell_width_px = sanitize_screen_dim(self.faux_cell_width_px, 32, 160);
        self.faux_cell_height_px = sanitize_screen_dim(self.faux_cell_height_px, 32, 160);
        self.faux_height_step_px = self.faux_height_step_px.clamp(4, 96);
        self.faux_side_face_width_px = self.faux_side_face_width_px.clamp(2, 48);
        self.height_step_px = self.height_step_px.clamp(4, 96);
    }
}

fn sanitize_screen_dim(value: u32, min: u32, max: u32) -> u32 {
    let rounded = value.clamp(min, max).div_ceil(4) * 4;
    rounded.clamp(min, max)
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
    pub generate_structure_faces: bool,
    pub face_shadow_strength: f32,
    pub face_lip_strength: f32,
    pub face_detail_density: f32,
    pub cutaway_alpha: f32,
    pub cutaway_radius_px: u32,
    pub projection: ProjectionSpec,
}

impl Default for TilesetRecipe {
    fn default() -> Self {
        Self {
            id: "dry_upland_outpost".to_string(),
            palette_id: "muted_field_32".to_string(),
            tile_size: 64,
            seed: 1337,
            variants_per_material: 8,
            detail_density: 0.55,
            shadow_strength: 0.50,
            highlight_strength: 0.28,
            outline_strength: 0.45,
            light_direction: LightDirection::Northwest,
            generate_transitions: true,
            transition_feather: 0.24,
            mask_strength: 0.75,
            seam_warning_threshold: 38.0,
            generate_structure_faces: true,
            face_shadow_strength: 0.72,
            face_lip_strength: 0.56,
            face_detail_density: 0.70,
            cutaway_alpha: 0.42,
            cutaway_radius_px: 128,
            projection: ProjectionSpec::default(),
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
            48..=63 => 48,
            64..=95 => 64,
            _ => 96,
        };
        self.variants_per_material = self.variants_per_material.clamp(1, 16);
        self.detail_density = self.detail_density.clamp(0.0, 1.0);
        self.shadow_strength = self.shadow_strength.clamp(0.0, 1.0);
        self.highlight_strength = self.highlight_strength.clamp(0.0, 1.0);
        self.outline_strength = self.outline_strength.clamp(0.0, 1.0);
        self.transition_feather = self.transition_feather.clamp(0.05, 0.45);
        self.mask_strength = self.mask_strength.clamp(0.0, 2.0);
        self.seam_warning_threshold = self.seam_warning_threshold.clamp(8.0, 160.0);
        self.face_shadow_strength = self.face_shadow_strength.clamp(0.0, 1.0);
        self.face_lip_strength = self.face_lip_strength.clamp(0.0, 1.0);
        self.face_detail_density = self.face_detail_density.clamp(0.0, 1.0);
        self.cutaway_alpha = self.cutaway_alpha.clamp(0.15, 1.0);
        self.cutaway_radius_px = self.cutaway_radius_px.clamp(16, 384);
        self.projection.sanitize(self.tile_size);
        if self.palette_id.trim().is_empty() {
            self.palette_id = "muted_field_32".to_string();
        }
        if self.id.trim().is_empty() {
            self.id = "unnamed_tileset".to_string();
        }
    }
}
