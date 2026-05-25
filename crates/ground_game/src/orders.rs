use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::hazards::{is_preparable_rolling_log, predict_rolling_log_path, rolling_log_direction};
use crate::{
    CellCoord, CellRect, CoverClass, Direction, EarthState, EnvironmentObject,
    EnvironmentObjectKind, GroundKind, LocalMaterialKind, LocalMaterialStock, LogState, MissionMap,
    MissionSpec, MissionState, ObstacleState, ToolKind, TreeState,
};

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

pub fn run_work_order_script(spec: MissionSpec, script: &WorkOrderScript) -> MissionState {
    let mut state = MissionState::from_spec(spec);
    for scripted in &script.orders {
        state.queue_work_order(scripted.kind, scripted.target.clone());
        state.run_next_queued_order();
    }
    state
}

pub(crate) fn build_work_order(
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

pub(crate) fn order_with_status(mut order: WorkOrder, status: WorkOrderStatus) -> WorkOrder {
    order.status = status;
    order
}

pub(crate) fn material_requirements(stock: &LocalMaterialStock) -> [(LocalMaterialKind, i32); 7] {
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

pub(crate) fn available_nearby_material(
    map: &MissionMap,
    origin: CellCoord,
    kind: LocalMaterialKind,
) -> i32 {
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

pub(crate) fn validate_order_target(
    map: &MissionMap,
    order: &WorkOrder,
) -> std::result::Result<(), String> {
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

pub(crate) fn apply_order_effects(map: &mut MissionMap, order: &mut WorkOrder) -> Result<()> {
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
