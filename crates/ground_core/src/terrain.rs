use serde::{Deserialize, Serialize};

use crate::recipe::GroundMaterial;
use crate::tileset::hash01;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverKind {
    None,
    Partial,
    Strong,
}

impl CoverKind {
    pub fn label(self) -> &'static str {
        match self {
            CoverKind::None => "none",
            CoverKind::Partial => "partial",
            CoverKind::Strong => "strong",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainCell {
    pub height: i8,
    pub ground: GroundMaterial,
    pub trench_depth: u8,
    pub berm_height: u8,
    pub cover: CoverKind,
    pub blocks_sight: bool,
}

impl TerrainCell {
    pub fn new(height: i8, ground: GroundMaterial) -> Self {
        Self {
            height,
            ground,
            trench_depth: 0,
            berm_height: 0,
            cover: CoverKind::None,
            blocks_sight: false,
        }
    }

    pub fn effective_height(&self) -> f32 {
        self.height as f32 + self.berm_height as f32 * 0.65 - self.trench_depth as f32 * 0.35
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainMap {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<TerrainCell>,
    pub objective: (u32, u32),
    pub spawn: (u32, u32),
}

impl TerrainMap {
    pub fn new(width: u32, height: u32, fill: TerrainCell) -> Self {
        let cells = vec![fill; width as usize * height as usize];
        Self {
            width,
            height,
            cells,
            objective: (width.saturating_sub(4), height / 2),
            spawn: (2, height / 2),
        }
    }

    /// Default preview map used by the workbench. This is intentionally art-directed:
    /// broad readable material regions, coherent ledges, and obvious trench/berm test
    /// features. Use `stress_test` when the renderer needs noisy edge-case coverage.
    pub fn demo(width: u32, height: u32, seed: u64) -> Self {
        Self::visual_target(width, height, seed)
    }

    /// Small hand-composed target scene used to judge the actual art direction.
    /// Unlike `stress_test` and `art_preview`, this is intentionally not a noisy
    /// generator. It creates a compact outpost/approach composition with broad
    /// floor regions, a visible road, a raised defended pad, a trench, a berm,
    /// a mud basin, and a rock outcrop. The simulation grid remains underneath,
    /// but the visual renderer can derive larger scene forms from it.
    pub fn visual_target(width: u32, height: u32, _seed: u64) -> Self {
        let width = width.max(16);
        let height = height.max(12);
        let mut map = TerrainMap::new(width, height, TerrainCell::new(2, GroundMaterial::Grass));
        map.spawn = (1, height.saturating_sub(3));
        map.objective = (width.saturating_sub(4), 4);

        // Small hero scene: three clear shelves, with the debug grid hidden by default.
        for y in 0..height {
            for x in 0..width {
                let h = if y <= 2 {
                    4
                } else if y <= 6 {
                    3
                } else if y >= height.saturating_sub(3) {
                    1
                } else {
                    2
                };
                set_map_cell(&mut map, x, y, h, GroundMaterial::Grass);
            }
        }

        // Worn approach road from the lower-left spawn toward the raised pad.
        fill_rect_cells(
            &mut map,
            0,
            height.saturating_sub(4),
            5,
            3,
            1,
            GroundMaterial::Dirt,
        );
        fill_rect_cells(
            &mut map,
            4,
            height.saturating_sub(5),
            4,
            2,
            2,
            GroundMaterial::Dirt,
        );
        fill_rect_cells(&mut map, 7, height / 2 + 1, 4, 2, 2, GroundMaterial::Dirt);
        fill_rect_cells(
            &mut map,
            width.saturating_sub(7),
            height / 2,
            6,
            2,
            3,
            GroundMaterial::Dirt,
        );

        // Raised objective platform with stone center and dirt shoulder.
        fill_rect_cells(
            &mut map,
            width.saturating_sub(7),
            2,
            6,
            4,
            4,
            GroundMaterial::Dirt,
        );
        fill_rect_cells(
            &mut map,
            width.saturating_sub(6),
            3,
            4,
            2,
            5,
            GroundMaterial::Rock,
        );

        // One readable trench, one berm, one mud basin, and one small stone outcrop.
        fill_trench_cells(&mut map, 4, height / 2 - 1, 6, 1, 1);
        fill_trench_cells(&mut map, 9, height / 2 - 1, 1, 3, 1);
        fill_berm_cells(&mut map, 7, height.saturating_sub(3), 6, 1, 1);
        fill_berm_cells(&mut map, width.saturating_sub(8), 5, 1, 3, 1);
        fill_rect_cells(
            &mut map,
            5,
            height.saturating_sub(4),
            4,
            2,
            1,
            GroundMaterial::Mud,
        );
        fill_rect_cells(&mut map, 2, 1, 3, 2, 4, GroundMaterial::Rock);

        map
    }

    /// Stress-test map kept for validation/debug exports. It deliberately contains many
    /// isolated material and height changes, so it should not be used as the default art read.
    pub fn stress_test(width: u32, height: u32, seed: u64) -> Self {
        let mut map = TerrainMap::new(width, height, TerrainCell::new(2, GroundMaterial::Grass));
        map.objective = (width.saturating_sub(5), height / 2);
        map.spawn = (2, height / 2 + 2);

        let hill_center = (width as f32 * 0.68, height as f32 * 0.45);
        let valley_center_y = height as f32 * 0.62;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - hill_center.0;
                let dy = y as f32 - hill_center.1;
                let hill =
                    4.5 * (1.0 - ((dx * dx + dy * dy).sqrt() / (width as f32 * 0.50))).max(0.0);
                let valley = -1.8
                    * (1.0 - ((y as f32 - valley_center_y).abs() / (height as f32 * 0.18)))
                        .max(0.0);
                let west_slope = (x as f32 / width.max(1) as f32) * 2.2;
                let noise = (hash01(seed, x, y, 777) - 0.5) * 1.3;
                let h = (2.0 + hill + valley + west_slope + noise)
                    .round()
                    .clamp(0.0, 8.0) as i8;

                let ground = if (y as i32 - map.spawn.1 as i32).abs() <= 1 && x < width / 2 {
                    GroundMaterial::Dirt
                } else if valley < -0.75 && hash01(seed, x, y, 33) > 0.35 {
                    GroundMaterial::Mud
                } else if h >= 6 && hash01(seed, x, y, 9) > 0.72 {
                    GroundMaterial::Rock
                } else if hash01(seed, x, y, 18) > 0.88 {
                    GroundMaterial::Dirt
                } else {
                    GroundMaterial::Grass
                };
                let idx = map
                    .index(x, y)
                    .expect("stress-test coordinates are in bounds");
                map.cells[idx] = TerrainCell::new(h, ground);
            }
        }

        map.apply_brush(
            map.objective.0,
            map.objective.1,
            Brush::new(BrushKind::Flatten, 3, 1),
        );
        map.apply_brush(
            map.spawn.0,
            map.spawn.1,
            Brush::new(BrushKind::Paint(GroundMaterial::Dirt), 2, 1),
        );
        map
    }

    /// Art-directed preview map. It has large regions, a clear approach road, coherent
    /// elevation shelves, a mud basin, a rock outcrop, and explicit trench/berm runs so
    /// the faux-perspective renderer can be judged visually instead of as a tile stress test.
    pub fn art_preview(width: u32, height: u32, seed: u64) -> Self {
        let mut map = TerrainMap::new(width, height, TerrainCell::new(2, GroundMaterial::Grass));
        map.objective = (width.saturating_sub(5), (height / 2).saturating_sub(2));
        map.spawn = (2, height / 2 + 3);

        let w = width.max(1) as f32;
        let hgt = height.max(1) as f32;
        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / w;
                let fy = y as f32 / hgt;
                let mut height_value = 2.0;

                // Broad readable shelves: high rear ground, mid plateau, lower foreground.
                if fy < 0.26 {
                    height_value += 2.4;
                } else if fy < 0.48 {
                    height_value += 1.4;
                } else if fy > 0.72 {
                    height_value -= 0.8;
                }
                if fx > 0.58 && fy < 0.66 {
                    height_value += 0.8;
                }
                if fx < 0.22 && fy > 0.62 {
                    height_value -= 0.6;
                }

                // Subtle coherent terrain variation, not noisy one-cell chaos.
                let large_noise = (hash01(seed, x / 3, y / 3, 1147) - 0.5) * 0.55;
                let detail_noise = (hash01(seed, x, y, 7721) - 0.5) * 0.18;
                let h = (height_value + large_noise + detail_noise)
                    .round()
                    .clamp(0.0, 8.0) as i8;

                let road_y =
                    map.spawn.1 as f32 - (x as f32 / w) * 5.0 + ((x as f32 * 0.42).sin() * 1.15);
                let road_dist = (y as f32 - road_y).abs();
                let mut ground = if road_dist < 1.45 {
                    GroundMaterial::Dirt
                } else if fx > 0.60 && fx < 0.84 && fy > 0.36 && fy < 0.58 {
                    GroundMaterial::Rock
                } else if fx > 0.34 && fx < 0.60 && fy > 0.55 && fy < 0.73 {
                    GroundMaterial::Mud
                } else if hash01(seed, x / 4, y / 4, 908) > 0.82 && road_dist < 3.0 {
                    GroundMaterial::Dirt
                } else {
                    GroundMaterial::Grass
                };

                // Keep a little natural variation, but at patch scale.
                if matches!(ground, GroundMaterial::Grass) && hash01(seed, x / 5, y / 5, 441) > 0.92
                {
                    ground = GroundMaterial::Dirt;
                }

                let idx = map
                    .index(x, y)
                    .expect("art-preview coordinates are in bounds");
                map.cells[idx] = TerrainCell::new(h, ground);
            }
        }

        // Deliberate road and objective pads.
        for x in 0..width {
            let road_y =
                map.spawn.1 as f32 - (x as f32 / w) * 5.0 + ((x as f32 * 0.42).sin() * 1.15);
            for dy in -1..=1 {
                let y = road_y.round() as i32 + dy;
                if y >= 0 && y < height as i32 {
                    if let Some(cell) = map.cell_mut(x, y as u32) {
                        cell.ground = GroundMaterial::Dirt;
                        recompute_semantics(cell);
                    }
                }
            }
        }

        map.apply_brush(
            map.spawn.0,
            map.spawn.1,
            Brush::new(BrushKind::Flatten, 2, 1),
        );
        map.apply_brush(
            map.spawn.0,
            map.spawn.1,
            Brush::new(BrushKind::Paint(GroundMaterial::Dirt), 2, 1),
        );
        map.apply_brush(
            map.objective.0,
            map.objective.1,
            Brush::new(BrushKind::Flatten, 3, 1),
        );
        map.apply_brush(
            map.objective.0,
            map.objective.1,
            Brush::new(BrushKind::Paint(GroundMaterial::Dirt), 2, 1),
        );

        // Continuous trench run near the road and a shorter fallback trench.
        let trench_y = height / 2;
        for x in 7..width.saturating_sub(12) {
            if x % 2 == 0 || x < width / 2 {
                map.apply_brush(
                    x,
                    trench_y.saturating_sub(3),
                    Brush::new(BrushKind::DigTrench, 1, 2),
                );
            }
        }
        let fallback_x = width.saturating_sub(9);
        for y in height / 3..height.saturating_sub(5) {
            if y % 2 == 0 || y < height / 2 {
                map.apply_brush(fallback_x, y, Brush::new(BrushKind::DigTrench, 1, 1));
            }
        }

        // Berms that visibly raise ground around a defended pad and shape the route.
        for x in width.saturating_sub(12)..width.saturating_sub(3) {
            map.apply_brush(
                x,
                map.objective.1.saturating_add(3),
                Brush::new(BrushKind::RaiseBerm, 1, 1),
            );
        }
        for y in map.objective.1.saturating_sub(4)
            ..=map
                .objective
                .1
                .saturating_add(2)
                .min(height.saturating_sub(1))
        {
            map.apply_brush(
                width.saturating_sub(11),
                y,
                Brush::new(BrushKind::RaiseBerm, 1, 1),
            );
        }
        for x in 5_u32..13_u32.min(width) {
            map.apply_brush(
                x,
                height.saturating_sub(6),
                Brush::new(BrushKind::RaiseBerm, 1, 1),
            );
        }

        // A compact rock outcrop with coherent shape instead of isolated grey cells.
        let outcrop_cx = (width as f32 * 0.70) as i32;
        let outcrop_cy = (height as f32 * 0.40) as i32;
        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - outcrop_cx;
                let dy = y as i32 - outcrop_cy;
                if dx * dx + dy * dy <= 18 {
                    if let Some(cell) = map.cell_mut(x, y) {
                        cell.ground = GroundMaterial::Rock;
                        cell.height = (cell.height + 1).clamp(0, 9);
                        recompute_semantics(cell);
                    }
                }
            }
        }

        map
    }

    pub fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    pub fn cell(&self, x: u32, y: u32) -> Option<&TerrainCell> {
        self.index(x, y).map(|idx| &self.cells[idx])
    }

    pub fn cell_mut(&mut self, x: u32, y: u32) -> Option<&mut TerrainCell> {
        self.index(x, y).map(move |idx| &mut self.cells[idx])
    }

    pub fn height_at(&self, x: u32, y: u32) -> i8 {
        self.cell(x, y).map(|c| c.height).unwrap_or(0)
    }

    pub fn slope_at(&self, x: u32, y: u32) -> f32 {
        let Some(cell) = self.cell(x, y) else {
            return 0.0;
        };
        let h = cell.effective_height();
        let mut max_delta: f32 = 0.0;
        for (nx, ny) in self.neighbors4(x, y) {
            if let Some(n) = self.cell(nx, ny) {
                max_delta = max_delta.max((h - n.effective_height()).abs());
            }
        }
        max_delta
    }

    pub fn movement_cost_at(&self, x: u32, y: u32) -> f32 {
        let Some(cell) = self.cell(x, y) else {
            return f32::INFINITY;
        };
        let slope = self.slope_at(x, y);
        let trench_cost = cell.trench_depth as f32 * 0.45;
        let berm_cost = cell.berm_height as f32 * 0.35;
        cell.ground.base_movement_cost() + slope * 0.35 + trench_cost + berm_cost
    }

    pub fn neighbors4(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        let mut out = Vec::with_capacity(4);
        if x > 0 {
            out.push((x - 1, y));
        }
        if y > 0 {
            out.push((x, y - 1));
        }
        if x + 1 < self.width {
            out.push((x + 1, y));
        }
        if y + 1 < self.height {
            out.push((x, y + 1));
        }
        out
    }

    pub fn apply_brush(&mut self, center_x: u32, center_y: u32, brush: Brush) {
        let radius = brush.radius as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = center_x as i32 + dx;
                let y = center_y as i32 + dy;
                if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                    continue;
                }
                let dist2 = dx * dx + dy * dy;
                if dist2 > radius * radius {
                    continue;
                }
                self.apply_brush_to_cell(x as u32, y as u32, brush.kind, brush.intensity);
            }
        }
    }

    fn apply_brush_to_cell(&mut self, x: u32, y: u32, kind: BrushKind, intensity: u8) {
        let Some(idx) = self.index(x, y) else {
            return;
        };
        let intensity = intensity.clamp(1, 4) as i8;
        match kind {
            BrushKind::Paint(material) => {
                self.cells[idx].ground = material;
                if !matches!(
                    material,
                    GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall
                ) {
                    self.cells[idx].trench_depth = 0;
                }
                if !matches!(material, GroundMaterial::BermTop | GroundMaterial::BermFace) {
                    self.cells[idx].berm_height = 0;
                }
                recompute_semantics(&mut self.cells[idx]);
            }
            BrushKind::DigTrench => {
                let cell = &mut self.cells[idx];
                cell.height = (cell.height - intensity).clamp(0, 9);
                cell.ground = GroundMaterial::TrenchFloor;
                cell.trench_depth = cell.trench_depth.saturating_add(intensity as u8).min(4);
                cell.berm_height = 0;
                cell.cover = CoverKind::Strong;
                cell.blocks_sight = false;
            }
            BrushKind::RaiseBerm => {
                let cell = &mut self.cells[idx];
                cell.height = (cell.height + intensity).clamp(0, 9);
                cell.ground = GroundMaterial::BermTop;
                cell.berm_height = cell.berm_height.saturating_add(intensity as u8).min(4);
                cell.trench_depth = 0;
                cell.cover = CoverKind::Partial;
                cell.blocks_sight = cell.berm_height >= 2;
            }
            BrushKind::Ditch => {
                let cell = &mut self.cells[idx];
                cell.height = (cell.height - 1).clamp(0, 9);
                cell.ground = GroundMaterial::Mud;
                cell.trench_depth = cell.trench_depth.saturating_add(1).min(2);
                cell.cover = CoverKind::Partial;
                cell.blocks_sight = false;
            }
            BrushKind::Flatten => {
                let target = self.average_height_around(x, y);
                let cell = &mut self.cells[idx];
                cell.height = target;
                cell.trench_depth = 0;
                cell.berm_height = 0;
                if matches!(
                    cell.ground,
                    GroundMaterial::TrenchFloor | GroundMaterial::BermTop
                ) {
                    cell.ground = GroundMaterial::Dirt;
                }
                recompute_semantics(cell);
            }
        }
    }

    fn average_height_around(&self, x: u32, y: u32) -> i8 {
        let mut total = self.height_at(x, y) as i32;
        let mut count = 1;
        for (nx, ny) in self.neighbors4(x, y) {
            total += self.height_at(nx, ny) as i32;
            count += 1;
        }
        (total as f32 / count as f32).round().clamp(0.0, 9.0) as i8
    }
}

fn fill_rect_cells(
    map: &mut TerrainMap,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    terrain_height: i8,
    material: GroundMaterial,
) {
    for y in y0..(y0 + height).min(map.height) {
        for x in x0..(x0 + width).min(map.width) {
            set_map_cell(map, x, y, terrain_height, material);
        }
    }
}

fn fill_trench_cells(map: &mut TerrainMap, x0: u32, y0: u32, width: u32, height: u32, depth: u8) {
    for y in y0..(y0 + height).min(map.height) {
        for x in x0..(x0 + width).min(map.width) {
            if let Some(cell) = map.cell_mut(x, y) {
                cell.height = (cell.height - depth as i8).clamp(0, 9);
                cell.ground = GroundMaterial::TrenchFloor;
                cell.trench_depth = depth.clamp(1, 4);
                cell.berm_height = 0;
                cell.cover = CoverKind::Strong;
                cell.blocks_sight = false;
            }
        }
    }
}

fn fill_berm_cells(map: &mut TerrainMap, x0: u32, y0: u32, width: u32, height: u32, lift: u8) {
    for y in y0..(y0 + height).min(map.height) {
        for x in x0..(x0 + width).min(map.width) {
            if let Some(cell) = map.cell_mut(x, y) {
                cell.height = (cell.height + lift as i8).clamp(0, 9);
                cell.ground = GroundMaterial::BermTop;
                cell.trench_depth = 0;
                cell.berm_height = lift.clamp(1, 4);
                cell.cover = CoverKind::Partial;
                cell.blocks_sight = cell.berm_height >= 2;
            }
        }
    }
}

fn set_map_cell(
    map: &mut TerrainMap,
    x: u32,
    y: u32,
    terrain_height: i8,
    material: GroundMaterial,
) {
    if let Some(cell) = map.cell_mut(x, y) {
        cell.height = terrain_height.clamp(0, 9);
        cell.ground = material;
        cell.trench_depth = 0;
        cell.berm_height = 0;
        recompute_semantics(cell);
    }
}

fn recompute_semantics(cell: &mut TerrainCell) {
    match cell.ground {
        GroundMaterial::TrenchFloor => {
            cell.cover = CoverKind::Strong;
            cell.blocks_sight = false;
        }
        GroundMaterial::TrenchWall | GroundMaterial::BermFace => {
            cell.cover = CoverKind::Partial;
            cell.blocks_sight = true;
        }
        GroundMaterial::BermTop => {
            cell.cover = CoverKind::Partial;
            cell.blocks_sight = cell.berm_height >= 2;
        }
        _ => {
            cell.cover = CoverKind::None;
            cell.blocks_sight = false;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushKind {
    Paint(GroundMaterial),
    DigTrench,
    RaiseBerm,
    Ditch,
    Flatten,
}

impl BrushKind {
    pub fn label(self) -> String {
        match self {
            BrushKind::Paint(material) => format!("Paint {}", material.display_name()),
            BrushKind::DigTrench => "Dig trench".to_string(),
            BrushKind::RaiseBerm => "Raise berm".to_string(),
            BrushKind::Ditch => "Create ditch".to_string(),
            BrushKind::Flatten => "Flatten".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brush {
    pub kind: BrushKind,
    pub radius: u8,
    pub intensity: u8,
}

impl Brush {
    pub fn new(kind: BrushKind, radius: u8, intensity: u8) -> Self {
        Self {
            kind,
            radius: radius.clamp(1, 8),
            intensity: intensity.clamp(1, 4),
        }
    }
}
