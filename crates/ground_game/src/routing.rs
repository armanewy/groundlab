use std::cmp::Ordering;
use std::collections::BinaryHeap;

use serde::{Deserialize, Serialize};

use crate::{
    CellCoord, CoverClass, EarthState, EnemyDoctrine, EnemyGroupSpec, EnvironmentObjectKind,
    GroundKind, MissionMap, MissionState,
};

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
