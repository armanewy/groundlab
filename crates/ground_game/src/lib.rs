use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MISSION_EXPORT_DIR: &str = "exports/gamepivot_01";

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
    pub objective: MissionObjective,
    pub prep_time_seconds: u32,
    pub map: MissionMap,
    pub starting_tools: ToolLoadout,
    pub crew: CrewPool,
    pub enemy_groups: Vec<EnemyGroupSpec>,
    pub constraints: MissionConstraints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionObjective {
    pub label: String,
    pub defend_cell: CellCoord,
    pub objective_health: u32,
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
    Loose { direction: Direction },
    DragPrepared { direction: Direction },
    Rolling { direction: Direction },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct MissionState {
    pub spec: MissionSpec,
    pub map: MissionMap,
    pub remaining_prep_seconds: u32,
    pub remaining_labor_seconds: u32,
    pub work_orders: Vec<WorkOrder>,
    pub event_log: Vec<String>,
}

impl MissionState {
    pub fn from_spec(spec: MissionSpec) -> Self {
        Self {
            remaining_prep_seconds: spec.prep_time_seconds,
            remaining_labor_seconds: spec.crew.labor_seconds_available,
            map: spec.map.clone(),
            spec,
            work_orders: Vec::new(),
            event_log: Vec::new(),
        }
    }

    pub fn road_below_seed() -> Self {
        Self::from_spec(road_below_spec())
    }

    pub fn apply_work_order(&mut self, kind: WorkOrderKind, target: WorkTarget) -> WorkOrder {
        let id = self.work_orders.len() as u32 + 1;
        let mut order = build_work_order(id, kind, target, &self.map);
        if !self.spec.starting_tools.has_all(&order.required_tools) {
            order.status = WorkOrderStatus::Rejected {
                reason: format!(
                    "missing required tools: {}",
                    order
                        .required_tools
                        .iter()
                        .filter(|tool| !self.spec.starting_tools.has(**tool))
                        .map(|tool| tool.label())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
        } else if order.labor_seconds > self.remaining_labor_seconds
            || order.labor_seconds > self.remaining_prep_seconds
        {
            order.status = WorkOrderStatus::Rejected {
                reason: "not enough prep time or crew labor".to_string(),
            };
        } else {
            match apply_order_effects(&mut self.map, &mut order) {
                Ok(()) => {
                    self.remaining_prep_seconds = self
                        .remaining_prep_seconds
                        .saturating_sub(order.labor_seconds);
                    self.remaining_labor_seconds = self
                        .remaining_labor_seconds
                        .saturating_sub(order.labor_seconds);
                    order.progress_seconds = order.labor_seconds;
                    order.status = WorkOrderStatus::Completed;
                }
                Err(err) => {
                    order.status = WorkOrderStatus::Rejected {
                        reason: err.to_string(),
                    };
                }
            }
        }

        self.event_log
            .push(format!("{}: {}", order.kind.label(), order.status.label()));
        self.work_orders.push(order.clone());
        order
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: u32,
    pub kind: WorkOrderKind,
    pub target: WorkTarget,
    pub required_tools: Vec<ToolKind>,
    pub labor_seconds: u32,
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
    InProgress,
    Completed,
    Rejected { reason: String },
}

impl WorkOrderStatus {
    pub fn label(&self) -> String {
        match self {
            WorkOrderStatus::Planned => "planned".to_string(),
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

pub fn road_below_spec() -> MissionSpec {
    let mut map = MissionMap::new(12, 8, MissionCell::new(1, GroundKind::Grass));
    map.spawn_cells.push(CellCoord::new(1, 7));

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

    MissionSpec {
        id: "road_below".to_string(),
        title: "The Road Below".to_string(),
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
        enemy_groups: vec![EnemyGroupSpec {
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
        }],
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

pub fn export_road_below_seed(out_dir: impl AsRef<Path>) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let spec = road_below_spec();
    let before = MissionState::from_spec(spec.clone());
    let mut after = MissionState::from_spec(spec.clone());
    after.apply_seed_orders();

    write_json(out_dir.join("mission_spec.json"), &spec)?;
    write_ron(out_dir.join("mission_spec.ron"), &spec)?;
    write_json(out_dir.join("mission_before.json"), &before)?;
    write_json(
        out_dir.join("scripted_work_orders.json"),
        &after.work_orders,
    )?;
    write_json(out_dir.join("mission_after.json"), &after)?;
    fs::write(out_dir.join("mission_before_map.txt"), ascii_map(&before))?;
    fs::write(out_dir.join("mission_after_map.txt"), ascii_map(&after))?;
    fs::write(out_dir.join("mission_summary.txt"), mission_summary(&after))?;
    Ok(())
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

pub fn save_mission_spec(path: impl AsRef<Path>, spec: &MissionSpec) -> Result<()> {
    let path = path.as_ref();
    if matches!(path.extension().and_then(|ext| ext.to_str()), Some("json")) {
        write_json(path, spec)
    } else {
        write_ron(path, spec)
    }
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
    let (required_tools, labor_seconds, material_inputs, material_outputs, notes) = match kind {
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
    };

    WorkOrder {
        id,
        kind,
        target,
        required_tools,
        labor_seconds,
        material_inputs,
        material_outputs: material_outputs.clone(),
        progress_seconds: 0,
        status: WorkOrderStatus::Planned,
        preview: WorkOrderPreview {
            affected_cells,
            affected_objects: Vec::new(),
            material_delta: material_outputs,
            notes,
        },
    }
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
            if consume_nearby_material(map, cell_coord, LocalMaterialKind::Stakes, 1).is_err() {
                consume_nearby_material(map, cell_coord, LocalMaterialKind::Logs, 1)
                    .context("not enough nearby stakes or logs to place stakes")?;
            }
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
            "- #{:02} {} · {} · {}s\n",
            order.id,
            order.kind.label(),
            order.status.label(),
            order.labor_seconds
        ));
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
}
