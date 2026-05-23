use crate::color::Rgba8;
use crate::edit_patch::{
    build_edit_patches, PatchRect, TerrainEditPatch, TerrainEditPatchKind,
    TerrainEditPatchOperation,
};
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, ViewOrientation};
use crate::terrain::{TerrainCell, TerrainMap};
use crate::tileset::Tileset;
use crate::visual_target::{ImageCellRect, VisualTarget};

/// Milestone 4.12 renderer options.
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
    pub show_dirty_cells: bool,
    pub show_patch_bounds: bool,
    pub show_patch_signatures: bool,
    pub show_cover_patches_only: bool,
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
            show_dirty_cells: true,
            show_patch_bounds: true,
            show_patch_signatures: true,
            show_cover_patches_only: false,
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
    let edit_patches = build_edit_patches(map, &baseline, &target);

    draw_edit_patches(
        &mut image,
        map,
        &target,
        &edit_patches,
        options.show_cover_patches_only,
    );

    if options.show_debug {
        draw_patch_debug(&mut image, map, &target, &edit_patches, options);
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
    target: &VisualTarget,
    edit_patches: &[TerrainEditPatch],
    cover_patches_only: bool,
) {
    for patch in edit_patches {
        draw_patch_blend(image, patch.bounds_px, patch.kind);
        if patch.cover_required {
            draw_cover_patch(image, map, target, patch);
        }
        if cover_patches_only {
            continue;
        }
        if draw_group_patch(image, map, target, patch) {
            continue;
        }
        for &(x, y) in &patch.cells {
            let Some(cell) = map.cell(x, y) else {
                continue;
            };
            let Some(rect) = target.cell_rect((x, y)) else {
                continue;
            };
            draw_local_replacement_patch(image, target, rect, cell, hash3(x, y, 0x4100));
        }
    }
}

fn draw_group_patch(
    image: &mut PixelImage,
    map: &TerrainMap,
    target: &VisualTarget,
    patch: &TerrainEditPatch,
) -> bool {
    if patch.cells.len() < 2 {
        return false;
    }
    let context = context_average_color(target, patch);
    match patch.kind {
        TerrainEditPatchKind::Trench => {
            draw_group_trench_patch(image, patch, context);
            true
        }
        TerrainEditPatchKind::Berm => {
            draw_group_berm_patch(image, patch, context);
            true
        }
        TerrainEditPatchKind::Road => {
            draw_group_material_patch(
                image,
                patch,
                context.blend(Rgba8::opaque(165, 113, 64), 0.80),
                0.72,
                0x8341,
            );
            true
        }
        TerrainEditPatchKind::Grass if patch.cover_required => {
            draw_group_material_patch(
                image,
                patch,
                context.blend(Rgba8::opaque(76, 119, 55), 0.86),
                0.86,
                0x8342,
            );
            true
        }
        TerrainEditPatchKind::Stone => {
            for &(x, y) in &patch.cells {
                let Some(cell) = map.cell(x, y) else {
                    continue;
                };
                let Some(rect) = target.cell_rect((x, y)) else {
                    continue;
                };
                draw_local_replacement_patch(image, target, rect, cell, hash3(x, y, 0x4100));
            }
            true
        }
        TerrainEditPatchKind::Grass | TerrainEditPatchKind::Mud | TerrainEditPatchKind::Mixed => {
            false
        }
    }
}

fn draw_cover_patch(
    image: &mut PixelImage,
    map: &TerrainMap,
    target: &VisualTarget,
    patch: &TerrainEditPatch,
) {
    let context = context_average_color(target, patch);
    let representative = patch
        .cells
        .first()
        .and_then(|&(x, y)| map.cell(x, y))
        .map(|cell| cover_color_for_cell(cell, context))
        .unwrap_or_else(|| context.blend(Rgba8::opaque(83, 110, 61), 0.75));
    let rect = patch_group_rect(patch, 12);
    draw_noisy_ellipse(
        image,
        rect,
        representative,
        0.91,
        hash3(rect.width, rect.height, 0x7211),
    );
    draw_cover_edge_noise(
        image,
        rect,
        representative,
        hash3(rect.width, rect.height, 0x7212),
    );
}

fn cover_color_for_cell(cell: &TerrainCell, context: Rgba8) -> Rgba8 {
    if cell.trench_depth > 0 || matches!(cell.ground, GroundMaterial::TrenchFloor) {
        context.blend(Rgba8::opaque(58, 42, 30), 0.88)
    } else if cell.berm_height > 0 || matches!(cell.ground, GroundMaterial::BermTop) {
        context.blend(Rgba8::opaque(126, 88, 50), 0.82)
    } else {
        match cell.ground {
            GroundMaterial::Grass => context.blend(Rgba8::opaque(75, 119, 53), 0.86),
            GroundMaterial::Dirt => context.blend(Rgba8::opaque(163, 111, 64), 0.84),
            GroundMaterial::Mud => context.blend(Rgba8::opaque(68, 60, 45), 0.86),
            GroundMaterial::Rock => context.blend(Rgba8::opaque(118, 125, 113), 0.84),
            GroundMaterial::TrenchWall | GroundMaterial::TrenchFloor => {
                context.blend(Rgba8::opaque(58, 42, 30), 0.88)
            }
            GroundMaterial::BermFace | GroundMaterial::BermTop => {
                context.blend(Rgba8::opaque(126, 88, 50), 0.82)
            }
        }
    }
}

fn draw_cover_edge_noise(image: &mut PixelImage, rect: ImageCellRect, color: Rgba8, seed: u32) {
    for i in 0..28 {
        let x = rect.x + (hash3(i, seed, 0x11) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(i, seed, 0x12) % rect.height.max(1)) as i32;
        let detail = if i % 2 == 0 {
            color.lighten(0.18)
        } else {
            color.darken(0.22)
        };
        blend_rect(
            image,
            x,
            y,
            1 + hash3(i, seed, 0x13) % 5,
            1 + hash3(i, seed, 0x14) % 3,
            detail,
            0.34,
        );
    }
}

fn draw_group_trench_patch(image: &mut PixelImage, patch: &TerrainEditPatch, context: Rgba8) {
    let rect = patch_group_rect(patch, 10);
    let lip = context.blend(Rgba8::opaque(116, 78, 47), 0.84);
    let floor = context.blend(Rgba8::opaque(29, 24, 20), 0.92);
    draw_soft_shadow(
        image,
        rect.x + 4,
        rect.y + rect.height as i32 / 5,
        rect.width.saturating_sub(8),
        (rect.height * 3 / 4).max(1),
        0.30,
    );
    draw_noisy_ellipse(
        image,
        rect,
        lip,
        0.86,
        hash3(rect.width, rect.height, 0x9131),
    );
    let inner = ImageCellRect {
        x: rect.x + (rect.width / 10) as i32,
        y: rect.y + (rect.height / 4) as i32,
        width: rect.width.saturating_sub(rect.width / 5),
        height: rect.height.saturating_sub(rect.height / 2),
    };
    draw_noisy_ellipse(
        image,
        inner,
        floor,
        0.95,
        hash3(rect.width, rect.height, 0x9132),
    );
    let plank = context.blend(Rgba8::opaque(83, 56, 35), 0.86);
    let mut x = inner.x + 10;
    while x < inner.x + inner.width as i32 - 8 {
        draw_line(
            image,
            x,
            inner.y + 4,
            x + 4,
            inner.y + inner.height as i32 - 4,
            plank,
            0.55,
        );
        x += 18;
    }
}

fn draw_group_berm_patch(image: &mut PixelImage, patch: &TerrainEditPatch, context: Rgba8) {
    let rect = patch_group_rect(patch, 12);
    let mound = context.blend(Rgba8::opaque(124, 86, 49), 0.82);
    draw_soft_shadow(
        image,
        rect.x + 8,
        rect.y + rect.height as i32 / 2,
        rect.width.saturating_sub(12),
        rect.height / 2,
        0.22,
    );
    draw_noisy_ellipse(
        image,
        rect,
        mound,
        0.88,
        hash3(rect.width, rect.height, 0x9141),
    );
    draw_cover_edge_noise(image, rect, mound, hash3(rect.width, rect.height, 0x9142));
}

fn draw_group_material_patch(
    image: &mut PixelImage,
    patch: &TerrainEditPatch,
    color: Rgba8,
    alpha: f32,
    seed: u32,
) {
    let rect = patch_group_rect(patch, 8);
    draw_noisy_ellipse(
        image,
        rect,
        color,
        alpha,
        hash3(rect.width, rect.height, seed),
    );
    draw_cover_edge_noise(
        image,
        rect,
        color,
        hash3(rect.width, rect.height, seed ^ 0x7777),
    );
}

fn patch_group_rect(patch: &TerrainEditPatch, inset: i32) -> ImageCellRect {
    let x = patch.bounds_px.x + inset;
    let y = patch.bounds_px.y + inset;
    let width = patch
        .bounds_px
        .width
        .saturating_sub((inset.max(0) as u32) * 2)
        .max(1);
    let height = patch
        .bounds_px
        .height
        .saturating_sub((inset.max(0) as u32) * 2)
        .max(1);
    ImageCellRect {
        x,
        y,
        width,
        height,
    }
}

fn draw_local_replacement_patch(
    image: &mut PixelImage,
    target: &VisualTarget,
    rect: ImageCellRect,
    cell: &TerrainCell,
    seed: u32,
) {
    let local_average = average_color_in_rect(&target.image, rect);
    if cell.trench_depth > 0 || matches!(cell.ground, GroundMaterial::TrenchFloor) {
        draw_trench_patch(image, rect, local_average, seed);
    } else if cell.berm_height > 0 || matches!(cell.ground, GroundMaterial::BermTop) {
        draw_berm_patch(image, rect, local_average, seed);
    } else {
        match cell.ground {
            GroundMaterial::Grass => draw_material_patch(
                image,
                rect,
                local_average.blend(Rgba8::opaque(74, 114, 54), 0.62),
                0.70,
                seed,
            ),
            GroundMaterial::Dirt => draw_material_patch(
                image,
                rect,
                local_average.blend(Rgba8::opaque(166, 113, 63), 0.72),
                0.78,
                seed,
            ),
            GroundMaterial::Mud => draw_material_patch(
                image,
                rect,
                local_average.blend(Rgba8::opaque(70, 63, 47), 0.76),
                0.82,
                seed,
            ),
            GroundMaterial::Rock => draw_stone_patch(image, rect, local_average, seed),
            GroundMaterial::TrenchWall => draw_trench_patch(image, rect, local_average, seed),
            GroundMaterial::BermFace => draw_berm_patch(image, rect, local_average, seed),
            GroundMaterial::TrenchFloor | GroundMaterial::BermTop => unreachable!(),
        }
    }
}

fn draw_patch_blend(image: &mut PixelImage, bounds: PatchRect, kind: TerrainEditPatchKind) {
    let color = match kind {
        TerrainEditPatchKind::Grass => Rgba8::opaque(71, 104, 49),
        TerrainEditPatchKind::Road => Rgba8::opaque(147, 104, 62),
        TerrainEditPatchKind::Mud => Rgba8::opaque(65, 58, 45),
        TerrainEditPatchKind::Stone => Rgba8::opaque(105, 112, 101),
        TerrainEditPatchKind::Trench => Rgba8::opaque(55, 39, 28),
        TerrainEditPatchKind::Berm => Rgba8::opaque(116, 82, 48),
        TerrainEditPatchKind::Mixed => Rgba8::opaque(103, 92, 70),
    };
    for yy in 0..bounds.height {
        let v = yy as f32 / bounds.height.max(1) as f32;
        for xx in 0..bounds.width {
            let u = xx as f32 / bounds.width.max(1) as f32;
            let edge = (u.min(1.0 - u).min(v.min(1.0 - v)) * 8.0).clamp(0.0, 1.0);
            blend_pixel(
                image,
                bounds.x + xx as i32,
                bounds.y + yy as i32,
                color,
                0.08 * edge,
            );
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

fn draw_trench_patch(image: &mut PixelImage, rect: ImageCellRect, local_average: Rgba8, seed: u32) {
    draw_soft_shadow(
        image,
        rect.x + 4,
        rect.y + 8,
        rect.width.saturating_sub(8),
        rect.height.saturating_sub(8),
        0.36,
    );
    let lip = local_average.blend(Rgba8::opaque(116, 78, 47), 0.82);
    let floor = local_average.blend(Rgba8::opaque(29, 24, 20), 0.90);
    draw_noisy_ellipse(image, rect, lip, 0.86, seed ^ 0x5511);
    let inner = ImageCellRect {
        x: rect.x + (rect.width / 7) as i32,
        y: rect.y + (rect.height / 5) as i32,
        width: rect.width.saturating_sub(rect.width / 4),
        height: rect.height.saturating_sub(rect.height / 3),
    };
    draw_noisy_ellipse(image, inner, floor, 0.94, seed ^ 0x5512);
    let plank = local_average.blend(Rgba8::opaque(83, 56, 35), 0.86);
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

fn draw_berm_patch(image: &mut PixelImage, rect: ImageCellRect, local_average: Rgba8, seed: u32) {
    draw_soft_shadow(
        image,
        rect.x + 8,
        rect.y + rect.height as i32 / 2,
        rect.width.saturating_sub(10),
        rect.height / 2,
        0.24,
    );
    let mound = local_average.blend(Rgba8::opaque(120, 83, 48), 0.80);
    draw_noisy_ellipse(image, rect, mound, 0.88, seed ^ 0x6611);
    for i in 0..22 {
        let x = rect.x + (hash3(i, seed, 0x6612) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(i, seed, 0x6613) % rect.height.max(1)) as i32;
        blend_rect(
            image,
            x,
            y,
            2 + hash3(i, seed, 0x6614) % 6,
            2,
            mound.darken(0.30),
            0.45,
        );
    }
}

fn draw_stone_patch(image: &mut PixelImage, rect: ImageCellRect, local_average: Rgba8, seed: u32) {
    let stone = local_average.blend(Rgba8::opaque(119, 126, 111), 0.78);
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

fn draw_patch_debug(
    image: &mut PixelImage,
    map: &TerrainMap,
    target: &VisualTarget,
    edit_patches: &[TerrainEditPatch],
    options: &TargetLookRenderOptions,
) {
    if options.show_patch_signatures {
        for y in 0..target.spec.map_size_cells.1.min(map.height) {
            for x in 0..target.spec.map_size_cells.0.min(map.width) {
                let Some(cell) = map.cell(x, y) else {
                    continue;
                };
                let Some(rect) = target.cell_rect((x, y)) else {
                    continue;
                };
                blend_rect(
                    image,
                    rect.x + 3,
                    rect.y + 3,
                    9,
                    9,
                    material_debug_color(cell.ground),
                    0.52,
                );
                let bars = cell.height.max(0) as u32;
                for i in 0..bars.min(8) {
                    blend_rect(
                        image,
                        rect.x + 3 + i as i32 * 3,
                        rect.y + rect.height as i32 - 8,
                        2,
                        5,
                        Rgba8::opaque(235, 224, 132),
                        0.60,
                    );
                }
            }
        }
    }

    for patch in edit_patches {
        if options.show_dirty_cells {
            for &cell in &patch.neighbor_cells {
                draw_cell_outline(image, target, cell, Rgba8::opaque(210, 213, 204), 0.18);
            }
        }
        let color = edit_patch_debug_color(patch);
        if options.show_dirty_cells {
            for &cell in &patch.cells {
                draw_cell_outline(image, target, cell, color, 0.78);
            }
        }
        if options.show_patch_bounds {
            draw_rect_outline(image, patch.bounds_px, color.lighten(0.18), 0.82);
        }
    }
}

fn material_debug_color(material: GroundMaterial) -> Rgba8 {
    match material {
        GroundMaterial::Grass => Rgba8::opaque(87, 170, 75),
        GroundMaterial::Dirt => Rgba8::opaque(222, 157, 81),
        GroundMaterial::Mud => Rgba8::opaque(103, 90, 70),
        GroundMaterial::Rock => Rgba8::opaque(166, 180, 184),
        GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall => Rgba8::opaque(78, 190, 232),
        GroundMaterial::BermTop | GroundMaterial::BermFace => Rgba8::opaque(242, 196, 78),
    }
}

fn edit_patch_debug_color(patch: &TerrainEditPatch) -> Rgba8 {
    if patch.operation == TerrainEditPatchOperation::Cover {
        return Rgba8::opaque(255, 105, 75);
    }
    match patch.kind {
        TerrainEditPatchKind::Grass => Rgba8::opaque(71, 205, 104),
        TerrainEditPatchKind::Road => Rgba8::opaque(244, 171, 81),
        TerrainEditPatchKind::Mud => Rgba8::opaque(142, 118, 85),
        TerrainEditPatchKind::Stone => Rgba8::opaque(176, 211, 228),
        TerrainEditPatchKind::Trench => Rgba8::opaque(73, 203, 244),
        TerrainEditPatchKind::Berm => Rgba8::opaque(246, 215, 88),
        TerrainEditPatchKind::Mixed => Rgba8::opaque(220, 134, 229),
    }
}

fn draw_rect_outline(image: &mut PixelImage, rect: PatchRect, color: Rgba8, alpha: f32) {
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.width as i32 - 1;
    let y1 = rect.y + rect.height as i32 - 1;
    draw_line(image, x0, y0, x1, y0, color, alpha);
    draw_line(image, x1, y0, x1, y1, color, alpha);
    draw_line(image, x1, y1, x0, y1, color, alpha);
    draw_line(image, x0, y1, x0, y0, color, alpha);
}

fn average_color_in_rect(image: &PixelImage, rect: ImageCellRect) -> Rgba8 {
    let x0 = rect.x.max(0) as u32;
    let y0 = rect.y.max(0) as u32;
    let x1 = (rect.x + rect.width as i32).clamp(0, image.width as i32) as u32;
    let y1 = (rect.y + rect.height as i32).clamp(0, image.height as i32) as u32;
    if x0 >= x1 || y0 >= y1 {
        return Rgba8::opaque(96, 91, 72);
    }

    let mut r = 0_u64;
    let mut g = 0_u64;
    let mut b = 0_u64;
    let mut count = 0_u64;
    let step_x = ((x1 - x0) / 12).max(1);
    let step_y = ((y1 - y0) / 12).max(1);
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            let px = image.get(x, y);
            r += px.r as u64;
            g += px.g as u64;
            b += px.b as u64;
            count += 1;
            x = x.saturating_add(step_x);
        }
        y = y.saturating_add(step_y);
    }

    if count == 0 {
        return Rgba8::opaque(96, 91, 72);
    }
    Rgba8::opaque((r / count) as u8, (g / count) as u8, (b / count) as u8)
}

fn context_average_color(target: &VisualTarget, patch: &TerrainEditPatch) -> Rgba8 {
    let mut r = 0_u64;
    let mut g = 0_u64;
    let mut b = 0_u64;
    let mut count = 0_u64;
    for &cell in &patch.neighbor_cells {
        let Some(rect) = target.cell_rect(cell) else {
            continue;
        };
        let px = average_color_in_rect(&target.image, rect);
        r += px.r as u64;
        g += px.g as u64;
        b += px.b as u64;
        count += 1;
    }
    if count == 0 {
        return average_color_in_rect(
            &target.image,
            ImageCellRect {
                x: patch.bounds_px.x,
                y: patch.bounds_px.y,
                width: patch.bounds_px.width,
                height: patch.bounds_px.height,
            },
        );
    }
    Rgba8::opaque((r / count) as u8, (g / count) as u8, (b / count) as u8)
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
