use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::assault::{assault_event, AssaultEventCause, AssaultEventKind};
use crate::{
    AssaultTimelineEvent, CellCoord, CellInfluence, Direction, EarthState, EnemyAgent,
    EnemyAgentStatus, EnvironmentObjectKind, LogState, MissionMap, MissionSpec, ObstacleState,
    TreeState,
};

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

pub(crate) fn process_rolling_hazards(
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

pub(crate) fn planned_rolling_hazards_for_map(
    map: &MissionMap,
    release_tick: u32,
) -> Vec<RollingHazardState> {
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

pub(crate) fn rolling_log_direction(kind: &EnvironmentObjectKind) -> Option<Direction> {
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

pub(crate) fn is_preparable_rolling_log(kind: &EnvironmentObjectKind) -> bool {
    matches!(
        kind,
        EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Log(LogState::Loose { .. })
            | EnvironmentObjectKind::Log(LogState::DragPrepared { .. })
            | EnvironmentObjectKind::Log(LogState::Positioned { .. })
            | EnvironmentObjectKind::Log(LogState::Braced { .. })
    )
}
