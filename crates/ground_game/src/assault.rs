use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::rating::rate_mission_outcome;
use crate::{
    CellCoord, DoctrineRouteSet, EarthState, EnemyDoctrine, EnvironmentObject,
    EnvironmentObjectKind, MissionMap, MissionRating, MissionSpec, MissionState, ObstacleState,
    RollingHazardImpactSummary, RollingHazardState, RollingHazardStatus, TreeState,
};

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

pub(crate) fn assault_event(
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

pub(crate) fn step_agent_assault(
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

pub(crate) fn enemy_hp_for_doctrine(doctrine: EnemyDoctrine) -> i32 {
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

pub(crate) fn summarize_assault(spec: &MissionSpec, assault: &AssaultState) -> AssaultSummary {
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

pub(crate) fn build_assault_debrief(state: &MissionState) -> Option<AssaultDebrief> {
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

pub(crate) fn build_assault_influence(
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

pub(crate) fn build_rolling_hazard_summary(
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

pub(crate) fn actual_agent_path(agent: &EnemyAgent) -> Vec<CellCoord> {
    if agent.route.is_empty() {
        return vec![agent.cell];
    }
    let end = agent.route_index.min(agent.route.len().saturating_sub(1));
    agent.route.iter().take(end + 1).copied().collect()
}
