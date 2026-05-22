use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::terrain::TerrainMap;

#[derive(Clone, Debug)]
pub struct PathResult {
    pub points: Vec<(u32, u32)>,
    pub total_cost: f32,
    pub reached_goal: bool,
}

#[derive(Clone, Copy, Debug)]
struct QueueNode {
    idx: usize,
    f_score: f32,
}

impl PartialEq for QueueNode {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.f_score.to_bits() == other.f_score.to_bits()
    }
}

impl Eq for QueueNode {}

impl PartialOrd for QueueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .total_cmp(&self.f_score)
            .then_with(|| self.idx.cmp(&other.idx))
    }
}

pub fn find_path(map: &TerrainMap, start: (u32, u32), goal: (u32, u32)) -> PathResult {
    let Some(start_idx) = map.index(start.0, start.1) else {
        return PathResult {
            points: Vec::new(),
            total_cost: 0.0,
            reached_goal: false,
        };
    };
    let Some(goal_idx) = map.index(goal.0, goal.1) else {
        return PathResult {
            points: Vec::new(),
            total_cost: 0.0,
            reached_goal: false,
        };
    };

    let len = map.cells.len();
    let mut open = BinaryHeap::new();
    let mut came_from: Vec<Option<usize>> = vec![None; len];
    let mut g_score = vec![f32::INFINITY; len];
    let mut best_idx = start_idx;
    let mut best_h = heuristic(start, goal);

    g_score[start_idx] = 0.0;
    open.push(QueueNode {
        idx: start_idx,
        f_score: best_h,
    });

    while let Some(node) = open.pop() {
        let pos = idx_to_pos(map, node.idx);
        let h = heuristic(pos, goal);
        if h < best_h {
            best_h = h;
            best_idx = node.idx;
        }
        if node.idx == goal_idx {
            let points = reconstruct(map, &came_from, goal_idx);
            return PathResult {
                points,
                total_cost: g_score[goal_idx],
                reached_goal: true,
            };
        }

        for neighbor in map.neighbors4(pos.0, pos.1) {
            let Some(n_idx) = map.index(neighbor.0, neighbor.1) else {
                continue;
            };
            let Some(current_cell) = map.cell(pos.0, pos.1) else {
                continue;
            };
            let Some(next_cell) = map.cell(neighbor.0, neighbor.1) else {
                continue;
            };
            let height_delta =
                (current_cell.effective_height() - next_cell.effective_height()).abs();
            if height_delta > 5.0 {
                continue;
            }
            let tentative = g_score[node.idx]
                + map.movement_cost_at(neighbor.0, neighbor.1)
                + height_delta * 0.35;
            if tentative < g_score[n_idx] {
                came_from[n_idx] = Some(node.idx);
                g_score[n_idx] = tentative;
                open.push(QueueNode {
                    idx: n_idx,
                    f_score: tentative + heuristic(neighbor, goal),
                });
            }
        }
    }

    PathResult {
        points: reconstruct(map, &came_from, best_idx),
        total_cost: g_score[best_idx],
        reached_goal: false,
    }
}

fn heuristic(a: (u32, u32), b: (u32, u32)) -> f32 {
    let dx = a.0.abs_diff(b.0) as f32;
    let dy = a.1.abs_diff(b.1) as f32;
    dx + dy
}

fn idx_to_pos(map: &TerrainMap, idx: usize) -> (u32, u32) {
    let x = idx as u32 % map.width;
    let y = idx as u32 / map.width;
    (x, y)
}

fn reconstruct(map: &TerrainMap, came_from: &[Option<usize>], mut idx: usize) -> Vec<(u32, u32)> {
    let mut out = vec![idx_to_pos(map, idx)];
    while let Some(prev) = came_from[idx] {
        idx = prev;
        out.push(idx_to_pos(map, idx));
    }
    out.reverse();
    out
}
