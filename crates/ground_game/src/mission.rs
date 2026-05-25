use ground_core::DEFAULT_SPRITE_STYLE_PATH;
use serde::{Deserialize, Serialize};

use crate::assault::{
    assault_event, build_assault_debrief, enemy_hp_for_doctrine, step_agent_assault,
    summarize_assault,
};
use crate::fixtures::{road_below_seed_orders, road_below_spec};
use crate::hazards::{planned_rolling_hazards_for_map, process_rolling_hazards};
use crate::orders::{
    apply_order_effects, available_nearby_material, build_work_order, material_requirements,
    order_with_status, validate_order_target,
};
use crate::routing::route_preview_for_state;
use crate::{
    AssaultDebrief, AssaultEventCause, AssaultEventKind, AssaultState, AssaultStatus,
    AssaultSummary, AssaultTimelineEvent, CrewPool, DoctrineRouteSet, EnemyAgent, EnemyAgentStatus,
    EnemyGroupSpec, EnvironmentObject, EnvironmentObjectKind, LocalMaterialStock,
    MaterialLedgerEntry, MissionConstraints, MissionPhase, OrderValidationEntry,
    OrderValidationSeverity, RollingHazardState, RollingHazardStatus, ToolLoadout, TreeState,
    WorkOrder, WorkOrderKind, WorkOrderStatus, WorkTarget,
};

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
