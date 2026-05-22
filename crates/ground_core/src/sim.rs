use serde::{Deserialize, Serialize};

use crate::terrain::TerrainMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RollingHazard {
    pub cell: (u32, u32),
    pub velocity: (f32, f32),
    pub mass: f32,
    pub damage: f32,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingTrace {
    pub points: Vec<(u32, u32)>,
    pub stopped_reason: String,
}

impl RollingHazard {
    pub fn log_at(cell: (u32, u32)) -> Self {
        Self {
            cell,
            velocity: (0.0, 0.0),
            mass: 8.0,
            damage: 10.0,
            active: true,
        }
    }
}

pub fn simulate_simple_roll(map: &TerrainMap, start: (u32, u32), max_steps: usize) -> RollingTrace {
    let mut current = start;
    let mut points = vec![current];

    for _ in 0..max_steps {
        let Some(next) = steepest_downhill_neighbor(map, current) else {
            return RollingTrace {
                points,
                stopped_reason: "no downhill neighbor".to_string(),
            };
        };
        if next == current {
            return RollingTrace {
                points,
                stopped_reason: "flat or blocked".to_string(),
            };
        }
        current = next;
        points.push(current);
    }

    RollingTrace {
        points,
        stopped_reason: "max steps reached".to_string(),
    }
}

fn steepest_downhill_neighbor(map: &TerrainMap, current: (u32, u32)) -> Option<(u32, u32)> {
    let current_h = map.cell(current.0, current.1)?.effective_height();
    let mut best = current;
    let mut best_drop = 0.0;
    for n in map.neighbors4(current.0, current.1) {
        let Some(cell) = map.cell(n.0, n.1) else {
            continue;
        };
        let drop = current_h - cell.effective_height();
        if drop > best_drop {
            best_drop = drop;
            best = n;
        }
    }
    Some(best)
}
