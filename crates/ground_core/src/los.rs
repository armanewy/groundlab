use serde::{Deserialize, Serialize};

use crate::terrain::TerrainMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Visible,
    Blocked,
}

#[derive(Clone, Debug)]
pub struct VisibilityGrid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Visibility>,
}

impl VisibilityGrid {
    pub fn get(&self, x: u32, y: u32) -> Visibility {
        if x < self.width && y < self.height {
            self.cells[y as usize * self.width as usize + x as usize]
        } else {
            Visibility::Blocked
        }
    }
}

pub fn visibility_grid(map: &TerrainMap, source: (u32, u32), max_range: u32) -> VisibilityGrid {
    let mut cells = vec![Visibility::Blocked; map.width as usize * map.height as usize];
    for y in 0..map.height {
        for x in 0..map.width {
            let dist = source.0.abs_diff(x).max(source.1.abs_diff(y));
            let vis = if dist <= max_range && line_of_sight(map, source, (x, y), 1.4, 0.8) {
                Visibility::Visible
            } else {
                Visibility::Blocked
            };
            cells[y as usize * map.width as usize + x as usize] = vis;
        }
    }
    VisibilityGrid {
        width: map.width,
        height: map.height,
        cells,
    }
}

pub fn line_of_sight(
    map: &TerrainMap,
    source: (u32, u32),
    target: (u32, u32),
    source_eye_height: f32,
    target_eye_height: f32,
) -> bool {
    if source == target {
        return true;
    }

    let Some(src_cell) = map.cell(source.0, source.1) else {
        return false;
    };
    let Some(dst_cell) = map.cell(target.0, target.1) else {
        return false;
    };
    let source_h = src_cell.effective_height() + source_eye_height;
    let target_h = dst_cell.effective_height() + target_eye_height;

    let points = bresenham(
        source.0 as i32,
        source.1 as i32,
        target.0 as i32,
        target.1 as i32,
    );
    let denom = (points.len().saturating_sub(1)).max(1) as f32;

    for (i, (x, y)) in points
        .iter()
        .enumerate()
        .skip(1)
        .take(points.len().saturating_sub(2))
    {
        if *x < 0 || *y < 0 || *x >= map.width as i32 || *y >= map.height as i32 {
            return false;
        }
        let t = i as f32 / denom;
        let expected = source_h + (target_h - source_h) * t;
        let Some(cell) = map.cell(*x as u32, *y as u32) else {
            return false;
        };
        let blocker_height = cell.effective_height() + if cell.blocks_sight { 1.1 } else { 0.15 };
        if blocker_height > expected + 0.25 {
            return false;
        }
    }
    true
}

pub fn bresenham(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push((x0, y0));
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
    points
}
