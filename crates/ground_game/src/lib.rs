use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use ground_core::{
    generate_effective_terrain_sprites, GeneratedTerrainSprite, PixelImage, Rgba8,
    TerrainSpriteKind, TerrainSpriteRecipe, DEFAULT_SPRITE_STYLE_PATH,
};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MISSION_EXPORT_DIR: &str = "exports/gamepivot_05_1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellCoord {
    pub x: u32,
    pub y: u32,
}

impl CellCoord {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn manhattan(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellRect {
    pub origin: CellCoord,
    pub width: u32,
    pub height: u32,
}

impl CellRect {
    pub const fn single(cell: CellCoord) -> Self {
        Self {
            origin: cell,
            width: 1,
            height: 1,
        }
    }

    pub fn cells(self) -> impl Iterator<Item = CellCoord> {
        let x0 = self.origin.x;
        let y0 = self.origin.y;
        let width = self.width;
        let height = self.height;
        (0..height).flat_map(move |dy| {
            (0..width).map(move |dx| CellCoord {
                x: x0 + dx,
                y: y0 + dy,
            })
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionSpec {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub briefing: MissionBriefing,
    #[serde(default)]
    pub visual_theme: MissionVisualTheme,
    pub objective: MissionObjective,
    pub prep_time_seconds: u32,
    pub map: MissionMap,
    pub starting_tools: ToolLoadout,
    pub crew: CrewPool,
    pub enemy_groups: Vec<EnemyGroupSpec>,
    #[serde(default)]
    pub defender_positions: Vec<DefenderPositionSpec>,
    pub constraints: MissionConstraints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionVisualTheme {
    pub sprite_style_profile: String,
    pub render_projection: MissionRenderProjection,
}

impl Default for MissionVisualTheme {
    fn default() -> Self {
        Self {
            sprite_style_profile: DEFAULT_SPRITE_STYLE_PATH.to_string(),
            render_projection: MissionRenderProjection::HighOblique2D,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionRenderProjection {
    HighOblique2D,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MissionBriefing {
    pub summary: String,
    pub primary: String,
    pub optional_objectives: Vec<String>,
    pub intel: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionObjective {
    pub label: String,
    pub defend_cell: CellCoord,
    pub objective_health: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefenderPositionSpec {
    pub id: String,
    pub label: String,
    pub cell: CellCoord,
    pub range: u32,
    pub pressure_per_step: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionMap {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MissionCell>,
    pub objects: Vec<EnvironmentObject>,
    pub spawn_cells: Vec<CellCoord>,
}

impl MissionMap {
    pub fn new(width: u32, height: u32, fill: MissionCell) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width as usize * height as usize],
            objects: Vec::new(),
            spawn_cells: Vec::new(),
        }
    }

    pub fn index(&self, cell: CellCoord) -> Option<usize> {
        if cell.x < self.width && cell.y < self.height {
            Some((cell.y * self.width + cell.x) as usize)
        } else {
            None
        }
    }

    pub fn cell(&self, cell: CellCoord) -> Option<&MissionCell> {
        self.index(cell).and_then(|idx| self.cells.get(idx))
    }

    pub fn cell_mut(&mut self, cell: CellCoord) -> Option<&mut MissionCell> {
        self.index(cell).and_then(|idx| self.cells.get_mut(idx))
    }

    pub fn object_at_mut(&mut self, object_id: &str) -> Option<&mut EnvironmentObject> {
        self.objects
            .iter_mut()
            .find(|object| object.id == object_id)
    }

    pub fn object_at_cell_mut(
        &mut self,
        cell: CellCoord,
        predicate: impl Fn(&EnvironmentObjectKind) -> bool,
    ) -> Option<&mut EnvironmentObject> {
        self.objects
            .iter_mut()
            .find(|object| object.cell == cell && predicate(&object.kind))
    }

    pub fn objects_at_cell(&self, cell: CellCoord) -> impl Iterator<Item = &EnvironmentObject> {
        self.objects
            .iter()
            .filter(move |object| object.cell == cell)
    }

    pub fn neighbors4(&self, cell: CellCoord) -> Vec<CellCoord> {
        let mut out = Vec::with_capacity(4);
        if cell.y > 0 {
            out.push(CellCoord::new(cell.x, cell.y - 1));
        }
        if cell.x + 1 < self.width {
            out.push(CellCoord::new(cell.x + 1, cell.y));
        }
        if cell.y + 1 < self.height {
            out.push(CellCoord::new(cell.x, cell.y + 1));
        }
        if cell.x > 0 {
            out.push(CellCoord::new(cell.x - 1, cell.y));
        }
        out
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionCell {
    pub height: i8,
    pub ground: GroundKind,
    pub earth_state: EarthState,
    pub cover: CoverClass,
    pub movement_cost: f32,
    pub blocks_sight: bool,
    pub local_material: LocalMaterialStock,
}

impl MissionCell {
    pub fn new(height: i8, ground: GroundKind) -> Self {
        Self {
            height,
            ground,
            earth_state: EarthState::Normal,
            cover: CoverClass::None,
            movement_cost: ground.base_movement_cost(),
            blocks_sight: false,
            local_material: LocalMaterialStock::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundKind {
    Grass,
    Dirt,
    Mud,
    Rock,
    Road,
}

impl GroundKind {
    pub fn base_movement_cost(self) -> f32 {
        match self {
            GroundKind::Road => 0.85,
            GroundKind::Grass => 1.0,
            GroundKind::Dirt => 1.05,
            GroundKind::Rock => 1.35,
            GroundKind::Mud => 1.8,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GroundKind::Grass => "grass",
            GroundKind::Dirt => "dirt",
            GroundKind::Mud => "mud",
            GroundKind::Rock => "rock",
            GroundKind::Road => "road",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EarthState {
    Normal,
    Scraped,
    Ditch,
    Trench,
    DeepTrench,
    SpoilPile,
    Berm,
    Unstable,
    Muddy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverClass {
    None,
    Light,
    Partial,
    Strong,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMaterialStock {
    pub earth_spoil: i32,
    pub timber: i32,
    pub logs: i32,
    pub stakes: i32,
    pub loose_stone: i32,
    pub scrap: i32,
    pub rope_uses: i32,
}

impl LocalMaterialStock {
    pub fn get(&self, kind: LocalMaterialKind) -> i32 {
        match kind {
            LocalMaterialKind::EarthSpoil => self.earth_spoil,
            LocalMaterialKind::Timber => self.timber,
            LocalMaterialKind::Logs => self.logs,
            LocalMaterialKind::Stakes => self.stakes,
            LocalMaterialKind::LooseStone => self.loose_stone,
            LocalMaterialKind::Scrap => self.scrap,
            LocalMaterialKind::RopeUses => self.rope_uses,
        }
    }

    pub fn add(&mut self, kind: LocalMaterialKind, amount: i32) {
        match kind {
            LocalMaterialKind::EarthSpoil => self.earth_spoil += amount,
            LocalMaterialKind::Timber => self.timber += amount,
            LocalMaterialKind::Logs => self.logs += amount,
            LocalMaterialKind::Stakes => self.stakes += amount,
            LocalMaterialKind::LooseStone => self.loose_stone += amount,
            LocalMaterialKind::Scrap => self.scrap += amount,
            LocalMaterialKind::RopeUses => self.rope_uses += amount,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.earth_spoil == 0
            && self.timber == 0
            && self.logs == 0
            && self.stakes == 0
            && self.loose_stone == 0
            && self.scrap == 0
            && self.rope_uses == 0
    }

    pub fn net(outputs: &Self, inputs: &Self) -> Self {
        Self {
            earth_spoil: outputs.earth_spoil - inputs.earth_spoil,
            timber: outputs.timber - inputs.timber,
            logs: outputs.logs - inputs.logs,
            stakes: outputs.stakes - inputs.stakes,
            loose_stone: outputs.loose_stone - inputs.loose_stone,
            scrap: outputs.scrap - inputs.scrap,
            rope_uses: outputs.rope_uses - inputs.rope_uses,
        }
    }

    pub fn signed_summary(&self) -> Vec<String> {
        [
            (self.earth_spoil, "spoil"),
            (self.timber, "timber"),
            (self.logs, "logs"),
            (self.stakes, "stakes"),
            (self.loose_stone, "stone"),
            (self.scrap, "scrap"),
            (self.rope_uses, "rope"),
        ]
        .into_iter()
        .filter(|(value, _)| *value != 0)
        .map(|(value, label)| {
            if value > 0 {
                format!("{label} +{value}")
            } else {
                format!("{label} {value}")
            }
        })
        .collect()
    }

    pub fn positive_summary(&self) -> Vec<String> {
        [
            (LocalMaterialKind::EarthSpoil, "spoil"),
            (LocalMaterialKind::Timber, "timber"),
            (LocalMaterialKind::Logs, "logs"),
            (LocalMaterialKind::Stakes, "stakes"),
            (LocalMaterialKind::LooseStone, "stone"),
            (LocalMaterialKind::Scrap, "scrap"),
            (LocalMaterialKind::RopeUses, "rope"),
        ]
        .into_iter()
        .filter_map(|(kind, label)| {
            let value = self.get(kind);
            (value > 0).then(|| format!("{label}: {value}"))
        })
        .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalMaterialKind {
    EarthSpoil,
    Timber,
    Logs,
    Stakes,
    LooseStone,
    Scrap,
    RopeUses,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentObject {
    pub id: String,
    pub label: String,
    pub kind: EnvironmentObjectKind,
    pub cell: CellCoord,
    pub footprint: (u32, u32),
    pub blocks_sight: bool,
    pub cover: CoverClass,
    pub movement_cost_delta: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnvironmentObjectKind {
    Tree(TreeState),
    Log(LogState),
    Rock(RockState),
    Wall(WallState),
    Wire(ObstacleState),
    Stakes(ObstacleState),
    FightingPosition(PositionState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreeState {
    Standing,
    PartiallyCut { progress: u8 },
    Falling { direction: Direction },
    FallenTrunk { direction: Direction },
    CutLogs,
    StakesBundle,
    Stump,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogState {
    Loose {
        direction: Direction,
    },
    DragPrepared {
        direction: Direction,
    },
    Positioned {
        direction: Direction,
    },
    Braced {
        direction: Direction,
    },
    PreparedRoll {
        direction: Direction,
        release_cell: CellCoord,
        predicted_path: Vec<CellCoord>,
    },
    Released {
        direction: Direction,
    },
    Rolling {
        direction: Direction,
    },
    Spent {
        direction: Direction,
    },
    Piled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RockState {
    Stable,
    Cracked,
    Rubble,
    RollingStone { direction: Direction },
    BlockedRubblePile,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WallState {
    Intact,
    Damaged,
    Breached,
    CollapsedRubble,
    ClearedRubble,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObstacleState {
    Placed,
    Damaged,
    Cleared,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PositionState {
    DugIn,
    Reinforced,
    Collapsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolLoadout {
    pub tools: Vec<ToolKind>,
}

impl ToolLoadout {
    pub fn basic_field_kit() -> Self {
        Self {
            tools: vec![
                ToolKind::Shovel,
                ToolKind::Axe,
                ToolKind::Hammer,
                ToolKind::Rope,
            ],
        }
    }

    pub fn has(&self, tool: ToolKind) -> bool {
        self.tools.contains(&tool)
    }

    pub fn has_all(&self, tools: &[ToolKind]) -> bool {
        tools.iter().all(|tool| self.has(*tool))
    }
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::East => "east",
            Direction::South => "south",
            Direction::West => "west",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Shovel,
    Axe,
    Hammer,
    Rope,
    SawKit,
    Mattock,
    Winch,
    BraceKit,
}

impl ToolKind {
    pub fn label(self) -> &'static str {
        match self {
            ToolKind::Shovel => "shovel",
            ToolKind::Axe => "axe",
            ToolKind::Hammer => "hammer",
            ToolKind::Rope => "rope",
            ToolKind::SawKit => "saw kit",
            ToolKind::Mattock => "mattock",
            ToolKind::Winch => "winch",
            ToolKind::BraceKit => "brace kit",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewPool {
    pub crews: u32,
    pub labor_seconds_available: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionConstraints {
    pub max_work_orders: u32,
    pub allow_assault_preview: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyGroupSpec {
    pub label: String,
    pub count: u32,
    pub doctrine: EnemyDoctrine,
    pub spawn: CellCoord,
    pub objective: CellCoord,
    pub movement_profile: MovementProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyDoctrine {
    RushShortest,
    PreferCover,
    FlankViaConcealment,
    AvoidObstacles,
    PushThroughLightObstacles,
    ClearObstacles,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementProfile {
    pub base_speed: f32,
    pub obstacle_tolerance: f32,
    pub cover_preference: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctrineRouteSet {
    pub mission_id: String,
    pub routes: Vec<EnemyRoutePreview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyRoutePreview {
    pub group_label: String,
    pub doctrine: EnemyDoctrine,
    pub spawn: CellCoord,
    pub objective: CellCoord,
    pub points: Vec<CellCoord>,
    pub total_cost: f32,
    pub reached_goal: bool,
    pub explanation: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteDeltaReport {
    pub mission_id: String,
    pub groups: Vec<EnemyRouteDelta>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyRouteDelta {
    pub group_label: String,
    pub doctrine: EnemyDoctrine,
    pub initial_cost: f32,
    pub after_cost: f32,
    pub cost_delta: f32,
    pub shared_cells: u32,
    pub initial_only_cells: Vec<CellCoord>,
    pub after_only_cells: Vec<CellCoord>,
    pub changed: bool,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionPhase {
    Briefing,
    #[default]
    Prep,
    Assault,
    Debrief,
}

impl MissionPhase {
    pub fn label(self) -> &'static str {
        match self {
            MissionPhase::Briefing => "briefing",
            MissionPhase::Prep => "prep",
            MissionPhase::Assault => "assault",
            MissionPhase::Debrief => "debrief",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssaultState {
    pub tick: u32,
    pub status: AssaultStatus,
    pub objective_health: i32,
    pub initial_routes: DoctrineRouteSet,
    pub agents: Vec<EnemyAgent>,
    pub rolling_hazards: Vec<RollingHazardState>,
    pub timeline: Vec<AssaultTimelineEvent>,
    pub summary: Option<AssaultSummary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssaultStatus {
    Ready,
    Running,
    Victory,
    Defeat,
}

impl AssaultStatus {
    pub fn label(self) -> &'static str {
        match self {
            AssaultStatus::Ready => "ready",
            AssaultStatus::Running => "running",
            AssaultStatus::Victory => "victory",
            AssaultStatus::Defeat => "defeat",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyAgent {
    pub id: u32,
    pub group_label: String,
    pub doctrine: EnemyDoctrine,
    pub cell: CellCoord,
    pub route: Vec<CellCoord>,
    pub route_index: usize,
    pub hp: i32,
    pub delay_ticks: u32,
    pub status: EnemyAgentStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyAgentStatus {
    Advancing,
    Delayed,
    Eliminated,
    ReachedObjective,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingHazardState {
    pub object_id: String,
    pub label: String,
    pub direction: Direction,
    pub release_tick: u32,
    pub path: Vec<RollingHazardStep>,
    pub status: RollingHazardStatus,
    pub path_index: usize,
    pub enemies_hit: u32,
    pub obstacles_destroyed: u32,
    pub spent_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingHazardStep {
    pub cell: CellCoord,
    pub height_delta: i8,
    pub energy: i32,
    pub blocked_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollingHazardStatus {
    Prepared,
    Released,
    Spent,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RollingHazardImpactSummary {
    pub prepared_count: u32,
    pub released_count: u32,
    pub spent_count: u32,
    pub enemies_hit: u32,
    pub enemies_eliminated: u32,
    pub obstacles_destroyed: u32,
    pub friendly_risk_cells: Vec<CellCoord>,
    pub best_hazard_cell: Option<CellInfluence>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionRating {
    pub stars: u8,
    pub label: String,
    pub objective_survived: bool,
    pub stopped_ratio: f32,
    pub objective_health_ratio: f32,
    pub prep_time_used_seconds: u32,
    pub prep_time_efficiency: f32,
    pub friendly_risk_count: u32,
    pub unused_defense_count: u32,
    pub hazard_enemies_hit: u32,
    pub score: i32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionBalanceScenarioReport {
    pub id: String,
    pub label: String,
    pub order_count: u32,
    pub prep_time_used_seconds: u32,
    pub summary: AssaultSummary,
    pub rating: MissionRating,
    pub route_prediction_accuracy: RoutePredictionAccuracyReport,
    pub rolling_hazards: RollingHazardImpactSummary,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionBalanceReport {
    pub mission_id: String,
    pub mission_title: String,
    pub scenarios: Vec<MissionBalanceScenarioReport>,
    pub route_shift_summary: Vec<String>,
    pub hazard_effectiveness: Vec<String>,
    pub rating_breakdown: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionGeneratorSpec {
    pub seed: u64,
    pub theme: MissionTheme,
    pub terrain_archetype: TerrainArchetype,
    pub difficulty: DifficultyBand,
    pub objective_kind: ObjectiveKind,
    pub enemy_doctrine_mix: DoctrineMix,
    pub material_budget_style: MaterialBudgetStyle,
    pub required_affordances: Vec<GeneratedAffordance>,
}

impl MissionGeneratorSpec {
    pub fn road_below(seed: u64) -> Self {
        Self {
            seed,
            theme: MissionTheme::DryRoadBelow,
            terrain_archetype: TerrainArchetype::RoadRidge,
            difficulty: DifficultyBand::Standard,
            objective_kind: ObjectiveKind::HoldMarker,
            enemy_doctrine_mix: DoctrineMix::BalancedRoadPush,
            material_budget_style: MaterialBudgetStyle::LocalSparse,
            required_affordances: vec![
                GeneratedAffordance::RoadApproach,
                GeneratedAffordance::Ridge,
                GeneratedAffordance::TreeCluster,
                GeneratedAffordance::RollingLogOpportunity,
                GeneratedAffordance::TrenchableSoil,
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionTheme {
    DryRoadBelow,
    OrchardApproach,
    DryWash,
    RidgeTrap,
    OldWall,
    SplitApproach,
}

impl MissionTheme {
    pub const GENERATABLE: [MissionTheme; 6] = [
        MissionTheme::DryRoadBelow,
        MissionTheme::OrchardApproach,
        MissionTheme::DryWash,
        MissionTheme::RidgeTrap,
        MissionTheme::OldWall,
        MissionTheme::SplitApproach,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            MissionTheme::DryRoadBelow => "dry_road_below",
            MissionTheme::OrchardApproach => "orchard_approach",
            MissionTheme::DryWash => "dry_wash",
            MissionTheme::RidgeTrap => "ridge_trap",
            MissionTheme::OldWall => "old_wall",
            MissionTheme::SplitApproach => "split_approach",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MissionTheme::DryRoadBelow => "Dry Road Below",
            MissionTheme::OrchardApproach => "Orchard Approach",
            MissionTheme::DryWash => "Dry Wash",
            MissionTheme::RidgeTrap => "Ridge Trap",
            MissionTheme::OldWall => "Old Wall",
            MissionTheme::SplitApproach => "Split Approach",
        }
    }
}

impl std::str::FromStr for MissionTheme {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "dry_road_below" | "road_below" | "dry-road-below" => Ok(MissionTheme::DryRoadBelow),
            "orchard_approach" | "orchard" | "orchard-approach" => {
                Ok(MissionTheme::OrchardApproach)
            }
            "dry_wash" | "dry-wash" => Ok(MissionTheme::DryWash),
            "ridge_trap" | "ridge-trap" => Ok(MissionTheme::RidgeTrap),
            "old_wall" | "old-wall" => Ok(MissionTheme::OldWall),
            "split_approach" | "split" | "split-approach" => Ok(MissionTheme::SplitApproach),
            other => Err(format!("unknown mission theme `{other}`")),
        }
    }
}

pub fn mission_visual_theme_for_theme(theme: MissionTheme) -> MissionVisualTheme {
    let sprite_style_profile = match theme {
        MissionTheme::DryRoadBelow | MissionTheme::OldWall | MissionTheme::SplitApproach => {
            "assets/sprite_styles/cozy_upland/style.ron"
        }
        MissionTheme::OrchardApproach => "assets/sprite_styles/cozy_upland_lush/style.ron",
        MissionTheme::DryWash | MissionTheme::RidgeTrap => {
            "assets/sprite_styles/cozy_upland_sparse/style.ron"
        }
    };
    MissionVisualTheme {
        sprite_style_profile: sprite_style_profile.to_string(),
        render_projection: MissionRenderProjection::HighOblique2D,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainArchetype {
    RoadRidge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyBand {
    Intro,
    Standard,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveKind {
    HoldMarker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctrineMix {
    BalancedRoadPush,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialBudgetStyle {
    LocalSparse,
    TimberRich,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratedAffordance {
    RoadApproach,
    Ridge,
    TreeCluster,
    RollingLogOpportunity,
    TrenchableSoil,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionCandidate {
    pub seed: u64,
    pub theme: MissionTheme,
    pub spec: MissionSpec,
    pub affordance_report: GeneratedMissionAffordanceReport,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeneratedMissionAffordanceReport {
    pub road_cell_count: u32,
    pub ridge_cell_count: u32,
    pub trenchable_soil_cells: u32,
    pub tree_count: u32,
    pub loose_log_count: u32,
    pub spawn_count: u32,
    pub route_count: u32,
    pub rolling_hazard_path_cells: u32,
    pub rolling_hazard_route_intersections: u32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionScenarioScore {
    pub id: String,
    pub label: String,
    pub stars: u8,
    pub score: i32,
    pub victory: bool,
    pub stopped: u32,
    pub reached: u32,
    pub prep_time_used_seconds: u32,
    pub hazard_enemies_hit: u32,
    pub validation_issue_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneratedMissionRejectionKind {
    TooEasyNoPrep,
    TooHardAllPlansFail,
    NoRouteDiversity,
    NoUsefulMaterials,
    NoHazardOpportunity,
    HazardTooDominant,
    ObjectiveUnreachable,
    TerrainTooFlat,
    SpawnTooClose,
    SpawnTooFar,
    InvalidMap,
    DuplicateCandidate,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeneratedMissionScoreBreakdown {
    pub baseline_pressure: i32,
    pub prep_delta: i32,
    pub route_diversity: i32,
    pub terrain_interest: i32,
    pub material_affordances: i32,
    pub work_order_opportunities: i32,
    pub hazard_viability: i32,
    pub doctrine_spread: i32,
    pub objective_vulnerability: i32,
    pub duplicate_penalty: i32,
}

impl GeneratedMissionScoreBreakdown {
    pub fn total(&self) -> i32 {
        self.baseline_pressure
            + self.prep_delta
            + self.route_diversity
            + self.terrain_interest
            + self.material_affordances
            + self.work_order_opportunities
            + self.hazard_viability
            + self.doctrine_spread
            + self.objective_vulnerability
            - self.duplicate_penalty
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionPlanSensitivity {
    pub baseline_score: i32,
    pub best_score: i32,
    pub worst_score: i32,
    pub best_minus_baseline: i32,
    pub best_minus_worst: i32,
    pub rolling_log_score: Option<i32>,
    pub rolling_log_to_best_ratio: Option<f32>,
    pub overbuilt_bad_plan_score: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionFingerprint {
    pub objective: CellCoord,
    pub spawns: Vec<CellCoord>,
    pub ridge_cells: Vec<CellCoord>,
    pub tree_cells: Vec<CellCoord>,
    pub route_cells: Vec<CellCoord>,
    pub route_lengths: Vec<u32>,
    pub rolling_hazard_route_intersections: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionEvaluation {
    pub seed: u64,
    pub mission_id: String,
    pub title: String,
    pub theme: MissionTheme,
    pub theme_slug: String,
    pub candidate_dir: Option<String>,
    pub mission_path: Option<String>,
    pub accepted: bool,
    pub tactical_interest_score: i32,
    pub score_breakdown: GeneratedMissionScoreBreakdown,
    pub plan_sensitivity: GeneratedMissionPlanSensitivity,
    pub rejection_kinds: Vec<GeneratedMissionRejectionKind>,
    pub rejection_reasons: Vec<String>,
    pub duplicate_of_seed: Option<u64>,
    pub similarity_to_duplicate: Option<f32>,
    pub baseline_rating: MissionRating,
    pub best_rating: MissionRating,
    pub best_plan_label: String,
    pub route_diversity_score: f32,
    pub height_interest_score: f32,
    pub local_material_score: f32,
    pub work_order_opportunity_score: f32,
    pub rolling_hazard_score: f32,
    pub doctrine_spread_score: f32,
    pub objective_vulnerability_score: f32,
    pub affordance_report: GeneratedMissionAffordanceReport,
    pub fingerprint: GeneratedMissionFingerprint,
    pub scenarios: Vec<GeneratedMissionScenarioScore>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionBatchReport {
    pub generator: MissionGeneratorSpec,
    pub generated_count: u32,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub ranked_candidates: Vec<GeneratedMissionEvaluation>,
    pub rejected_candidates: Vec<GeneratedMissionEvaluation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionThemeSummary {
    pub theme: MissionTheme,
    pub theme_slug: String,
    pub generated_count: u32,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub best_candidate: Option<GeneratedMissionEvaluation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionThemeBatchReport {
    pub seed: u64,
    pub count_per_theme: u32,
    pub theme_summaries: Vec<GeneratedMissionThemeSummary>,
    pub total_generated_count: u32,
    pub total_accepted_count: u32,
    pub total_rejected_count: u32,
    pub all_ranked_candidates: Vec<GeneratedMissionEvaluation>,
    pub all_rejected_candidates: Vec<GeneratedMissionEvaluation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionBrowserIndex {
    pub source_dir: String,
    pub generated_count: u32,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub theme_summaries: Vec<GeneratedMissionThemeSummary>,
    pub candidates: Vec<GeneratedMissionBrowserEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionBrowserEntry {
    pub title: String,
    pub mission_id: String,
    pub theme: MissionTheme,
    pub theme_slug: String,
    pub seed: u64,
    pub accepted: bool,
    pub tactical_interest_score: i32,
    pub best_plan_label: String,
    pub baseline_score: i32,
    pub best_score: i32,
    pub best_minus_baseline: i32,
    pub best_minus_worst: i32,
    pub route_diversity_score: f32,
    pub hazard_viability_score: f32,
    pub local_material_score: f32,
    pub difficulty_score: i32,
    pub complexity_score: i32,
    pub top_rejection_kind: Option<GeneratedMissionRejectionKind>,
    pub top_rejection_reason: Option<String>,
    pub primary_affordance: String,
    pub mission_path: Option<String>,
    pub candidate_dir: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionPack {
    pub id: String,
    pub label: String,
    pub seed: u64,
    pub missions: Vec<GeneratedMissionPackEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionPackEntry {
    pub order: u32,
    pub title: String,
    pub mission_id: String,
    pub theme: MissionTheme,
    pub theme_slug: String,
    pub seed: u64,
    pub tactical_interest_score: i32,
    pub difficulty_score: i32,
    pub complexity_score: i32,
    pub mission_path: String,
    pub best_plan_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionPackSummary {
    pub pack: GeneratedMissionPack,
    pub curve: MissionPackCurve,
    pub requested_missions: u32,
    pub candidate_count_per_theme: u32,
    pub source_batch_dir: String,
    pub total_generated_count: u32,
    pub total_accepted_count: u32,
    pub difficulty_curve: Vec<GeneratedMissionDifficultyPoint>,
    pub complexity_curve: Vec<GeneratedMissionComplexityPoint>,
    pub pack_diversity_report: GeneratedMissionPackDiversityReport,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionPackCurve {
    Balanced,
    Tutorial,
}

impl MissionPackCurve {
    pub fn label(self) -> &'static str {
        match self {
            MissionPackCurve::Balanced => "balanced",
            MissionPackCurve::Tutorial => "tutorial",
        }
    }
}

impl std::str::FromStr for MissionPackCurve {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "balanced" | "default" => Ok(MissionPackCurve::Balanced),
            "tutorial" | "teaching" => Ok(MissionPackCurve::Tutorial),
            other => Err(format!("unknown mission pack curve `{other}`")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionDifficultyPoint {
    pub order: u32,
    pub title: String,
    pub theme_slug: String,
    pub difficulty_score: i32,
    pub baseline_score: i32,
    pub best_score: i32,
    pub tactical_interest_score: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionComplexityPoint {
    pub order: u32,
    pub title: String,
    pub theme_slug: String,
    pub complexity_score: i32,
    pub route_count: u32,
    pub doctrine_count: u32,
    pub material_types_present: u32,
    pub hazard_count: u32,
    pub height_interest_score: f32,
    pub meaningful_affordances: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedMissionPackDiversityReport {
    pub mission_count: u32,
    pub unique_theme_count: u32,
    pub repeated_theme_count: u32,
    pub has_tree_material_mission: bool,
    pub has_hazard_mission: bool,
    pub has_split_approach_mission: bool,
    pub difficulty_curve_is_monotonic: bool,
    pub complexity_curve_is_monotonic: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeCalibrationReport {
    pub seed: u64,
    pub count_per_theme: u32,
    pub total_generated_count: u32,
    pub total_accepted_count: u32,
    pub total_rejected_count: u32,
    pub theme_summaries: Vec<ThemeCalibrationSummary>,
    pub global_rejection_reasons: Vec<RejectionReasonHistogramEntry>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeCalibrationSummary {
    pub theme: MissionTheme,
    pub theme_slug: String,
    pub generated_count: u32,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub acceptance_rate: f32,
    pub target_acceptance_min: f32,
    pub target_acceptance_max: f32,
    pub target_difficulty: String,
    pub average_score: f32,
    pub best_score: i32,
    pub average_difficulty_score: f32,
    pub average_complexity_score: f32,
    pub average_plan_sensitivity: f32,
    pub average_route_diversity: f32,
    pub average_hazard_usefulness: f32,
    pub average_material_affordance: f32,
    pub most_common_rejection: Option<GeneratedMissionRejectionKind>,
    pub rejection_reasons: Vec<RejectionReasonHistogramEntry>,
    pub recommendations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RejectionReasonHistogramEntry {
    pub kind: GeneratedMissionRejectionKind,
    pub count: u32,
    pub ratio: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionVisualAssetReport {
    pub mission_id: String,
    pub sprite_style_profile: String,
    pub render_projection: MissionRenderProjection,
    pub generated_sprite_count: usize,
    pub effective_sprite_count: usize,
    pub overridden_sprite_count: usize,
    pub override_issue_count: usize,
    pub missing_visual_pieces: Vec<String>,
    pub fallback_pieces_used: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
struct GeneratedMissionArtifact {
    spec: MissionSpec,
    candidate_dir: PathBuf,
    evaluation: GeneratedMissionEvaluation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssaultTimelineEvent {
    pub tick: u32,
    pub agent_id: Option<u32>,
    pub group_label: Option<String>,
    pub cell: Option<CellCoord>,
    pub kind: AssaultEventKind,
    pub cause: AssaultEventCause,
    pub magnitude: i32,
    pub note: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssaultEventKind {
    AssaultStarted,
    Spawned,
    Moved,
    Rerouted,
    RollingHazardReleased,
    RollingHazardMoved,
    RollingHazardHitEnemy,
    RollingHazardDestroyedObstacle,
    RollingHazardBlocked,
    RollingHazardSpent,
    DelayedByTerrain,
    DelayedByObstacle,
    SuppressedByDefender,
    DamagedByDefender,
    DamagedByObstacle,
    Eliminated,
    ReachedObjective,
    AssaultEnded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssaultEventCause {
    System,
    Spawn,
    Route,
    Terrain,
    Obstacle,
    Defender,
    Objective,
    RollingHazard,
}

impl AssaultEventCause {
    pub fn label(self) -> &'static str {
        match self {
            AssaultEventCause::System => "system",
            AssaultEventCause::Spawn => "spawn",
            AssaultEventCause::Route => "route",
            AssaultEventCause::Terrain => "terrain",
            AssaultEventCause::Obstacle => "obstacle",
            AssaultEventCause::Defender => "defender",
            AssaultEventCause::Objective => "objective",
            AssaultEventCause::RollingHazard => "rolling hazard",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssaultSummary {
    pub victory: bool,
    pub outcome_label: String,
    pub ticks_elapsed: u32,
    pub enemies_spawned: u32,
    pub enemies_eliminated: u32,
    pub enemies_reached_objective: u32,
    pub objective_health_remaining: i32,
    pub objective_damage_taken: i32,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssaultDebrief {
    pub mission_id: String,
    pub outcome_label: String,
    pub summary: AssaultSummary,
    pub rating: MissionRating,
    pub influence: AssaultInfluenceSummary,
    pub rolling_hazards: RollingHazardImpactSummary,
    pub route_prediction_accuracy: RoutePredictionAccuracyReport,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssaultInfluenceSummary {
    pub most_crossed_cells: Vec<CellInfluence>,
    pub most_delayed_cells: Vec<CellInfluence>,
    pub most_damaging_cells: Vec<CellInfluence>,
    pub defender_pressure_cells: Vec<CellInfluence>,
    pub breach_cells: Vec<CellInfluence>,
    pub most_effective_obstacle: Option<CellInfluence>,
    pub most_delayed_group: Option<GroupInfluence>,
    pub unused_defenses: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellInfluence {
    pub cell: CellCoord,
    pub count: u32,
    pub magnitude: i32,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupInfluence {
    pub group_label: String,
    pub count: u32,
    pub magnitude: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutePredictionAccuracyReport {
    pub groups: Vec<RoutePredictionAccuracy>,
    pub average_accuracy: f32,
    pub total_divergence_cells: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoutePredictionAccuracy {
    pub group_label: String,
    pub doctrine: EnemyDoctrine,
    pub predicted_cell_count: u32,
    pub actual_cell_count: u32,
    pub shared_cell_count: u32,
    pub divergence_cells: Vec<CellCoord>,
    pub accuracy: f32,
    pub explanation: String,
}

fn assault_event(
    tick: u32,
    agent: Option<&EnemyAgent>,
    cell: Option<CellCoord>,
    kind: AssaultEventKind,
    cause: AssaultEventCause,
    magnitude: i32,
    note: impl Into<String>,
) -> AssaultTimelineEvent {
    AssaultTimelineEvent {
        tick,
        agent_id: agent.map(|agent| agent.id),
        group_label: agent.map(|agent| agent.group_label.clone()),
        cell,
        kind,
        cause,
        magnitude,
        note: note.into(),
    }
}

#[derive(Clone, Debug)]
struct AssaultCellEffect {
    delay_ticks: u32,
    terrain_delay_ticks: u32,
    obstacle_delay_ticks: u32,
    obstacle_damage: i32,
    terrain_damage: i32,
    reasons: Vec<String>,
}

impl AssaultCellEffect {
    fn total_damage(&self) -> i32 {
        self.obstacle_damage + self.terrain_damage
    }

    fn reason_text(&self) -> String {
        if self.reasons.is_empty() {
            "clear ground".to_string()
        } else {
            self.reasons.join(", ")
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DoctrineWeights {
    trench_cost: f32,
    berm_cost: f32,
    obstacle_cost: f32,
    cover_discount: f32,
    concealment_discount: f32,
    road_bias: f32,
    height_cost: f32,
}

impl EnemyDoctrine {
    fn weights(self) -> DoctrineWeights {
        match self {
            EnemyDoctrine::RushShortest => DoctrineWeights {
                trench_cost: 1.3,
                berm_cost: 1.1,
                obstacle_cost: 1.4,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.18,
                height_cost: 0.28,
            },
            EnemyDoctrine::PreferCover => DoctrineWeights {
                trench_cost: 0.9,
                berm_cost: 0.6,
                obstacle_cost: 1.1,
                cover_discount: 0.32,
                concealment_discount: 0.18,
                road_bias: 0.02,
                height_cost: 0.24,
            },
            EnemyDoctrine::FlankViaConcealment => DoctrineWeights {
                trench_cost: 0.8,
                berm_cost: 0.7,
                obstacle_cost: 1.0,
                cover_discount: 0.18,
                concealment_discount: 0.42,
                road_bias: 0.24,
                height_cost: 0.20,
            },
            EnemyDoctrine::AvoidObstacles => DoctrineWeights {
                trench_cost: 2.2,
                berm_cost: 2.0,
                obstacle_cost: 2.7,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.05,
                height_cost: 0.42,
            },
            EnemyDoctrine::PushThroughLightObstacles => DoctrineWeights {
                trench_cost: 1.5,
                berm_cost: 1.2,
                obstacle_cost: 0.75,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.12,
                height_cost: 0.35,
            },
            EnemyDoctrine::ClearObstacles => DoctrineWeights {
                trench_cost: 1.4,
                berm_cost: 1.1,
                obstacle_cost: 0.45,
                cover_discount: 0.04,
                concealment_discount: 0.0,
                road_bias: -0.10,
                height_cost: 0.32,
            },
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EnemyDoctrine::RushShortest => "rush shortest",
            EnemyDoctrine::PreferCover => "prefer cover",
            EnemyDoctrine::FlankViaConcealment => "flank via concealment",
            EnemyDoctrine::AvoidObstacles => "avoid obstacles",
            EnemyDoctrine::PushThroughLightObstacles => "push through light obstacles",
            EnemyDoctrine::ClearObstacles => "clear obstacles",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionState {
    pub spec: MissionSpec,
    pub map: MissionMap,
    #[serde(default)]
    pub phase: MissionPhase,
    pub remaining_prep_seconds: u32,
    pub remaining_labor_seconds: u32,
    pub work_queue: Vec<WorkOrder>,
    pub work_orders: Vec<WorkOrder>,
    pub material_ledger: Vec<MaterialLedgerEntry>,
    pub order_validation: Vec<OrderValidationEntry>,
    pub event_log: Vec<String>,
    #[serde(default)]
    pub assault: Option<AssaultState>,
}

impl MissionState {
    pub fn from_spec(spec: MissionSpec) -> Self {
        Self {
            phase: MissionPhase::Prep,
            remaining_prep_seconds: spec.prep_time_seconds,
            remaining_labor_seconds: spec.crew.labor_seconds_available,
            map: spec.map.clone(),
            spec,
            work_queue: Vec::new(),
            work_orders: Vec::new(),
            material_ledger: Vec::new(),
            order_validation: Vec::new(),
            event_log: Vec::new(),
            assault: None,
        }
    }

    pub fn road_below_seed() -> Self {
        Self::from_spec(road_below_spec())
    }

    pub fn apply_work_order(&mut self, kind: WorkOrderKind, target: WorkTarget) -> WorkOrder {
        let order = self.queue_work_order(kind, target);
        if matches!(order.status, WorkOrderStatus::Queued) {
            self.run_next_queued_order().unwrap_or_else(|| {
                order_with_status(
                    order,
                    WorkOrderStatus::Rejected {
                        reason: "queued order disappeared before execution".to_string(),
                    },
                )
            })
        } else {
            order
        }
    }

    pub fn preview_work_order(&self, kind: WorkOrderKind, target: WorkTarget) -> WorkOrder {
        let id = self.work_orders.len() as u32 + self.work_queue.len() as u32 + 1;
        let mut order = build_work_order(id, kind, target, &self.map);
        if let Err(reason) = self.validate_work_order(&order) {
            order.status = WorkOrderStatus::Rejected { reason };
        }
        order
    }

    pub fn queue_work_order(&mut self, kind: WorkOrderKind, target: WorkTarget) -> WorkOrder {
        let id = self.work_orders.len() as u32 + self.work_queue.len() as u32 + 1;
        let mut order = build_work_order(id, kind, target, &self.map);
        match self.validate_work_order(&order) {
            Ok(()) => {
                order.status = WorkOrderStatus::Queued;
                self.event_log.push(format!(
                    "queued {}: {}s with {} crew",
                    order.kind.label(),
                    order.duration_seconds,
                    order.assigned_crews
                ));
                self.work_queue.push(order.clone());
            }
            Err(reason) => {
                order.status = WorkOrderStatus::Rejected {
                    reason: reason.clone(),
                };
                self.order_validation.push(OrderValidationEntry {
                    order_id: Some(order.id),
                    severity: OrderValidationSeverity::Error,
                    message: reason.clone(),
                });
                self.event_log
                    .push(format!("{}: {}", order.kind.label(), order.status.label()));
                self.work_orders.push(order.clone());
            }
        }
        order
    }

    pub fn run_next_queued_order(&mut self) -> Option<WorkOrder> {
        if self.work_queue.is_empty() {
            return None;
        }
        let mut order = self.work_queue.remove(0);
        order.status = WorkOrderStatus::InProgress;
        match self
            .validate_work_order(&order)
            .and_then(|()| self.validate_work_order_materials(&order))
        {
            Ok(()) => match apply_order_effects(&mut self.map, &mut order) {
                Ok(()) => {
                    self.remaining_prep_seconds = self
                        .remaining_prep_seconds
                        .saturating_sub(order.duration_seconds);
                    self.remaining_labor_seconds = self
                        .remaining_labor_seconds
                        .saturating_sub(order.labor_seconds);
                    order.progress_seconds = order.duration_seconds;
                    order.status = WorkOrderStatus::Completed;
                    self.record_material_ledger(&order);
                }
                Err(err) => {
                    order.status = WorkOrderStatus::Rejected {
                        reason: err.to_string(),
                    };
                    self.order_validation.push(OrderValidationEntry {
                        order_id: Some(order.id),
                        severity: OrderValidationSeverity::Error,
                        message: err.to_string(),
                    });
                }
            },
            Err(reason) => {
                order.status = WorkOrderStatus::Rejected {
                    reason: reason.clone(),
                };
                self.order_validation.push(OrderValidationEntry {
                    order_id: Some(order.id),
                    severity: OrderValidationSeverity::Error,
                    message: reason,
                });
            }
        }

        self.event_log
            .push(format!("{}: {}", order.kind.label(), order.status.label()));
        self.work_orders.push(order.clone());
        Some(order)
    }

    pub fn run_all_queued_orders(&mut self) {
        while !self.work_queue.is_empty() {
            self.run_next_queued_order();
        }
    }

    pub fn validate_work_order(&self, order: &WorkOrder) -> std::result::Result<(), String> {
        if !self.spec.starting_tools.has_all(&order.required_tools) {
            return Err(format!(
                "missing required tools: {}",
                order
                    .required_tools
                    .iter()
                    .filter(|tool| !self.spec.starting_tools.has(**tool))
                    .map(|tool| tool.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if order.duration_seconds > self.remaining_prep_seconds
            || order.labor_seconds > self.remaining_labor_seconds
        {
            return Err("not enough prep time or crew labor".to_string());
        }
        if order.crew_required > self.spec.crew.crews {
            return Err(format!(
                "not enough crews: need {}, have {}",
                order.crew_required, self.spec.crew.crews
            ));
        }
        if self.work_orders.len() as u32 + self.work_queue.len() as u32
            >= self.spec.constraints.max_work_orders
        {
            return Err("mission work-order limit reached".to_string());
        }
        validate_order_target(&self.map, order)?;
        Ok(())
    }

    pub fn validate_work_order_materials(
        &self,
        order: &WorkOrder,
    ) -> std::result::Result<(), String> {
        for (kind, required) in material_requirements(&order.material_inputs) {
            if required > 0 {
                let available = order
                    .preview
                    .affected_cells
                    .first()
                    .map(|origin| available_nearby_material(&self.map, *origin, kind))
                    .unwrap_or_else(|| self.material_totals().get(kind));
                if available < required {
                    return Err(format!(
                        "not enough nearby {kind:?}: need {required}, have {available}"
                    ));
                }
            }
        }
        Ok(())
    }

    fn record_material_ledger(&mut self, order: &WorkOrder) {
        let net = LocalMaterialStock::net(&order.material_outputs, &order.material_inputs);
        if net.is_zero() {
            return;
        }
        self.material_ledger.push(MaterialLedgerEntry {
            order_id: order.id,
            order_kind: order.kind,
            inputs: order.material_inputs.clone(),
            outputs: order.material_outputs.clone(),
            net,
            note: order
                .preview
                .notes
                .first()
                .cloned()
                .unwrap_or_else(|| "work order material change".to_string()),
        });
    }

    pub fn apply_seed_orders(&mut self) {
        for (kind, target) in road_below_seed_orders() {
            self.apply_work_order(kind, target);
        }
    }

    pub fn material_totals(&self) -> LocalMaterialStock {
        let mut totals = LocalMaterialStock::default();
        for cell in &self.map.cells {
            totals.earth_spoil += cell.local_material.earth_spoil;
            totals.timber += cell.local_material.timber;
            totals.logs += cell.local_material.logs;
            totals.stakes += cell.local_material.stakes;
            totals.loose_stone += cell.local_material.loose_stone;
            totals.scrap += cell.local_material.scrap;
            totals.rope_uses += cell.local_material.rope_uses;
        }
        totals
    }

    pub fn route_preview(&self) -> DoctrineRouteSet {
        route_preview_for_state(self)
    }

    pub fn start_assault(&mut self) {
        let routes = self.route_preview();
        let rolling_hazards = planned_rolling_hazards_for_map(&self.map, 7);
        let mut agents = Vec::new();
        let mut agent_id = 1;
        for group in &self.spec.enemy_groups {
            let route = routes
                .routes
                .iter()
                .find(|route| route.group_label == group.label)
                .map(|route| route.points.clone())
                .unwrap_or_default();
            for _ in 0..group.count {
                agents.push(EnemyAgent {
                    id: agent_id,
                    group_label: group.label.clone(),
                    doctrine: group.doctrine,
                    cell: group.spawn,
                    route: route.clone(),
                    route_index: 0,
                    hp: enemy_hp_for_doctrine(group.doctrine),
                    delay_ticks: 0,
                    status: EnemyAgentStatus::Advancing,
                });
                agent_id += 1;
            }
        }

        let mut timeline = Vec::new();
        timeline.push(assault_event(
            0,
            None,
            None,
            AssaultEventKind::AssaultStarted,
            AssaultEventCause::System,
            agents.len() as i32,
            format!(
                "Assault started with {} enemy agent(s) and {} defender position(s).",
                agents.len(),
                self.spec.defender_positions.len()
            ),
        ));
        for agent in &agents {
            timeline.push(assault_event(
                0,
                Some(agent),
                Some(agent.cell),
                AssaultEventKind::Spawned,
                AssaultEventCause::Spawn,
                1,
                format!(
                    "{} spawned at ({}, {}).",
                    agent.group_label, agent.cell.x, agent.cell.y
                ),
            ));
        }

        self.phase = MissionPhase::Assault;
        self.assault = Some(AssaultState {
            tick: 0,
            status: AssaultStatus::Running,
            objective_health: self.spec.objective.objective_health as i32,
            initial_routes: routes,
            agents,
            rolling_hazards,
            timeline,
            summary: None,
        });
    }

    pub fn step_assault(&mut self) -> Vec<AssaultTimelineEvent> {
        if self.assault.is_none() {
            self.start_assault();
        }
        let Some(mut assault) = self.assault.take() else {
            return Vec::new();
        };
        if !matches!(assault.status, AssaultStatus::Running) {
            self.assault = Some(assault);
            return Vec::new();
        }

        assault.tick += 1;
        let mut events = Vec::new();
        for index in 0..assault.agents.len() {
            if assault.objective_health <= 0 {
                break;
            }
            let event_count_before = events.len();
            step_agent_assault(
                &self.map,
                &self.spec,
                assault.tick,
                &mut assault.objective_health,
                &mut assault.agents[index],
                &mut events,
            );
            if events.len() == event_count_before
                && matches!(assault.agents[index].status, EnemyAgentStatus::Advancing)
            {
                events.push(assault_event(
                    assault.tick,
                    Some(&assault.agents[index]),
                    Some(assault.agents[index].cell),
                    AssaultEventKind::DelayedByTerrain,
                    AssaultEventCause::Route,
                    1,
                    "agent held position",
                ));
            }
        }

        process_rolling_hazards(
            &mut self.map,
            &self.spec,
            assault.tick,
            &mut assault.agents,
            &mut assault.rolling_hazards,
            &mut events,
        );

        let active_agents = assault
            .agents
            .iter()
            .filter(|agent| {
                matches!(
                    agent.status,
                    EnemyAgentStatus::Advancing | EnemyAgentStatus::Delayed
                )
            })
            .count();
        if assault.objective_health <= 0 || active_agents == 0 {
            let summary = summarize_assault(&self.spec, &assault);
            assault.status = if summary.victory {
                AssaultStatus::Victory
            } else {
                AssaultStatus::Defeat
            };
            events.push(assault_event(
                assault.tick,
                None,
                Some(self.spec.objective.defend_cell),
                AssaultEventKind::AssaultEnded,
                AssaultEventCause::System,
                if summary.victory { 1 } else { -1 },
                summary.outcome_label.clone(),
            ));
            assault.summary = Some(summary);
            self.phase = MissionPhase::Debrief;
        }
        assault.timeline.extend(events.clone());
        self.assault = Some(assault);
        events
    }

    pub fn run_assault_to_completion(&mut self, max_ticks: u32) -> AssaultSummary {
        if self.assault.is_none() {
            self.start_assault();
        }
        for _ in 0..max_ticks {
            let done = self
                .assault
                .as_ref()
                .map(|assault| !matches!(assault.status, AssaultStatus::Running))
                .unwrap_or(true);
            if done {
                break;
            }
            self.step_assault();
        }
        if self
            .assault
            .as_ref()
            .and_then(|assault| assault.summary.clone())
            .is_none()
        {
            if let Some(mut assault) = self.assault.take() {
                let summary = summarize_assault(&self.spec, &assault);
                assault.status = if summary.victory {
                    AssaultStatus::Victory
                } else {
                    AssaultStatus::Defeat
                };
                assault.summary = Some(summary);
                self.phase = MissionPhase::Debrief;
                self.assault = Some(assault);
            }
        }
        self.assault
            .as_ref()
            .and_then(|assault| assault.summary.clone())
            .unwrap_or_else(|| AssaultSummary {
                victory: false,
                outcome_label: "assault did not start".to_string(),
                ticks_elapsed: 0,
                enemies_spawned: 0,
                enemies_eliminated: 0,
                enemies_reached_objective: 0,
                objective_health_remaining: self.spec.objective.objective_health as i32,
                objective_damage_taken: 0,
                notes: vec!["no assault state was available".to_string()],
            })
    }

    pub fn reset_assault(&mut self) {
        self.phase = MissionPhase::Prep;
        self.assault = None;
    }

    pub fn assault_debrief(&self) -> Option<AssaultDebrief> {
        build_assault_debrief(self)
    }

    pub fn rolling_hazard_plans(&self) -> Vec<RollingHazardState> {
        planned_rolling_hazards_for_map(&self.map, 6)
    }

    pub fn release_prepared_rolling_hazards(&mut self) -> usize {
        if self.assault.is_none() {
            self.start_assault();
        }
        let Some(assault) = &mut self.assault else {
            return 0;
        };
        let release_tick = assault.tick + 1;
        let mut count = 0;
        for hazard in &mut assault.rolling_hazards {
            if hazard.status == RollingHazardStatus::Prepared {
                hazard.release_tick = release_tick;
                count += 1;
            }
        }
        count
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialLedgerEntry {
    pub order_id: u32,
    pub order_kind: WorkOrderKind,
    pub inputs: LocalMaterialStock,
    pub outputs: LocalMaterialStock,
    pub net: LocalMaterialStock,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderValidationEntry {
    pub order_id: Option<u32>,
    pub severity: OrderValidationSeverity,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: u32,
    pub kind: WorkOrderKind,
    pub target: WorkTarget,
    pub required_tools: Vec<ToolKind>,
    pub crew_required: u32,
    pub assigned_crews: u32,
    pub labor_seconds: u32,
    pub duration_seconds: u32,
    pub material_inputs: LocalMaterialStock,
    pub material_outputs: LocalMaterialStock,
    pub progress_seconds: u32,
    pub status: WorkOrderStatus,
    pub preview: WorkOrderPreview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderKind {
    DigTrench,
    RaiseBerm,
    Flatten,
    FellTree,
    CutIntoLogs,
    PlaceStakes,
    PrepareRollingLog,
}

impl WorkOrderKind {
    pub fn label(self) -> &'static str {
        match self {
            WorkOrderKind::DigTrench => "dig trench",
            WorkOrderKind::RaiseBerm => "raise berm",
            WorkOrderKind::Flatten => "flatten",
            WorkOrderKind::FellTree => "fell tree",
            WorkOrderKind::CutIntoLogs => "cut into logs",
            WorkOrderKind::PlaceStakes => "place stakes",
            WorkOrderKind::PrepareRollingLog => "prepare rolling log",
        }
    }

    pub fn default_crew_required(self) -> u32 {
        match self {
            WorkOrderKind::DigTrench => 2,
            WorkOrderKind::RaiseBerm => 2,
            WorkOrderKind::Flatten => 1,
            WorkOrderKind::FellTree => 2,
            WorkOrderKind::CutIntoLogs => 1,
            WorkOrderKind::PlaceStakes => 1,
            WorkOrderKind::PrepareRollingLog => 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkTarget {
    Cell(CellCoord),
    Rect(CellRect),
    Object(String),
}

impl WorkTarget {
    pub fn affected_cells(&self) -> Vec<CellCoord> {
        match self {
            WorkTarget::Cell(cell) => vec![*cell],
            WorkTarget::Rect(rect) => rect.cells().collect(),
            WorkTarget::Object(_) => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkOrderStatus {
    Planned,
    Queued,
    InProgress,
    Completed,
    Rejected { reason: String },
}

impl WorkOrderStatus {
    pub fn label(&self) -> String {
        match self {
            WorkOrderStatus::Planned => "planned".to_string(),
            WorkOrderStatus::Queued => "queued".to_string(),
            WorkOrderStatus::InProgress => "in progress".to_string(),
            WorkOrderStatus::Completed => "completed".to_string(),
            WorkOrderStatus::Rejected { reason } => format!("rejected: {reason}"),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkOrderPreview {
    pub affected_cells: Vec<CellCoord>,
    pub affected_objects: Vec<String>,
    pub material_delta: LocalMaterialStock,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOrderScript {
    pub id: String,
    pub label: String,
    pub orders: Vec<ScriptedWorkOrder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptedWorkOrder {
    pub kind: WorkOrderKind,
    pub target: WorkTarget,
}

pub fn road_below_spec() -> MissionSpec {
    let mut map = MissionMap::new(12, 8, MissionCell::new(1, GroundKind::Grass));
    map.spawn_cells.push(CellCoord::new(1, 7));
    map.spawn_cells.push(CellCoord::new(0, 6));

    for x in 0..map.width {
        let cell = CellCoord::new(x, 4);
        let Some(tile) = map.cell_mut(cell) else {
            continue;
        };
        tile.ground = GroundKind::Road;
        tile.movement_cost = GroundKind::Road.base_movement_cost();
    }

    for x in 7..=11 {
        if let Some(tile) = map.cell_mut(CellCoord::new(x, 2)) {
            tile.height = 2;
        }
        if let Some(tile) = map.cell_mut(CellCoord::new(x, 3)) {
            tile.height = 2;
        }
    }

    for (id, label, cell) in [
        ("tree_west_01", "roadside pine", CellCoord::new(3, 2)),
        ("tree_west_02", "screening pine", CellCoord::new(4, 2)),
        ("tree_east_01", "low orchard tree", CellCoord::new(8, 5)),
    ] {
        map.objects.push(EnvironmentObject {
            id: id.to_string(),
            label: label.to_string(),
            kind: EnvironmentObjectKind::Tree(TreeState::Standing),
            cell,
            footprint: (1, 1),
            blocks_sight: true,
            cover: CoverClass::Partial,
            movement_cost_delta: 0.4,
        });
    }

    map.objects.push(EnvironmentObject {
        id: "ridge_stone_01".to_string(),
        label: "loose ridge stone".to_string(),
        kind: EnvironmentObjectKind::Rock(RockState::Stable),
        cell: CellCoord::new(8, 2),
        footprint: (1, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.3,
    });

    map.objects.push(EnvironmentObject {
        id: "ridge_log_01".to_string(),
        label: "ridge rolling log".to_string(),
        kind: EnvironmentObjectKind::Log(LogState::Loose {
            direction: Direction::South,
        }),
        cell: CellCoord::new(7, 3),
        footprint: (2, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.8,
    });

    MissionSpec {
        id: "road_below".to_string(),
        title: "The Road Below".to_string(),
        briefing: MissionBriefing {
            summary: "A southern road climbs toward a low ridge marker. Use limited prep time to shape the approach before the assault.".to_string(),
            primary: "Keep the ridge marker intact until the attack breaks.".to_string(),
            optional_objectives: vec![
                "Stop at least 70% of attackers.".to_string(),
                "Spend less than six minutes of prep time.".to_string(),
                "Avoid friendly-risk hazard paths.".to_string(),
            ],
            intel: vec![
                "Rushers favor the road and shortest climb.".to_string(),
                "Skirmishers look for cover and concealment on the orchard side.".to_string(),
                "The ridge log can punish bunching but crosses a friendly-risk cell.".to_string(),
            ],
        },
        visual_theme: mission_visual_theme_for_theme(MissionTheme::DryRoadBelow),
        objective: MissionObjective {
            label: "Hold the ridge marker".to_string(),
            defend_cell: CellCoord::new(10, 3),
            objective_health: 100,
        },
        prep_time_seconds: 480,
        map,
        starting_tools: ToolLoadout::basic_field_kit(),
        crew: CrewPool {
            crews: 3,
            labor_seconds_available: 480,
        },
        enemy_groups: vec![
            EnemyGroupSpec {
                label: "southern rushers".to_string(),
                count: 12,
                doctrine: EnemyDoctrine::RushShortest,
                spawn: CellCoord::new(1, 7),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 1.0,
                    obstacle_tolerance: 0.35,
                    cover_preference: 0.1,
                },
            },
            EnemyGroupSpec {
                label: "cautious riflemen".to_string(),
                count: 8,
                doctrine: EnemyDoctrine::PreferCover,
                spawn: CellCoord::new(0, 6),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 0.85,
                    obstacle_tolerance: 0.2,
                    cover_preference: 0.65,
                },
            },
            EnemyGroupSpec {
                label: "orchard skirmishers".to_string(),
                count: 6,
                doctrine: EnemyDoctrine::FlankViaConcealment,
                spawn: CellCoord::new(0, 6),
                objective: CellCoord::new(10, 3),
                movement_profile: MovementProfile {
                    base_speed: 0.95,
                    obstacle_tolerance: 0.3,
                    cover_preference: 0.45,
                },
            },
        ],
        defender_positions: vec![
            DefenderPositionSpec {
                id: "ridge_team_01".to_string(),
                label: "ridge rifle pit".to_string(),
                cell: CellCoord::new(9, 3),
                range: 5,
                pressure_per_step: 2,
            },
            DefenderPositionSpec {
                id: "road_team_01".to_string(),
                label: "road overwatch".to_string(),
                cell: CellCoord::new(7, 3),
                range: 4,
                pressure_per_step: 1,
            },
        ],
        constraints: MissionConstraints {
            max_work_orders: 12,
            allow_assault_preview: false,
        },
    }
}

pub fn road_below_seed_orders() -> Vec<(WorkOrderKind, WorkTarget)> {
    vec![
        (
            WorkOrderKind::DigTrench,
            WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 4),
                width: 2,
                height: 1,
            }),
        ),
        (
            WorkOrderKind::RaiseBerm,
            WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 3),
                width: 2,
                height: 1,
            }),
        ),
        (
            WorkOrderKind::FellTree,
            WorkTarget::Object("tree_west_01".to_string()),
        ),
        (
            WorkOrderKind::CutIntoLogs,
            WorkTarget::Object("tree_west_01".to_string()),
        ),
        (
            WorkOrderKind::PlaceStakes,
            WorkTarget::Cell(CellCoord::new(3, 4)),
        ),
    ]
}

pub fn road_below_basic_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "road_below_basic_prep".to_string(),
        label: "Road Below basic prep".to_string(),
        orders: road_below_seed_orders()
            .into_iter()
            .map(|(kind, target)| ScriptedWorkOrder { kind, target })
            .collect(),
    }
}

pub fn road_below_hazard_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "road_below_hazard_prep".to_string(),
        label: "Road Below rolling hazard prep".to_string(),
        orders: vec![ScriptedWorkOrder {
            kind: WorkOrderKind::PrepareRollingLog,
            target: WorkTarget::Object("ridge_log_01".to_string()),
        }],
    }
}

pub fn road_below_no_prep_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "baseline_no_prep".to_string(),
        label: "Baseline: no prep".to_string(),
        orders: Vec::new(),
    }
}

pub fn road_below_trench_line_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "basic_trench_line".to_string(),
        label: "Basic trench line".to_string(),
        orders: vec![ScriptedWorkOrder {
            kind: WorkOrderKind::DigTrench,
            target: WorkTarget::Rect(CellRect {
                origin: CellCoord::new(5, 4),
                width: 2,
                height: 1,
            }),
        }],
    }
}

pub fn road_below_berm_and_stakes_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "berm_and_stakes".to_string(),
        label: "Berm and stakes".to_string(),
        orders: vec![
            ScriptedWorkOrder {
                kind: WorkOrderKind::DigTrench,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(5, 4),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::RaiseBerm,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(5, 3),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::FellTree,
                target: WorkTarget::Object("tree_west_01".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::CutIntoLogs,
                target: WorkTarget::Object("tree_west_01".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::PlaceStakes,
                target: WorkTarget::Cell(CellCoord::new(3, 4)),
            },
        ],
    }
}

pub fn road_below_ridge_chokepoint_script() -> WorkOrderScript {
    WorkOrderScript {
        id: "ridge_chokepoint".to_string(),
        label: "Ridge chokepoint".to_string(),
        orders: vec![
            ScriptedWorkOrder {
                kind: WorkOrderKind::FellTree,
                target: WorkTarget::Object("tree_west_02".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::CutIntoLogs,
                target: WorkTarget::Object("tree_west_02".to_string()),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::DigTrench,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(2, 6),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::RaiseBerm,
                target: WorkTarget::Rect(CellRect {
                    origin: CellCoord::new(2, 5),
                    width: 2,
                    height: 1,
                }),
            },
            ScriptedWorkOrder {
                kind: WorkOrderKind::PlaceStakes,
                target: WorkTarget::Cell(CellCoord::new(4, 5)),
            },
        ],
    }
}

pub fn road_below_overbuilt_bad_plan_script() -> WorkOrderScript {
    let mut orders = road_below_seed_orders()
        .into_iter()
        .map(|(kind, target)| ScriptedWorkOrder { kind, target })
        .collect::<Vec<_>>();
    orders.push(ScriptedWorkOrder {
        kind: WorkOrderKind::FellTree,
        target: WorkTarget::Object("tree_east_01".to_string()),
    });
    WorkOrderScript {
        id: "overbuilt_bad_plan".to_string(),
        label: "Overbuilt bad plan".to_string(),
        orders,
    }
}

pub fn road_below_balance_scripts() -> Vec<WorkOrderScript> {
    vec![
        road_below_no_prep_script(),
        road_below_trench_line_script(),
        road_below_berm_and_stakes_script(),
        road_below_basic_prep_script(),
        road_below_hazard_prep_script(),
        road_below_ridge_chokepoint_script(),
        road_below_overbuilt_bad_plan_script(),
    ]
}

pub fn generate_mission_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    match generator.theme {
        MissionTheme::DryRoadBelow | MissionTheme::RidgeTrap => {
            generate_road_ridge_candidate(generator, seed)
        }
        MissionTheme::OrchardApproach => generate_orchard_approach_candidate(generator, seed),
        MissionTheme::DryWash => generate_dry_wash_candidate(generator, seed),
        MissionTheme::OldWall => generate_old_wall_candidate(generator, seed),
        MissionTheme::SplitApproach => generate_split_approach_candidate(generator, seed),
    }
}

pub fn export_generated_mission_batch(
    out_dir: impl AsRef<Path>,
    generator: MissionGeneratorSpec,
    count: u32,
) -> Result<GeneratedMissionBatchReport> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    write_json(out_dir.join("generator_spec.json"), &generator)?;
    write_ron(out_dir.join("generator_spec.ron"), &generator)?;

    let mut artifacts = Vec::new();
    for index in 0..count {
        let seed = generator
            .seed
            .wrapping_add((index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let candidate = generate_mission_candidate(&generator, seed);
        let candidate_dir = out_dir
            .join("candidates")
            .join(format!("seed_{:04}", index + 1));
        fs::create_dir_all(&candidate_dir)
            .with_context(|| format!("failed to create {}", candidate_dir.display()))?;

        let initial_state = MissionState::from_spec(candidate.spec.clone());
        let initial_routes = initial_state.route_preview();
        write_json(candidate_dir.join("mission.json"), &candidate.spec)?;
        write_ron(candidate_dir.join("mission.ron"), &candidate.spec)?;
        write_json(
            candidate_dir.join("affordance_report.json"),
            &candidate.affordance_report,
        )?;
        write_json(
            candidate_dir.join("enemy_routes_initial.json"),
            &initial_routes,
        )?;
        save_mission_preview_png(candidate_dir.join("mission_preview.png"), &initial_state)?;
        save_mission_route_debug_png(
            candidate_dir.join("route_preview.png"),
            &initial_state,
            &initial_routes,
        )?;
        let visual_report = save_mission_visual_preview_png(
            candidate_dir.join("mission_visual_preview.png"),
            &initial_state,
        )?;
        save_mission_visual_routes_png(
            candidate_dir.join("mission_visual_routes.png"),
            &initial_state,
        )?;
        save_mission_visual_debug_png(
            candidate_dir.join("mission_visual_debug.png"),
            &initial_state,
        )?;
        write_json(
            candidate_dir.join("visual_asset_report.json"),
            &visual_report,
        )?;

        let balance_dir = candidate_dir.join("balance");
        let balance_report = export_mission_balance_run(&balance_dir, candidate.spec.clone())?;
        write_json(candidate_dir.join("balance_summary.json"), &balance_report)?;
        if let Some(best) = balance_report
            .scenarios
            .iter()
            .max_by_key(|scenario| (scenario.rating.stars, scenario.rating.score))
        {
            write_json(candidate_dir.join("assault_summary.json"), &best.summary)?;
        }

        let evaluation =
            evaluate_generated_mission_candidate(&candidate, &initial_routes, &balance_report);
        let mut evaluation = evaluation;
        evaluation.candidate_dir = Some(candidate_dir.to_string_lossy().to_string());
        evaluation.mission_path = Some(
            candidate_dir
                .join("mission.ron")
                .to_string_lossy()
                .to_string(),
        );
        artifacts.push(GeneratedMissionArtifact {
            spec: candidate.spec,
            candidate_dir,
            evaluation,
        });
    }

    artifacts.sort_by(|a, b| {
        b.evaluation
            .tactical_interest_score
            .cmp(&a.evaluation.tactical_interest_score)
            .then_with(|| a.evaluation.seed.cmp(&b.evaluation.seed))
    });

    let mut kept_accepted = Vec::new();
    let mut rejected_artifacts = Vec::new();
    for mut artifact in artifacts {
        if artifact.evaluation.accepted {
            if let Some((duplicate_seed, similarity)) =
                first_similar_candidate(&artifact.evaluation, &kept_accepted)
            {
                mark_duplicate_candidate(&mut artifact.evaluation, duplicate_seed, similarity);
                rejected_artifacts.push(artifact);
            } else {
                kept_accepted.push(artifact);
            }
        } else {
            rejected_artifacts.push(artifact);
        }
    }

    let ranked_candidates = kept_accepted
        .iter()
        .map(|artifact| artifact.evaluation.clone())
        .collect::<Vec<_>>();
    let mut rejected_candidates = rejected_artifacts
        .iter()
        .map(|artifact| artifact.evaluation.clone())
        .collect::<Vec<_>>();
    rejected_candidates.sort_by(|a, b| {
        b.tactical_interest_score
            .cmp(&a.tactical_interest_score)
            .then_with(|| a.seed.cmp(&b.seed))
    });

    for artifact in kept_accepted.iter().chain(rejected_artifacts.iter()) {
        write_json(
            artifact.candidate_dir.join("candidate_evaluation.json"),
            &artifact.evaluation,
        )?;
    }

    save_generated_mission_contact_sheet(
        out_dir.join("top_10_contact_sheet.png"),
        &kept_accepted,
        10,
    )?;
    save_generated_mission_contact_sheet(
        out_dir.join("top_ranked_contact_sheet.png"),
        &kept_accepted,
        10,
    )?;
    save_generated_mission_contact_sheet(
        out_dir.join("accepted_contact_sheet.png"),
        &kept_accepted,
        25,
    )?;
    save_generated_mission_contact_sheet(
        out_dir.join("rejected_contact_sheet.png"),
        &rejected_artifacts,
        25,
    )?;
    save_generated_mission_visual_contact_sheet(
        out_dir.join("top_ranked_visual_contact_sheet.png"),
        &kept_accepted,
        10,
    )?;
    save_generated_mission_visual_contact_sheet(
        out_dir.join("accepted_visual_contact_sheet.png"),
        &kept_accepted,
        25,
    )?;
    save_generated_mission_visual_contact_sheet(
        out_dir.join("rejected_visual_contact_sheet.png"),
        &rejected_artifacts,
        25,
    )?;

    let report = GeneratedMissionBatchReport {
        generator,
        generated_count: count,
        accepted_count: ranked_candidates.len() as u32,
        rejected_count: rejected_candidates.len() as u32,
        ranked_candidates,
        rejected_candidates,
    };
    write_json(
        out_dir.join("ranked_candidates.json"),
        &report.ranked_candidates,
    )?;
    write_json(
        out_dir.join("rejected_candidates.json"),
        &report.rejected_candidates,
    )?;
    let theme_summary = GeneratedMissionThemeSummary {
        theme: report.generator.theme,
        theme_slug: report.generator.theme.slug().to_string(),
        generated_count: report.generated_count,
        accepted_count: report.accepted_count,
        rejected_count: report.rejected_count,
        best_candidate: report.ranked_candidates.first().cloned(),
    };
    let browser_index = generated_mission_browser_index(
        out_dir,
        vec![theme_summary],
        &report.ranked_candidates,
        &report.rejected_candidates,
    );
    write_json(out_dir.join("browser_index.json"), &browser_index)?;
    write_json(out_dir.join("generator_summary.json"), &report)?;
    Ok(report)
}

pub fn export_generated_mission_theme_batch(
    out_dir: impl AsRef<Path>,
    base_generator: MissionGeneratorSpec,
    count_per_theme: u32,
    themes: &[MissionTheme],
) -> Result<GeneratedMissionThemeBatchReport> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let per_theme_dir = out_dir.join("per_theme");
    let contact_sheet_dir = out_dir.join("contact_sheets");
    fs::create_dir_all(&per_theme_dir)
        .with_context(|| format!("failed to create {}", per_theme_dir.display()))?;
    fs::create_dir_all(&contact_sheet_dir)
        .with_context(|| format!("failed to create {}", contact_sheet_dir.display()))?;

    let mut theme_summaries = Vec::new();
    let mut all_ranked_candidates = Vec::new();
    let mut all_rejected_candidates = Vec::new();
    for (theme_index, theme) in themes.iter().copied().enumerate() {
        let mut generator = base_generator.clone();
        generator.theme = theme;
        generator.seed = base_generator
            .seed
            .wrapping_add((theme_index as u64).wrapping_mul(0xd1b5_4a32_d192_ed03));
        let theme_report = export_generated_mission_batch(
            per_theme_dir.join(theme.slug()),
            generator,
            count_per_theme,
        )?;
        theme_summaries.push(GeneratedMissionThemeSummary {
            theme,
            theme_slug: theme.slug().to_string(),
            generated_count: theme_report.generated_count,
            accepted_count: theme_report.accepted_count,
            rejected_count: theme_report.rejected_count,
            best_candidate: theme_report.ranked_candidates.first().cloned(),
        });
        all_ranked_candidates.extend(theme_report.ranked_candidates);
        all_rejected_candidates.extend(theme_report.rejected_candidates);
    }

    all_ranked_candidates.sort_by(|a, b| {
        b.tactical_interest_score
            .cmp(&a.tactical_interest_score)
            .then_with(|| a.theme_slug.cmp(&b.theme_slug))
            .then_with(|| a.seed.cmp(&b.seed))
    });
    all_rejected_candidates.sort_by(|a, b| {
        a.rejection_kinds
            .first()
            .map(|kind| format!("{kind:?}"))
            .cmp(&b.rejection_kinds.first().map(|kind| format!("{kind:?}")))
            .then_with(|| b.tactical_interest_score.cmp(&a.tactical_interest_score))
    });

    save_generated_mission_evaluation_contact_sheet(
        contact_sheet_dir.join("top_ranked_all_themes.png"),
        &all_ranked_candidates,
        25,
    )?;
    save_generated_mission_evaluation_contact_sheet(
        contact_sheet_dir.join("accepted_by_theme.png"),
        &all_ranked_candidates,
        30,
    )?;
    save_generated_mission_evaluation_contact_sheet(
        contact_sheet_dir.join("rejected_by_reason.png"),
        &all_rejected_candidates,
        30,
    )?;
    save_generated_mission_evaluation_visual_contact_sheet(
        contact_sheet_dir.join("top_ranked_all_themes_visual.png"),
        &all_ranked_candidates,
        25,
    )?;
    save_generated_mission_evaluation_visual_contact_sheet(
        contact_sheet_dir.join("accepted_by_theme_visual.png"),
        &all_ranked_candidates,
        30,
    )?;
    save_generated_mission_evaluation_visual_contact_sheet(
        contact_sheet_dir.join("rejected_by_reason_visual.png"),
        &all_rejected_candidates,
        30,
    )?;

    let report = GeneratedMissionThemeBatchReport {
        seed: base_generator.seed,
        count_per_theme,
        total_generated_count: count_per_theme * themes.len() as u32,
        total_accepted_count: all_ranked_candidates.len() as u32,
        total_rejected_count: all_rejected_candidates.len() as u32,
        theme_summaries,
        all_ranked_candidates,
        all_rejected_candidates,
    };
    write_json(out_dir.join("theme_summary.json"), &report)?;
    write_json(
        out_dir.join("all_ranked_candidates.json"),
        &report.all_ranked_candidates,
    )?;
    write_json(
        out_dir.join("all_rejected_candidates.json"),
        &report.all_rejected_candidates,
    )?;
    let browser_index = generated_mission_browser_index(
        out_dir,
        report.theme_summaries.clone(),
        &report.all_ranked_candidates,
        &report.all_rejected_candidates,
    );
    write_json(out_dir.join("browser_index.json"), &browser_index)?;
    Ok(report)
}

pub fn export_generated_mission_pack(
    out_dir: impl AsRef<Path>,
    base_generator: MissionGeneratorSpec,
    requested_missions: u32,
    candidate_count_per_theme: u32,
) -> Result<GeneratedMissionPackSummary> {
    export_generated_mission_pack_with_curve(
        out_dir,
        base_generator,
        requested_missions,
        candidate_count_per_theme,
        MissionPackCurve::Balanced,
    )
}

pub fn export_generated_mission_pack_with_curve(
    out_dir: impl AsRef<Path>,
    base_generator: MissionGeneratorSpec,
    requested_missions: u32,
    candidate_count_per_theme: u32,
    curve: MissionPackCurve,
) -> Result<GeneratedMissionPackSummary> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let source_batch_dir = out_dir.join("source_candidates");
    let theme_report = export_generated_mission_theme_batch(
        &source_batch_dir,
        base_generator.clone(),
        candidate_count_per_theme,
        &MissionTheme::GENERATABLE,
    )?;
    let selected = select_mission_pack_candidates(
        &theme_report.all_ranked_candidates,
        requested_missions as usize,
        curve,
    );
    let mut entries = Vec::new();
    let mut difficulty_curve = Vec::new();
    let mut complexity_curve = Vec::new();
    for (index, candidate) in selected.iter().enumerate() {
        let order = index as u32 + 1;
        let difficulty_score = generated_mission_difficulty_score(candidate);
        let complexity_score = generated_mission_complexity_score(candidate);
        let mission_path = candidate
            .mission_path
            .clone()
            .unwrap_or_else(|| "<missing mission path>".to_string());
        entries.push(GeneratedMissionPackEntry {
            order,
            title: candidate.title.clone(),
            mission_id: candidate.mission_id.clone(),
            theme: candidate.theme,
            theme_slug: candidate.theme_slug.clone(),
            seed: candidate.seed,
            tactical_interest_score: candidate.tactical_interest_score,
            difficulty_score,
            complexity_score,
            mission_path,
            best_plan_label: candidate.best_plan_label.clone(),
        });
        difficulty_curve.push(GeneratedMissionDifficultyPoint {
            order,
            title: candidate.title.clone(),
            theme_slug: candidate.theme_slug.clone(),
            difficulty_score,
            baseline_score: candidate.baseline_rating.score,
            best_score: candidate.best_rating.score,
            tactical_interest_score: candidate.tactical_interest_score,
        });
        complexity_curve.push(generated_mission_complexity_point(
            order,
            candidate,
            complexity_score,
        ));
    }
    let pack = GeneratedMissionPack {
        id: format!("generated_pack_{:016x}", base_generator.seed),
        label: format!("Generated mission pack {}", base_generator.seed),
        seed: base_generator.seed,
        missions: entries,
    };
    let mut notes = Vec::new();
    if pack.missions.len() < requested_missions as usize {
        notes.push(format!(
            "Only {} accepted candidate(s) were available for {} requested mission(s).",
            pack.missions.len(),
            requested_missions
        ));
    }
    if pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::RidgeTrap)
    {
        notes.push("Pack includes a ridge-trap / rolling-hazard mission.".to_string());
    }
    if pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::SplitApproach)
    {
        notes.push("Pack includes a split-approach prioritization mission.".to_string());
    }
    if pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::OrchardApproach)
    {
        notes.push("Pack includes a tree/material-heavy orchard mission.".to_string());
    }
    let pack_diversity_report =
        generated_mission_pack_diversity_report(&pack, &difficulty_curve, &complexity_curve);
    let summary = GeneratedMissionPackSummary {
        pack,
        curve,
        requested_missions,
        candidate_count_per_theme,
        source_batch_dir: source_batch_dir.to_string_lossy().to_string(),
        total_generated_count: theme_report.total_generated_count,
        total_accepted_count: theme_report.total_accepted_count,
        difficulty_curve,
        complexity_curve,
        pack_diversity_report,
        notes,
    };
    write_ron(out_dir.join("mission_pack.ron"), &summary.pack)?;
    write_json(out_dir.join("mission_pack_summary.json"), &summary)?;
    write_json(
        out_dir.join("difficulty_curve.json"),
        &summary.difficulty_curve,
    )?;
    write_json(
        out_dir.join("complexity_curve.json"),
        &summary.complexity_curve,
    )?;
    write_json(
        out_dir.join("pack_diversity_report.json"),
        &summary.pack_diversity_report,
    )?;
    save_generated_mission_evaluation_contact_sheet(
        out_dir.join("mission_pack_contact_sheet.png"),
        &selected,
        selected.len(),
    )?;
    save_generated_mission_evaluation_visual_contact_sheet(
        out_dir.join("mission_pack_visual_sheet.png"),
        &selected,
        selected.len(),
    )?;
    Ok(summary)
}

pub fn export_theme_calibration_report(
    out_dir: impl AsRef<Path>,
    base_generator: MissionGeneratorSpec,
    count_per_theme: u32,
) -> Result<ThemeCalibrationReport> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let theme_report = export_generated_mission_theme_batch(
        out_dir,
        base_generator.clone(),
        count_per_theme,
        &MissionTheme::GENERATABLE,
    )?;
    let calibration = build_theme_calibration_report(&theme_report);
    write_json(out_dir.join("theme_calibration_report.json"), &calibration)?;
    save_theme_calibration_summary_image(
        out_dir.join("theme_calibration_summary.png"),
        &calibration,
    )?;
    save_rejection_reason_histogram_image(
        out_dir.join("rejection_reason_histogram.png"),
        &calibration.global_rejection_reasons,
    )?;
    save_difficulty_complexity_scatter_image(
        out_dir.join("difficulty_complexity_scatter.png"),
        &theme_report.all_ranked_candidates,
    )?;
    Ok(calibration)
}

pub fn load_generated_mission_browser_index(
    path: impl AsRef<Path>,
) -> Result<GeneratedMissionBrowserIndex> {
    let path = path.as_ref();
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse mission browser index {}", path.display()))
}

fn generated_mission_browser_index(
    out_dir: &Path,
    theme_summaries: Vec<GeneratedMissionThemeSummary>,
    ranked_candidates: &[GeneratedMissionEvaluation],
    rejected_candidates: &[GeneratedMissionEvaluation],
) -> GeneratedMissionBrowserIndex {
    let candidates = ranked_candidates
        .iter()
        .chain(rejected_candidates.iter())
        .map(browser_entry_from_evaluation)
        .collect::<Vec<_>>();
    GeneratedMissionBrowserIndex {
        source_dir: out_dir.to_string_lossy().to_string(),
        generated_count: candidates.len() as u32,
        accepted_count: ranked_candidates.len() as u32,
        rejected_count: rejected_candidates.len() as u32,
        theme_summaries,
        candidates,
    }
}

fn browser_entry_from_evaluation(
    evaluation: &GeneratedMissionEvaluation,
) -> GeneratedMissionBrowserEntry {
    GeneratedMissionBrowserEntry {
        title: evaluation.title.clone(),
        mission_id: evaluation.mission_id.clone(),
        theme: evaluation.theme,
        theme_slug: evaluation.theme_slug.clone(),
        seed: evaluation.seed,
        accepted: evaluation.accepted,
        tactical_interest_score: evaluation.tactical_interest_score,
        best_plan_label: evaluation.best_plan_label.clone(),
        baseline_score: evaluation.baseline_rating.score,
        best_score: evaluation.best_rating.score,
        best_minus_baseline: evaluation.plan_sensitivity.best_minus_baseline,
        best_minus_worst: evaluation.plan_sensitivity.best_minus_worst,
        route_diversity_score: evaluation.route_diversity_score,
        hazard_viability_score: evaluation.rolling_hazard_score,
        local_material_score: evaluation.local_material_score,
        difficulty_score: generated_mission_difficulty_score(evaluation),
        complexity_score: generated_mission_complexity_score(evaluation),
        top_rejection_kind: evaluation.rejection_kinds.first().copied(),
        top_rejection_reason: evaluation.rejection_reasons.first().cloned(),
        primary_affordance: primary_affordance_label(evaluation),
        mission_path: evaluation.mission_path.clone(),
        candidate_dir: evaluation.candidate_dir.clone(),
    }
}

fn primary_affordance_label(evaluation: &GeneratedMissionEvaluation) -> String {
    match evaluation.theme {
        MissionTheme::OrchardApproach => "tree/material dilemma".to_string(),
        MissionTheme::DryWash => "low wash / dead ground".to_string(),
        MissionTheme::RidgeTrap => "slope / rolling hazard".to_string(),
        MissionTheme::OldWall => "wall cover / breach lane".to_string(),
        MissionTheme::SplitApproach => "two approach lanes".to_string(),
        MissionTheme::DryRoadBelow => {
            if evaluation.rolling_hazard_score >= 0.45 {
                "road ridge / rolling hazard".to_string()
            } else if evaluation.affordance_report.tree_count >= 5 {
                "road ridge / local timber".to_string()
            } else {
                "road ridge defense".to_string()
            }
        }
    }
}

fn select_mission_pack_candidates(
    ranked_candidates: &[GeneratedMissionEvaluation],
    requested_count: usize,
    curve: MissionPackCurve,
) -> Vec<GeneratedMissionEvaluation> {
    if requested_count == 0 {
        return Vec::new();
    }

    let mut selected = Vec::new();
    let mut selected_ids = HashSet::new();
    for theme in mission_pack_theme_sequence(curve) {
        if selected.len() >= requested_count {
            break;
        }
        let candidate = best_pack_candidate_for_theme(
            ranked_candidates,
            theme,
            curve,
            selected.len() + 1,
            false,
        )
        .into_iter()
        .find(|candidate| !selected_ids.contains(&candidate.mission_id));
        if let Some(candidate) = candidate {
            selected_ids.insert(candidate.mission_id.clone());
            selected.push(candidate);
        }
    }

    for candidate in ranked_candidates {
        if selected.len() >= requested_count {
            break;
        }
        if candidate.mission_path.is_some()
            && generated_mission_pack_prefers(candidate)
            && selected_ids.insert(candidate.mission_id.clone())
        {
            selected.push(candidate.clone());
        }
    }
    for candidate in ranked_candidates {
        if selected.len() >= requested_count {
            break;
        }
        if candidate.mission_path.is_some() && selected_ids.insert(candidate.mission_id.clone()) {
            selected.push(candidate.clone());
        }
    }

    selected.sort_by(|a, b| match curve {
        MissionPackCurve::Balanced => generated_mission_difficulty_score(a)
            .cmp(&generated_mission_difficulty_score(b))
            .then_with(|| b.tactical_interest_score.cmp(&a.tactical_interest_score))
            .then_with(|| a.seed.cmp(&b.seed)),
        MissionPackCurve::Tutorial => generated_mission_difficulty_score(a)
            .cmp(&generated_mission_difficulty_score(b))
            .then_with(|| {
                generated_mission_complexity_score(a).cmp(&generated_mission_complexity_score(b))
            })
            .then_with(|| tutorial_theme_order(a.theme).cmp(&tutorial_theme_order(b.theme))),
    });
    selected
}

fn mission_pack_theme_sequence(curve: MissionPackCurve) -> [MissionTheme; 6] {
    match curve {
        MissionPackCurve::Balanced => MissionTheme::GENERATABLE,
        MissionPackCurve::Tutorial => [
            MissionTheme::DryRoadBelow,
            MissionTheme::OrchardApproach,
            MissionTheme::DryWash,
            MissionTheme::RidgeTrap,
            MissionTheme::SplitApproach,
            MissionTheme::OldWall,
        ],
    }
}

fn best_pack_candidate_for_theme(
    ranked_candidates: &[GeneratedMissionEvaluation],
    theme: MissionTheme,
    curve: MissionPackCurve,
    order: usize,
    allow_fallback: bool,
) -> Option<GeneratedMissionEvaluation> {
    let mut candidates = ranked_candidates
        .iter()
        .filter(|candidate| {
            candidate.theme == theme
                && candidate.mission_path.is_some()
                && (allow_fallback || generated_mission_pack_prefers(candidate))
        })
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        let a_preferred = generated_mission_pack_prefers(a);
        let b_preferred = generated_mission_pack_prefers(b);
        b_preferred
            .cmp(&a_preferred)
            .then_with(|| match curve {
                MissionPackCurve::Balanced => b
                    .tactical_interest_score
                    .cmp(&a.tactical_interest_score)
                    .then_with(|| {
                        generated_mission_difficulty_score(a)
                            .cmp(&generated_mission_difficulty_score(b))
                    }),
                MissionPackCurve::Tutorial => {
                    let target_complexity = tutorial_target_complexity(order);
                    let a_distance =
                        (generated_mission_complexity_score(a) - target_complexity).abs();
                    let b_distance =
                        (generated_mission_complexity_score(b) - target_complexity).abs();
                    a_distance
                        .cmp(&b_distance)
                        .then_with(|| {
                            generated_mission_difficulty_score(a)
                                .cmp(&generated_mission_difficulty_score(b))
                        })
                        .then_with(|| b.tactical_interest_score.cmp(&a.tactical_interest_score))
                }
            })
            .then_with(|| a.seed.cmp(&b.seed))
    });
    candidates.into_iter().next()
}

fn tutorial_theme_order(theme: MissionTheme) -> u8 {
    match theme {
        MissionTheme::DryRoadBelow => 0,
        MissionTheme::OrchardApproach => 1,
        MissionTheme::DryWash => 2,
        MissionTheme::RidgeTrap => 3,
        MissionTheme::SplitApproach => 4,
        MissionTheme::OldWall => 5,
    }
}

fn tutorial_target_complexity(order: usize) -> i32 {
    match order {
        0 | 1 => 22,
        2 => 34,
        3 => 46,
        4 => 58,
        5 => 70,
        _ => 78,
    }
}

fn generated_mission_pack_prefers(evaluation: &GeneratedMissionEvaluation) -> bool {
    let plan = evaluation.best_plan_label.to_ascii_lowercase();
    !plan.contains("baseline")
        && !plan.contains("overbuilt")
        && evaluation.plan_sensitivity.best_minus_baseline >= 5
}

fn generated_mission_difficulty_score(evaluation: &GeneratedMissionEvaluation) -> i32 {
    let baseline_pressure = (100 - evaluation.baseline_rating.score).clamp(0, 100);
    let best_pressure = (100 - evaluation.best_rating.score).clamp(0, 100) / 2;
    let sensitivity = (evaluation.plan_sensitivity.best_minus_worst / 4).clamp(0, 25);
    let hazard_complexity = (evaluation.rolling_hazard_score * 10.0).round() as i32;
    baseline_pressure + best_pressure + sensitivity + hazard_complexity
}

fn generated_mission_complexity_score(evaluation: &GeneratedMissionEvaluation) -> i32 {
    let route_count = evaluation.affordance_report.route_count.min(4) as i32 * 3;
    let doctrine_count = evaluation
        .scenarios
        .first()
        .map(|_| (evaluation.doctrine_spread_score * 12.0).round() as i32)
        .unwrap_or(0);
    let material_types = generated_mission_material_type_count(evaluation) as i32 * 4;
    let hazard = (evaluation.rolling_hazard_score * 10.0).round() as i32;
    let height = (evaluation.height_interest_score * 14.0).round() as i32;
    let affordances = generated_mission_meaningful_affordance_count(evaluation) as i32 * 4;
    (route_count + doctrine_count + material_types + hazard + height + affordances).clamp(0, 100)
}

fn generated_mission_material_type_count(evaluation: &GeneratedMissionEvaluation) -> u32 {
    let mut count = 0;
    if evaluation.affordance_report.tree_count > 0 {
        count += 1;
    }
    if evaluation.affordance_report.loose_log_count > 0 {
        count += 1;
    }
    if evaluation.affordance_report.trenchable_soil_cells > 0 {
        count += 1;
    }
    if evaluation.theme == MissionTheme::OldWall {
        count += 1;
    }
    count
}

fn generated_mission_meaningful_affordance_count(evaluation: &GeneratedMissionEvaluation) -> u32 {
    let mut count = 0;
    if evaluation.affordance_report.road_cell_count >= 8 {
        count += 1;
    }
    if evaluation.affordance_report.ridge_cell_count >= 6 {
        count += 1;
    }
    if evaluation.affordance_report.tree_count >= 3 {
        count += 1;
    }
    if evaluation.affordance_report.trenchable_soil_cells >= 36 {
        count += 1;
    }
    if evaluation.rolling_hazard_score > 0.0 {
        count += 1;
    }
    if evaluation.route_diversity_score >= 0.4 {
        count += 1;
    }
    count
}

fn generated_mission_complexity_point(
    order: u32,
    evaluation: &GeneratedMissionEvaluation,
    complexity_score: i32,
) -> GeneratedMissionComplexityPoint {
    GeneratedMissionComplexityPoint {
        order,
        title: evaluation.title.clone(),
        theme_slug: evaluation.theme_slug.clone(),
        complexity_score,
        route_count: evaluation.affordance_report.route_count,
        doctrine_count: ((evaluation.doctrine_spread_score * 3.0).round() as u32).max(1),
        material_types_present: generated_mission_material_type_count(evaluation),
        hazard_count: (evaluation.rolling_hazard_score > 0.0) as u32,
        height_interest_score: evaluation.height_interest_score,
        meaningful_affordances: generated_mission_meaningful_affordance_count(evaluation),
    }
}

fn generated_mission_pack_diversity_report(
    pack: &GeneratedMissionPack,
    difficulty_curve: &[GeneratedMissionDifficultyPoint],
    complexity_curve: &[GeneratedMissionComplexityPoint],
) -> GeneratedMissionPackDiversityReport {
    let themes = pack
        .missions
        .iter()
        .map(|mission| mission.theme)
        .collect::<HashSet<_>>();
    let difficulty_curve_is_monotonic = difficulty_curve
        .windows(2)
        .all(|window| window[0].difficulty_score <= window[1].difficulty_score);
    let complexity_curve_is_monotonic = complexity_curve
        .windows(2)
        .all(|window| window[0].complexity_score <= window[1].complexity_score + 5);
    let has_tree_material_mission = pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::OrchardApproach);
    let has_hazard_mission = pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::RidgeTrap);
    let has_split_approach_mission = pack
        .missions
        .iter()
        .any(|mission| mission.theme == MissionTheme::SplitApproach);
    let repeated_theme_count = pack.missions.len().saturating_sub(themes.len()) as u32;
    let mut notes = Vec::new();
    if has_tree_material_mission {
        notes.push("Includes at least one tree/material dilemma.".to_string());
    } else {
        notes.push("Missing a tree/material dilemma mission.".to_string());
    }
    if has_hazard_mission {
        notes.push("Includes at least one rolling-hazard mission.".to_string());
    } else {
        notes.push("Missing a rolling-hazard mission.".to_string());
    }
    if has_split_approach_mission {
        notes.push("Includes at least one split-route prioritization mission.".to_string());
    } else {
        notes.push("Missing a split-route prioritization mission.".to_string());
    }
    if repeated_theme_count > 0 {
        notes.push(format!(
            "Pack repeats {repeated_theme_count} theme slot(s) because not enough preferred candidates were available."
        ));
    }
    if !difficulty_curve_is_monotonic {
        notes.push("Difficulty curve has a local spike or dip.".to_string());
    }
    if !complexity_curve_is_monotonic {
        notes.push("Complexity curve has a local spike or dip.".to_string());
    }

    GeneratedMissionPackDiversityReport {
        mission_count: pack.missions.len() as u32,
        unique_theme_count: themes.len() as u32,
        repeated_theme_count,
        has_tree_material_mission,
        has_hazard_mission,
        has_split_approach_mission,
        difficulty_curve_is_monotonic,
        complexity_curve_is_monotonic,
        notes,
    }
}

fn build_theme_calibration_report(
    theme_report: &GeneratedMissionThemeBatchReport,
) -> ThemeCalibrationReport {
    let mut theme_summaries = Vec::new();
    for theme in MissionTheme::GENERATABLE {
        let accepted = theme_report
            .all_ranked_candidates
            .iter()
            .filter(|candidate| candidate.theme == theme)
            .collect::<Vec<_>>();
        let rejected = theme_report
            .all_rejected_candidates
            .iter()
            .filter(|candidate| candidate.theme == theme)
            .collect::<Vec<_>>();
        let generated_count = accepted.len() as u32 + rejected.len() as u32;
        let accepted_count = accepted.len() as u32;
        let rejected_count = rejected.len() as u32;
        let acceptance_rate = ratio(accepted_count, generated_count);
        let (target_min, target_max, target_difficulty) = theme_calibration_target(theme);
        let all = accepted
            .iter()
            .copied()
            .chain(rejected.iter().copied())
            .collect::<Vec<_>>();
        let rejection_reasons = rejection_histogram(rejected.iter().copied(), rejected_count);
        let most_common_rejection = rejection_reasons.first().map(|entry| entry.kind);
        let average_score = average_i32(
            all.iter()
                .map(|candidate| candidate.tactical_interest_score),
        );
        let best_score = all
            .iter()
            .map(|candidate| candidate.tactical_interest_score)
            .max()
            .unwrap_or(0);
        let average_difficulty_score = average_i32(
            all.iter()
                .map(|candidate| generated_mission_difficulty_score(candidate)),
        );
        let average_complexity_score = average_i32(
            all.iter()
                .map(|candidate| generated_mission_complexity_score(candidate)),
        );
        let average_plan_sensitivity = average_i32(
            all.iter()
                .map(|candidate| candidate.plan_sensitivity.best_minus_worst),
        );
        let average_route_diversity =
            average_f32(all.iter().map(|candidate| candidate.route_diversity_score));
        let average_hazard_usefulness =
            average_f32(all.iter().map(|candidate| candidate.rolling_hazard_score));
        let average_material_affordance =
            average_f32(all.iter().map(|candidate| candidate.local_material_score));
        let recommendations = theme_calibration_recommendations(
            theme,
            acceptance_rate,
            target_min,
            target_max,
            &rejection_reasons,
        );
        theme_summaries.push(ThemeCalibrationSummary {
            theme,
            theme_slug: theme.slug().to_string(),
            generated_count,
            accepted_count,
            rejected_count,
            acceptance_rate,
            target_acceptance_min: target_min,
            target_acceptance_max: target_max,
            target_difficulty: target_difficulty.to_string(),
            average_score,
            best_score,
            average_difficulty_score,
            average_complexity_score,
            average_plan_sensitivity,
            average_route_diversity,
            average_hazard_usefulness,
            average_material_affordance,
            most_common_rejection,
            rejection_reasons,
            recommendations,
        });
    }

    let global_rejection_reasons = rejection_histogram(
        theme_report.all_rejected_candidates.iter(),
        theme_report.total_rejected_count,
    );
    let mut notes = Vec::new();
    for summary in &theme_summaries {
        if summary.acceptance_rate < summary.target_acceptance_min {
            notes.push(format!(
                "{} acceptance is below target ({:.0}% vs {:.0}%).",
                summary.theme.label(),
                summary.acceptance_rate * 100.0,
                summary.target_acceptance_min * 100.0
            ));
        } else if summary.acceptance_rate > summary.target_acceptance_max {
            notes.push(format!(
                "{} acceptance is above target ({:.0}% vs {:.0}%).",
                summary.theme.label(),
                summary.acceptance_rate * 100.0,
                summary.target_acceptance_max * 100.0
            ));
        }
    }
    if notes.is_empty() {
        notes.push("All theme acceptance rates are inside target bands.".to_string());
    }

    ThemeCalibrationReport {
        seed: theme_report.seed,
        count_per_theme: theme_report.count_per_theme,
        total_generated_count: theme_report.total_generated_count,
        total_accepted_count: theme_report.total_accepted_count,
        total_rejected_count: theme_report.total_rejected_count,
        theme_summaries,
        global_rejection_reasons,
        notes,
    }
}

fn theme_calibration_target(theme: MissionTheme) -> (f32, f32, &'static str) {
    match theme {
        MissionTheme::DryRoadBelow => (0.15, 0.35, "early"),
        MissionTheme::OrchardApproach => (0.10, 0.30, "early-mid"),
        MissionTheme::DryWash => (0.10, 0.30, "mid"),
        MissionTheme::RidgeTrap => (0.08, 0.25, "mid"),
        MissionTheme::OldWall => (0.08, 0.25, "late"),
        MissionTheme::SplitApproach => (0.08, 0.20, "late"),
    }
}

fn theme_calibration_recommendations(
    theme: MissionTheme,
    acceptance_rate: f32,
    target_min: f32,
    target_max: f32,
    rejection_reasons: &[RejectionReasonHistogramEntry],
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if acceptance_rate < target_min {
        recommendations.push("Acceptance is low; loosen or enrich this theme grammar.".to_string());
    }
    if acceptance_rate > target_max {
        recommendations
            .push("Acceptance is high; add stricter tactical-interest gates.".to_string());
    }
    if let Some(top) = rejection_reasons.first() {
        if top.ratio >= 0.35 {
            recommendations.push(theme_rejection_recommendation(theme, top.kind).to_string());
        }
    }
    if recommendations.is_empty() {
        recommendations
            .push("Theme is inside target bands; keep monitoring pack diversity.".to_string());
    }
    recommendations
}

fn theme_rejection_recommendation(
    theme: MissionTheme,
    kind: GeneratedMissionRejectionKind,
) -> &'static str {
    match (theme, kind) {
        (MissionTheme::RidgeTrap, GeneratedMissionRejectionKind::NoHazardOpportunity) => {
            "Increase slope/log co-location or route crossing probability."
        }
        (MissionTheme::SplitApproach, GeneratedMissionRejectionKind::NoRouteDiversity) => {
            "Increase spawn separation or route-lane obstacles."
        }
        (MissionTheme::OrchardApproach, GeneratedMissionRejectionKind::NoUsefulMaterials) => {
            "Increase useful tree clusters near plausible prep zones."
        }
        (MissionTheme::DryWash, GeneratedMissionRejectionKind::TerrainTooFlat) => {
            "Deepen the wash or add stronger overlook height contrast."
        }
        (MissionTheme::OldWall, GeneratedMissionRejectionKind::NoRouteDiversity) => {
            "Vary breach/cover gaps so wall routes diverge more clearly."
        }
        (_, GeneratedMissionRejectionKind::TooEasyNoPrep) => {
            "Raise baseline pressure or expose the objective to more doctrine variation."
        }
        (_, GeneratedMissionRejectionKind::TooHardAllPlansFail) => {
            "Add stronger local materials or reduce enemy pressure."
        }
        (_, GeneratedMissionRejectionKind::SpawnTooClose) => {
            "Move spawns farther from the objective."
        }
        (_, GeneratedMissionRejectionKind::NoHazardOpportunity) => {
            "Add optional hazard affordance or relax hazard requirement for this theme."
        }
        (_, GeneratedMissionRejectionKind::NoUsefulMaterials) => {
            "Increase local material affordances near the defensive problem."
        }
        (_, GeneratedMissionRejectionKind::DuplicateCandidate) => {
            "Increase layout variance or reduce repeated route/objective patterns."
        }
        _ => {
            "Inspect the top rejected candidates and tune the terrain grammar around that failure."
        }
    }
}

fn rejection_histogram<'a>(
    evaluations: impl Iterator<Item = &'a GeneratedMissionEvaluation>,
    total_rejected_count: u32,
) -> Vec<RejectionReasonHistogramEntry> {
    let mut counts: HashMap<GeneratedMissionRejectionKind, u32> = HashMap::new();
    for evaluation in evaluations {
        for kind in &evaluation.rejection_kinds {
            *counts.entry(*kind).or_default() += 1;
        }
    }
    let mut entries = counts
        .into_iter()
        .map(|(kind, count)| RejectionReasonHistogramEntry {
            kind,
            count,
            ratio: ratio(count, total_rejected_count),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| format!("{:?}", a.kind).cmp(&format!("{:?}", b.kind)))
    });
    entries
}

fn average_i32(values: impl Iterator<Item = i32>) -> f32 {
    let mut total = 0;
    let mut count = 0;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total as f32 / count as f32
    }
}

fn average_f32(values: impl Iterator<Item = f32>) -> f32 {
    let mut total = 0.0;
    let mut count = 0;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn ratio(numerator: u32, denominator: u32) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn generate_road_ridge_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    let mut rng = MissionRng::new(seed);
    let mut map = MissionMap::new(12, 8, MissionCell::new(1, GroundKind::Grass));
    let road_y = 4;
    let south_spawn = CellCoord::new(rng.range_u32(0, 1), 7);
    let west_spawn = CellCoord::new(0, rng.range_u32(5, 6));
    map.spawn_cells.push(south_spawn);
    map.spawn_cells.push(west_spawn);

    for x in 0..map.width {
        set_ground(&mut map, CellCoord::new(x, road_y), GroundKind::Road);
        if rng.chance(1, 5) && road_y > 0 {
            set_ground(&mut map, CellCoord::new(x, road_y - 1), GroundKind::Dirt);
        }
    }

    let ridge_start = 7 + rng.range_u32(0, 1);
    for x in ridge_start..map.width {
        for y in 2..=3 {
            if let Some(cell) = map.cell_mut(CellCoord::new(x, y)) {
                cell.height = 2;
            }
        }
    }
    if rng.chance(1, 2) {
        for x in ridge_start + 1..map.width {
            if let Some(cell) = map.cell_mut(CellCoord::new(x, 1)) {
                cell.height = 2;
            }
        }
    }

    let objective = CellCoord::new(10, 3 - rng.range_u32(0, 1));
    let west_tree_x = 3 + rng.range_u32(0, 1);
    let west_tree_two_x = if west_tree_x == 3 { 4 } else { 3 };
    add_tree_object(
        &mut map,
        "tree_west_01",
        "roadside pine",
        CellCoord::new(west_tree_x, 2),
    );
    add_tree_object(
        &mut map,
        "tree_west_02",
        "screening pine",
        CellCoord::new(west_tree_two_x, 2),
    );
    add_tree_object(
        &mut map,
        "tree_east_01",
        "low orchard tree",
        CellCoord::new(8 + rng.range_u32(0, 1), 5),
    );
    for index in 0..rng.range_u32(1, 3) {
        let x = 1 + rng.range_u32(0, 3);
        let y = 5 + rng.range_u32(0, 1);
        add_tree_object(
            &mut map,
            format!("tree_extra_{index:02}"),
            "scrub tree",
            CellCoord::new(x, y),
        );
    }

    map.objects.push(EnvironmentObject {
        id: "ridge_stone_01".to_string(),
        label: "loose ridge stone".to_string(),
        kind: EnvironmentObjectKind::Rock(RockState::Stable),
        cell: CellCoord::new(8 + rng.range_u32(0, 1), 2),
        footprint: (1, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.3,
    });
    map.objects.push(EnvironmentObject {
        id: "ridge_log_01".to_string(),
        label: "ridge rolling log".to_string(),
        kind: EnvironmentObjectKind::Log(LogState::Loose {
            direction: Direction::South,
        }),
        cell: CellCoord::new(7, 3),
        footprint: (2, 1),
        blocks_sight: false,
        cover: CoverClass::Light,
        movement_cost_delta: 0.8,
    });

    let difficulty_extra = match generator.difficulty {
        DifficultyBand::Intro => 0,
        DifficultyBand::Standard => 2,
        DifficultyBand::Hard => 5,
    };
    let title_prefix = match generator.theme {
        MissionTheme::RidgeTrap => "Ridge Trap",
        MissionTheme::DryRoadBelow => "Generated Road Below",
        other => other.label(),
    };
    let title = format!("{title_prefix} {:04x}", (seed & 0xffff) as u32);
    let spec = MissionSpec {
        id: format!("procgen_{}_{seed:016x}", generator.theme.slug()),
        title,
        briefing: MissionBriefing {
            summary: format!(
                "{} seed {seed}: a compact generated road/ridge defense problem.",
                generator.theme.label()
            ),
            primary: "Keep the ridge marker intact through the assault.".to_string(),
            optional_objectives: vec![
                "Find a prep plan that outperforms no prep.".to_string(),
                "Use local material without wasting defenses.".to_string(),
                "Treat rolling logs as a risky opportunity, not a guaranteed answer.".to_string(),
            ],
            intel: vec![
                "Rushers test the road line.".to_string(),
                "Cover-seekers and skirmishers probe orchard and side cover.".to_string(),
                "The generator requires a ridge, timber, trenchable soil, and at least one rolling-log opportunity.".to_string(),
            ],
        },
        visual_theme: mission_visual_theme_for_theme(generator.theme),
        objective: MissionObjective {
            label: "Hold the ridge marker".to_string(),
            defend_cell: objective,
            objective_health: 100,
        },
        prep_time_seconds: 420 + rng.range_u32(0, 2) * 30,
        map,
        starting_tools: ToolLoadout::basic_field_kit(),
        crew: CrewPool {
            crews: 3,
            labor_seconds_available: 440 + rng.range_u32(0, 2) * 20,
        },
        enemy_groups: vec![
            EnemyGroupSpec {
                label: "southern rushers".to_string(),
                count: 10 + difficulty_extra + rng.range_u32(0, 3),
                doctrine: EnemyDoctrine::RushShortest,
                spawn: south_spawn,
                objective,
                movement_profile: MovementProfile {
                    base_speed: 1.0,
                    obstacle_tolerance: 0.35,
                    cover_preference: 0.1,
                },
            },
            EnemyGroupSpec {
                label: "cautious riflemen".to_string(),
                count: 6 + rng.range_u32(0, 2),
                doctrine: EnemyDoctrine::PreferCover,
                spawn: west_spawn,
                objective,
                movement_profile: MovementProfile {
                    base_speed: 0.85,
                    obstacle_tolerance: 0.2,
                    cover_preference: 0.65,
                },
            },
            EnemyGroupSpec {
                label: "orchard skirmishers".to_string(),
                count: 5 + rng.range_u32(0, 2),
                doctrine: EnemyDoctrine::FlankViaConcealment,
                spawn: west_spawn,
                objective,
                movement_profile: MovementProfile {
                    base_speed: 0.95,
                    obstacle_tolerance: 0.3,
                    cover_preference: 0.45,
                },
            },
        ],
        defender_positions: vec![
            DefenderPositionSpec {
                id: "ridge_team_01".to_string(),
                label: "ridge rifle pit".to_string(),
                cell: CellCoord::new(objective.x.saturating_sub(1), objective.y),
                range: 5,
                pressure_per_step: 2,
            },
            DefenderPositionSpec {
                id: "road_team_01".to_string(),
                label: "road overwatch".to_string(),
                cell: CellCoord::new(7, 3),
                range: 4,
                pressure_per_step: 1,
            },
        ],
        constraints: MissionConstraints {
            max_work_orders: 12,
            allow_assault_preview: false,
        },
    };
    let affordance_report = build_generated_affordance_report(&spec);
    GeneratedMissionCandidate {
        seed,
        theme: generator.theme,
        spec,
        affordance_report,
    }
}

fn generate_orchard_approach_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    let mut candidate = generate_road_ridge_candidate(generator, seed);
    let mut rng = MissionRng::new(seed ^ 0x0a2c_0fc0);
    candidate.spec.title = format!("Orchard Approach {:04x}", (seed & 0xffff) as u32);
    candidate.spec.id = format!("procgen_orchard_approach_{seed:016x}");
    candidate.spec.briefing.summary =
        "A generated orchard approach where trees are both cover and material.".to_string();
    candidate.spec.briefing.primary = "Hold the orchard road marker.".to_string();
    candidate.spec.briefing.optional_objectives = vec![
        "Use timber without clearing every line-of-sight blocker.".to_string(),
        "Make the cover-seeking route worse than the road route.".to_string(),
        "Avoid wasting the ridge log if the orchard already slows attackers.".to_string(),
    ];
    candidate.spec.briefing.intel = vec![
        "Skirmishers prefer the tree cover and can punish over-cleared orchards.".to_string(),
        "Rushers still test the road if the orchard becomes too costly.".to_string(),
        "Local trees can become logs or stakes, but each cut changes route cover.".to_string(),
    ];
    candidate.spec.prep_time_seconds = 450 + rng.range_u32(0, 1) * 30;
    candidate.spec.crew.labor_seconds_available = 470;
    candidate.spec.objective.objective_health = 100;
    for group in &mut candidate.spec.enemy_groups {
        group.count += 1;
    }

    for x in 0..candidate.spec.map.width {
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 4),
            GroundKind::Road,
        );
        if x % 3 == 0 {
            set_ground(
                &mut candidate.spec.map,
                CellCoord::new(x, 3),
                GroundKind::Dirt,
            );
        }
    }
    for x in 7..candidate.spec.map.width {
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 2)) {
            cell.height = 2;
        }
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 3)) {
            cell.height = 2;
        }
    }
    let orchard_cells = [
        CellCoord::new(1, 1),
        CellCoord::new(2, 1),
        CellCoord::new(6, 6),
    ];
    for (index, cell) in orchard_cells.iter().enumerate() {
        add_tree_object(
            &mut candidate.spec.map,
            format!("orchard_tree_{index:02}"),
            "orchard tree",
            *cell,
        );
    }
    move_object(
        &mut candidate.spec.map,
        "tree_west_01",
        CellCoord::new(3, 2),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_west_02",
        CellCoord::new(4, 2),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_east_01",
        CellCoord::new(8, 5),
    );
    move_object(
        &mut candidate.spec.map,
        "ridge_log_01",
        CellCoord::new(7, 3),
    );
    refresh_generated_candidate(candidate, generator.theme)
}

fn generate_dry_wash_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    let mut candidate = generate_road_ridge_candidate(generator, seed);
    let mut rng = MissionRng::new(seed ^ 0x0d27_0a5a);
    candidate.spec.title = format!("Dry Wash {:04x}", (seed & 0xffff) as u32);
    candidate.spec.id = format!("procgen_dry_wash_{seed:016x}");
    candidate.spec.briefing.summary =
        "A generated dry wash where low ground creates dead-ground approach choices.".to_string();
    candidate.spec.briefing.primary = "Hold the wash crossing marker.".to_string();
    candidate.spec.briefing.optional_objectives = vec![
        "Decide whether to deepen, block, or cover the wash.".to_string(),
        "Use berms to overlook low ground without blocking defenders.".to_string(),
        "Keep the rolling hazard from simply solving the crossing.".to_string(),
    ];
    candidate.spec.briefing.intel = vec![
        "Rushers use the easiest wash crossing unless it becomes costly.".to_string(),
        "Cover-seekers may treat low ground as safer than the open road.".to_string(),
        "The dry wash is trenchable, but it can also create dead ground.".to_string(),
    ];
    candidate.spec.prep_time_seconds = 450;
    candidate.spec.crew.labor_seconds_available = 460 + rng.range_u32(0, 1) * 20;

    for x in 0..candidate.spec.map.width {
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 5),
            GroundKind::Road,
        );
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 5)) {
            cell.height = 0;
        }
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 4),
            GroundKind::Dirt,
        );
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 4)) {
            cell.height = 0;
            cell.earth_state = EarthState::Ditch;
            cell.cover = CoverClass::Light;
            cell.movement_cost = 1.35;
        }
        if rng.chance(1, 3) {
            set_ground(
                &mut candidate.spec.map,
                CellCoord::new(x, 3),
                GroundKind::Mud,
            );
        }
    }
    for x in 7..candidate.spec.map.width {
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 2)) {
            cell.height = 2;
        }
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 3)) {
            cell.height = 2;
        }
    }
    move_object(
        &mut candidate.spec.map,
        "tree_west_01",
        CellCoord::new(2, 6),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_west_02",
        CellCoord::new(4, 6),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_east_01",
        CellCoord::new(9, 5),
    );
    add_tree_object(
        &mut candidate.spec.map,
        "wash_tree_01",
        "wash scrub tree",
        CellCoord::new(1, 3),
    );
    move_object(
        &mut candidate.spec.map,
        "ridge_log_01",
        CellCoord::new(5, 3),
    );
    if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(5, 3)) {
        cell.height = 2;
    }
    refresh_generated_candidate(candidate, generator.theme)
}

fn generate_old_wall_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    let mut candidate = generate_road_ridge_candidate(generator, seed);
    let mut rng = MissionRng::new(seed ^ 0x0d0d_aa11);
    candidate.spec.title = format!("Old Wall {:04x}", (seed & 0xffff) as u32);
    candidate.spec.id = format!("procgen_old_wall_{seed:016x}");
    candidate.spec.briefing.summary =
        "A generated ruined-wall approach with hard cover and breach choices.".to_string();
    candidate.spec.briefing.primary = "Hold the wall-side marker.".to_string();
    candidate.spec.briefing.optional_objectives = vec![
        "Use the ruin as cover without giving attackers a protected route.".to_string(),
        "Force enemies through a chosen breach rather than every gap.".to_string(),
        "Avoid overbuilding earthworks that make the wall irrelevant.".to_string(),
    ];
    candidate.spec.briefing.intel = vec![
        "Riflemen value wall cover more than rushers do.".to_string(),
        "Ruined segments can make one route safer for attackers.".to_string(),
        "Earthworks around the wall should shape, not erase, the hard-cover problem.".to_string(),
    ];
    candidate.spec.prep_time_seconds = 420 + rng.range_u32(0, 1) * 30;

    for x in 0..candidate.spec.map.width {
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 4),
            GroundKind::Road,
        );
        if x >= 5 {
            set_ground(
                &mut candidate.spec.map,
                CellCoord::new(x, 3),
                GroundKind::Rock,
            );
        }
    }
    for x in 7..candidate.spec.map.width {
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 2)) {
            cell.height = 2;
        }
    }
    let wall_cells = [
        (CellCoord::new(5, 3), WallState::Damaged),
        (CellCoord::new(6, 3), WallState::Breached),
        (CellCoord::new(7, 3), WallState::Damaged),
        (CellCoord::new(8, 4), WallState::CollapsedRubble),
    ];
    for (index, (cell, state)) in wall_cells.iter().enumerate() {
        candidate.spec.map.objects.push(EnvironmentObject {
            id: format!("old_wall_{index:02}"),
            label: "old wall segment".to_string(),
            kind: EnvironmentObjectKind::Wall(state.clone()),
            cell: *cell,
            footprint: (1, 1),
            blocks_sight: !matches!(state, WallState::Breached | WallState::CollapsedRubble),
            cover: CoverClass::Strong,
            movement_cost_delta: if matches!(state, WallState::Breached) {
                0.2
            } else {
                1.1
            },
        });
    }
    move_object(
        &mut candidate.spec.map,
        "tree_west_01",
        CellCoord::new(2, 2),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_west_02",
        CellCoord::new(2, 6),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_east_01",
        CellCoord::new(9, 5),
    );
    move_object(
        &mut candidate.spec.map,
        "ridge_log_01",
        CellCoord::new(7, 3),
    );
    refresh_generated_candidate(candidate, generator.theme)
}

fn generate_split_approach_candidate(
    generator: &MissionGeneratorSpec,
    seed: u64,
) -> GeneratedMissionCandidate {
    let mut candidate = generate_road_ridge_candidate(generator, seed);
    let mut rng = MissionRng::new(seed ^ 0x5f11_7001);
    candidate.spec.title = format!("Split Approach {:04x}", (seed & 0xffff) as u32);
    candidate.spec.id = format!("procgen_split_approach_{seed:016x}");
    candidate.spec.briefing.summary =
        "A generated split approach with two lanes and not enough time to perfect both."
            .to_string();
    candidate.spec.briefing.primary = "Hold the split-road marker.".to_string();
    candidate.spec.briefing.optional_objectives = vec![
        "Avoid overcommitting all prep to one approach.".to_string(),
        "Use cheap obstacles to bias one lane while strengthening the other.".to_string(),
        "Keep defenders from being screened by your own berms.".to_string(),
    ];
    candidate.spec.briefing.intel = vec![
        "Rushers pressure the southern road.".to_string(),
        "Skirmishers and cautious riflemen can exploit the side lane.".to_string(),
        "Prep time is intentionally tight for two complete defenses.".to_string(),
    ];
    candidate.spec.prep_time_seconds = 390 + rng.range_u32(0, 1) * 30;
    candidate.spec.crew.labor_seconds_available = 410;
    candidate.spec.objective.objective_health = 125;
    candidate.spec.map.spawn_cells = vec![CellCoord::new(0, 6), CellCoord::new(0, 2)];

    for x in 0..candidate.spec.map.width {
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 5),
            GroundKind::Road,
        );
        set_ground(
            &mut candidate.spec.map,
            CellCoord::new(x, 2),
            GroundKind::Road,
        );
        if x % 2 == 0 {
            set_ground(
                &mut candidate.spec.map,
                CellCoord::new(x, 3),
                GroundKind::Dirt,
            );
        }
    }
    for x in 7..candidate.spec.map.width {
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 3)) {
            cell.height = 2;
        }
        if let Some(cell) = candidate.spec.map.cell_mut(CellCoord::new(x, 4)) {
            cell.height = 2;
        }
    }
    let objective = CellCoord::new(10, 3);
    candidate.spec.objective.defend_cell = objective;
    for group in &mut candidate.spec.enemy_groups {
        group.objective = objective;
    }
    if let Some(group) = candidate.spec.enemy_groups.get_mut(0) {
        group.spawn = CellCoord::new(0, 6);
    }
    if let Some(group) = candidate.spec.enemy_groups.get_mut(1) {
        group.spawn = CellCoord::new(0, 2);
    }
    if let Some(group) = candidate.spec.enemy_groups.get_mut(2) {
        group.spawn = CellCoord::new(0, 2);
    }
    move_object(
        &mut candidate.spec.map,
        "tree_west_01",
        CellCoord::new(3, 2),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_west_02",
        CellCoord::new(3, 6),
    );
    move_object(
        &mut candidate.spec.map,
        "tree_east_01",
        CellCoord::new(8, 5),
    );
    move_object(
        &mut candidate.spec.map,
        "ridge_log_01",
        CellCoord::new(7, 3),
    );
    refresh_generated_candidate(candidate, generator.theme)
}

fn evaluate_generated_mission_candidate(
    candidate: &GeneratedMissionCandidate,
    initial_routes: &DoctrineRouteSet,
    balance_report: &MissionBalanceReport,
) -> GeneratedMissionEvaluation {
    let scenarios = balance_report
        .scenarios
        .iter()
        .map(|scenario| GeneratedMissionScenarioScore {
            id: scenario.id.clone(),
            label: scenario.label.clone(),
            stars: scenario.rating.stars,
            score: scenario.rating.score,
            victory: scenario.summary.victory,
            stopped: scenario.summary.enemies_eliminated,
            reached: scenario.summary.enemies_reached_objective,
            prep_time_used_seconds: scenario.prep_time_used_seconds,
            hazard_enemies_hit: scenario.rolling_hazards.enemies_hit,
            validation_issue_count: scenario
                .notes
                .iter()
                .find_map(|note| parse_validation_issue_count(note))
                .unwrap_or(0),
        })
        .collect::<Vec<_>>();
    let baseline = balance_report
        .scenarios
        .iter()
        .find(|scenario| scenario.id == "baseline_no_prep")
        .unwrap_or_else(|| {
            balance_report
                .scenarios
                .first()
                .expect("mission balance must include at least one scenario")
        });
    let best = balance_report
        .scenarios
        .iter()
        .max_by_key(|scenario| (scenario.rating.stars, scenario.rating.score))
        .unwrap_or(baseline);

    let route_diversity_score = route_diversity_score(initial_routes);
    let height_interest_score = height_interest_score(&candidate.spec.map);
    let local_material_score = (candidate.affordance_report.tree_count as f32 / 4.0)
        .min(1.0)
        .max((candidate.affordance_report.loose_log_count as f32 / 1.0).min(1.0));
    let work_order_opportunity_score =
        ((candidate.affordance_report.trenchable_soil_cells as f32 / 48.0) * 0.55
            + (candidate.affordance_report.tree_count as f32 / 4.0).min(1.0) * 0.45)
            .min(1.0);
    let rolling_hazard_score = if candidate.affordance_report.rolling_hazard_path_cells >= 4 {
        (candidate
            .affordance_report
            .rolling_hazard_route_intersections as f32
            / 3.0)
            .min(1.0)
    } else {
        0.0
    };
    let doctrine_spread_score = doctrine_spread_score(&candidate.spec.enemy_groups);
    let objective_vulnerability_score: f32 = match baseline.rating.stars {
        0 => 0.7,
        1 => 1.0,
        2 => 0.55,
        _ => 0.0,
    };
    let rolling_log_rating = balance_report
        .scenarios
        .iter()
        .find(|scenario| scenario.id == "rolling_log_plan")
        .or_else(|| {
            balance_report
                .scenarios
                .iter()
                .find(|scenario| scenario.id == "road_below_hazard_prep")
        })
        .map(|scenario| scenario.rating.score);
    let worst_score = balance_report
        .scenarios
        .iter()
        .map(|scenario| scenario.rating.score)
        .min()
        .unwrap_or(baseline.rating.score);
    let overbuilt_bad_plan_score = balance_report
        .scenarios
        .iter()
        .find(|scenario| scenario.id == "overbuilt_bad_plan")
        .map(|scenario| scenario.rating.score);
    let plan_sensitivity = GeneratedMissionPlanSensitivity {
        baseline_score: baseline.rating.score,
        best_score: best.rating.score,
        worst_score,
        best_minus_baseline: best.rating.score - baseline.rating.score,
        best_minus_worst: best.rating.score - worst_score,
        rolling_log_score: rolling_log_rating,
        rolling_log_to_best_ratio: rolling_log_rating
            .map(|score| score as f32 / best.rating.score.max(1) as f32),
        overbuilt_bad_plan_score,
    };

    let mut rejection_kinds = Vec::new();
    let mut rejection_reasons = Vec::new();
    if initial_routes
        .routes
        .iter()
        .any(|route| !route.reached_goal)
    {
        rejection_kinds.push(GeneratedMissionRejectionKind::ObjectiveUnreachable);
        rejection_reasons.push("one or more enemy routes cannot reach the objective".to_string());
    }
    if baseline.rating.stars >= 3 {
        rejection_kinds.push(GeneratedMissionRejectionKind::TooEasyNoPrep);
        rejection_reasons.push("no-prep baseline earns 3 stars".to_string());
    }
    if best.rating.stars == 0 {
        rejection_kinds.push(GeneratedMissionRejectionKind::TooHardAllPlansFail);
        rejection_reasons.push("all known prep scripts fail".to_string());
    }
    if route_diversity_score < 0.25 {
        rejection_kinds.push(GeneratedMissionRejectionKind::NoRouteDiversity);
        rejection_reasons.push("enemy routes are too similar".to_string());
    }
    if height_interest_score < 0.12 {
        rejection_kinds.push(GeneratedMissionRejectionKind::TerrainTooFlat);
        rejection_reasons.push("terrain has too little height variation".to_string());
    }
    if candidate.affordance_report.tree_count < 2 {
        rejection_kinds.push(GeneratedMissionRejectionKind::NoUsefulMaterials);
        rejection_reasons.push("not enough local timber affordance".to_string());
    }
    if candidate.affordance_report.trenchable_soil_cells < 36 {
        rejection_kinds.push(GeneratedMissionRejectionKind::NoUsefulMaterials);
        rejection_reasons.push("not enough trenchable soil".to_string());
    }
    if rolling_hazard_score <= 0.0 && theme_requires_rolling_hazard(candidate.theme) {
        rejection_kinds.push(GeneratedMissionRejectionKind::NoHazardOpportunity);
        rejection_reasons.push("rolling-log opportunity does not cross likely routes".to_string());
    }
    if rolling_log_rating
        .map(|score| best.rating.score > 0 && score >= best.rating.score && score >= 95)
        .unwrap_or(false)
        && plan_sensitivity.best_minus_baseline < 10
    {
        rejection_kinds.push(GeneratedMissionRejectionKind::HazardTooDominant);
        rejection_reasons
            .push("rolling-log plan dominates without enough broader prep sensitivity".to_string());
    }
    if candidate
        .spec
        .map
        .spawn_cells
        .iter()
        .any(|spawn| spawn.manhattan(candidate.spec.objective.defend_cell) <= 4)
    {
        rejection_kinds.push(GeneratedMissionRejectionKind::SpawnTooClose);
        rejection_reasons.push("enemy spawn is too close to the objective".to_string());
    }

    let rating_delta = (best.rating.score - baseline.rating.score).max(0) as f32 / 100.0;
    let score_breakdown = GeneratedMissionScoreBreakdown {
        baseline_pressure: (objective_vulnerability_score * 12.0).round() as i32,
        prep_delta: (rating_delta * 20.0).round() as i32,
        route_diversity: (route_diversity_score * 14.0).round() as i32,
        terrain_interest: (height_interest_score * 12.0).round() as i32,
        material_affordances: (local_material_score * 10.0).round() as i32,
        work_order_opportunities: (work_order_opportunity_score * 14.0).round() as i32,
        hazard_viability: (rolling_hazard_score * 18.0).round() as i32,
        doctrine_spread: (doctrine_spread_score * 10.0).round() as i32,
        objective_vulnerability: 0,
        duplicate_penalty: 0,
    };
    let tactical_interest_score = score_breakdown.total();

    GeneratedMissionEvaluation {
        seed: candidate.seed,
        mission_id: candidate.spec.id.clone(),
        title: candidate.spec.title.clone(),
        theme: candidate.theme,
        theme_slug: candidate.theme.slug().to_string(),
        candidate_dir: None,
        mission_path: None,
        accepted: rejection_kinds.is_empty(),
        tactical_interest_score,
        score_breakdown,
        plan_sensitivity,
        rejection_kinds,
        rejection_reasons,
        duplicate_of_seed: None,
        similarity_to_duplicate: None,
        baseline_rating: baseline.rating.clone(),
        best_rating: best.rating.clone(),
        best_plan_label: best.label.clone(),
        route_diversity_score,
        height_interest_score,
        local_material_score,
        work_order_opportunity_score,
        rolling_hazard_score,
        doctrine_spread_score,
        objective_vulnerability_score,
        affordance_report: candidate.affordance_report.clone(),
        fingerprint: generated_mission_fingerprint(&candidate.spec, initial_routes),
        scenarios,
    }
}

fn first_similar_candidate(
    evaluation: &GeneratedMissionEvaluation,
    accepted: &[GeneratedMissionArtifact],
) -> Option<(u64, f32)> {
    accepted
        .iter()
        .map(|artifact| {
            (
                artifact.evaluation.seed,
                generated_mission_similarity(
                    &evaluation.fingerprint,
                    &artifact.evaluation.fingerprint,
                ),
            )
        })
        .filter(|(_, similarity)| *similarity >= 0.92)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
}

fn mark_duplicate_candidate(
    evaluation: &mut GeneratedMissionEvaluation,
    duplicate_seed: u64,
    similarity: f32,
) {
    evaluation.accepted = false;
    evaluation.duplicate_of_seed = Some(duplicate_seed);
    evaluation.similarity_to_duplicate = Some(similarity);
    evaluation
        .rejection_kinds
        .push(GeneratedMissionRejectionKind::DuplicateCandidate);
    evaluation.rejection_reasons.push(format!(
        "near-duplicate of higher-ranked seed {duplicate_seed} ({similarity:.2} similarity)"
    ));
    evaluation.score_breakdown.duplicate_penalty = 12;
    evaluation.tactical_interest_score = evaluation.score_breakdown.total();
}

fn generated_mission_fingerprint(
    spec: &MissionSpec,
    routes: &DoctrineRouteSet,
) -> GeneratedMissionFingerprint {
    let mut ridge_cells = Vec::new();
    for y in 0..spec.map.height {
        for x in 0..spec.map.width {
            let coord = CellCoord::new(x, y);
            let Some(cell) = spec.map.cell(coord) else {
                continue;
            };
            if cell.height >= 2 {
                ridge_cells.push(coord);
            }
        }
    }
    let mut tree_cells = spec
        .map
        .objects
        .iter()
        .filter_map(|object| match object.kind {
            EnvironmentObjectKind::Tree(TreeState::Standing)
            | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => Some(object.cell),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut route_cells = routes
        .routes
        .iter()
        .flat_map(|route| route.points.iter().copied())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let route_lengths = routes
        .routes
        .iter()
        .map(|route| route.points.len() as u32)
        .collect::<Vec<_>>();
    ridge_cells.sort_by_key(|cell| (cell.y, cell.x));
    tree_cells.sort_by_key(|cell| (cell.y, cell.x));
    route_cells.sort_by_key(|cell| (cell.y, cell.x));

    GeneratedMissionFingerprint {
        objective: spec.objective.defend_cell,
        spawns: spec.map.spawn_cells.clone(),
        ridge_cells,
        tree_cells,
        route_cells,
        route_lengths,
        rolling_hazard_route_intersections: build_generated_affordance_report(spec)
            .rolling_hazard_route_intersections,
    }
}

fn generated_mission_similarity(
    a: &GeneratedMissionFingerprint,
    b: &GeneratedMissionFingerprint,
) -> f32 {
    let objective_score = if a.objective.manhattan(b.objective) <= 1 {
        1.0
    } else {
        0.0
    };
    let route_score = coord_jaccard(&a.route_cells, &b.route_cells);
    let ridge_score = coord_jaccard(&a.ridge_cells, &b.ridge_cells);
    let tree_score = coord_jaccard(&a.tree_cells, &b.tree_cells);
    let spawn_score = coord_jaccard(&a.spawns, &b.spawns);
    let hazard_score =
        if a.rolling_hazard_route_intersections == b.rolling_hazard_route_intersections {
            1.0
        } else {
            0.4
        };
    (route_score * 0.35)
        + (ridge_score * 0.22)
        + (tree_score * 0.15)
        + (spawn_score * 0.1)
        + (objective_score * 0.1)
        + (hazard_score * 0.08)
}

fn coord_jaccard(a: &[CellCoord], b: &[CellCoord]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let left = a.iter().copied().collect::<HashSet<_>>();
    let right = b.iter().copied().collect::<HashSet<_>>();
    let union = left.union(&right).count().max(1) as f32;
    left.intersection(&right).count() as f32 / union
}

fn theme_requires_rolling_hazard(theme: MissionTheme) -> bool {
    matches!(theme, MissionTheme::DryRoadBelow | MissionTheme::RidgeTrap)
}

#[derive(Clone, Debug)]
struct MissionRng {
    state: u64,
}

impl MissionRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xa076_1d64_78bd_642f,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        ((self.state.wrapping_mul(0x2545_f491_4f6c_dd1d)) >> 32) as u32
    }

    fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        min + self.next_u32() % (max - min + 1)
    }

    fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
        denominator == 0 || self.range_u32(1, denominator) <= numerator
    }
}

fn set_ground(map: &mut MissionMap, cell: CellCoord, ground: GroundKind) {
    if let Some(tile) = map.cell_mut(cell) {
        tile.ground = ground;
        tile.movement_cost = ground.base_movement_cost();
    }
}

fn add_tree_object(
    map: &mut MissionMap,
    id: impl Into<String>,
    label: impl Into<String>,
    cell: CellCoord,
) {
    map.objects.push(EnvironmentObject {
        id: id.into(),
        label: label.into(),
        kind: EnvironmentObjectKind::Tree(TreeState::Standing),
        cell,
        footprint: (1, 1),
        blocks_sight: true,
        cover: CoverClass::Partial,
        movement_cost_delta: 0.4,
    });
}

fn move_object(map: &mut MissionMap, object_id: &str, cell: CellCoord) {
    if let Some(object) = map.object_at_mut(object_id) {
        object.cell = cell;
    }
}

fn refresh_generated_candidate(
    mut candidate: GeneratedMissionCandidate,
    theme: MissionTheme,
) -> GeneratedMissionCandidate {
    candidate.theme = theme;
    candidate.spec.visual_theme = mission_visual_theme_for_theme(theme);
    candidate.affordance_report = build_generated_affordance_report(&candidate.spec);
    candidate
}

fn build_generated_affordance_report(spec: &MissionSpec) -> GeneratedMissionAffordanceReport {
    let state = MissionState::from_spec(spec.clone());
    let routes = state.route_preview();
    let route_cells = routes
        .routes
        .iter()
        .flat_map(|route| route.points.iter().copied())
        .collect::<HashSet<_>>();
    let mut report = GeneratedMissionAffordanceReport {
        spawn_count: spec.map.spawn_cells.len() as u32,
        route_count: routes.routes.len() as u32,
        ..Default::default()
    };

    for cell in &spec.map.cells {
        if cell.ground == GroundKind::Road {
            report.road_cell_count += 1;
        }
        if cell.height >= 2 {
            report.ridge_cell_count += 1;
        }
        if !matches!(cell.ground, GroundKind::Rock | GroundKind::Mud) {
            report.trenchable_soil_cells += 1;
        }
    }
    for object in &spec.map.objects {
        match &object.kind {
            EnvironmentObjectKind::Tree(TreeState::Standing)
            | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => {
                report.tree_count += 1;
            }
            EnvironmentObjectKind::Log(kind) => {
                report.loose_log_count += 1;
                if let Some(direction) =
                    rolling_log_direction(&EnvironmentObjectKind::Log(kind.clone()))
                {
                    let path = predict_rolling_log_path(&spec.map, object.cell, direction);
                    report.rolling_hazard_path_cells =
                        report.rolling_hazard_path_cells.max(path.len() as u32);
                    report.rolling_hazard_route_intersections +=
                        path.iter()
                            .filter(|step| route_cells.contains(&step.cell))
                            .count() as u32;
                }
            }
            _ => {}
        }
    }

    if report.road_cell_count > 0 {
        report.notes.push(format!(
            "{} road cell(s) define the main approach.",
            report.road_cell_count
        ));
    }
    if report.ridge_cell_count > 0 {
        report.notes.push(format!(
            "{} raised ridge cell(s) create height interest.",
            report.ridge_cell_count
        ));
    }
    if report.tree_count > 0 {
        report.notes.push(format!(
            "{} standing tree(s) create timber/LOS tradeoffs.",
            report.tree_count
        ));
    }
    if report.rolling_hazard_route_intersections > 0 {
        report.notes.push(format!(
            "Rolling hazard path crosses likely routes at {} cell(s).",
            report.rolling_hazard_route_intersections
        ));
    }
    report
}

fn route_diversity_score(routes: &DoctrineRouteSet) -> f32 {
    if routes.routes.len() < 2 {
        return 0.0;
    }
    let route_sets = routes
        .routes
        .iter()
        .map(|route| route.points.iter().copied().collect::<HashSet<_>>())
        .collect::<Vec<_>>();
    let mut total = 0.0;
    let mut pairs = 0;
    for i in 0..route_sets.len() {
        for j in i + 1..route_sets.len() {
            let shared = route_sets[i].intersection(&route_sets[j]).count() as f32;
            let union = route_sets[i].union(&route_sets[j]).count().max(1) as f32;
            total += 1.0 - shared / union;
            pairs += 1;
        }
    }
    if pairs == 0 {
        0.0
    } else {
        (total / pairs as f32).clamp(0.0, 1.0)
    }
}

fn height_interest_score(map: &MissionMap) -> f32 {
    let mut transitions = 0;
    let mut checked = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let cell = CellCoord::new(x, y);
            let Some(source) = map.cell(cell) else {
                continue;
            };
            for neighbor in map.neighbors4(cell) {
                if neighbor.x < x || neighbor.y < y {
                    continue;
                }
                let Some(target) = map.cell(neighbor) else {
                    continue;
                };
                checked += 1;
                if source.height != target.height {
                    transitions += 1;
                }
            }
        }
    }
    if checked == 0 {
        0.0
    } else {
        (transitions as f32 / checked as f32 * 4.0).clamp(0.0, 1.0)
    }
}

fn doctrine_spread_score(groups: &[EnemyGroupSpec]) -> f32 {
    let unique = groups
        .iter()
        .map(|group| group.doctrine)
        .collect::<HashSet<_>>()
        .len();
    (unique as f32 / 3.0).min(1.0)
}

fn parse_validation_issue_count(note: &str) -> Option<u32> {
    let (_, tail) = note.split_once(", ")?;
    let (count, _) = tail.split_once(" validation issue")?;
    count.parse().ok()
}

fn save_theme_calibration_summary_image(
    path: impl AsRef<Path>,
    report: &ThemeCalibrationReport,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let width = 820;
    let row_h = 42;
    let height = 24 + report.theme_summaries.len() as u32 * row_h;
    let mut image = RgbaImage::from_pixel(width, height.max(64), Rgba([23, 26, 25, 255]));
    for (index, summary) in report.theme_summaries.iter().enumerate() {
        let y = 12 + index as u32 * row_h;
        let base = theme_preview_color(summary.theme);
        fill_preview_rect(&mut image, 12, y, 28, 28, base);
        let bar_x = 56;
        let bar_w = 220;
        fill_preview_rect(&mut image, bar_x, y + 2, bar_w, 7, Rgba([47, 52, 49, 255]));
        fill_preview_rect(
            &mut image,
            bar_x,
            y + 2,
            scaled_bar(summary.acceptance_rate, bar_w),
            7,
            Rgba([82, 184, 104, 255]),
        );
        fill_preview_rect(&mut image, bar_x, y + 12, bar_w, 5, Rgba([47, 52, 49, 255]));
        fill_preview_rect(
            &mut image,
            bar_x,
            y + 12,
            scaled_bar(summary.average_score / 100.0, bar_w),
            5,
            Rgba([226, 196, 88, 255]),
        );
        fill_preview_rect(&mut image, bar_x, y + 21, bar_w, 5, Rgba([47, 52, 49, 255]));
        fill_preview_rect(
            &mut image,
            bar_x,
            y + 21,
            scaled_bar(summary.average_complexity_score / 100.0, bar_w),
            5,
            Rgba([84, 156, 218, 255]),
        );

        let target_x = 304;
        let target_w = 170;
        fill_preview_rect(
            &mut image,
            target_x,
            y + 2,
            target_w,
            7,
            Rgba([47, 52, 49, 255]),
        );
        fill_preview_rect(
            &mut image,
            target_x + scaled_bar(summary.target_acceptance_min, target_w),
            y + 1,
            2,
            9,
            Rgba([205, 205, 180, 255]),
        );
        fill_preview_rect(
            &mut image,
            target_x + scaled_bar(summary.target_acceptance_max, target_w),
            y + 1,
            2,
            9,
            Rgba([205, 205, 180, 255]),
        );
        fill_preview_rect(
            &mut image,
            target_x,
            y + 15,
            scaled_bar(summary.average_route_diversity, target_w),
            5,
            Rgba([142, 196, 111, 255]),
        );
        fill_preview_rect(
            &mut image,
            target_x,
            y + 24,
            scaled_bar(summary.average_hazard_usefulness, target_w),
            5,
            Rgba([211, 112, 81, 255]),
        );

        let reject_w = 230;
        fill_preview_rect(&mut image, 506, y + 2, reject_w, 6, Rgba([47, 52, 49, 255]));
        if let Some(top) = summary.rejection_reasons.first() {
            fill_preview_rect(
                &mut image,
                506,
                y + 2,
                scaled_bar(top.ratio, reject_w),
                6,
                Rgba([198, 82, 74, 255]),
            );
        }
        let accepted_w = (summary.accepted_count.min(20) * 4).max(1);
        fill_preview_rect(
            &mut image,
            506,
            y + 17,
            accepted_w,
            7,
            Rgba([82, 184, 104, 255]),
        );
        let rejected_w = (summary.rejected_count.min(40) * 3).max(1);
        fill_preview_rect(
            &mut image,
            506,
            y + 27,
            rejected_w,
            7,
            Rgba([198, 82, 74, 255]),
        );
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn save_rejection_reason_histogram_image(
    path: impl AsRef<Path>,
    histogram: &[RejectionReasonHistogramEntry],
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let width = 720;
    let row_h = 24;
    let height = 28 + histogram.len().max(1) as u32 * row_h;
    let mut image = RgbaImage::from_pixel(width, height.max(64), Rgba([23, 26, 25, 255]));
    let max_count = histogram.iter().map(|entry| entry.count).max().unwrap_or(1);
    for (index, entry) in histogram.iter().enumerate() {
        let y = 14 + index as u32 * row_h;
        let color = rejection_preview_color(entry.kind);
        fill_preview_rect(&mut image, 12, y, 20, 12, color);
        fill_preview_rect(&mut image, 44, y + 2, 600, 8, Rgba([47, 52, 49, 255]));
        let width = ((entry.count as f32 / max_count as f32) * 600.0).round() as u32;
        fill_preview_rect(&mut image, 44, y + 2, width.max(1), 8, color);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn save_difficulty_complexity_scatter_image(
    path: impl AsRef<Path>,
    evaluations: &[GeneratedMissionEvaluation],
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let width = 640;
    let height = 420;
    let mut image = RgbaImage::from_pixel(width, height, Rgba([23, 26, 25, 255]));
    fill_preview_rect(&mut image, 48, 32, 544, 328, Rgba([32, 36, 34, 255]));
    for tick in 0..=4 {
        let x = 48 + tick * 136;
        fill_preview_rect(&mut image, x, 32, 1, 328, Rgba([54, 60, 56, 255]));
        let y = 32 + tick * 82;
        fill_preview_rect(&mut image, 48, y, 544, 1, Rgba([54, 60, 56, 255]));
    }
    for evaluation in evaluations {
        let difficulty = generated_mission_difficulty_score(evaluation).clamp(0, 100) as u32;
        let complexity = generated_mission_complexity_score(evaluation).clamp(0, 100) as u32;
        let x = 48 + (difficulty * 544) / 100;
        let y = 360 - (complexity * 328) / 100;
        fill_preview_rect(
            &mut image,
            x.saturating_sub(2),
            y.saturating_sub(2),
            5,
            5,
            theme_preview_color(evaluation.theme),
        );
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn scaled_bar(value: f32, width: u32) -> u32 {
    ((value.clamp(0.0, 1.0) * width as f32).round() as u32).max(1)
}

fn theme_preview_color(theme: MissionTheme) -> Rgba<u8> {
    match theme {
        MissionTheme::DryRoadBelow => Rgba([216, 173, 88, 255]),
        MissionTheme::OrchardApproach => Rgba([104, 174, 88, 255]),
        MissionTheme::DryWash => Rgba([170, 134, 92, 255]),
        MissionTheme::RidgeTrap => Rgba([124, 151, 188, 255]),
        MissionTheme::OldWall => Rgba([158, 158, 143, 255]),
        MissionTheme::SplitApproach => Rgba([205, 112, 92, 255]),
    }
}

fn rejection_preview_color(kind: GeneratedMissionRejectionKind) -> Rgba<u8> {
    match kind {
        GeneratedMissionRejectionKind::TooEasyNoPrep => Rgba([216, 173, 88, 255]),
        GeneratedMissionRejectionKind::TooHardAllPlansFail => Rgba([190, 74, 66, 255]),
        GeneratedMissionRejectionKind::NoRouteDiversity => Rgba([88, 143, 205, 255]),
        GeneratedMissionRejectionKind::NoUsefulMaterials => Rgba([104, 174, 88, 255]),
        GeneratedMissionRejectionKind::NoHazardOpportunity => Rgba([211, 112, 81, 255]),
        GeneratedMissionRejectionKind::HazardTooDominant => Rgba([236, 96, 112, 255]),
        GeneratedMissionRejectionKind::ObjectiveUnreachable => Rgba([110, 92, 174, 255]),
        GeneratedMissionRejectionKind::TerrainTooFlat => Rgba([170, 134, 92, 255]),
        GeneratedMissionRejectionKind::SpawnTooClose => Rgba([230, 130, 76, 255]),
        GeneratedMissionRejectionKind::SpawnTooFar => Rgba([118, 186, 186, 255]),
        GeneratedMissionRejectionKind::InvalidMap => Rgba([218, 218, 218, 255]),
        GeneratedMissionRejectionKind::DuplicateCandidate => Rgba([210, 150, 54, 255]),
    }
}

fn save_generated_mission_contact_sheet(
    path: impl AsRef<Path>,
    artifacts: &[GeneratedMissionArtifact],
    limit: usize,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entries = artifacts.iter().take(limit).collect::<Vec<_>>();
    let cell_px = 16;
    let preview_w = 12 * cell_px;
    let label_h = 10;
    let preview_h = 8 * cell_px + label_h;
    let gap = 8;
    let columns = entries.len().clamp(1, 5) as u32;
    let rows = (entries.len() as u32).div_ceil(columns).max(1);
    let mut sheet = RgbaImage::from_pixel(
        columns * preview_w + (columns + 1) * gap,
        rows * preview_h + (rows + 1) * gap,
        Rgba([24, 27, 26, 255]),
    );
    for (index, artifact) in entries.iter().enumerate() {
        let state = MissionState::from_spec(artifact.spec.clone());
        let mut image = mission_preview_image(&state, cell_px);
        let route_colors = [
            Rgba([68, 132, 226, 145]),
            Rgba([230, 194, 70, 145]),
            Rgba([210, 82, 112, 145]),
        ];
        for (route_index, route) in state.route_preview().routes.iter().enumerate() {
            draw_route_path(
                &mut image,
                route,
                cell_px,
                route_colors[route_index % route_colors.len()],
                3,
            );
        }
        draw_defender_positions(&mut image, &state.spec, cell_px);
        let col = index as u32 % columns;
        let row = index as u32 / columns;
        let x0 = gap + col * (preview_w + gap);
        let y0 = gap + row * (preview_h + gap);
        blit_image(&mut sheet, &image, x0, y0);
        let border_color = if artifact.evaluation.accepted {
            Rgba([78, 184, 92, 255])
        } else if artifact
            .evaluation
            .rejection_kinds
            .contains(&GeneratedMissionRejectionKind::DuplicateCandidate)
        {
            Rgba([210, 150, 54, 255])
        } else {
            Rgba([202, 76, 68, 255])
        };
        draw_preview_border(&mut sheet, x0, y0, preview_w, 8 * cell_px, border_color);
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + 8 * cell_px,
            preview_w,
            label_h,
            Rgba([34, 37, 34, 255]),
        );
        let score_width = ((artifact.evaluation.tactical_interest_score.max(0) as u32).min(100)
            * preview_w)
            / 100;
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + 8 * cell_px + 2,
            score_width.max(1),
            3,
            border_color,
        );
        let sensitivity_width =
            ((artifact.evaluation.plan_sensitivity.best_minus_worst.max(0) as u32).min(100)
                * preview_w)
                / 100;
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + 8 * cell_px + 6,
            sensitivity_width.max(1),
            2,
            Rgba([230, 198, 84, 255]),
        );
    }
    sheet
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn save_generated_mission_visual_contact_sheet(
    path: impl AsRef<Path>,
    artifacts: &[GeneratedMissionArtifact],
    limit: usize,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entries = artifacts.iter().take(limit).collect::<Vec<_>>();
    let preview_w = 240;
    let preview_h = 172;
    let label_h = 12;
    let gap = 10;
    let columns = entries.len().clamp(1, 3) as u32;
    let rows = (entries.len() as u32).div_ceil(columns).max(1);
    let mut sheet = RgbaImage::from_pixel(
        columns * preview_w + (columns + 1) * gap,
        rows * (preview_h + label_h) + (rows + 1) * gap,
        Rgba([22, 24, 23, 255]),
    );
    for (index, artifact) in entries.iter().enumerate() {
        let visual_path = artifact.candidate_dir.join("mission_visual_routes.png");
        let visual = if visual_path.is_file() {
            image::open(&visual_path)
                .with_context(|| format!("failed to read {}", visual_path.display()))?
                .to_rgba8()
        } else {
            let state = MissionState::from_spec(artifact.spec.clone());
            let (visual, _) = mission_visual_image(&state, MissionVisualOverlay::Routes)?;
            visual
        };
        let thumb = image::imageops::resize(
            &visual,
            preview_w,
            preview_h,
            image::imageops::FilterType::Nearest,
        );
        let col = index as u32 % columns;
        let row = index as u32 / columns;
        let x0 = gap + col * (preview_w + gap);
        let y0 = gap + row * (preview_h + label_h + gap);
        blit_image(&mut sheet, &thumb, x0, y0);
        let border_color = if artifact.evaluation.accepted {
            Rgba([78, 184, 92, 255])
        } else if artifact
            .evaluation
            .rejection_kinds
            .contains(&GeneratedMissionRejectionKind::DuplicateCandidate)
        {
            Rgba([210, 150, 54, 255])
        } else {
            Rgba([202, 76, 68, 255])
        };
        draw_preview_border(&mut sheet, x0, y0, preview_w, preview_h, border_color);
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + preview_h,
            preview_w,
            label_h,
            Rgba([34, 37, 34, 255]),
        );
        let score_width = ((artifact.evaluation.tactical_interest_score.max(0) as u32).min(100)
            * preview_w)
            / 100;
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + preview_h + 3,
            score_width.max(1),
            3,
            border_color,
        );
        let difficulty_width = (generated_mission_difficulty_score(&artifact.evaluation)
            .clamp(0, 100) as u32
            * preview_w)
            / 100;
        fill_preview_rect(
            &mut sheet,
            x0,
            y0 + preview_h + 8,
            difficulty_width.max(1),
            2,
            Rgba([230, 198, 84, 255]),
        );
    }
    sheet
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn save_generated_mission_evaluation_contact_sheet(
    path: impl AsRef<Path>,
    evaluations: &[GeneratedMissionEvaluation],
    limit: usize,
) -> Result<()> {
    let mut artifacts = Vec::new();
    for evaluation in evaluations.iter().take(limit) {
        let Some(mission_path) = &evaluation.mission_path else {
            continue;
        };
        let spec = load_mission_spec(mission_path)?;
        artifacts.push(GeneratedMissionArtifact {
            spec,
            candidate_dir: evaluation
                .candidate_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_default(),
            evaluation: evaluation.clone(),
        });
    }
    save_generated_mission_contact_sheet(path, &artifacts, limit)
}

fn save_generated_mission_evaluation_visual_contact_sheet(
    path: impl AsRef<Path>,
    evaluations: &[GeneratedMissionEvaluation],
    limit: usize,
) -> Result<()> {
    let mut artifacts = Vec::new();
    for evaluation in evaluations.iter().take(limit) {
        let Some(mission_path) = &evaluation.mission_path else {
            continue;
        };
        let spec = load_mission_spec(mission_path)?;
        artifacts.push(GeneratedMissionArtifact {
            spec,
            candidate_dir: evaluation
                .candidate_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_default(),
            evaluation: evaluation.clone(),
        });
    }
    save_generated_mission_visual_contact_sheet(path, &artifacts, limit)
}

fn blit_image(target: &mut RgbaImage, source: &RgbaImage, x0: u32, y0: u32) {
    for y in 0..source.height() {
        for x in 0..source.width() {
            let tx = x0 + x;
            let ty = y0 + y;
            if tx < target.width() && ty < target.height() {
                target.put_pixel(tx, ty, *source.get_pixel(x, y));
            }
        }
    }
}

pub fn run_work_order_script(spec: MissionSpec, script: &WorkOrderScript) -> MissionState {
    let mut state = MissionState::from_spec(spec);
    for scripted in &script.orders {
        state.queue_work_order(scripted.kind, scripted.target.clone());
        state.run_next_queued_order();
    }
    state
}

pub fn route_preview_for_state(state: &MissionState) -> DoctrineRouteSet {
    DoctrineRouteSet {
        mission_id: state.spec.id.clone(),
        routes: state
            .spec
            .enemy_groups
            .iter()
            .map(|group| route_preview_for_group(&state.map, group))
            .collect(),
    }
}

pub fn route_delta_report(
    mission_id: impl Into<String>,
    initial: &DoctrineRouteSet,
    after: &DoctrineRouteSet,
) -> RouteDeltaReport {
    let mut groups = Vec::new();
    for initial_route in &initial.routes {
        let Some(after_route) = after
            .routes
            .iter()
            .find(|route| route.group_label == initial_route.group_label)
        else {
            continue;
        };
        let initial_only_cells: Vec<CellCoord> = initial_route
            .points
            .iter()
            .copied()
            .filter(|cell| !after_route.points.contains(cell))
            .collect();
        let after_only_cells: Vec<CellCoord> = after_route
            .points
            .iter()
            .copied()
            .filter(|cell| !initial_route.points.contains(cell))
            .collect();
        let shared_cells = initial_route
            .points
            .iter()
            .filter(|cell| after_route.points.contains(cell))
            .count() as u32;
        let cost_delta = after_route.total_cost - initial_route.total_cost;
        let changed = !initial_only_cells.is_empty()
            || !after_only_cells.is_empty()
            || cost_delta.abs() > 0.2;
        let terrain_notes = after_route
            .explanation
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let explanation = if changed {
            let mut text = format!(
                "{} route changed by {:.1} cost; {} old-only cell(s), {} new-only cell(s).",
                initial_route.group_label,
                cost_delta,
                initial_only_cells.len(),
                after_only_cells.len()
            );
            if !terrain_notes.is_empty() {
                text.push(' ');
                text.push_str(&terrain_notes);
            }
            text
        } else {
            format!(
                "{} route stayed effectively stable; cost delta {:.1}.",
                initial_route.group_label, cost_delta
            )
        };
        groups.push(EnemyRouteDelta {
            group_label: initial_route.group_label.clone(),
            doctrine: initial_route.doctrine,
            initial_cost: initial_route.total_cost,
            after_cost: after_route.total_cost,
            cost_delta,
            shared_cells,
            initial_only_cells,
            after_only_cells,
            changed,
            explanation,
        });
    }
    RouteDeltaReport {
        mission_id: mission_id.into(),
        groups,
    }
}

pub fn export_road_below_seed(out_dir: impl AsRef<Path>) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let spec = road_below_spec();
    let script = road_below_basic_prep_script();
    let before = MissionState::from_spec(spec.clone());
    let after = run_work_order_script(spec.clone(), &script);
    let initial_routes = before.route_preview();
    let after_routes = after.route_preview();
    let route_delta = route_delta_report(spec.id.clone(), &initial_routes, &after_routes);

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("mission_before.json"), &before)?;
    write_json(out_dir.join("order_script.json"), &script)?;
    write_ron(out_dir.join("order_script.ron"), &script)?;
    write_json(
        out_dir.join("scripted_work_orders.json"),
        &after.work_orders,
    )?;
    write_json(out_dir.join("material_ledger.json"), &after.material_ledger)?;
    write_json(
        out_dir.join("order_validation.json"),
        &after.order_validation,
    )?;
    write_json(out_dir.join("mission_after.json"), &after)?;
    write_json(out_dir.join("enemy_routes_initial.json"), &initial_routes)?;
    write_json(
        out_dir.join("enemy_routes_after_orders.json"),
        &after_routes,
    )?;
    write_json(out_dir.join("enemy_route_delta.json"), &route_delta)?;
    fs::write(out_dir.join("mission_before_map.txt"), ascii_map(&before))?;
    fs::write(out_dir.join("mission_after_map.txt"), ascii_map(&after))?;
    fs::write(out_dir.join("mission_summary.txt"), mission_summary(&after))?;
    save_mission_preview_png(out_dir.join("mission_preview.png"), &after)?;
    save_mission_route_preview_png(
        out_dir.join("mission_route_preview.png"),
        &after,
        &initial_routes,
        &after_routes,
    )?;
    save_mission_route_debug_png(
        out_dir.join("mission_route_debug.png"),
        &after,
        &after_routes,
    )?;
    Ok(())
}

pub fn export_order_script_run(
    out_dir: impl AsRef<Path>,
    spec: MissionSpec,
    script: WorkOrderScript,
) -> Result<MissionState> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let initial = MissionState::from_spec(spec.clone());
    let after = run_work_order_script(spec.clone(), &script);
    let initial_routes = initial.route_preview();
    let after_routes = after.route_preview();
    let route_delta = route_delta_report(spec.id.clone(), &initial_routes, &after_routes);

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("order_script.json"), &script)?;
    write_ron(out_dir.join("order_script.ron"), &script)?;
    write_json(out_dir.join("mission_initial.json"), &initial)?;
    write_json(out_dir.join("mission_after_orders.json"), &after)?;
    write_json(out_dir.join("work_log.json"), &after.work_orders)?;
    write_json(out_dir.join("material_ledger.json"), &after.material_ledger)?;
    write_json(out_dir.join("enemy_routes_initial.json"), &initial_routes)?;
    write_json(
        out_dir.join("enemy_routes_after_orders.json"),
        &after_routes,
    )?;
    write_json(out_dir.join("enemy_route_delta.json"), &route_delta)?;
    write_json(
        out_dir.join("order_validation.json"),
        &after.order_validation,
    )?;
    fs::write(out_dir.join("mission_initial_map.txt"), ascii_map(&initial))?;
    fs::write(
        out_dir.join("mission_after_orders_map.txt"),
        ascii_map(&after),
    )?;
    fs::write(out_dir.join("mission_summary.txt"), mission_summary(&after))?;
    save_mission_preview_png(out_dir.join("mission_preview.png"), &after)?;
    save_mission_route_preview_png(
        out_dir.join("mission_route_preview.png"),
        &after,
        &initial_routes,
        &after_routes,
    )?;
    save_mission_route_debug_png(
        out_dir.join("mission_route_debug.png"),
        &after,
        &after_routes,
    )?;
    Ok(after)
}

pub fn export_assault_run(
    out_dir: impl AsRef<Path>,
    spec: MissionSpec,
    script: WorkOrderScript,
) -> Result<AssaultSummary> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let mut prep = run_work_order_script(spec.clone(), &script);
    let assault_initial_routes = prep.route_preview();

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("order_script.json"), &script)?;
    write_ron(out_dir.join("order_script.ron"), &script)?;
    write_json(out_dir.join("mission_prep_final.json"), &prep)?;
    write_json(
        out_dir.join("assault_initial_routes.json"),
        &assault_initial_routes,
    )?;
    save_mission_route_debug_png(
        out_dir.join("assault_path_trace.png"),
        &prep,
        &assault_initial_routes,
    )?;

    prep.start_assault();
    save_mission_assault_preview_png(out_dir.join("assault_preview_start.png"), &prep)?;
    let summary = prep.run_assault_to_completion(160);
    if let Some(assault) = &prep.assault {
        write_json(out_dir.join("assault_state_final.json"), assault)?;
        write_json(out_dir.join("assault_timeline.json"), &assault.timeline)?;
    }
    write_json(out_dir.join("assault_summary.json"), &summary)?;
    if let Some(debrief) = prep.assault_debrief() {
        write_json(out_dir.join("assault_debrief.json"), &debrief)?;
        write_json(out_dir.join("rating_breakdown.json"), &debrief.rating)?;
        write_json(
            out_dir.join("route_prediction_accuracy.json"),
            &debrief.route_prediction_accuracy,
        )?;
        save_assault_delay_heatmap_png(out_dir.join("assault_delay_heatmap.png"), &prep)?;
        save_assault_pressure_heatmap_png(out_dir.join("assault_pressure_heatmap.png"), &prep)?;
        save_assault_prediction_vs_actual_png(
            out_dir.join("assault_prediction_vs_actual.png"),
            &prep,
        )?;
    }
    save_mission_assault_preview_png(out_dir.join("assault_preview_end.png"), &prep)?;
    Ok(summary)
}

pub fn export_hazard_sandbox_run(
    out_dir: impl AsRef<Path>,
    spec: MissionSpec,
    script: WorkOrderScript,
) -> Result<AssaultSummary> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let mut prep = run_work_order_script(spec.clone(), &script);
    let assault_initial_routes = prep.route_preview();
    let rolling_hazards = prep.rolling_hazard_plans();

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("order_script.json"), &script)?;
    write_ron(out_dir.join("order_script.ron"), &script)?;
    write_json(out_dir.join("mission_prep_final.json"), &prep)?;
    write_json(out_dir.join("rolling_hazards.json"), &rolling_hazards)?;
    write_json(
        out_dir.join("assault_initial_routes.json"),
        &assault_initial_routes,
    )?;
    save_rolling_hazard_preview_png(out_dir.join("rolling_hazard_preview.png"), &prep)?;
    save_rolling_hazard_preview_png(out_dir.join("rolling_hazard_path_debug.png"), &prep)?;
    save_mission_route_debug_png(
        out_dir.join("assault_path_trace.png"),
        &prep,
        &assault_initial_routes,
    )?;

    prep.start_assault();
    save_mission_assault_preview_png(out_dir.join("assault_preview_start.png"), &prep)?;
    let summary = prep.run_assault_to_completion(160);
    if let Some(assault) = &prep.assault {
        write_json(
            out_dir.join("rolling_hazards_final.json"),
            &assault.rolling_hazards,
        )?;
        write_json(out_dir.join("assault_state_final.json"), assault)?;
        write_json(out_dir.join("assault_timeline.json"), &assault.timeline)?;
    }
    write_json(out_dir.join("assault_summary.json"), &summary)?;
    if let Some(debrief) = prep.assault_debrief() {
        write_json(out_dir.join("assault_debrief.json"), &debrief)?;
        write_json(out_dir.join("rating_breakdown.json"), &debrief.rating)?;
        write_json(
            out_dir.join("assault_hazard_summary.json"),
            &debrief.rolling_hazards,
        )?;
        write_json(
            out_dir.join("route_prediction_accuracy.json"),
            &debrief.route_prediction_accuracy,
        )?;
        save_assault_delay_heatmap_png(out_dir.join("assault_delay_heatmap.png"), &prep)?;
        save_assault_pressure_heatmap_png(out_dir.join("assault_pressure_heatmap.png"), &prep)?;
        save_assault_prediction_vs_actual_png(
            out_dir.join("assault_prediction_vs_actual.png"),
            &prep,
        )?;
    }
    save_mission_assault_preview_png(out_dir.join("assault_preview_end.png"), &prep)?;
    Ok(summary)
}

pub fn export_mission_balance_run(
    out_dir: impl AsRef<Path>,
    spec: MissionSpec,
) -> Result<MissionBalanceReport> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let scripts = road_below_balance_scripts();
    let initial = MissionState::from_spec(spec.clone());
    let initial_routes = initial.route_preview();
    let mut scenario_reports = Vec::new();
    let mut route_shift_summary = Vec::new();
    let mut hazard_effectiveness = Vec::new();
    let mut rating_breakdown = Vec::new();

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("enemy_routes_initial.json"), &initial_routes)?;

    for script in scripts {
        let scenario_dir = out_dir.join("scenarios").join(&script.id);
        fs::create_dir_all(&scenario_dir)
            .with_context(|| format!("failed to create {}", scenario_dir.display()))?;

        let mut prep = run_work_order_script(spec.clone(), &script);
        let after_routes = prep.route_preview();
        let route_delta = route_delta_report(spec.id.clone(), &initial_routes, &after_routes);
        let prep_time_used_seconds = spec
            .prep_time_seconds
            .saturating_sub(prep.remaining_prep_seconds);

        write_json(scenario_dir.join("order_script.json"), &script)?;
        write_ron(scenario_dir.join("order_script.ron"), &script)?;
        write_json(scenario_dir.join("mission_prep_final.json"), &prep)?;
        write_json(scenario_dir.join("work_log.json"), &prep.work_orders)?;
        write_json(
            scenario_dir.join("material_ledger.json"),
            &prep.material_ledger,
        )?;
        write_json(
            scenario_dir.join("order_validation.json"),
            &prep.order_validation,
        )?;
        write_json(
            scenario_dir.join("enemy_routes_after_orders.json"),
            &after_routes,
        )?;
        write_json(scenario_dir.join("enemy_route_delta.json"), &route_delta)?;
        fs::write(scenario_dir.join("mission_prep_map.txt"), ascii_map(&prep))?;
        save_mission_preview_png(scenario_dir.join("mission_preview.png"), &prep)?;
        save_mission_route_preview_png(
            scenario_dir.join("mission_route_preview.png"),
            &prep,
            &initial_routes,
            &after_routes,
        )?;
        save_mission_route_debug_png(
            scenario_dir.join("mission_route_debug.png"),
            &prep,
            &after_routes,
        )?;
        save_rolling_hazard_preview_png(scenario_dir.join("rolling_hazard_preview.png"), &prep)?;

        prep.start_assault();
        save_mission_assault_preview_png(scenario_dir.join("assault_preview_start.png"), &prep)?;
        let summary = prep.run_assault_to_completion(160);
        save_mission_assault_preview_png(scenario_dir.join("assault_preview_end.png"), &prep)?;

        if let Some(assault) = &prep.assault {
            write_json(scenario_dir.join("assault_state_final.json"), assault)?;
            write_json(
                scenario_dir.join("assault_timeline.json"),
                &assault.timeline,
            )?;
            write_json(
                scenario_dir.join("rolling_hazards_final.json"),
                &assault.rolling_hazards,
            )?;
        }
        write_json(scenario_dir.join("assault_summary.json"), &summary)?;

        let debrief = prep
            .assault_debrief()
            .context("assault completed without a debrief")?;
        write_json(scenario_dir.join("assault_debrief.json"), &debrief)?;
        write_json(scenario_dir.join("rating_breakdown.json"), &debrief.rating)?;
        write_json(
            scenario_dir.join("route_prediction_accuracy.json"),
            &debrief.route_prediction_accuracy,
        )?;
        write_json(
            scenario_dir.join("assault_hazard_summary.json"),
            &debrief.rolling_hazards,
        )?;
        save_assault_delay_heatmap_png(scenario_dir.join("assault_delay_heatmap.png"), &prep)?;
        save_assault_pressure_heatmap_png(
            scenario_dir.join("assault_pressure_heatmap.png"),
            &prep,
        )?;
        save_assault_prediction_vs_actual_png(
            scenario_dir.join("assault_prediction_vs_actual.png"),
            &prep,
        )?;

        let route_notes = route_delta
            .groups
            .iter()
            .filter(|group| group.changed)
            .map(|group| {
                format!(
                    "{} / {}: {}",
                    script.label, group.group_label, group.explanation
                )
            })
            .collect::<Vec<_>>();
        if route_notes.is_empty() {
            route_shift_summary.push(format!("{}: no major route shift.", script.label));
        } else {
            route_shift_summary.extend(route_notes);
        }

        if debrief.rolling_hazards.prepared_count > 0 {
            hazard_effectiveness.push(format!(
                "{}: {} prepared, {} released, {} enemy hit(s), {} friendly-risk cell(s).",
                script.label,
                debrief.rolling_hazards.prepared_count,
                debrief.rolling_hazards.released_count,
                debrief.rolling_hazards.enemies_hit,
                debrief.rolling_hazards.friendly_risk_cells.len()
            ));
        } else {
            hazard_effectiveness.push(format!("{}: no rolling hazard prepared.", script.label));
        }

        rating_breakdown.push(format!(
            "{}: {} star(s), score {}, {}",
            script.label, debrief.rating.stars, debrief.rating.score, debrief.rating.label
        ));

        let mut notes = vec![format!(
            "{} completed order(s), {} validation issue(s).",
            prep.work_orders
                .iter()
                .filter(|order| matches!(order.status, WorkOrderStatus::Completed))
                .count(),
            prep.order_validation.len()
        )];
        notes.extend(debrief.rating.notes.clone());

        scenario_reports.push(MissionBalanceScenarioReport {
            id: script.id,
            label: script.label,
            order_count: prep.work_orders.len() as u32,
            prep_time_used_seconds,
            summary,
            rating: debrief.rating,
            route_prediction_accuracy: debrief.route_prediction_accuracy,
            rolling_hazards: debrief.rolling_hazards,
            notes,
        });
    }

    let report = MissionBalanceReport {
        mission_id: spec.id,
        mission_title: spec.title,
        scenarios: scenario_reports,
        route_shift_summary,
        hazard_effectiveness,
        rating_breakdown,
    };

    write_json(out_dir.join("mission_balance_summary.json"), &report)?;
    write_json(out_dir.join("scenario_comparison.json"), &report.scenarios)?;
    write_json(
        out_dir.join("route_shift_summary.json"),
        &report.route_shift_summary,
    )?;
    write_json(
        out_dir.join("hazard_effectiveness.json"),
        &report.hazard_effectiveness,
    )?;
    write_json(
        out_dir.join("rating_breakdown.json"),
        &report.rating_breakdown,
    )?;
    Ok(report)
}

pub fn load_mission_spec(path: impl AsRef<Path>) -> Result<MissionSpec> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read mission spec {}", path.display()))?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&text)
            .with_context(|| format!("failed to parse JSON mission {}", path.display())),
        _ => ron::from_str(&text)
            .with_context(|| format!("failed to parse RON mission {}", path.display())),
    }
}

pub fn load_work_order_script(path: impl AsRef<Path>) -> Result<WorkOrderScript> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read work-order script {}", path.display()))?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&text)
            .with_context(|| format!("failed to parse JSON order script {}", path.display())),
        _ => ron::from_str(&text)
            .with_context(|| format!("failed to parse RON order script {}", path.display())),
    }
}

pub fn save_mission_spec(path: impl AsRef<Path>, spec: &MissionSpec) -> Result<()> {
    let path = path.as_ref();
    if matches!(path.extension().and_then(|ext| ext.to_str()), Some("json")) {
        write_json(path, spec)
    } else {
        write_ron(path, spec)
    }
}

pub fn save_work_order_script(path: impl AsRef<Path>, script: &WorkOrderScript) -> Result<()> {
    let path = path.as_ref();
    if matches!(path.extension().and_then(|ext| ext.to_str()), Some("json")) {
        write_json(path, script)
    } else {
        write_ron(path, script)
    }
}

#[derive(Clone, Copy, Debug)]
struct RouteQueueNode {
    idx: usize,
    f_score: f32,
}

impl PartialEq for RouteQueueNode {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.f_score.to_bits() == other.f_score.to_bits()
    }
}

impl Eq for RouteQueueNode {}

impl PartialOrd for RouteQueueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RouteQueueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .total_cmp(&self.f_score)
            .then_with(|| self.idx.cmp(&other.idx))
    }
}

fn route_preview_for_group(map: &MissionMap, group: &EnemyGroupSpec) -> EnemyRoutePreview {
    let Some(start_idx) = map.index(group.spawn) else {
        return empty_route(group, "spawn is outside the mission map");
    };
    let Some(goal_idx) = map.index(group.objective) else {
        return empty_route(group, "objective is outside the mission map");
    };

    let len = map.cells.len();
    let mut open = BinaryHeap::new();
    let mut came_from: Vec<Option<usize>> = vec![None; len];
    let mut g_score = vec![f32::INFINITY; len];
    let mut best_idx = start_idx;
    let mut best_h = route_heuristic(group.spawn, group.objective);

    g_score[start_idx] = 0.0;
    open.push(RouteQueueNode {
        idx: start_idx,
        f_score: best_h,
    });

    while let Some(node) = open.pop() {
        let pos = idx_to_coord(map, node.idx);
        let h = route_heuristic(pos, group.objective);
        if h < best_h {
            best_h = h;
            best_idx = node.idx;
        }
        if node.idx == goal_idx {
            let points = reconstruct_route(map, &came_from, goal_idx);
            return EnemyRoutePreview {
                group_label: group.label.clone(),
                doctrine: group.doctrine,
                spawn: group.spawn,
                objective: group.objective,
                total_cost: g_score[goal_idx],
                reached_goal: true,
                explanation: explain_route(map, group, &points, g_score[goal_idx], true),
                points,
            };
        }

        for neighbor in map.neighbors4(pos) {
            let Some(n_idx) = map.index(neighbor) else {
                continue;
            };
            let step = doctrine_step_cost(map, group, pos, neighbor);
            if !step.is_finite() {
                continue;
            }
            let tentative = g_score[node.idx] + step;
            if tentative < g_score[n_idx] {
                came_from[n_idx] = Some(node.idx);
                g_score[n_idx] = tentative;
                open.push(RouteQueueNode {
                    idx: n_idx,
                    f_score: tentative + route_heuristic(neighbor, group.objective),
                });
            }
        }
    }

    let points = reconstruct_route(map, &came_from, best_idx);
    let total_cost = g_score[best_idx];
    EnemyRoutePreview {
        group_label: group.label.clone(),
        doctrine: group.doctrine,
        spawn: group.spawn,
        objective: group.objective,
        total_cost,
        reached_goal: false,
        explanation: explain_route(map, group, &points, total_cost, false),
        points,
    }
}

fn empty_route(group: &EnemyGroupSpec, reason: &str) -> EnemyRoutePreview {
    EnemyRoutePreview {
        group_label: group.label.clone(),
        doctrine: group.doctrine,
        spawn: group.spawn,
        objective: group.objective,
        points: Vec::new(),
        total_cost: 999_999.0,
        reached_goal: false,
        explanation: vec![reason.to_string()],
    }
}

fn doctrine_step_cost(
    map: &MissionMap,
    group: &EnemyGroupSpec,
    from: CellCoord,
    to: CellCoord,
) -> f32 {
    let Some(from_cell) = map.cell(from) else {
        return f32::INFINITY;
    };
    let Some(to_cell) = map.cell(to) else {
        return f32::INFINITY;
    };
    let weights = group.doctrine.weights();
    let height_delta = (from_cell.height - to_cell.height).abs() as f32;
    if height_delta > 4.0 {
        return f32::INFINITY;
    }

    let mut cost = to_cell.movement_cost.max(0.2);
    cost += height_delta * weights.height_cost;
    cost += match to_cell.earth_state {
        EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch => weights.trench_cost,
        EarthState::Berm | EarthState::SpoilPile => weights.berm_cost,
        EarthState::Muddy => 0.7,
        EarthState::Unstable => 0.45,
        EarthState::Scraped | EarthState::Normal => 0.0,
    };

    for object in map.objects_at_cell(to) {
        cost += object.movement_cost_delta * weights.obstacle_cost;
        if object.blocks_sight {
            cost -= weights.concealment_discount;
        }
        if matches!(
            object.kind,
            EnvironmentObjectKind::Stakes(_) | EnvironmentObjectKind::Wire(_)
        ) {
            cost += weights.obstacle_cost;
        }
        if matches!(object.cover, CoverClass::Partial | CoverClass::Strong) {
            cost -= weights.cover_discount;
        }
    }

    match to_cell.cover {
        CoverClass::Strong => cost -= weights.cover_discount,
        CoverClass::Partial => cost -= weights.cover_discount * 0.7,
        CoverClass::Light => cost -= weights.cover_discount * 0.35,
        CoverClass::None => {}
    }
    if matches!(to_cell.ground, GroundKind::Road) {
        cost += weights.road_bias;
    }

    (cost / group.movement_profile.base_speed.max(0.1)).max(0.1)
}

fn explain_route(
    map: &MissionMap,
    group: &EnemyGroupSpec,
    points: &[CellCoord],
    total_cost: f32,
    reached_goal: bool,
) -> Vec<String> {
    let mut trench_cells = 0;
    let mut berm_cells = 0;
    let mut obstacle_cells = 0;
    let mut covered_cells = 0;
    let mut road_cells = 0;
    for point in points {
        if let Some(cell) = map.cell(*point) {
            if matches!(
                cell.earth_state,
                EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch
            ) {
                trench_cells += 1;
            }
            if matches!(cell.earth_state, EarthState::Berm | EarthState::SpoilPile) {
                berm_cells += 1;
            }
            if matches!(cell.cover, CoverClass::Partial | CoverClass::Strong) {
                covered_cells += 1;
            }
            if matches!(cell.ground, GroundKind::Road) {
                road_cells += 1;
            }
        }
        if map.objects_at_cell(*point).next().is_some() {
            obstacle_cells += 1;
        }
    }

    let mut notes = vec![format!(
        "{} doctrine `{}` {} objective with {:.1} route cost over {} cell(s).",
        group.label,
        group.doctrine.label(),
        if reached_goal {
            "reached"
        } else {
            "did not reach"
        },
        total_cost,
        points.len()
    )];
    if trench_cells > 0 {
        notes.push(format!(
            "Route crosses {trench_cells} trench/ditch cell(s), so digging is influencing movement."
        ));
    }
    if berm_cells > 0 {
        notes.push(format!(
            "Route touches {berm_cells} berm/spoil cell(s), creating a height/cover tradeoff."
        ));
    }
    if obstacle_cells > 0 {
        notes.push(format!(
            "Route touches {obstacle_cells} object/obstacle cell(s)."
        ));
    }
    if covered_cells > 0 {
        notes.push(format!(
            "Route uses {covered_cells} covered cell(s), relevant for cover-seeking doctrines."
        ));
    }
    if road_cells > 0 {
        notes.push(format!("Route uses {road_cells} road cell(s)."));
    }
    notes
}

fn route_heuristic(a: CellCoord, b: CellCoord) -> f32 {
    a.manhattan(b) as f32 * 0.1
}

fn idx_to_coord(map: &MissionMap, idx: usize) -> CellCoord {
    CellCoord::new(idx as u32 % map.width, idx as u32 / map.width)
}

fn reconstruct_route(
    map: &MissionMap,
    came_from: &[Option<usize>],
    mut idx: usize,
) -> Vec<CellCoord> {
    let mut out = vec![idx_to_coord(map, idx)];
    while let Some(prev) = came_from[idx] {
        idx = prev;
        out.push(idx_to_coord(map, idx));
    }
    out.reverse();
    out
}

fn process_rolling_hazards(
    map: &mut MissionMap,
    spec: &MissionSpec,
    tick: u32,
    agents: &mut [EnemyAgent],
    hazards: &mut [RollingHazardState],
    events: &mut Vec<AssaultTimelineEvent>,
) {
    for hazard in hazards {
        if hazard.status != RollingHazardStatus::Prepared || tick < hazard.release_tick {
            continue;
        }
        hazard.status = RollingHazardStatus::Released;
        let Some(first) = hazard.path.first().cloned() else {
            hazard.status = RollingHazardStatus::Spent;
            hazard.spent_reason = Some("no predicted path".to_string());
            continue;
        };
        events.push(AssaultTimelineEvent {
            tick,
            agent_id: None,
            group_label: Some(hazard.label.clone()),
            cell: Some(first.cell),
            kind: AssaultEventKind::RollingHazardReleased,
            cause: AssaultEventCause::RollingHazard,
            magnitude: hazard.path.len() as i32,
            note: format!(
                "{} released {} from ({}, {}) toward {}.",
                hazard.label,
                hazard.object_id,
                first.cell.x,
                first.cell.y,
                hazard.direction.label()
            ),
        });

        let mut spent_cell = first.cell;
        for (index, step) in hazard.path.iter().enumerate().skip(1) {
            hazard.path_index = index;
            spent_cell = step.cell;
            events.push(AssaultTimelineEvent {
                tick,
                agent_id: None,
                group_label: Some(hazard.label.clone()),
                cell: Some(step.cell),
                kind: AssaultEventKind::RollingHazardMoved,
                cause: AssaultEventCause::RollingHazard,
                magnitude: step.energy,
                note: format!(
                    "{} rolled through ({}, {}) with energy {}.",
                    hazard.label, step.cell.x, step.cell.y, step.energy
                ),
            });

            for agent in agents.iter_mut().filter(|agent| {
                agent.cell == step.cell
                    && matches!(
                        agent.status,
                        EnemyAgentStatus::Advancing | EnemyAgentStatus::Delayed
                    )
            }) {
                let damage = 3 + (step.energy / 4).max(0);
                agent.hp -= damage;
                agent.delay_ticks = agent.delay_ticks.max(1);
                if agent.hp > 0 {
                    agent.status = EnemyAgentStatus::Delayed;
                }
                hazard.enemies_hit += 1;
                events.push(assault_event(
                    tick,
                    Some(agent),
                    Some(step.cell),
                    AssaultEventKind::RollingHazardHitEnemy,
                    AssaultEventCause::RollingHazard,
                    damage,
                    format!(
                        "{} hit {} for {damage} damage at ({}, {}).",
                        hazard.label, agent.group_label, step.cell.x, step.cell.y
                    ),
                ));
                if agent.hp <= 0 {
                    agent.status = EnemyAgentStatus::Eliminated;
                    events.push(assault_event(
                        tick,
                        Some(agent),
                        Some(step.cell),
                        AssaultEventKind::Eliminated,
                        AssaultEventCause::RollingHazard,
                        1,
                        format!("{} was stopped by {}.", agent.group_label, hazard.label),
                    ));
                }
            }

            let mut destroyed = Vec::new();
            for object in map.objects.iter_mut().filter(|object| {
                object.cell == step.cell
                    && object.id != hazard.object_id
                    && matches!(
                        object.kind,
                        EnvironmentObjectKind::Stakes(ObstacleState::Placed)
                            | EnvironmentObjectKind::Wire(ObstacleState::Placed)
                    )
            }) {
                match object.kind {
                    EnvironmentObjectKind::Stakes(_) => {
                        object.kind = EnvironmentObjectKind::Stakes(ObstacleState::Cleared)
                    }
                    EnvironmentObjectKind::Wire(_) => {
                        object.kind = EnvironmentObjectKind::Wire(ObstacleState::Cleared)
                    }
                    _ => {}
                }
                object.movement_cost_delta = 0.0;
                destroyed.push(object.label.clone());
            }
            for label in destroyed {
                hazard.obstacles_destroyed += 1;
                events.push(AssaultTimelineEvent {
                    tick,
                    agent_id: None,
                    group_label: Some(hazard.label.clone()),
                    cell: Some(step.cell),
                    kind: AssaultEventKind::RollingHazardDestroyedObstacle,
                    cause: AssaultEventCause::RollingHazard,
                    magnitude: 1,
                    note: format!(
                        "{} destroyed {label} at ({}, {}).",
                        hazard.label, step.cell.x, step.cell.y
                    ),
                });
            }

            if step.cell == spec.objective.defend_cell {
                events.push(AssaultTimelineEvent {
                    tick,
                    agent_id: None,
                    group_label: Some(hazard.label.clone()),
                    cell: Some(step.cell),
                    kind: AssaultEventKind::RollingHazardBlocked,
                    cause: AssaultEventCause::RollingHazard,
                    magnitude: 1,
                    note: format!("{} crossed the defended objective cell.", hazard.label),
                });
            }

            if let Some(reason) = &step.blocked_reason {
                hazard.spent_reason = Some(reason.clone());
                events.push(AssaultTimelineEvent {
                    tick,
                    agent_id: None,
                    group_label: Some(hazard.label.clone()),
                    cell: Some(step.cell),
                    kind: AssaultEventKind::RollingHazardBlocked,
                    cause: AssaultEventCause::RollingHazard,
                    magnitude: step.energy,
                    note: format!("{} was blocked by {reason}.", hazard.label),
                });
                break;
            }
        }

        hazard.status = RollingHazardStatus::Spent;
        let reason = hazard
            .spent_reason
            .clone()
            .unwrap_or_else(|| "ran out of safe path".to_string());
        events.push(AssaultTimelineEvent {
            tick,
            agent_id: None,
            group_label: Some(hazard.label.clone()),
            cell: Some(spent_cell),
            kind: AssaultEventKind::RollingHazardSpent,
            cause: AssaultEventCause::RollingHazard,
            magnitude: hazard.enemies_hit as i32,
            note: format!(
                "{} spent at ({}, {}) after hitting {} enemy agent(s): {reason}.",
                hazard.label, spent_cell.x, spent_cell.y, hazard.enemies_hit
            ),
        });

        if let Some(object) = map.object_at_mut(&hazard.object_id) {
            object.kind = EnvironmentObjectKind::Log(LogState::Spent {
                direction: hazard.direction,
            });
            object.cell = spent_cell;
            object.movement_cost_delta = 0.6;
        }
    }
}

fn step_agent_assault(
    map: &MissionMap,
    spec: &MissionSpec,
    tick: u32,
    objective_health: &mut i32,
    agent: &mut EnemyAgent,
    events: &mut Vec<AssaultTimelineEvent>,
) {
    if !matches!(
        agent.status,
        EnemyAgentStatus::Advancing | EnemyAgentStatus::Delayed
    ) {
        return;
    }
    if agent.delay_ticks > 0 {
        agent.delay_ticks -= 1;
        agent.status = if agent.delay_ticks == 0 {
            EnemyAgentStatus::Advancing
        } else {
            EnemyAgentStatus::Delayed
        };
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::DelayedByTerrain,
            AssaultEventCause::Route,
            agent.delay_ticks as i32,
            format!("{} is still delayed before moving.", agent.group_label),
        ));
        return;
    }

    let defender_hit = defender_pressure_at_cell(map, spec, agent.cell);
    if defender_hit > 0 {
        agent.hp -= defender_hit;
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::SuppressedByDefender,
            AssaultEventCause::Defender,
            defender_hit,
            format!(
                "{} was pinned by defender pressure at ({}, {}).",
                agent.group_label, agent.cell.x, agent.cell.y
            ),
        ));
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::DamagedByDefender,
            AssaultEventCause::Defender,
            defender_hit,
            format!(
                "{} took {defender_hit} pressure from defender positions.",
                agent.group_label
            ),
        ));
        if agent.hp <= 0 {
            agent.status = EnemyAgentStatus::Eliminated;
            events.push(assault_event(
                tick,
                Some(agent),
                Some(agent.cell),
                AssaultEventKind::Eliminated,
                AssaultEventCause::Defender,
                1,
                format!(
                    "{} was stopped before reaching the objective.",
                    agent.group_label
                ),
            ));
            return;
        }
    }

    if agent.route_index + 1 >= agent.route.len() {
        agent.status = EnemyAgentStatus::ReachedObjective;
        let damage = objective_damage_for_doctrine(agent.doctrine);
        *objective_health -= damage;
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::ReachedObjective,
            AssaultEventCause::Objective,
            damage,
            format!("{} reached the objective.", agent.group_label),
        ));
        return;
    }

    let next = agent.route[agent.route_index + 1];
    let effect = assault_cell_effect(map, next, agent.doctrine);
    let damage = effect.total_damage();
    let reason = effect.reason_text();
    agent.cell = next;
    agent.route_index += 1;
    events.push(assault_event(
        tick,
        Some(agent),
        Some(agent.cell),
        AssaultEventKind::Moved,
        AssaultEventCause::Route,
        1,
        format!(
            "{} advanced to ({}, {}).",
            agent.group_label, next.x, next.y
        ),
    ));

    if damage > 0 {
        agent.hp -= damage;
        let cause = if effect.obstacle_damage > 0 {
            AssaultEventCause::Obstacle
        } else {
            AssaultEventCause::Terrain
        };
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::DamagedByObstacle,
            cause,
            damage,
            format!("{} took {damage} damage from {reason}.", agent.group_label),
        ));
        if agent.hp <= 0 {
            agent.status = EnemyAgentStatus::Eliminated;
            events.push(assault_event(
                tick,
                Some(agent),
                Some(agent.cell),
                AssaultEventKind::Eliminated,
                cause,
                1,
                format!("{} was stopped by {reason}.", agent.group_label),
            ));
            return;
        }
    }

    if next == spec.objective.defend_cell {
        agent.status = EnemyAgentStatus::ReachedObjective;
        let damage = objective_damage_for_doctrine(agent.doctrine);
        *objective_health -= damage;
        events.push(assault_event(
            tick,
            Some(agent),
            Some(agent.cell),
            AssaultEventKind::ReachedObjective,
            AssaultEventCause::Objective,
            damage,
            format!("{} damaged the objective for {damage}.", agent.group_label),
        ));
        return;
    }

    if effect.delay_ticks > 0 {
        agent.delay_ticks = effect.delay_ticks;
        agent.status = EnemyAgentStatus::Delayed;
        if effect.terrain_delay_ticks > 0 {
            events.push(assault_event(
                tick,
                Some(agent),
                Some(agent.cell),
                AssaultEventKind::DelayedByTerrain,
                AssaultEventCause::Terrain,
                effect.terrain_delay_ticks as i32,
                format!("{} is delayed by {reason}.", agent.group_label),
            ));
        }
        if effect.obstacle_delay_ticks > 0 {
            events.push(assault_event(
                tick,
                Some(agent),
                Some(agent.cell),
                AssaultEventKind::DelayedByObstacle,
                AssaultEventCause::Obstacle,
                effect.obstacle_delay_ticks as i32,
                format!("{} is delayed by {reason}.", agent.group_label),
            ));
        }
    }
}

fn defender_pressure_at_cell(map: &MissionMap, spec: &MissionSpec, target: CellCoord) -> i32 {
    spec.defender_positions
        .iter()
        .filter(|position| position.cell.manhattan(target) <= position.range)
        .filter(|position| mission_line_of_sight_clear(map, position.cell, target))
        .map(|position| position.pressure_per_step)
        .sum()
}

fn mission_line_of_sight_clear(map: &MissionMap, from: CellCoord, to: CellCoord) -> bool {
    if from == to {
        return true;
    }
    let steps = from.x.abs_diff(to.x).max(from.y.abs_diff(to.y)).max(1);
    for step in 1..steps {
        let t = step as f32 / steps as f32;
        let x = (from.x as f32 + (to.x as f32 - from.x as f32) * t).round() as u32;
        let y = (from.y as f32 + (to.y as f32 - from.y as f32) * t).round() as u32;
        let cell = CellCoord::new(x, y);
        if cell == from || cell == to {
            continue;
        }
        if let Some(tile) = map.cell(cell) {
            if tile.blocks_sight || tile.height >= 3 || matches!(tile.earth_state, EarthState::Berm)
            {
                return false;
            }
        }
        if map.objects_at_cell(cell).any(|object| object.blocks_sight) {
            return false;
        }
    }
    true
}

fn assault_cell_effect(
    map: &MissionMap,
    cell: CellCoord,
    doctrine: EnemyDoctrine,
) -> AssaultCellEffect {
    let mut effect = AssaultCellEffect {
        delay_ticks: 0,
        terrain_delay_ticks: 0,
        obstacle_delay_ticks: 0,
        obstacle_damage: 0,
        terrain_damage: 0,
        reasons: Vec::new(),
    };
    if let Some(tile) = map.cell(cell) {
        match tile.earth_state {
            EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch => {
                let delay = match doctrine {
                    EnemyDoctrine::RushShortest | EnemyDoctrine::AvoidObstacles => 2,
                    EnemyDoctrine::PushThroughLightObstacles | EnemyDoctrine::ClearObstacles => 1,
                    _ => 1,
                };
                effect.delay_ticks += delay;
                effect.terrain_delay_ticks += delay;
                effect.reasons.push("trench crossing".to_string());
            }
            EarthState::Berm | EarthState::SpoilPile => {
                let delay = match doctrine {
                    EnemyDoctrine::AvoidObstacles => 2,
                    _ => 1,
                };
                effect.delay_ticks += delay;
                effect.terrain_delay_ticks += delay;
                effect.reasons.push("berm slope".to_string());
            }
            EarthState::Muddy => {
                effect.delay_ticks += 1;
                effect.terrain_delay_ticks += 1;
                effect.reasons.push("mud".to_string());
            }
            EarthState::Unstable => {
                effect.terrain_damage += 1;
                effect.reasons.push("unstable ground".to_string());
            }
            EarthState::Normal | EarthState::Scraped => {}
        }
    }
    for object in map.objects_at_cell(cell) {
        match object.kind {
            EnvironmentObjectKind::Stakes(ObstacleState::Placed) => {
                let delay = match doctrine {
                    EnemyDoctrine::ClearObstacles => 0,
                    EnemyDoctrine::PushThroughLightObstacles => 1,
                    _ => 2,
                };
                let damage = match doctrine {
                    EnemyDoctrine::PushThroughLightObstacles => 2,
                    EnemyDoctrine::ClearObstacles => 0,
                    _ => 1,
                };
                effect.delay_ticks += delay;
                effect.obstacle_delay_ticks += delay;
                effect.obstacle_damage += damage;
                effect.reasons.push("stakes".to_string());
            }
            EnvironmentObjectKind::Wire(ObstacleState::Placed) => {
                let delay = match doctrine {
                    EnemyDoctrine::ClearObstacles => 1,
                    _ => 2,
                };
                effect.delay_ticks += delay;
                effect.obstacle_delay_ticks += delay;
                effect.reasons.push("wire".to_string());
            }
            EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Log(_) => {
                let delay = match doctrine {
                    EnemyDoctrine::PushThroughLightObstacles | EnemyDoctrine::ClearObstacles => 1,
                    _ => 2,
                };
                effect.delay_ticks += delay;
                effect.obstacle_delay_ticks += delay;
                effect.reasons.push("log obstacle".to_string());
            }
            _ => {}
        }
    }
    effect
}

fn enemy_hp_for_doctrine(doctrine: EnemyDoctrine) -> i32 {
    match doctrine {
        EnemyDoctrine::RushShortest => 18,
        EnemyDoctrine::PreferCover => 19,
        EnemyDoctrine::FlankViaConcealment => 18,
        EnemyDoctrine::AvoidObstacles => 19,
        EnemyDoctrine::PushThroughLightObstacles => 22,
        EnemyDoctrine::ClearObstacles => 19,
    }
}

fn objective_damage_for_doctrine(doctrine: EnemyDoctrine) -> i32 {
    match doctrine {
        EnemyDoctrine::PushThroughLightObstacles => 12,
        EnemyDoctrine::RushShortest => 10,
        _ => 8,
    }
}

fn summarize_assault(spec: &MissionSpec, assault: &AssaultState) -> AssaultSummary {
    let enemies_spawned = assault.agents.len() as u32;
    let enemies_eliminated = assault
        .agents
        .iter()
        .filter(|agent| matches!(agent.status, EnemyAgentStatus::Eliminated))
        .count() as u32;
    let enemies_reached_objective = assault
        .agents
        .iter()
        .filter(|agent| matches!(agent.status, EnemyAgentStatus::ReachedObjective))
        .count() as u32;
    let objective_damage_taken =
        spec.objective.objective_health as i32 - assault.objective_health.max(0);
    let victory = assault.objective_health > 0;
    AssaultSummary {
        victory,
        outcome_label: if victory {
            format!(
                "Victory: objective held with {} health.",
                assault.objective_health
            )
        } else {
            "Defeat: objective was overrun.".to_string()
        },
        ticks_elapsed: assault.tick,
        enemies_spawned,
        enemies_eliminated,
        enemies_reached_objective,
        objective_health_remaining: assault.objective_health.max(0),
        objective_damage_taken,
        notes: vec![
            format!("{enemies_eliminated}/{enemies_spawned} enemy agents stopped."),
            format!("{enemies_reached_objective} enemy agent(s) reached the objective."),
            format!("Objective damage taken: {objective_damage_taken}."),
        ],
    }
}

pub fn mission_rating_for_state(state: &MissionState) -> Option<MissionRating> {
    let assault = state.assault.as_ref()?;
    let summary = assault
        .summary
        .clone()
        .unwrap_or_else(|| summarize_assault(&state.spec, assault));
    let influence = build_assault_influence(state, assault);
    let rolling_hazards = build_rolling_hazard_summary(state, assault);
    Some(rate_mission_outcome(
        state,
        &summary,
        &influence,
        &rolling_hazards,
    ))
}

fn rate_mission_outcome(
    state: &MissionState,
    summary: &AssaultSummary,
    influence: &AssaultInfluenceSummary,
    rolling_hazards: &RollingHazardImpactSummary,
) -> MissionRating {
    let objective_survived = summary.victory;
    let stopped_ratio = if summary.enemies_spawned == 0 {
        1.0
    } else {
        summary.enemies_eliminated as f32 / summary.enemies_spawned as f32
    };
    let objective_health_ratio = if state.spec.objective.objective_health == 0 {
        0.0
    } else {
        summary.objective_health_remaining.max(0) as f32
            / state.spec.objective.objective_health as f32
    };
    let prep_time_used_seconds = state
        .spec
        .prep_time_seconds
        .saturating_sub(state.remaining_prep_seconds);
    let prep_time_efficiency = if state.spec.prep_time_seconds == 0 {
        0.0
    } else {
        state.remaining_prep_seconds as f32 / state.spec.prep_time_seconds as f32
    };
    let friendly_risk_count = rolling_hazards.friendly_risk_cells.len() as u32;
    let unused_defense_count = influence.unused_defenses.len() as u32;
    let hazard_enemies_hit = rolling_hazards.enemies_hit;

    let mut score = 0;
    if objective_survived {
        score += 50;
    }
    score += (stopped_ratio * 25.0).round() as i32;
    score += (objective_health_ratio * 15.0).round() as i32;
    score += (prep_time_efficiency * 10.0).round() as i32;
    score += hazard_enemies_hit.min(10) as i32;
    score -= friendly_risk_count as i32 * 10;
    score -= unused_defense_count.min(4) as i32 * 2;
    score = score.max(0);

    let stars = if !objective_survived {
        0
    } else if objective_health_ratio >= 0.90
        && stopped_ratio >= 0.85
        && prep_time_used_seconds <= 360
        && friendly_risk_count == 0
    {
        3
    } else if objective_health_ratio >= 0.70 && stopped_ratio >= 0.70 {
        2
    } else {
        1
    };
    let label = match stars {
        3 => "Decisive defense",
        2 => "Solid defense",
        1 => "Objective held",
        _ => "Objective lost",
    }
    .to_string();

    let mut notes = Vec::new();
    notes.push(if objective_survived {
        format!(
            "Objective survived with {:.0}% health.",
            objective_health_ratio * 100.0
        )
    } else {
        "Objective was overrun.".to_string()
    });
    notes.push(format!(
        "Stopped {:.0}% of attackers ({} of {}).",
        stopped_ratio * 100.0,
        summary.enemies_eliminated,
        summary.enemies_spawned
    ));
    notes.push(format!(
        "Prep used {}s of {}s.",
        prep_time_used_seconds, state.spec.prep_time_seconds
    ));
    if friendly_risk_count > 0 {
        notes.push(format!(
            "{friendly_risk_count} friendly-risk hazard cell(s) were flagged."
        ));
    }
    if unused_defense_count > 0 {
        notes.push(format!(
            "{unused_defense_count} prepared defense(s) did not affect enemy paths."
        ));
    }
    if hazard_enemies_hit > 0 {
        notes.push(format!(
            "Rolling hazards hit {hazard_enemies_hit} enemy agent(s)."
        ));
    }

    MissionRating {
        stars,
        label,
        objective_survived,
        stopped_ratio,
        objective_health_ratio,
        prep_time_used_seconds,
        prep_time_efficiency,
        friendly_risk_count,
        unused_defense_count,
        hazard_enemies_hit,
        score,
        notes,
    }
}

fn build_assault_debrief(state: &MissionState) -> Option<AssaultDebrief> {
    let assault = state.assault.as_ref()?;
    let summary = assault
        .summary
        .clone()
        .unwrap_or_else(|| summarize_assault(&state.spec, assault));
    let influence = build_assault_influence(state, assault);
    let rolling_hazards = build_rolling_hazard_summary(state, assault);
    let route_prediction_accuracy = build_route_prediction_accuracy(assault);
    let rating = rate_mission_outcome(state, &summary, &influence, &rolling_hazards);
    let mut notes = summary.notes.clone();
    if let Some(group) = &influence.most_delayed_group {
        notes.push(format!(
            "Most delayed group: {} with {} delay event(s) totaling {} tick(s).",
            group.group_label, group.count, group.magnitude
        ));
    }
    if let Some(cell) = &influence.most_effective_obstacle {
        notes.push(format!(
            "Most effective obstacle cell: ({}, {}) · {}.",
            cell.cell.x, cell.cell.y, cell.label
        ));
    }
    notes.push(format!(
        "Prediction accuracy averaged {:.0}% across {} doctrine route(s).",
        route_prediction_accuracy.average_accuracy * 100.0,
        route_prediction_accuracy.groups.len()
    ));
    if rolling_hazards.enemies_hit > 0 || rolling_hazards.obstacles_destroyed > 0 {
        notes.push(format!(
            "Rolling hazards hit {} enemy agent(s) and destroyed {} obstacle(s).",
            rolling_hazards.enemies_hit, rolling_hazards.obstacles_destroyed
        ));
    }

    Some(AssaultDebrief {
        mission_id: state.spec.id.clone(),
        outcome_label: summary.outcome_label.clone(),
        summary,
        rating,
        influence,
        rolling_hazards,
        route_prediction_accuracy,
        notes,
    })
}

#[derive(Clone, Debug, Default)]
struct CellAgg {
    count: u32,
    magnitude: i32,
    label: String,
}

fn add_cell_agg(
    map: &mut HashMap<CellCoord, CellAgg>,
    cell: CellCoord,
    magnitude: i32,
    label: impl Into<String>,
) {
    let entry = map.entry(cell).or_insert_with(|| CellAgg {
        count: 0,
        magnitude: 0,
        label: label.into(),
    });
    entry.count += 1;
    entry.magnitude += magnitude.max(1);
}

fn build_assault_influence(
    state: &MissionState,
    assault: &AssaultState,
) -> AssaultInfluenceSummary {
    let mut crossed = HashMap::new();
    let mut delayed = HashMap::new();
    let mut damaging = HashMap::new();
    let mut defender_pressure = HashMap::new();
    let mut breach = HashMap::new();
    let mut obstacle_effects = HashMap::new();
    let mut group_delay: HashMap<String, GroupInfluence> = HashMap::new();
    let mut used_defenders = HashSet::new();
    let mut used_obstacle_cells = HashSet::new();

    for agent in &assault.agents {
        for cell in actual_agent_path(agent) {
            add_cell_agg(&mut crossed, cell, 1, "enemy crossing traffic");
        }
    }

    for event in &assault.timeline {
        let Some(cell) = event.cell else {
            continue;
        };
        match event.kind {
            AssaultEventKind::DelayedByTerrain | AssaultEventKind::DelayedByObstacle => {
                add_cell_agg(&mut delayed, cell, event.magnitude, event.note.clone());
                if event.cause == AssaultEventCause::Obstacle {
                    add_cell_agg(
                        &mut obstacle_effects,
                        cell,
                        event.magnitude,
                        event.note.clone(),
                    );
                    used_obstacle_cells.insert(cell);
                }
                if let Some(group_label) = &event.group_label {
                    let entry =
                        group_delay
                            .entry(group_label.clone())
                            .or_insert_with(|| GroupInfluence {
                                group_label: group_label.clone(),
                                count: 0,
                                magnitude: 0,
                            });
                    entry.count += 1;
                    entry.magnitude += event.magnitude.max(1);
                }
            }
            AssaultEventKind::DamagedByObstacle | AssaultEventKind::DamagedByDefender => {
                add_cell_agg(&mut damaging, cell, event.magnitude, event.note.clone());
                if event.kind == AssaultEventKind::DamagedByDefender {
                    add_cell_agg(
                        &mut defender_pressure,
                        cell,
                        event.magnitude,
                        event.note.clone(),
                    );
                    for defender in &state.spec.defender_positions {
                        if defender.cell.manhattan(cell) <= defender.range
                            && mission_line_of_sight_clear(&state.map, defender.cell, cell)
                        {
                            used_defenders.insert(defender.id.clone());
                        }
                    }
                } else if event.cause == AssaultEventCause::Obstacle {
                    add_cell_agg(
                        &mut obstacle_effects,
                        cell,
                        event.magnitude,
                        event.note.clone(),
                    );
                    used_obstacle_cells.insert(cell);
                }
            }
            AssaultEventKind::SuppressedByDefender => {
                add_cell_agg(
                    &mut defender_pressure,
                    cell,
                    event.magnitude,
                    event.note.clone(),
                );
            }
            AssaultEventKind::ReachedObjective => {
                add_cell_agg(&mut breach, cell, event.magnitude, event.note.clone());
            }
            AssaultEventKind::RollingHazardHitEnemy => {
                add_cell_agg(&mut damaging, cell, event.magnitude, event.note.clone());
            }
            AssaultEventKind::AssaultStarted
            | AssaultEventKind::Spawned
            | AssaultEventKind::Moved
            | AssaultEventKind::Rerouted
            | AssaultEventKind::RollingHazardReleased
            | AssaultEventKind::RollingHazardMoved
            | AssaultEventKind::RollingHazardDestroyedObstacle
            | AssaultEventKind::RollingHazardBlocked
            | AssaultEventKind::RollingHazardSpent
            | AssaultEventKind::Eliminated
            | AssaultEventKind::AssaultEnded => {}
        }
    }

    let mut unused_defenses = Vec::new();
    for defender in &state.spec.defender_positions {
        if !used_defenders.contains(&defender.id) {
            unused_defenses.push(format!(
                "{} did not apply stopping pressure.",
                defender.label
            ));
        }
    }
    for object in &state.map.objects {
        if is_assault_obstacle(object) && !used_obstacle_cells.contains(&object.cell) {
            unused_defenses.push(format!(
                "{} at ({}, {}) did not affect an enemy path.",
                object.id, object.cell.x, object.cell.y
            ));
        }
    }

    let most_effective_obstacle = top_cell_influences(obstacle_effects, 1).into_iter().next();
    let most_delayed_group = group_delay
        .into_values()
        .max_by_key(|group| (group.magnitude, group.count));

    AssaultInfluenceSummary {
        most_crossed_cells: top_cell_influences(crossed, 8),
        most_delayed_cells: top_cell_influences(delayed, 8),
        most_damaging_cells: top_cell_influences(damaging, 8),
        defender_pressure_cells: top_cell_influences(defender_pressure, 8),
        breach_cells: top_cell_influences(breach, 4),
        most_effective_obstacle,
        most_delayed_group,
        unused_defenses,
    }
}

fn build_rolling_hazard_summary(
    state: &MissionState,
    assault: &AssaultState,
) -> RollingHazardImpactSummary {
    let mut cell_hits = HashMap::new();
    let mut enemies_eliminated = 0;
    for event in &assault.timeline {
        match event.kind {
            AssaultEventKind::RollingHazardHitEnemy => {
                if let Some(cell) = event.cell {
                    add_cell_agg(&mut cell_hits, cell, event.magnitude, event.note.clone());
                }
            }
            AssaultEventKind::Eliminated if event.cause == AssaultEventCause::RollingHazard => {
                enemies_eliminated += 1;
            }
            _ => {}
        }
    }

    let mut friendly_risk = HashSet::new();
    for hazard in &assault.rolling_hazards {
        for step in &hazard.path {
            if step.cell == state.spec.objective.defend_cell
                || state
                    .spec
                    .defender_positions
                    .iter()
                    .any(|defender| defender.cell == step.cell)
            {
                friendly_risk.insert(step.cell);
            }
        }
    }
    let mut friendly_risk_cells: Vec<_> = friendly_risk.into_iter().collect();
    friendly_risk_cells.sort_by_key(|cell| (cell.y, cell.x));
    let best_hazard_cell = top_cell_influences(cell_hits, 1).into_iter().next();
    let prepared_count = assault.rolling_hazards.len() as u32;
    let released_count = assault
        .rolling_hazards
        .iter()
        .filter(|hazard| hazard.status != RollingHazardStatus::Prepared)
        .count() as u32;
    let spent_count = assault
        .rolling_hazards
        .iter()
        .filter(|hazard| hazard.status == RollingHazardStatus::Spent)
        .count() as u32;
    let enemies_hit = assault
        .rolling_hazards
        .iter()
        .map(|hazard| hazard.enemies_hit)
        .sum();
    let obstacles_destroyed = assault
        .rolling_hazards
        .iter()
        .map(|hazard| hazard.obstacles_destroyed)
        .sum();
    let mut notes = Vec::new();
    for hazard in &assault.rolling_hazards {
        notes.push(format!(
            "{}: {} path cell(s), {} enemy hit(s), {} obstacle(s) destroyed.",
            hazard.label,
            hazard.path.len(),
            hazard.enemies_hit,
            hazard.obstacles_destroyed
        ));
    }

    RollingHazardImpactSummary {
        prepared_count,
        released_count,
        spent_count,
        enemies_hit,
        enemies_eliminated,
        obstacles_destroyed,
        friendly_risk_cells,
        best_hazard_cell,
        notes,
    }
}

fn is_assault_obstacle(object: &EnvironmentObject) -> bool {
    matches!(
        object.kind,
        EnvironmentObjectKind::Stakes(ObstacleState::Placed)
            | EnvironmentObjectKind::Wire(ObstacleState::Placed)
            | EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Log(_)
    )
}

fn top_cell_influences(map: HashMap<CellCoord, CellAgg>, limit: usize) -> Vec<CellInfluence> {
    let mut cells: Vec<_> = map
        .into_iter()
        .map(|(cell, agg)| CellInfluence {
            cell,
            count: agg.count,
            magnitude: agg.magnitude,
            label: agg.label,
        })
        .collect();
    cells.sort_by(|a, b| {
        b.magnitude
            .cmp(&a.magnitude)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.cell.y.cmp(&b.cell.y))
            .then_with(|| a.cell.x.cmp(&b.cell.x))
    });
    cells.truncate(limit);
    cells
}

fn build_route_prediction_accuracy(assault: &AssaultState) -> RoutePredictionAccuracyReport {
    let mut groups = Vec::new();
    for route in &assault.initial_routes.routes {
        let predicted: HashSet<_> = route.points.iter().copied().collect();
        let mut actual = HashSet::new();
        for agent in assault
            .agents
            .iter()
            .filter(|agent| agent.group_label == route.group_label)
        {
            actual.extend(actual_agent_path(agent));
        }
        let shared_cell_count = actual.intersection(&predicted).count() as u32;
        let mut divergence_cells: Vec<_> = actual.difference(&predicted).copied().collect();
        divergence_cells.sort_by_key(|cell| (cell.y, cell.x));
        let denominator = actual.len().max(1) as f32;
        let accuracy = shared_cell_count as f32 / denominator;
        let explanation = if divergence_cells.is_empty() {
            format!(
                "{} actual paths stayed on the predicted doctrine route through {} observed cell(s).",
                route.group_label,
                actual.len()
            )
        } else {
            format!(
                "{} diverged across {} cell(s) from the predicted doctrine route.",
                route.group_label,
                divergence_cells.len()
            )
        };
        groups.push(RoutePredictionAccuracy {
            group_label: route.group_label.clone(),
            doctrine: route.doctrine,
            predicted_cell_count: predicted.len() as u32,
            actual_cell_count: actual.len() as u32,
            shared_cell_count,
            divergence_cells,
            accuracy,
            explanation,
        });
    }

    let average_accuracy = if groups.is_empty() {
        1.0
    } else {
        groups.iter().map(|group| group.accuracy).sum::<f32>() / groups.len() as f32
    };
    let total_divergence_cells = groups
        .iter()
        .map(|group| group.divergence_cells.len() as u32)
        .sum();

    RoutePredictionAccuracyReport {
        groups,
        average_accuracy,
        total_divergence_cells,
    }
}

fn actual_agent_path(agent: &EnemyAgent) -> Vec<CellCoord> {
    if agent.route.is_empty() {
        return vec![agent.cell];
    }
    let end = agent.route_index.min(agent.route.len().saturating_sub(1));
    agent.route.iter().take(end + 1).copied().collect()
}

fn planned_rolling_hazards_for_map(map: &MissionMap, release_tick: u32) -> Vec<RollingHazardState> {
    map.objects
        .iter()
        .filter_map(|object| {
            let EnvironmentObjectKind::Log(LogState::PreparedRoll {
                direction,
                release_cell,
                ..
            }) = object.kind
            else {
                return None;
            };
            let path = predict_rolling_log_path(map, release_cell, direction);
            (path.len() > 1).then(|| RollingHazardState {
                object_id: object.id.clone(),
                label: object.label.clone(),
                direction,
                release_tick,
                path,
                status: RollingHazardStatus::Prepared,
                path_index: 0,
                enemies_hit: 0,
                obstacles_destroyed: 0,
                spent_reason: None,
            })
        })
        .collect()
}

pub fn predict_rolling_log_path(
    map: &MissionMap,
    start: CellCoord,
    direction: Direction,
) -> Vec<RollingHazardStep> {
    let Some(start_cell) = map.cell(start) else {
        return Vec::new();
    };
    let mut path = vec![RollingHazardStep {
        cell: start,
        height_delta: 0,
        energy: 4,
        blocked_reason: blocking_reason_for_rolling_log(map, start, None),
    }];
    let mut current = start;
    let mut current_height = start_cell.height;
    let mut energy = 4;
    let mut visited = HashSet::new();
    visited.insert(start);

    for _ in 0..8 {
        let Some((next, delta)) = next_rolling_log_cell(map, current, current_height, direction)
        else {
            break;
        };
        if visited.contains(&next) {
            break;
        }
        let next_height = map
            .cell(next)
            .map(|cell| cell.height)
            .unwrap_or(current_height);
        energy += delta.max(0) as i32;
        if delta <= 0 {
            energy -= 1;
        }
        if energy <= 0 {
            break;
        }
        let blocked_reason = blocking_reason_for_rolling_log(map, next, None);
        path.push(RollingHazardStep {
            cell: next,
            height_delta: delta,
            energy,
            blocked_reason: blocked_reason.clone(),
        });
        visited.insert(next);
        current = next;
        current_height = next_height;
        if blocked_reason.is_some() {
            break;
        }
    }

    path
}

fn next_rolling_log_cell(
    map: &MissionMap,
    current: CellCoord,
    current_height: i8,
    direction: Direction,
) -> Option<(CellCoord, i8)> {
    if let Some(forward) = offset_cell(map, current, direction) {
        if let Some(forward_height) = map.cell(forward).map(|cell| cell.height) {
            let delta = current_height - forward_height;
            if delta >= 0 {
                return Some((forward, delta));
            }
        }
    }

    let mut candidates = map
        .neighbors4(current)
        .into_iter()
        .filter_map(|cell| {
            let next_height = map.cell(cell)?.height;
            let delta = current_height - next_height;
            (delta >= 0).then_some((cell, delta))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|(cell, delta)| (-(*delta as i16), cell.y, cell.x));
    candidates.into_iter().next()
}

fn offset_cell(map: &MissionMap, cell: CellCoord, direction: Direction) -> Option<CellCoord> {
    let (dx, dy) = direction.delta();
    let x = cell.x as i32 + dx;
    let y = cell.y as i32 + dy;
    (x >= 0 && y >= 0)
        .then_some(CellCoord::new(x as u32, y as u32))
        .filter(|coord| map.cell(*coord).is_some())
}

fn blocking_reason_for_rolling_log(
    map: &MissionMap,
    cell: CellCoord,
    source_object_id: Option<&str>,
) -> Option<String> {
    let tile = map.cell(cell)?;
    if matches!(
        tile.earth_state,
        EarthState::Berm | EarthState::DeepTrench | EarthState::Trench
    ) {
        return Some(format!("{:?}", tile.earth_state));
    }
    for object in map.objects_at_cell(cell) {
        if Some(object.id.as_str()) == source_object_id {
            continue;
        }
        if matches!(
            object.kind,
            EnvironmentObjectKind::Tree(TreeState::Standing)
                | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. })
                | EnvironmentObjectKind::Rock(_)
                | EnvironmentObjectKind::Wall(_)
        ) {
            return Some(object.label.clone());
        }
    }
    None
}

fn rolling_log_direction(kind: &EnvironmentObjectKind) -> Option<Direction> {
    match kind {
        EnvironmentObjectKind::Tree(TreeState::FallenTrunk { direction })
        | EnvironmentObjectKind::Log(LogState::Loose { direction })
        | EnvironmentObjectKind::Log(LogState::DragPrepared { direction })
        | EnvironmentObjectKind::Log(LogState::Positioned { direction })
        | EnvironmentObjectKind::Log(LogState::Braced { direction })
        | EnvironmentObjectKind::Log(LogState::Released { direction })
        | EnvironmentObjectKind::Log(LogState::Rolling { direction })
        | EnvironmentObjectKind::Log(LogState::Spent { direction }) => Some(*direction),
        EnvironmentObjectKind::Log(LogState::PreparedRoll { direction, .. }) => Some(*direction),
        _ => None,
    }
}

fn is_preparable_rolling_log(kind: &EnvironmentObjectKind) -> bool {
    matches!(
        kind,
        EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Log(LogState::Loose { .. })
            | EnvironmentObjectKind::Log(LogState::DragPrepared { .. })
            | EnvironmentObjectKind::Log(LogState::Positioned { .. })
            | EnvironmentObjectKind::Log(LogState::Braced { .. })
    )
}

fn build_work_order(
    id: u32,
    kind: WorkOrderKind,
    target: WorkTarget,
    map: &MissionMap,
) -> WorkOrder {
    let affected_cells = match &target {
        WorkTarget::Object(object_id) => map
            .objects
            .iter()
            .find(|object| &object.id == object_id)
            .map(|object| vec![object.cell])
            .unwrap_or_default(),
        _ => target.affected_cells(),
    };
    let affected_count = affected_cells.len().max(1) as u32;
    let (required_tools, labor_seconds, material_inputs, material_outputs, mut notes) = match kind {
        WorkOrderKind::DigTrench => {
            let outputs = LocalMaterialStock {
                earth_spoil: affected_count as i32 * 2,
                ..Default::default()
            };
            (
                vec![ToolKind::Shovel],
                affected_count * 40,
                LocalMaterialStock::default(),
                outputs,
                vec!["lowers earth and creates local spoil".to_string()],
            )
        }
        WorkOrderKind::RaiseBerm => {
            let inputs = LocalMaterialStock {
                earth_spoil: affected_count as i32 * 2,
                ..Default::default()
            };
            (
                vec![ToolKind::Shovel],
                affected_count * 35,
                inputs,
                LocalMaterialStock::default(),
                vec!["consumes nearby spoil and raises cover".to_string()],
            )
        }
        WorkOrderKind::Flatten => (
            vec![ToolKind::Shovel],
            affected_count * 25,
            LocalMaterialStock::default(),
            LocalMaterialStock::default(),
            vec!["removes trench/berm state and leaves scraped ground".to_string()],
        ),
        WorkOrderKind::FellTree => (
            vec![ToolKind::Axe],
            60,
            LocalMaterialStock::default(),
            LocalMaterialStock::default(),
            vec!["turns a standing tree into a fallen trunk".to_string()],
        ),
        WorkOrderKind::CutIntoLogs => {
            let outputs = LocalMaterialStock {
                logs: 2,
                timber: 1,
                ..Default::default()
            };
            (
                vec![ToolKind::Axe],
                45,
                LocalMaterialStock::default(),
                outputs,
                vec!["converts a fallen trunk into local usable timber".to_string()],
            )
        }
        WorkOrderKind::PlaceStakes => {
            let inputs = LocalMaterialStock {
                logs: 1,
                ..Default::default()
            };
            (
                vec![ToolKind::Hammer],
                35,
                inputs,
                LocalMaterialStock::default(),
                vec!["converts one nearby log into a crude stake obstacle".to_string()],
            )
        }
        WorkOrderKind::PrepareRollingLog => (
            vec![ToolKind::Rope],
            80,
            LocalMaterialStock::default(),
            LocalMaterialStock::default(),
            vec![
                "positions and braces a physical log as a deterministic rolling hazard".to_string(),
            ],
        ),
    };
    let crew_required = kind.default_crew_required();
    let assigned_crews = crew_required.max(1);
    let duration_seconds = labor_seconds.div_ceil(assigned_crews);
    let material_delta = LocalMaterialStock::net(&material_outputs, &material_inputs);
    if kind == WorkOrderKind::PrepareRollingLog {
        if let Ok(object) = target_rolling_log_object(map, &target) {
            let direction = rolling_log_direction(&object.kind).unwrap_or(Direction::South);
            let path = predict_rolling_log_path(map, object.cell, direction);
            notes.push(format!(
                "predicted path: {} cell(s) toward {}",
                path.len(),
                direction.label()
            ));
            if path.iter().any(|step| step.blocked_reason.is_some()) {
                notes.push("path includes a blocking terrain/object stop".to_string());
            }
        }
    }

    WorkOrder {
        id,
        kind,
        target,
        required_tools,
        crew_required,
        assigned_crews,
        labor_seconds,
        duration_seconds,
        material_inputs,
        material_outputs: material_outputs.clone(),
        progress_seconds: 0,
        status: WorkOrderStatus::Planned,
        preview: WorkOrderPreview {
            affected_cells,
            affected_objects: Vec::new(),
            material_delta,
            notes,
        },
    }
}

fn order_with_status(mut order: WorkOrder, status: WorkOrderStatus) -> WorkOrder {
    order.status = status;
    order
}

fn material_requirements(stock: &LocalMaterialStock) -> [(LocalMaterialKind, i32); 7] {
    [
        (LocalMaterialKind::EarthSpoil, stock.earth_spoil),
        (LocalMaterialKind::Timber, stock.timber),
        (LocalMaterialKind::Logs, stock.logs),
        (LocalMaterialKind::Stakes, stock.stakes),
        (LocalMaterialKind::LooseStone, stock.loose_stone),
        (LocalMaterialKind::Scrap, stock.scrap),
        (LocalMaterialKind::RopeUses, stock.rope_uses),
    ]
}

fn available_nearby_material(map: &MissionMap, origin: CellCoord, kind: LocalMaterialKind) -> i32 {
    let mut total = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let cell_coord = CellCoord::new(x, y);
            if origin.manhattan(cell_coord) <= 3 {
                if let Some(cell) = map.cell(cell_coord) {
                    total += cell.local_material.get(kind).max(0);
                }
            }
        }
    }
    total
}

fn validate_order_target(map: &MissionMap, order: &WorkOrder) -> std::result::Result<(), String> {
    match order.kind {
        WorkOrderKind::DigTrench => {
            for cell_coord in order.target.affected_cells() {
                let cell = map.cell(cell_coord).ok_or_else(|| {
                    format!(
                        "target cell ({}, {}) is outside the mission map",
                        cell_coord.x, cell_coord.y
                    )
                })?;
                if matches!(cell.ground, GroundKind::Rock | GroundKind::Mud) {
                    return Err(format!(
                        "cannot dig trench into {} at ({}, {}) with the basic kit",
                        cell.ground.label(),
                        cell_coord.x,
                        cell_coord.y
                    ));
                }
                if matches!(cell.earth_state, EarthState::Berm) {
                    return Err(format!(
                        "flatten the berm at ({}, {}) before digging",
                        cell_coord.x, cell_coord.y
                    ));
                }
            }
        }
        WorkOrderKind::RaiseBerm => {
            for cell_coord in order.target.affected_cells() {
                let cell = map.cell(cell_coord).ok_or_else(|| {
                    format!(
                        "target cell ({}, {}) is outside the mission map",
                        cell_coord.x, cell_coord.y
                    )
                })?;
                if matches!(
                    cell.earth_state,
                    EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch
                ) {
                    return Err(format!(
                        "fill or flatten the cut at ({}, {}) before raising a berm",
                        cell_coord.x, cell_coord.y
                    ));
                }
            }
        }
        WorkOrderKind::Flatten => {
            if order.target.affected_cells().is_empty() {
                return Err("flatten requires at least one target cell".to_string());
            }
        }
        WorkOrderKind::FellTree => {
            target_tree_object(map, &order.target)?;
        }
        WorkOrderKind::CutIntoLogs => {
            target_tree_object(map, &order.target)?;
        }
        WorkOrderKind::PlaceStakes => {
            let target = match order.target {
                WorkTarget::Cell(cell) => cell,
                WorkTarget::Rect(rect) => rect.origin,
                WorkTarget::Object(_) => {
                    return Err("place stakes requires a cell target".to_string());
                }
            };
            map.cell(target).ok_or_else(|| {
                format!(
                    "target cell ({}, {}) is outside the mission map",
                    target.x, target.y
                )
            })?;
            if map.objects.iter().any(|object| {
                object.cell == target && matches!(object.kind, EnvironmentObjectKind::Stakes(_))
            }) {
                return Err(format!(
                    "stakes already occupy ({}, {})",
                    target.x, target.y
                ));
            }
        }
        WorkOrderKind::PrepareRollingLog => {
            let object = target_rolling_log_object(map, &order.target)?;
            let direction = rolling_log_direction(&object.kind).unwrap_or(Direction::South);
            let path = predict_rolling_log_path(map, object.cell, direction);
            if path.len() < 2 {
                return Err(format!(
                    "{} has no useful downhill or forward roll path",
                    object.label
                ));
            }
        }
    }
    Ok(())
}

fn apply_order_effects(map: &mut MissionMap, order: &mut WorkOrder) -> Result<()> {
    match order.kind {
        WorkOrderKind::DigTrench => {
            for cell_coord in order.target.affected_cells() {
                let Some(cell) = map.cell_mut(cell_coord) else {
                    bail!(
                        "target cell ({}, {}) is outside the mission map",
                        cell_coord.x,
                        cell_coord.y
                    );
                };
                cell.height = (cell.height - 1).clamp(0, 9);
                cell.earth_state = match cell.earth_state {
                    EarthState::Trench | EarthState::DeepTrench => EarthState::DeepTrench,
                    _ => EarthState::Trench,
                };
                cell.cover = CoverClass::Strong;
                cell.movement_cost = cell.ground.base_movement_cost() + 1.2;
                cell.blocks_sight = false;
                cell.local_material.earth_spoil += 2;
            }
        }
        WorkOrderKind::RaiseBerm => {
            for cell_coord in order.target.affected_cells() {
                consume_nearby_material(map, cell_coord, LocalMaterialKind::EarthSpoil, 2)
                    .with_context(|| {
                        format!(
                            "not enough nearby spoil to raise berm at ({}, {})",
                            cell_coord.x, cell_coord.y
                        )
                    })?;
                let Some(cell) = map.cell_mut(cell_coord) else {
                    bail!(
                        "target cell ({}, {}) is outside the mission map",
                        cell_coord.x,
                        cell_coord.y
                    );
                };
                cell.height = (cell.height + 1).clamp(0, 9);
                cell.earth_state = EarthState::Berm;
                cell.cover = CoverClass::Strong;
                cell.movement_cost = cell.ground.base_movement_cost() + 0.8;
                cell.blocks_sight = true;
            }
        }
        WorkOrderKind::Flatten => {
            for cell_coord in order.target.affected_cells() {
                let Some(cell) = map.cell_mut(cell_coord) else {
                    bail!(
                        "target cell ({}, {}) is outside the mission map",
                        cell_coord.x,
                        cell_coord.y
                    );
                };
                if matches!(cell.earth_state, EarthState::Berm | EarthState::SpoilPile) {
                    cell.local_material.earth_spoil += 1;
                }
                cell.earth_state = EarthState::Scraped;
                cell.cover = CoverClass::None;
                cell.movement_cost = cell.ground.base_movement_cost() + 0.15;
                cell.blocks_sight = false;
            }
        }
        WorkOrderKind::FellTree => {
            let object = target_tree_object_mut(map, &order.target)?;
            match object.kind {
                EnvironmentObjectKind::Tree(TreeState::Standing)
                | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => {
                    object.kind = EnvironmentObjectKind::Tree(TreeState::FallenTrunk {
                        direction: Direction::East,
                    });
                    object.label = format!("fallen {}", object.label);
                    object.footprint = (2, 1);
                    object.blocks_sight = false;
                    object.cover = CoverClass::Light;
                    object.movement_cost_delta = 1.0;
                    order.preview.affected_objects.push(object.id.clone());
                }
                _ => bail!("target tree is not standing or partially cut"),
            }
        }
        WorkOrderKind::CutIntoLogs => {
            let (cell_coord, object_id) = {
                let object = target_tree_object_mut(map, &order.target)?;
                match object.kind {
                    EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. }) => {
                        object.kind = EnvironmentObjectKind::Tree(TreeState::CutLogs);
                        object.label = format!("cut {}", object.label);
                        object.footprint = (1, 1);
                        object.cover = CoverClass::Light;
                        object.movement_cost_delta = 0.2;
                        (object.cell, object.id.clone())
                    }
                    _ => bail!("target tree must be a fallen trunk before cutting logs"),
                }
            };
            let Some(cell) = map.cell_mut(cell_coord) else {
                bail!("tree cell is outside the mission map");
            };
            cell.local_material.logs += 2;
            cell.local_material.timber += 1;
            order.preview.affected_objects.push(object_id);
        }
        WorkOrderKind::PlaceStakes => {
            let cell_coord = match order.target {
                WorkTarget::Cell(cell) => cell,
                WorkTarget::Rect(rect) => rect.origin,
                WorkTarget::Object(_) => bail!("place stakes requires a cell target"),
            };
            consume_nearby_material(map, cell_coord, LocalMaterialKind::Logs, 1)
                .context("not enough nearby logs to place stakes")?;
            map.objects.push(EnvironmentObject {
                id: format!("stakes_{}_{}", cell_coord.x, cell_coord.y),
                label: "field stakes".to_string(),
                kind: EnvironmentObjectKind::Stakes(ObstacleState::Placed),
                cell: cell_coord,
                footprint: (1, 1),
                blocks_sight: false,
                cover: CoverClass::Light,
                movement_cost_delta: 2.2,
            });
            if let Some(cell) = map.cell_mut(cell_coord) {
                cell.movement_cost += 1.0;
            }
            order
                .preview
                .affected_objects
                .push(format!("stakes_{}_{}", cell_coord.x, cell_coord.y));
        }
        WorkOrderKind::PrepareRollingLog => {
            let (object_id, object_cell, direction) = {
                let object = target_rolling_log_object(map, &order.target)
                    .map_err(|err| anyhow::anyhow!(err))?;
                (
                    object.id.clone(),
                    object.cell,
                    rolling_log_direction(&object.kind).unwrap_or(Direction::South),
                )
            };
            let predicted_path = predict_rolling_log_path(map, object_cell, direction)
                .into_iter()
                .map(|step| step.cell)
                .collect::<Vec<_>>();
            if predicted_path.len() < 2 {
                bail!("rolling log has no useful predicted path");
            }
            let object = map
                .object_at_mut(&object_id)
                .ok_or_else(|| anyhow::anyhow!("rolling log object disappeared"))?;
            object.kind = EnvironmentObjectKind::Log(LogState::PreparedRoll {
                direction,
                release_cell: object_cell,
                predicted_path: predicted_path.clone(),
            });
            object.label = format!("prepared {}", object.label);
            object.blocks_sight = false;
            object.cover = CoverClass::Light;
            object.movement_cost_delta = 1.2;
            order.preview.affected_objects.push(object_id);
            order.preview.notes.push(format!(
                "predicted roll path: {} cell(s) toward {}",
                predicted_path.len(),
                direction.label()
            ));
        }
    }
    Ok(())
}

fn consume_nearby_material(
    map: &mut MissionMap,
    origin: CellCoord,
    kind: LocalMaterialKind,
    mut amount: i32,
) -> Result<()> {
    let mut candidates = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let cell = CellCoord::new(x, y);
            if origin.manhattan(cell) <= 3 {
                candidates.push(cell);
            }
        }
    }
    candidates.sort_by_key(|cell| (origin.manhattan(*cell), cell.y, cell.x));

    for cell_coord in candidates {
        let Some(cell) = map.cell_mut(cell_coord) else {
            continue;
        };
        let available = cell.local_material.get(kind).max(0);
        let take = available.min(amount);
        if take > 0 {
            cell.local_material.add(kind, -take);
            amount -= take;
        }
        if amount == 0 {
            return Ok(());
        }
    }

    bail!("needed {amount} more local material")
}

fn target_tree_object_mut<'a>(
    map: &'a mut MissionMap,
    target: &WorkTarget,
) -> Result<&'a mut EnvironmentObject> {
    match target {
        WorkTarget::Object(id) => map
            .object_at_mut(id)
            .with_context(|| format!("object `{id}` was not found")),
        WorkTarget::Cell(cell) => map
            .object_at_cell_mut(*cell, |kind| matches!(kind, EnvironmentObjectKind::Tree(_)))
            .with_context(|| format!("no tree object at ({}, {})", cell.x, cell.y)),
        WorkTarget::Rect(rect) => map
            .object_at_cell_mut(rect.origin, |kind| {
                matches!(kind, EnvironmentObjectKind::Tree(_))
            })
            .with_context(|| format!("no tree object at ({}, {})", rect.origin.x, rect.origin.y)),
    }
}

fn target_tree_object<'a>(
    map: &'a MissionMap,
    target: &WorkTarget,
) -> std::result::Result<&'a EnvironmentObject, String> {
    match target {
        WorkTarget::Object(id) => map
            .objects
            .iter()
            .find(|object| &object.id == id)
            .ok_or_else(|| format!("object `{id}` was not found")),
        WorkTarget::Cell(cell) => map
            .objects
            .iter()
            .find(|object| {
                object.cell == *cell && matches!(object.kind, EnvironmentObjectKind::Tree(_))
            })
            .ok_or_else(|| format!("no tree object at ({}, {})", cell.x, cell.y)),
        WorkTarget::Rect(rect) => map
            .objects
            .iter()
            .find(|object| {
                object.cell == rect.origin && matches!(object.kind, EnvironmentObjectKind::Tree(_))
            })
            .ok_or_else(|| format!("no tree object at ({}, {})", rect.origin.x, rect.origin.y)),
    }
}

fn target_rolling_log_object<'a>(
    map: &'a MissionMap,
    target: &WorkTarget,
) -> std::result::Result<&'a EnvironmentObject, String> {
    let object = match target {
        WorkTarget::Object(id) => map
            .objects
            .iter()
            .find(|object| &object.id == id)
            .ok_or_else(|| format!("object `{id}` was not found"))?,
        WorkTarget::Cell(cell) => map
            .objects
            .iter()
            .find(|object| object.cell == *cell && is_preparable_rolling_log(&object.kind))
            .ok_or_else(|| format!("no preparable log at ({}, {})", cell.x, cell.y))?,
        WorkTarget::Rect(rect) => map
            .objects
            .iter()
            .find(|object| object.cell == rect.origin && is_preparable_rolling_log(&object.kind))
            .ok_or_else(|| {
                format!(
                    "no preparable log at ({}, {})",
                    rect.origin.x, rect.origin.y
                )
            })?,
    };
    if is_preparable_rolling_log(&object.kind) {
        Ok(object)
    } else {
        Err(format!(
            "{} cannot be prepared as a rolling log",
            object.label
        ))
    }
}

pub fn ascii_map(state: &MissionState) -> String {
    let mut out = String::new();
    for y in 0..state.map.height {
        for x in 0..state.map.width {
            let coord = CellCoord::new(x, y);
            if state.spec.objective.defend_cell == coord {
                out.push('O');
                continue;
            }
            if state.map.spawn_cells.contains(&coord) {
                out.push('S');
                continue;
            }
            if let Some(object) = state.map.objects.iter().find(|object| object.cell == coord) {
                out.push(match object.kind {
                    EnvironmentObjectKind::Tree(TreeState::Standing)
                    | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => 'T',
                    EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
                    | EnvironmentObjectKind::Tree(TreeState::CutLogs)
                    | EnvironmentObjectKind::Log(_) => 'L',
                    EnvironmentObjectKind::Stakes(_) => '^',
                    EnvironmentObjectKind::Rock(_) => 'r',
                    EnvironmentObjectKind::Wall(_) => 'W',
                    EnvironmentObjectKind::Wire(_) => 'w',
                    EnvironmentObjectKind::FightingPosition(_) => 'F',
                    EnvironmentObjectKind::Tree(TreeState::Falling { .. })
                    | EnvironmentObjectKind::Tree(TreeState::StakesBundle)
                    | EnvironmentObjectKind::Tree(TreeState::Stump) => 't',
                });
                continue;
            }
            let cell = state
                .map
                .cell(coord)
                .expect("ascii map only accesses in-bounds cells");
            out.push(match cell.earth_state {
                EarthState::Trench | EarthState::DeepTrench => '=',
                EarthState::Berm | EarthState::SpoilPile => '#',
                EarthState::Scraped | EarthState::Ditch => '_',
                _ => match cell.ground {
                    GroundKind::Grass => '.',
                    GroundKind::Dirt => ',',
                    GroundKind::Mud => '~',
                    GroundKind::Rock => 'R',
                    GroundKind::Road => ':',
                },
            });
        }
        out.push('\n');
    }
    out
}

pub fn mission_summary(state: &MissionState) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}\n", state.spec.title));
    out.push_str(&format!("mission id: {}\n", state.spec.id));
    out.push_str(&format!(
        "prep remaining: {}s · labor remaining: {}s\n",
        state.remaining_prep_seconds, state.remaining_labor_seconds
    ));
    out.push_str(&format!(
        "objective: {} at ({}, {})\n",
        state.spec.objective.label,
        state.spec.objective.defend_cell.x,
        state.spec.objective.defend_cell.y
    ));
    out.push_str("work orders:\n");
    for order in &state.work_orders {
        out.push_str(&format!(
            "- #{:02} {} · {} · duration {}s · labor {}s · crew {}\n",
            order.id,
            order.kind.label(),
            order.status.label(),
            order.duration_seconds,
            order.labor_seconds,
            order.assigned_crews
        ));
    }
    if !state.material_ledger.is_empty() {
        out.push_str("material ledger:\n");
        for entry in &state.material_ledger {
            let summary = entry.net.signed_summary();
            out.push_str(&format!(
                "- #{:02} {} · {}\n",
                entry.order_id,
                entry.order_kind.label(),
                if summary.is_empty() {
                    "no material delta".to_string()
                } else {
                    summary.join(", ")
                }
            ));
        }
    }
    let materials = state.material_totals().positive_summary();
    out.push_str(&format!(
        "local material remaining: {}\n",
        if materials.is_empty() {
            "none".to_string()
        } else {
            materials.join(", ")
        }
    ));
    out.push_str("\nmap:\n");
    out.push_str(&ascii_map(state));
    out
}

pub fn save_mission_preview_png(path: impl AsRef<Path>, state: &MissionState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let image = mission_preview_image(state, cell_px);
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn export_mission_visuals(
    out_dir: impl AsRef<Path>,
    spec: MissionSpec,
) -> Result<MissionVisualAssetReport> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let state = MissionState::from_spec(spec);
    let report =
        save_mission_visual_preview_png(out_dir.join("mission_visual_preview.png"), &state)?;
    save_mission_visual_routes_png(out_dir.join("mission_visual_routes.png"), &state)?;
    save_mission_visual_debug_png(out_dir.join("mission_visual_debug.png"), &state)?;
    write_json(out_dir.join("visual_asset_report.json"), &report)?;
    Ok(report)
}

pub fn save_mission_visual_preview_png(
    path: impl AsRef<Path>,
    state: &MissionState,
) -> Result<MissionVisualAssetReport> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let (image, report) = mission_visual_image(state, MissionVisualOverlay::None)?;
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))?;
    Ok(report)
}

pub fn save_mission_visual_routes_png(path: impl AsRef<Path>, state: &MissionState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let (image, _) = mission_visual_image(state, MissionVisualOverlay::Routes)?;
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_mission_visual_debug_png(path: impl AsRef<Path>, state: &MissionState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let (image, _) = mission_visual_image(state, MissionVisualOverlay::Debug)?;
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MissionVisualOverlay {
    None,
    Routes,
    Debug,
}

struct MissionVisualProjection {
    tile_w: u32,
    tile_h: u32,
    height_step: i32,
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
}

impl MissionVisualProjection {
    fn new(map: &MissionMap) -> Self {
        let tile_w = 56;
        let tile_h = 36;
        let height_step = 10;
        let margin = 80;
        let width = (map.width + map.height) * tile_w / 2 + margin * 2;
        let height = (map.width + map.height) * tile_h / 2 + margin * 2;
        Self {
            tile_w,
            tile_h,
            height_step,
            origin_x: (map.height as i32 * tile_w as i32 / 2) + margin as i32,
            origin_y: margin as i32,
            width,
            height,
        }
    }

    fn center(&self, cell: CellCoord, height: i8) -> (i32, i32) {
        (
            self.origin_x + (cell.x as i32 - cell.y as i32) * self.tile_w as i32 / 2,
            self.origin_y + (cell.x as i32 + cell.y as i32) * self.tile_h as i32 / 2
                - height as i32 * self.height_step,
        )
    }

    fn top_left(&self, cell: CellCoord, height: i8) -> (i32, i32) {
        let (cx, cy) = self.center(cell, height);
        (cx - self.tile_w as i32 / 2, cy - self.tile_h as i32 / 2)
    }
}

fn mission_visual_image(
    state: &MissionState,
    overlay: MissionVisualOverlay,
) -> Result<(RgbaImage, MissionVisualAssetReport)> {
    let projection = MissionVisualProjection::new(&state.map);
    let mut image =
        RgbaImage::from_pixel(projection.width, projection.height, Rgba([21, 24, 24, 255]));
    let mut assets = mission_visual_asset_context(state);
    let mut missing = HashSet::new();
    let mut fallbacks = HashSet::new();

    for y in 0..state.map.height {
        for x in 0..state.map.width {
            let coord = CellCoord::new(x, y);
            let Some(cell) = state.map.cell(coord) else {
                continue;
            };
            draw_visual_cell_faces(&mut image, state, coord, cell, &projection);
            let kind = mission_cell_sprite_kind(state, coord, cell);
            let sprite = mission_visual_sprite(&assets.sprites, kind).or_else(|| {
                missing.insert(format!("{kind:?}"));
                let fallback = match cell.ground {
                    GroundKind::Road | GroundKind::Dirt | GroundKind::Mud => {
                        TerrainSpriteKind::DirtTile
                    }
                    GroundKind::Rock => TerrainSpriteKind::StoneFloorTop,
                    GroundKind::Grass => TerrainSpriteKind::GrassTile,
                };
                fallbacks.insert(format!("{kind:?}->{fallback:?}"));
                mission_visual_sprite(&assets.sprites, fallback)
            });
            if let Some(sprite) = sprite {
                draw_diamond_sprite(&mut image, &sprite.image, &projection, coord, cell.height);
            } else {
                draw_diamond_fill(
                    &mut image,
                    &projection,
                    coord,
                    cell.height,
                    preview_cell_color(cell),
                );
            }
            if matches!(overlay, MissionVisualOverlay::Debug) {
                draw_diamond_outline(
                    &mut image,
                    &projection,
                    coord,
                    cell.height,
                    Rgba([28, 32, 29, 180]),
                );
            }
        }
    }

    draw_visual_objects(&mut image, state, &projection);
    draw_visual_mission_markers(&mut image, state, &projection);
    if matches!(
        overlay,
        MissionVisualOverlay::Routes | MissionVisualOverlay::Debug
    ) {
        let routes = state.route_preview();
        let colors = [
            Rgba([242, 183, 62, 220]),
            Rgba([98, 205, 139, 220]),
            Rgba([220, 108, 92, 220]),
            Rgba([123, 171, 242, 220]),
        ];
        for (index, route) in routes.routes.iter().enumerate() {
            draw_visual_route(
                &mut image,
                state,
                &projection,
                route,
                colors[index % colors.len()],
            );
        }
    }

    assets.report.missing_visual_pieces = sorted_strings(missing);
    assets.report.fallback_pieces_used = sorted_strings(fallbacks);
    if !assets.report.missing_visual_pieces.is_empty() {
        assets.report.warnings.push(format!(
            "{} visual piece kind(s) were missing and used fallbacks.",
            assets.report.missing_visual_pieces.len()
        ));
    }
    Ok((image, assets.report))
}

struct MissionVisualAssetContext {
    sprites: Vec<GeneratedTerrainSprite>,
    report: MissionVisualAssetReport,
}

fn mission_visual_asset_context(state: &MissionState) -> MissionVisualAssetContext {
    let profile = state.spec.visual_theme.sprite_style_profile.clone();
    let mut warnings = Vec::new();
    let mut recipe = match TerrainSpriteRecipe::from_style_profile_path(&profile) {
        Ok(recipe) => recipe,
        Err(err) => {
            warnings.push(format!(
                "failed to load sprite profile {profile}: {err}; using default profile"
            ));
            TerrainSpriteRecipe::from_default_style_profile()
        }
    };
    recipe.sanitize();
    let bundle = generate_effective_terrain_sprites(&recipe);
    let report = MissionVisualAssetReport {
        mission_id: state.spec.id.clone(),
        sprite_style_profile: profile,
        render_projection: state.spec.visual_theme.render_projection,
        generated_sprite_count: bundle.report.generated_count,
        effective_sprite_count: bundle.effective.len(),
        overridden_sprite_count: bundle.report.overridden_count,
        override_issue_count: bundle.report.issue_count(),
        missing_visual_pieces: Vec::new(),
        fallback_pieces_used: Vec::new(),
        warnings,
    };
    MissionVisualAssetContext {
        sprites: bundle.effective,
        report,
    }
}

fn mission_visual_sprite(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
) -> Option<&GeneratedTerrainSprite> {
    sprites.iter().find(|sprite| sprite.kind == kind)
}

fn mission_cell_sprite_kind(
    state: &MissionState,
    coord: CellCoord,
    cell: &MissionCell,
) -> TerrainSpriteKind {
    match cell.earth_state {
        EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch => {
            TerrainSpriteKind::from_trench_mask(cardinal_mask_for_cells(state, coord, |cell| {
                matches!(
                    cell.earth_state,
                    EarthState::Trench | EarthState::DeepTrench | EarthState::Ditch
                )
            }))
            .unwrap_or(TerrainSpriteKind::TrenchMask15)
        }
        EarthState::Berm | EarthState::SpoilPile => {
            TerrainSpriteKind::from_berm_mask(cardinal_mask_for_cells(state, coord, |cell| {
                matches!(cell.earth_state, EarthState::Berm | EarthState::SpoilPile)
            }))
            .unwrap_or(TerrainSpriteKind::BermMask15)
        }
        _ => match cell.ground {
            GroundKind::Road => {
                TerrainSpriteKind::from_path_mask(cardinal_mask_for_cells(state, coord, |cell| {
                    cell.ground == GroundKind::Road
                }))
                .unwrap_or(TerrainSpriteKind::PathMask15)
            }
            GroundKind::Dirt | GroundKind::Mud => TerrainSpriteKind::DirtTile,
            GroundKind::Rock => TerrainSpriteKind::StoneFloorTop,
            GroundKind::Grass => TerrainSpriteKind::GrassTile,
        },
    }
}

fn cardinal_mask_for_cells(
    state: &MissionState,
    coord: CellCoord,
    include: impl Fn(&MissionCell) -> bool,
) -> u8 {
    let mut mask = 0;
    let checks = [
        (
            1,
            coord.y.checked_sub(1).map(|y| CellCoord::new(coord.x, y)),
        ),
        (2, Some(CellCoord::new(coord.x + 1, coord.y))),
        (4, Some(CellCoord::new(coord.x, coord.y + 1))),
        (
            8,
            coord.x.checked_sub(1).map(|x| CellCoord::new(x, coord.y)),
        ),
    ];
    for (bit, neighbor) in checks {
        if neighbor
            .and_then(|cell| state.map.cell(cell))
            .map(&include)
            .unwrap_or(false)
        {
            mask |= bit;
        }
    }
    mask
}

fn draw_visual_cell_faces(
    image: &mut RgbaImage,
    state: &MissionState,
    coord: CellCoord,
    cell: &MissionCell,
    projection: &MissionVisualProjection,
) {
    for neighbor in [
        CellCoord::new(coord.x + 1, coord.y),
        CellCoord::new(coord.x, coord.y + 1),
    ] {
        let neighbor_height = state
            .map
            .cell(neighbor)
            .map(|cell| cell.height)
            .unwrap_or(0);
        let delta = (cell.height - neighbor_height).max(0) as u32;
        if delta == 0 {
            continue;
        }
        let (cx, cy) = projection.center(coord, cell.height);
        let half_w = projection.tile_w as i32 / 2;
        let half_h = projection.tile_h as i32 / 2;
        let face_h = delta * projection.height_step as u32;
        let color = match cell.ground {
            GroundKind::Rock => Rgba([80, 82, 74, 255]),
            GroundKind::Road | GroundKind::Dirt | GroundKind::Mud => Rgba([94, 62, 38, 255]),
            GroundKind::Grass => Rgba([50, 76, 40, 255]),
        };
        let (x0, y0, x1, y1) = if neighbor.x > coord.x {
            (cx + half_w, cy, cx, cy + half_h)
        } else {
            (cx, cy + half_h, cx - half_w, cy)
        };
        for step in 0..face_h {
            draw_rgba_line(image, x0, y0 + step as i32, x1, y1 + step as i32, color, 1);
        }
    }
}

fn draw_diamond_sprite(
    image: &mut RgbaImage,
    sprite: &PixelImage,
    projection: &MissionVisualProjection,
    coord: CellCoord,
    height: i8,
) {
    let (x0, y0) = projection.top_left(coord, height);
    let tile_w = projection.tile_w as i32;
    let tile_h = projection.tile_h as i32;
    for dy in 0..tile_h {
        for dx in 0..tile_w {
            let nx = (dx as f32 + 0.5) / tile_w as f32 * 2.0 - 1.0;
            let ny = (dy as f32 + 0.5) / tile_h as f32 * 2.0 - 1.0;
            if nx.abs() + ny.abs() > 1.0 {
                continue;
            }
            let u = ((nx + ny + 1.0) * 0.5).clamp(0.0, 1.0);
            let v = ((ny - nx + 1.0) * 0.5).clamp(0.0, 1.0);
            let sx = (u * (sprite.width.saturating_sub(1)) as f32).round() as u32;
            let sy = (v * (sprite.height.saturating_sub(1)) as f32).round() as u32;
            let color = sprite.get(sx, sy);
            blend_pixel_rgba(
                image,
                x0 + dx,
                y0 + dy,
                Rgba([color.r, color.g, color.b, color.a]),
            );
        }
    }
}

fn draw_diamond_fill(
    image: &mut RgbaImage,
    projection: &MissionVisualProjection,
    coord: CellCoord,
    height: i8,
    color: Rgba<u8>,
) {
    let fallback = PixelImage::new(16, 16, Rgba8::new(color[0], color[1], color[2], color[3]));
    draw_diamond_sprite(image, &fallback, projection, coord, height);
}

fn draw_diamond_outline(
    image: &mut RgbaImage,
    projection: &MissionVisualProjection,
    coord: CellCoord,
    height: i8,
    color: Rgba<u8>,
) {
    let (cx, cy) = projection.center(coord, height);
    let hw = projection.tile_w as i32 / 2;
    let hh = projection.tile_h as i32 / 2;
    let points = [(cx, cy - hh), (cx + hw, cy), (cx, cy + hh), (cx - hw, cy)];
    for window in points.windows(2) {
        draw_rgba_line(
            image,
            window[0].0,
            window[0].1,
            window[1].0,
            window[1].1,
            color,
            1,
        );
    }
    draw_rgba_line(
        image,
        points[3].0,
        points[3].1,
        points[0].0,
        points[0].1,
        color,
        1,
    );
}

fn draw_visual_objects(
    image: &mut RgbaImage,
    state: &MissionState,
    projection: &MissionVisualProjection,
) {
    let mut objects = state.map.objects.iter().collect::<Vec<_>>();
    objects.sort_by_key(|object| (object.cell.x + object.cell.y, object.cell.y, object.cell.x));
    for object in objects {
        let height = state
            .map
            .cell(object.cell)
            .map(|cell| cell.height)
            .unwrap_or(0);
        let (cx, cy) = projection.center(object.cell, height);
        match &object.kind {
            EnvironmentObjectKind::Tree(TreeState::Standing)
            | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => {
                fill_rgba_rect(image, cx - 3, cy - 17, 6, 18, Rgba([82, 49, 30, 255]));
                fill_rgba_rect(image, cx - 11, cy - 27, 22, 16, Rgba([34, 88, 40, 245]));
                fill_rgba_rect(image, cx - 7, cy - 34, 14, 14, Rgba([44, 112, 52, 245]));
            }
            EnvironmentObjectKind::Tree(TreeState::Falling { .. })
            | EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Tree(TreeState::CutLogs)
            | EnvironmentObjectKind::Log(_) => {
                draw_rgba_line(
                    image,
                    cx - 15,
                    cy - 7,
                    cx + 15,
                    cy + 4,
                    Rgba([92, 55, 30, 255]),
                    5,
                );
                draw_rgba_line(
                    image,
                    cx - 15,
                    cy - 10,
                    cx + 15,
                    cy + 1,
                    Rgba([132, 83, 43, 255]),
                    2,
                );
            }
            EnvironmentObjectKind::Tree(TreeState::StakesBundle) => {
                for offset in [-9, -3, 3, 9] {
                    draw_rgba_line(
                        image,
                        cx + offset,
                        cy + 4,
                        cx + offset + 3,
                        cy - 11,
                        Rgba([214, 188, 122, 255]),
                        2,
                    );
                }
            }
            EnvironmentObjectKind::Tree(TreeState::Stump) => {
                fill_rgba_rect(image, cx - 5, cy - 10, 10, 8, Rgba([98, 62, 36, 255]));
                fill_rgba_rect(image, cx - 7, cy - 12, 14, 4, Rgba([132, 88, 48, 255]));
            }
            EnvironmentObjectKind::Stakes(_) => {
                for offset in [-8, 0, 8] {
                    draw_rgba_line(
                        image,
                        cx + offset,
                        cy + 4,
                        cx + offset + 3,
                        cy - 12,
                        Rgba([214, 188, 122, 255]),
                        2,
                    );
                }
            }
            EnvironmentObjectKind::Rock(_) => {
                fill_rgba_rect(image, cx - 8, cy - 9, 16, 10, Rgba([116, 120, 110, 255]));
                fill_rgba_rect(image, cx - 4, cy - 13, 10, 8, Rgba([145, 148, 132, 255]));
            }
            EnvironmentObjectKind::Wall(_) => {
                fill_rgba_rect(image, cx - 15, cy - 14, 30, 16, Rgba([100, 94, 78, 255]));
                fill_rgba_rect(image, cx - 13, cy - 16, 26, 4, Rgba([142, 136, 112, 255]));
            }
            EnvironmentObjectKind::Wire(_) => {
                draw_rgba_line(
                    image,
                    cx - 15,
                    cy - 3,
                    cx + 15,
                    cy + 3,
                    Rgba([142, 148, 150, 255]),
                    2,
                );
                draw_rgba_line(
                    image,
                    cx - 15,
                    cy + 3,
                    cx + 15,
                    cy - 3,
                    Rgba([142, 148, 150, 180]),
                    1,
                );
            }
            EnvironmentObjectKind::FightingPosition(_) => {
                fill_rgba_rect(image, cx - 10, cy - 8, 20, 10, Rgba([74, 52, 37, 255]));
                fill_rgba_rect(image, cx - 8, cy - 12, 16, 5, Rgba([112, 82, 48, 255]));
            }
        }
    }
}

fn draw_visual_mission_markers(
    image: &mut RgbaImage,
    state: &MissionState,
    projection: &MissionVisualProjection,
) {
    for spawn in &state.map.spawn_cells {
        let height = state.map.cell(*spawn).map(|cell| cell.height).unwrap_or(0);
        let (cx, cy) = projection.center(*spawn, height);
        fill_rgba_rect(image, cx - 6, cy - 21, 12, 12, Rgba([78, 130, 206, 235]));
    }
    let objective = state.spec.objective.defend_cell;
    let height = state
        .map
        .cell(objective)
        .map(|cell| cell.height)
        .unwrap_or(0);
    let (cx, cy) = projection.center(objective, height);
    fill_rgba_rect(image, cx - 8, cy - 25, 16, 16, Rgba([226, 202, 88, 245]));
    draw_preview_border_safe(image, cx - 8, cy - 25, 16, 16, Rgba([74, 62, 28, 255]));
}

fn draw_visual_route(
    image: &mut RgbaImage,
    state: &MissionState,
    projection: &MissionVisualProjection,
    route: &EnemyRoutePreview,
    color: Rgba<u8>,
) {
    for window in route.points.windows(2) {
        let a = visual_route_point(state, projection, window[0]);
        let b = visual_route_point(state, projection, window[1]);
        draw_rgba_line(image, a.0, a.1, b.0, b.1, color, 4);
    }
}

fn visual_route_point(
    state: &MissionState,
    projection: &MissionVisualProjection,
    cell: CellCoord,
) -> (i32, i32) {
    let height = state.map.cell(cell).map(|cell| cell.height).unwrap_or(0);
    let (cx, cy) = projection.center(cell, height);
    (cx, cy - 8)
}

fn sorted_strings(values: HashSet<String>) -> Vec<String> {
    let mut out = values.into_iter().collect::<Vec<_>>();
    out.sort();
    out
}

fn fill_rgba_rect(image: &mut RgbaImage, x: i32, y: i32, w: u32, h: u32, color: Rgba<u8>) {
    for dy in 0..h as i32 {
        for dx in 0..w as i32 {
            blend_pixel_rgba(image, x + dx, y + dy, color);
        }
    }
}

fn draw_preview_border_safe(
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    color: Rgba<u8>,
) {
    draw_rgba_line(image, x, y, x + w as i32, y, color, 1);
    draw_rgba_line(image, x, y + h as i32, x + w as i32, y + h as i32, color, 1);
    draw_rgba_line(image, x, y, x, y + h as i32, color, 1);
    draw_rgba_line(image, x + w as i32, y, x + w as i32, y + h as i32, color, 1);
}

fn draw_rgba_line(
    image: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: Rgba<u8>,
    thickness: u32,
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        let r = thickness as i32 / 2;
        fill_rgba_rect(image, x0 - r, y0 - r, thickness, thickness, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn blend_pixel_rgba(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 || color[3] == 0 {
        return;
    }
    let x = x as u32;
    let y = y as u32;
    let dst = *image.get_pixel(x, y);
    if color[3] == 255 {
        image.put_pixel(x, y, color);
        return;
    }
    let alpha = color[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    image.put_pixel(
        x,
        y,
        Rgba([
            (color[0] as f32 * alpha + dst[0] as f32 * inv).round() as u8,
            (color[1] as f32 * alpha + dst[1] as f32 * inv).round() as u8,
            (color[2] as f32 * alpha + dst[2] as f32 * inv).round() as u8,
            255,
        ]),
    );
}

pub fn save_mission_route_preview_png(
    path: impl AsRef<Path>,
    state: &MissionState,
    initial_routes: &DoctrineRouteSet,
    after_routes: &DoctrineRouteSet,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    for route in &initial_routes.routes {
        draw_route_path(&mut image, route, cell_px, Rgba([70, 130, 222, 150]), 3);
    }
    for route in &after_routes.routes {
        draw_route_path(&mut image, route, cell_px, Rgba([238, 196, 78, 205]), 5);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_mission_route_debug_png(
    path: impl AsRef<Path>,
    state: &MissionState,
    routes: &DoctrineRouteSet,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    let colors = [
        Rgba([242, 183, 62, 220]),
        Rgba([99, 205, 139, 220]),
        Rgba([220, 108, 92, 220]),
        Rgba([123, 171, 242, 220]),
        Rgba([205, 127, 226, 220]),
        Rgba([230, 230, 120, 220]),
    ];
    for (index, route) in routes.routes.iter().enumerate() {
        draw_route_path(&mut image, route, cell_px, colors[index % colors.len()], 4);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_mission_assault_preview_png(
    path: impl AsRef<Path>,
    state: &MissionState,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    draw_defender_positions(&mut image, &state.spec, cell_px);
    if let Some(assault) = &state.assault {
        for route in &assault.initial_routes.routes {
            draw_route_path(&mut image, route, cell_px, Rgba([90, 135, 210, 95]), 2);
        }
        draw_assault_agents(&mut image, assault, cell_px);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_assault_delay_heatmap_png(path: impl AsRef<Path>, state: &MissionState) -> Result<()> {
    save_assault_heatmap_png(
        path,
        state,
        |event| {
            matches!(
                event.kind,
                AssaultEventKind::DelayedByTerrain | AssaultEventKind::DelayedByObstacle
            )
        },
        Rgba([236, 174, 62, 185]),
    )
}

pub fn save_assault_pressure_heatmap_png(
    path: impl AsRef<Path>,
    state: &MissionState,
) -> Result<()> {
    save_assault_heatmap_png(
        path,
        state,
        |event| {
            matches!(
                event.kind,
                AssaultEventKind::SuppressedByDefender
                    | AssaultEventKind::DamagedByDefender
                    | AssaultEventKind::DamagedByObstacle
                    | AssaultEventKind::RollingHazardHitEnemy
            )
        },
        Rgba([218, 72, 82, 185]),
    )
}

fn save_assault_heatmap_png(
    path: impl AsRef<Path>,
    state: &MissionState,
    include: impl Fn(&AssaultTimelineEvent) -> bool,
    color: Rgba<u8>,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    if let Some(assault) = &state.assault {
        let mut cells: HashMap<CellCoord, i32> = HashMap::new();
        for event in assault
            .timeline
            .iter()
            .filter(|event| include(event))
            .filter_map(|event| event.cell.map(|cell| (cell, event.magnitude.max(1))))
        {
            *cells.entry(event.0).or_default() += event.1;
        }
        let max_value = cells.values().copied().max().unwrap_or(1).max(1) as f32;
        for (cell, value) in cells {
            let intensity = (value as f32 / max_value).clamp(0.2, 1.0);
            let alpha = (color[3] as f32 * intensity).round() as u8;
            blend_preview_rect(
                &mut image,
                cell.x * cell_px + 2,
                cell.y * cell_px + 2,
                cell_px - 4,
                cell_px - 4,
                Rgba([color[0], color[1], color[2], alpha]),
            );
        }
        draw_defender_positions(&mut image, &state.spec, cell_px);
        draw_assault_agents(&mut image, assault, cell_px);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_assault_prediction_vs_actual_png(
    path: impl AsRef<Path>,
    state: &MissionState,
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    if let Some(assault) = &state.assault {
        for route in &assault.initial_routes.routes {
            draw_route_path(&mut image, route, cell_px, Rgba([82, 130, 228, 120]), 2);
        }
        let actual_colors = [
            Rgba([238, 91, 71, 210]),
            Rgba([240, 184, 72, 210]),
            Rgba([210, 105, 230, 210]),
            Rgba([86, 214, 166, 210]),
        ];
        let mut drawn_groups = HashSet::new();
        for agent in &assault.agents {
            if drawn_groups.insert(agent.group_label.clone()) {
                let color = actual_colors[drawn_groups.len() % actual_colors.len()];
                draw_cell_path(&mut image, &actual_agent_path(agent), cell_px, color, 4);
            }
        }
        draw_defender_positions(&mut image, &state.spec, cell_px);
        draw_assault_agents(&mut image, assault, cell_px);
    }
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

pub fn save_rolling_hazard_preview_png(path: impl AsRef<Path>, state: &MissionState) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cell_px = 24;
    let mut image = mission_preview_image(state, cell_px);
    let hazards = state
        .assault
        .as_ref()
        .map(|assault| assault.rolling_hazards.clone())
        .unwrap_or_else(|| state.rolling_hazard_plans());
    for hazard in &hazards {
        let cells = hazard.path.iter().map(|step| step.cell).collect::<Vec<_>>();
        draw_cell_path(&mut image, &cells, cell_px, Rgba([236, 170, 64, 210]), 4);
        if let Some(first) = cells.first().copied() {
            let (cx, cy) = route_cell_center(first, cell_px);
            fill_preview_rect(
                &mut image,
                cx.saturating_sub(5),
                cy.saturating_sub(5),
                11,
                11,
                Rgba([118, 72, 38, 255]),
            );
        }
        if let Some(last) = cells.last().copied() {
            let (cx, cy) = route_cell_center(last, cell_px);
            blend_preview_rect(
                &mut image,
                cx.saturating_sub(6),
                cy.saturating_sub(6),
                13,
                13,
                Rgba([238, 82, 64, 210]),
            );
        }
    }
    draw_defender_positions(&mut image, &state.spec, cell_px);
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn mission_preview_image(state: &MissionState, cell_px: u32) -> RgbaImage {
    let mut image = RgbaImage::new(state.map.width * cell_px, state.map.height * cell_px);
    for y in 0..state.map.height {
        for x in 0..state.map.width {
            let coord = CellCoord::new(x, y);
            let cell = state
                .map
                .cell(coord)
                .expect("mission preview only reads in-bounds cells");
            let base = preview_cell_color(cell);
            fill_preview_rect(&mut image, x * cell_px, y * cell_px, cell_px, cell_px, base);
            draw_preview_border(
                &mut image,
                x * cell_px,
                y * cell_px,
                cell_px,
                cell_px,
                Rgba([32, 34, 28, 255]),
            );

            if state.spec.objective.defend_cell == coord {
                fill_preview_rect(
                    &mut image,
                    x * cell_px + 7,
                    y * cell_px + 7,
                    10,
                    10,
                    Rgba([226, 202, 88, 255]),
                );
            } else if state.map.spawn_cells.contains(&coord) {
                fill_preview_rect(
                    &mut image,
                    x * cell_px + 7,
                    y * cell_px + 7,
                    10,
                    10,
                    Rgba([78, 130, 206, 255]),
                );
            }

            if let Some(object) = state.map.objects.iter().find(|object| object.cell == coord) {
                let color = match object.kind {
                    EnvironmentObjectKind::Tree(TreeState::Standing)
                    | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => {
                        Rgba([35, 74, 34, 255])
                    }
                    EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
                    | EnvironmentObjectKind::Tree(TreeState::CutLogs)
                    | EnvironmentObjectKind::Log(_) => Rgba([92, 55, 30, 255]),
                    EnvironmentObjectKind::Stakes(_) => Rgba([214, 188, 122, 255]),
                    EnvironmentObjectKind::Rock(_) => Rgba([116, 120, 110, 255]),
                    EnvironmentObjectKind::Wall(_) => Rgba([100, 94, 78, 255]),
                    EnvironmentObjectKind::Wire(_) => Rgba([120, 126, 130, 255]),
                    EnvironmentObjectKind::FightingPosition(_) => Rgba([74, 52, 37, 255]),
                    _ => Rgba([82, 74, 52, 255]),
                };
                fill_preview_rect(&mut image, x * cell_px + 5, y * cell_px + 5, 14, 14, color);
            }
        }
    }

    image
}

fn draw_defender_positions(image: &mut RgbaImage, spec: &MissionSpec, cell_px: u32) {
    for defender in &spec.defender_positions {
        let (cx, cy) = route_cell_center(defender.cell, cell_px);
        fill_preview_rect(
            image,
            cx.saturating_sub(5),
            cy.saturating_sub(5),
            11,
            11,
            Rgba([90, 172, 226, 255]),
        );
    }
}

fn draw_assault_agents(image: &mut RgbaImage, assault: &AssaultState, cell_px: u32) {
    for agent in &assault.agents {
        let color = match agent.status {
            EnemyAgentStatus::Advancing => Rgba([218, 78, 62, 230]),
            EnemyAgentStatus::Delayed => Rgba([232, 150, 60, 230]),
            EnemyAgentStatus::Eliminated => Rgba([70, 72, 70, 210]),
            EnemyAgentStatus::ReachedObjective => Rgba([178, 70, 178, 230]),
        };
        let (cx, cy) = route_cell_center(agent.cell, cell_px);
        blend_preview_rect(
            image,
            cx.saturating_sub(4),
            cy.saturating_sub(4),
            9,
            9,
            color,
        );
    }
}

fn draw_route_path(
    image: &mut RgbaImage,
    route: &EnemyRoutePreview,
    cell_px: u32,
    color: Rgba<u8>,
    thickness: u32,
) {
    if route.points.is_empty() {
        return;
    }

    for window in route.points.windows(2) {
        let a = route_cell_center(window[0], cell_px);
        let b = route_cell_center(window[1], cell_px);
        draw_route_segment(image, a, b, color, thickness);
    }
    for point in &route.points {
        let (cx, cy) = route_cell_center(*point, cell_px);
        let radius = thickness + 1;
        blend_preview_rect(
            image,
            cx.saturating_sub(radius),
            cy.saturating_sub(radius),
            radius * 2 + 1,
            radius * 2 + 1,
            color,
        );
    }
}

fn draw_cell_path(
    image: &mut RgbaImage,
    points: &[CellCoord],
    cell_px: u32,
    color: Rgba<u8>,
    thickness: u32,
) {
    if points.is_empty() {
        return;
    }
    for window in points.windows(2) {
        let a = route_cell_center(window[0], cell_px);
        let b = route_cell_center(window[1], cell_px);
        draw_route_segment(image, a, b, color, thickness);
    }
    for point in points {
        let (cx, cy) = route_cell_center(*point, cell_px);
        let radius = thickness + 1;
        blend_preview_rect(
            image,
            cx.saturating_sub(radius),
            cy.saturating_sub(radius),
            radius * 2 + 1,
            radius * 2 + 1,
            color,
        );
    }
}

fn route_cell_center(cell: CellCoord, cell_px: u32) -> (u32, u32) {
    (
        cell.x * cell_px + cell_px / 2,
        cell.y * cell_px + cell_px / 2,
    )
}

fn draw_route_segment(
    image: &mut RgbaImage,
    a: (u32, u32),
    b: (u32, u32),
    color: Rgba<u8>,
    thickness: u32,
) {
    let half = thickness / 2 + 1;
    if a.1 == b.1 {
        let x0 = a.0.min(b.0).saturating_sub(half);
        let x1 = a.0.max(b.0) + half;
        blend_preview_rect(
            image,
            x0,
            a.1.saturating_sub(half),
            x1.saturating_sub(x0) + 1,
            half * 2 + 1,
            color,
        );
    } else if a.0 == b.0 {
        let y0 = a.1.min(b.1).saturating_sub(half);
        let y1 = a.1.max(b.1) + half;
        blend_preview_rect(
            image,
            a.0.saturating_sub(half),
            y0,
            half * 2 + 1,
            y1.saturating_sub(y0) + 1,
            color,
        );
    } else {
        blend_preview_rect(
            image,
            a.0.saturating_sub(half),
            a.1.saturating_sub(half),
            half * 2 + 1,
            half * 2 + 1,
            color,
        );
        blend_preview_rect(
            image,
            b.0.saturating_sub(half),
            b.1.saturating_sub(half),
            half * 2 + 1,
            half * 2 + 1,
            color,
        );
    }
}

fn preview_cell_color(cell: &MissionCell) -> Rgba<u8> {
    match cell.earth_state {
        EarthState::Trench | EarthState::DeepTrench => Rgba([45, 33, 26, 255]),
        EarthState::Berm | EarthState::SpoilPile => Rgba([132, 95, 54, 255]),
        EarthState::Scraped | EarthState::Ditch => Rgba([112, 88, 62, 255]),
        _ => match cell.ground {
            GroundKind::Grass => Rgba([76, 111, 58, 255]),
            GroundKind::Dirt => Rgba([132, 91, 57, 255]),
            GroundKind::Mud => Rgba([70, 62, 50, 255]),
            GroundKind::Rock => Rgba([105, 109, 100, 255]),
            GroundKind::Road => Rgba([156, 111, 68, 255]),
        },
    }
}

fn fill_preview_rect(
    image: &mut RgbaImage,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
) {
    for y in y0..(y0 + height).min(image.height()) {
        for x in x0..(x0 + width).min(image.width()) {
            image.put_pixel(x, y, color);
        }
    }
}

fn blend_preview_rect(
    image: &mut RgbaImage,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
) {
    let alpha = color[3] as f32 / 255.0;
    for y in y0..(y0 + height).min(image.height()) {
        for x in x0..(x0 + width).min(image.width()) {
            let dst = image.get_pixel(x, y);
            let blended = Rgba([
                (color[0] as f32 * alpha + dst[0] as f32 * (1.0 - alpha)).round() as u8,
                (color[1] as f32 * alpha + dst[1] as f32 * (1.0 - alpha)).round() as u8,
                (color[2] as f32 * alpha + dst[2] as f32 * (1.0 - alpha)).round() as u8,
                255,
            ]);
            image.put_pixel(x, y, blended);
        }
    }
}

fn draw_preview_border(
    image: &mut RgbaImage,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
) {
    if width == 0 || height == 0 {
        return;
    }
    let x1 = (x0 + width - 1).min(image.width().saturating_sub(1));
    let y1 = (y0 + height - 1).min(image.height().saturating_sub(1));
    for x in x0..=x1 {
        image.put_pixel(x, y0.min(image.height().saturating_sub(1)), color);
        image.put_pixel(x, y1, color);
    }
    for y in y0..=y1 {
        image.put_pixel(x0.min(image.width().saturating_sub(1)), y, color);
        image.put_pixel(x1, y, color);
    }
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))
}

fn write_ron(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let text = ron::ser::to_string_pretty(value, pretty)?;
    fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn road_below_seed_orders_change_terrain_and_objects() {
        let mut state = MissionState::road_below_seed();
        state.apply_seed_orders();

        assert!(state
            .work_orders
            .iter()
            .all(|order| matches!(order.status, WorkOrderStatus::Completed)));
        assert_eq!(
            state
                .map
                .cell(CellCoord::new(5, 4))
                .expect("trench cell")
                .earth_state,
            EarthState::Trench
        );
        assert_eq!(
            state
                .map
                .cell(CellCoord::new(5, 3))
                .expect("berm cell")
                .earth_state,
            EarthState::Berm
        );
        assert!(state.map.objects.iter().any(|object| matches!(
            object.kind,
            EnvironmentObjectKind::Stakes(ObstacleState::Placed)
        )));
    }

    #[test]
    fn queued_seed_plan_allows_order_dependencies() {
        let mut state = MissionState::road_below_seed();
        for (kind, target) in road_below_seed_orders() {
            state.queue_work_order(kind, target);
        }
        assert_eq!(state.work_queue.len(), 5);
        assert!(state.work_orders.is_empty());

        state.run_all_queued_orders();

        assert_eq!(state.work_queue.len(), 0);
        assert_eq!(state.work_orders.len(), 5);
        assert!(state.order_validation.is_empty());
        assert_eq!(state.material_totals().logs, 1);
        assert!(state.material_ledger.iter().any(|entry| {
            entry.order_kind == WorkOrderKind::RaiseBerm && entry.net.earth_spoil < 0
        }));
    }

    #[test]
    fn doctrine_route_preview_exports_changed_routes_after_prep() {
        let spec = road_below_spec();
        let initial = MissionState::from_spec(spec.clone());
        let after = run_work_order_script(spec.clone(), &road_below_basic_prep_script());

        let initial_routes = initial.route_preview();
        let after_routes = after.route_preview();
        let delta = route_delta_report(spec.id, &initial_routes, &after_routes);

        assert_eq!(initial_routes.routes.len(), 3);
        assert_eq!(after_routes.routes.len(), 3);
        assert!(after_routes.routes.iter().all(|route| route.reached_goal));
        assert!(after_routes
            .routes
            .iter()
            .all(|route| !route.explanation.is_empty()));
        assert_eq!(delta.groups.len(), 3);
        assert!(delta.groups.iter().any(|group| group.changed));
    }

    #[test]
    fn assault_sandbox_runs_to_deterministic_summary() {
        let spec = road_below_spec();
        let mut state = run_work_order_script(spec.clone(), &road_below_basic_prep_script());

        state.start_assault();
        assert!(matches!(state.phase, MissionPhase::Assault));
        assert_eq!(
            state.assault.as_ref().expect("assault").agents.len() as u32,
            spec.enemy_groups
                .iter()
                .map(|group| group.count)
                .sum::<u32>()
        );

        let summary = state.run_assault_to_completion(160);
        assert!(matches!(state.phase, MissionPhase::Debrief));
        assert!(summary.ticks_elapsed > 0);
        assert_eq!(summary.enemies_spawned, 26);
        assert!(
            summary.enemies_eliminated + summary.enemies_reached_objective
                <= summary.enemies_spawned
        );
        assert!(state
            .assault
            .as_ref()
            .expect("assault")
            .timeline
            .iter()
            .any(|event| matches!(event.kind, AssaultEventKind::AssaultEnded)));
        assert!(state
            .assault
            .as_ref()
            .expect("assault")
            .timeline
            .iter()
            .any(|event| {
                matches!(event.kind, AssaultEventKind::DamagedByDefender)
                    && event.cause == AssaultEventCause::Defender
                    && event.magnitude > 0
            }));

        let debrief = state.assault_debrief().expect("debrief");
        assert!(!debrief.influence.most_crossed_cells.is_empty());
        assert!(!debrief.influence.most_damaging_cells.is_empty());
        assert_eq!(debrief.route_prediction_accuracy.total_divergence_cells, 0);
        assert!(debrief.route_prediction_accuracy.average_accuracy > 0.99);
    }

    #[test]
    fn rolling_hazard_prepares_releases_and_reports_impacts() {
        let spec = road_below_spec();
        let mut state = run_work_order_script(spec, &road_below_hazard_prep_script());
        let hazards = state.rolling_hazard_plans();
        assert_eq!(hazards.len(), 1);
        assert!(hazards[0].path.len() > 2);

        let summary = state.run_assault_to_completion(160);
        assert!(summary.ticks_elapsed > 0);
        let debrief = state.assault_debrief().expect("debrief");
        assert_eq!(debrief.rolling_hazards.prepared_count, 1);
        assert_eq!(debrief.rolling_hazards.released_count, 1);
        assert_eq!(debrief.rolling_hazards.spent_count, 1);
        assert!(debrief.rolling_hazards.enemies_hit > 0);
        assert!(state
            .assault
            .as_ref()
            .expect("assault")
            .timeline
            .iter()
            .any(|event| matches!(event.kind, AssaultEventKind::RollingHazardReleased)));
    }

    #[test]
    fn balance_scripts_create_meaningful_rating_spread() {
        let spec = road_below_spec();

        let mut baseline = run_work_order_script(spec.clone(), &road_below_no_prep_script());
        baseline.run_assault_to_completion(160);
        let baseline_rating = mission_rating_for_state(&baseline).expect("baseline rating");

        let mut chokepoint =
            run_work_order_script(spec.clone(), &road_below_ridge_chokepoint_script());
        chokepoint.run_assault_to_completion(160);
        let chokepoint_rating = mission_rating_for_state(&chokepoint).expect("chokepoint rating");

        let mut bad = run_work_order_script(spec, &road_below_overbuilt_bad_plan_script());
        bad.run_assault_to_completion(160);
        let bad_rating = mission_rating_for_state(&bad).expect("bad plan rating");

        assert!(chokepoint_rating.score > baseline_rating.score);
        assert_eq!(chokepoint_rating.stars, 3);
        assert!(bad_rating.stars <= baseline_rating.stars);
    }

    #[test]
    fn procgen_road_below_candidate_has_tactical_affordances() {
        let generator = MissionGeneratorSpec::road_below(99_418_113);
        let candidate = generate_mission_candidate(&generator, generator.seed);

        assert_eq!(candidate.spec.map.width, 12);
        assert_eq!(candidate.spec.map.height, 8);
        assert!(candidate.affordance_report.road_cell_count >= 8);
        assert!(candidate.affordance_report.ridge_cell_count >= 6);
        assert!(candidate.affordance_report.tree_count >= 3);
        assert_eq!(candidate.affordance_report.loose_log_count, 1);
        assert!(
            candidate
                .affordance_report
                .rolling_hazard_route_intersections
                > 0
        );

        let state = MissionState::from_spec(candidate.spec);
        let routes = state.route_preview();
        assert_eq!(routes.routes.len(), 3);
        assert!(routes.routes.iter().all(|route| route.reached_goal));
    }

    #[test]
    fn procgen_theme_classes_generate_routeable_candidates() {
        for theme in MissionTheme::GENERATABLE {
            let mut generator = MissionGeneratorSpec::road_below(99_418_113);
            generator.theme = theme;
            let candidate = generate_mission_candidate(&generator, generator.seed);
            let routes = MissionState::from_spec(candidate.spec.clone()).route_preview();

            assert_eq!(candidate.theme, theme);
            assert_eq!(routes.routes.len(), 3, "theme {theme:?}");
            assert!(
                routes.routes.iter().all(|route| route.reached_goal),
                "theme {theme:?}"
            );
            assert!(
                candidate.affordance_report.tree_count >= 3,
                "theme {theme:?}"
            );
            if theme_requires_rolling_hazard(theme) {
                assert!(
                    candidate
                        .affordance_report
                        .rolling_hazard_route_intersections
                        > 0,
                    "theme {theme:?}"
                );
            }
        }
    }
}
