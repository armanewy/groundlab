use crate::color::Rgba8;
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, ViewOrientation};
use crate::target_style::{TerrainStampKind, TerrainStampResolver};
use crate::terrain::{TerrainCell, TerrainMap};
use crate::tileset::Tileset;
use crate::visual_target::{ImageCellRect, VisualTarget};

/// Milestone 4.10 renderer options.
///
/// The source image is the visual authority. The terrain grid remains the simulation/editing
/// authority, and edits render as local replacement patches over the target-derived base scene.
#[derive(Clone, Copy, Debug)]
pub struct TargetLookRenderOptions {
    pub show_route: bool,
    pub show_markers: bool,
    pub show_debug: bool,
    pub show_grid: bool,
    pub inspect_cell: Option<(u32, u32)>,
    pub view_orientation: ViewOrientation,
    pub height_step_px: u32,
}

impl Default for TargetLookRenderOptions {
    fn default() -> Self {
        Self {
            show_route: true,
            show_markers: true,
            show_debug: false,
            show_grid: false,
            inspect_cell: None,
            view_orientation: ViewOrientation::SouthEast,
            height_step_px: 32,
        }
    }
}

/// Render the editable target-derived scene.
///
/// This path intentionally does not try to repaint the whole battlefield procedurally. It uses the
/// generated target image as the base art source, then draws only local semantic edit patches and
/// tactical overlays from the current terrain map.
pub fn render_target_look_scene(
    map: &TerrainMap,
    _tileset: &Tileset,
    options: &TargetLookRenderOptions,
) -> PixelImage {
    let target = VisualTarget::load_default()
        .expect("assets/visual_targets/dry_upland_outpost_01 must contain visual_target.png");
    let mut image = target.image.clone();
    let baseline = TerrainMap::target_derived(
        target.spec.map_size_cells.0,
        target.spec.map_size_cells.1,
        0,
    );

    draw_edit_patches(&mut image, map, &baseline, &target);

    if options.show_debug {
        draw_stamp_debug(&mut image, map, &target);
    }
    if options.show_grid {
        draw_grid(&mut image, &target, Rgba8::opaque(30, 38, 31));
    }
    if let Some(cell) = options.inspect_cell {
        draw_cell_outline(
            &mut image,
            &target,
            cell,
            Rgba8::opaque(154, 215, 238),
            0.90,
        );
    }
    if options.show_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_route(
            &mut image,
            &target,
            &path.points,
            if path.reached_goal {
                Rgba8::opaque(242, 198, 82)
            } else {
                Rgba8::opaque(230, 74, 74)
            },
        );
    }
    if options.show_markers {
        draw_marker(&mut image, &target, map.spawn, Rgba8::opaque(83, 154, 222));
        draw_marker(
            &mut image,
            &target,
            map.objective,
            Rgba8::opaque(238, 205, 91),
        );
    }

    image
}

pub fn target_look_pixel_to_cell(
    _map: &TerrainMap,
    _tileset: &Tileset,
    _options: &TargetLookRenderOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let target = VisualTarget::load_default().ok()?;
    target.pixel_to_cell(px, py)
}

fn draw_edit_patches(
    image: &mut PixelImage,
    map: &TerrainMap,
    baseline: &TerrainMap,
    target: &VisualTarget,
) {
    let width = map.width.min(target.spec.map_size_cells.0);
    let height = map.height.min(target.spec.map_size_cells.1);
    for y in 0..height {
        for x in 0..width {
            let Some(cell) = map.cell(x, y) else {
                continue;
            };
            let baseline_cell = baseline.cell(x, y);
            if baseline_cell.is_some_and(|base| same_semantics(base, cell)) {
                continue;
            }
            let Some(rect) = target.cell_rect((x, y)) else {
                continue;
            };
            draw_local_replacement_patch(image, rect, cell, hash3(x, y, 0x4100));
        }
    }
}

fn same_semantics(a: &TerrainCell, b: &TerrainCell) -> bool {
    a.height == b.height
        && a.ground == b.ground
        && a.trench_depth == b.trench_depth
        && a.berm_height == b.berm_height
        && a.cover == b.cover
        && a.blocks_sight == b.blocks_sight
}

fn draw_local_replacement_patch(
    image: &mut PixelImage,
    rect: ImageCellRect,
    cell: &TerrainCell,
    seed: u32,
) {
    if cell.trench_depth > 0 || matches!(cell.ground, GroundMaterial::TrenchFloor) {
        draw_trench_patch(image, rect, seed);
    } else if cell.berm_height > 0 || matches!(cell.ground, GroundMaterial::BermTop) {
        draw_berm_patch(image, rect, seed);
    } else {
        match cell.ground {
            GroundMaterial::Grass => {
                draw_material_patch(image, rect, Rgba8::opaque(76, 116, 54), 0.78, seed)
            }
            GroundMaterial::Dirt => {
                draw_material_patch(image, rect, Rgba8::opaque(160, 111, 64), 0.82, seed)
            }
            GroundMaterial::Mud => {
                draw_material_patch(image, rect, Rgba8::opaque(76, 67, 48), 0.84, seed)
            }
            GroundMaterial::Rock => draw_stone_patch(image, rect, seed),
            GroundMaterial::TrenchWall => draw_trench_patch(image, rect, seed),
            GroundMaterial::BermFace => draw_berm_patch(image, rect, seed),
            GroundMaterial::TrenchFloor | GroundMaterial::BermTop => unreachable!(),
        }
    }
}

fn draw_material_patch(
    image: &mut PixelImage,
    rect: ImageCellRect,
    color: Rgba8,
    alpha: f32,
    seed: u32,
) {
    let inset_x = (rect.width / 10) as i32;
    let inset_y = (rect.height / 10) as i32;
    let patch = ImageCellRect {
        x: rect.x + inset_x,
        y: rect.y + inset_y,
        width: rect.width.saturating_sub((inset_x * 2) as u32).max(1),
        height: rect.height.saturating_sub((inset_y * 2) as u32).max(1),
    };
    draw_noisy_ellipse(image, patch, color, alpha, seed);
    for i in 0..18 {
        let x = patch.x + (hash3(i, seed, 0x2231) % patch.width.max(1)) as i32;
        let y = patch.y + (hash3(i, seed, 0x2232) % patch.height.max(1)) as i32;
        let detail = if i.is_multiple_of(3) {
            color.lighten(0.20)
        } else {
            color.darken(0.20)
        };
        blend_rect(image, x, y, 2 + hash3(i, seed, 0x2233) % 5, 1, detail, 0.35);
    }
}

fn draw_trench_patch(image: &mut PixelImage, rect: ImageCellRect, seed: u32) {
    draw_soft_shadow(
        image,
        rect.x + 4,
        rect.y + 8,
        rect.width.saturating_sub(8),
        rect.height.saturating_sub(8),
        0.36,
    );
    let lip = Rgba8::opaque(116, 78, 47);
    let floor = Rgba8::opaque(33, 27, 22);
    draw_noisy_ellipse(image, rect, lip, 0.86, seed ^ 0x5511);
    let inner = ImageCellRect {
        x: rect.x + (rect.width / 7) as i32,
        y: rect.y + (rect.height / 5) as i32,
        width: rect.width.saturating_sub(rect.width / 4),
        height: rect.height.saturating_sub(rect.height / 3),
    };
    draw_noisy_ellipse(image, inner, floor, 0.94, seed ^ 0x5512);
    let plank = Rgba8::opaque(83, 56, 35);
    let mut y = inner.y + 8;
    while y < inner.y + inner.height as i32 - 4 {
        draw_line(
            image,
            inner.x + 6,
            y,
            inner.x + inner.width as i32 - 8,
            y - 2,
            plank,
            0.62,
        );
        y += 10;
    }
}

fn draw_berm_patch(image: &mut PixelImage, rect: ImageCellRect, seed: u32) {
    draw_soft_shadow(
        image,
        rect.x + 8,
        rect.y + rect.height as i32 / 2,
        rect.width.saturating_sub(10),
        rect.height / 2,
        0.24,
    );
    draw_noisy_ellipse(image, rect, Rgba8::opaque(120, 83, 48), 0.88, seed ^ 0x6611);
    for i in 0..22 {
        let x = rect.x + (hash3(i, seed, 0x6612) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(i, seed, 0x6613) % rect.height.max(1)) as i32;
        blend_rect(
            image,
            x,
            y,
            2 + hash3(i, seed, 0x6614) % 6,
            2,
            Rgba8::opaque(74, 58, 42),
            0.45,
        );
    }
}

fn draw_stone_patch(image: &mut PixelImage, rect: ImageCellRect, seed: u32) {
    let stone = Rgba8::opaque(119, 126, 111);
    blend_rect(
        image,
        rect.x + 4,
        rect.y + 8,
        rect.width.saturating_sub(8),
        rect.height.saturating_sub(10),
        stone,
        0.80,
    );
    let line = stone.darken(0.32);
    let mut y = rect.y + 20;
    while y < rect.y + rect.height as i32 - 8 {
        draw_line(
            image,
            rect.x + 8,
            y,
            rect.x + rect.width as i32 - 8,
            y,
            line,
            0.55,
        );
        y += 18;
    }
    let mut x = rect.x + 24 + (seed % 9) as i32;
    while x < rect.x + rect.width as i32 - 8 {
        draw_line(
            image,
            x,
            rect.y + 10,
            x - 2,
            rect.y + rect.height as i32 - 10,
            line,
            0.42,
        );
        x += 30;
    }
}

fn draw_route(image: &mut PixelImage, target: &VisualTarget, points: &[(u32, u32)], color: Rgba8) {
    for pair in points.windows(2) {
        let Some(a) = target.cell_center(pair[0]) else {
            continue;
        };
        let Some(b) = target.cell_center(pair[1]) else {
            continue;
        };
        draw_line(
            image,
            a.0 + 1,
            a.1 + 1,
            b.0 + 1,
            b.1 + 1,
            Rgba8::BLACK,
            0.45,
        );
        draw_line(image, a.0, a.1, b.0, b.1, color, 0.86);
    }
}

fn draw_marker(image: &mut PixelImage, target: &VisualTarget, cell: (u32, u32), color: Rgba8) {
    let Some((cx, cy)) = target.cell_center(cell) else {
        return;
    };
    let r = 12;
    for yy in -r..=r {
        for xx in -r..=r {
            if xx * xx + yy * yy <= r * r {
                blend_pixel(image, cx + xx, cy + yy, color, 0.72);
            }
        }
    }
    draw_cell_outline(image, target, cell, color.lighten(0.22), 0.88);
}

fn draw_grid(image: &mut PixelImage, target: &VisualTarget, color: Rgba8) {
    for y in 0..target.spec.map_size_cells.1 {
        for x in 0..target.spec.map_size_cells.0 {
            draw_cell_outline(image, target, (x, y), color, 0.22);
        }
    }
}

fn draw_stamp_debug(image: &mut PixelImage, map: &TerrainMap, target: &VisualTarget) {
    for stamp in TerrainStampResolver::resolve(map) {
        let color = match stamp.kind {
            TerrainStampKind::GrassFieldPatch => Rgba8::opaque(72, 156, 86),
            TerrainStampKind::DirtRoadSegment | TerrainStampKind::DirtRoadJunction => {
                Rgba8::opaque(216, 161, 88)
            }
            TerrainStampKind::TrenchStraight
            | TerrainStampKind::TrenchCorner
            | TerrainStampKind::TrenchEndCap => Rgba8::opaque(72, 178, 222),
            TerrainStampKind::BermStraight | TerrainStampKind::BermCorner => {
                Rgba8::opaque(238, 196, 83)
            }
            TerrainStampKind::StonePlatform => Rgba8::opaque(157, 178, 196),
            TerrainStampKind::MudPatch => Rgba8::opaque(88, 78, 65),
            TerrainStampKind::GrassTuftCluster
            | TerrainStampKind::RockScatter
            | TerrainStampKind::CastShadow => Rgba8::opaque(210, 178, 112),
        };
        for cell in stamp.cells {
            draw_cell_outline(image, target, cell, color, 0.50);
        }
    }
}

fn draw_cell_outline(
    image: &mut PixelImage,
    target: &VisualTarget,
    cell: (u32, u32),
    color: Rgba8,
    alpha: f32,
) {
    let Some(rect) = target.cell_rect(cell) else {
        return;
    };
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.width as i32 - 1;
    let y1 = rect.y + rect.height as i32 - 1;
    draw_line(image, x0, y0, x1, y0, color, alpha);
    draw_line(image, x1, y0, x1, y1, color, alpha);
    draw_line(image, x1, y1, x0, y1, color, alpha);
    draw_line(image, x0, y1, x0, y0, color, alpha);
}

fn draw_noisy_ellipse(
    image: &mut PixelImage,
    rect: ImageCellRect,
    color: Rgba8,
    alpha: f32,
    seed: u32,
) {
    let rx = (rect.width / 2).max(1) as i32;
    let ry = (rect.height / 2).max(1) as i32;
    let cx = rect.x + rx;
    let cy = rect.y + ry;
    for yy in -ry..=ry {
        for xx in -rx..=rx {
            let nx = xx as f32 / rx as f32;
            let ny = yy as f32 / ry as f32;
            let n = noise01(seed, (xx + rx) as u32 / 5, (yy + ry) as u32 / 5);
            let edge = 1.0 + (n - 0.5) * 0.32;
            let dist = nx * nx + ny * ny;
            if dist > edge {
                continue;
            }
            let fade = ((edge - dist) / 0.24).clamp(0.0, 1.0);
            let detail = noise01(seed ^ 0x1f2f, (xx + rx) as u32 / 3, (yy + ry) as u32 / 3) - 0.5;
            let px = if detail >= 0.0 {
                color.lighten(detail * 0.16)
            } else {
                color.darken(-detail * 0.16)
            };
            blend_pixel(image, cx + xx, cy + yy, px, alpha * fade);
        }
    }
}

fn draw_soft_shadow(image: &mut PixelImage, x: i32, y: i32, width: u32, height: u32, alpha: f32) {
    for yy in 0..height {
        let v = yy as f32 / height.max(1) as f32;
        for xx in 0..width {
            let u = xx as f32 / width.max(1) as f32;
            let edge = 1.0 - ((u - 0.5).abs() * 1.8).min(1.0) * 0.5;
            blend_pixel(
                image,
                x + xx as i32,
                y + yy as i32,
                Rgba8::BLACK,
                alpha * edge * (1.0 - v * 0.55),
            );
        }
    }
}

fn draw_line(image: &mut PixelImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba8, alpha: f32) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        blend_pixel(image, x0, y0, color, alpha);
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
}

fn blend_rect(
    image: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Rgba8,
    alpha: f32,
) {
    for yy in 0..height {
        for xx in 0..width {
            blend_pixel(image, x + xx as i32, y + yy as i32, color, alpha);
        }
    }
}

fn blend_pixel(image: &mut PixelImage, x: i32, y: i32, color: Rgba8, alpha: f32) {
    if image.in_bounds(x, y) {
        image.blend_pixel(x as u32, y as u32, color, alpha);
    }
}

fn hash3(x: u32, y: u32, salt: u32) -> u32 {
    let mut v = salt ^ x.wrapping_mul(0x9e37_79b1) ^ y.wrapping_mul(0x85eb_ca6b);
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^ (v >> 16)
}

fn noise01(seed: u32, x: u32, y: u32) -> f32 {
    hash3(x, y, seed) as f32 / u32::MAX as f32
}
