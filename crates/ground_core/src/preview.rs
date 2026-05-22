use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::los::{visibility_grid, Visibility};
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::GroundMaterial;
use crate::terrain::TerrainMap;
use crate::tileset::{stable_tile_variant, Tileset};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewMode {
    Material,
    /// First true 2.5D visualization pass: top tiles are displaced upward by height,
    /// and exposed height deltas draw vertical terrain faces. This is intentionally
    /// still a software preview, not the final GPU renderer.
    ErectedTerrain,
    Height,
    Slope,
    MovementCost,
    Route,
    LineOfSight,
}

impl PreviewMode {
    pub const ALL: [PreviewMode; 7] = [
        PreviewMode::Material,
        PreviewMode::ErectedTerrain,
        PreviewMode::Height,
        PreviewMode::Slope,
        PreviewMode::MovementCost,
        PreviewMode::Route,
        PreviewMode::LineOfSight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PreviewMode::Material => "Material",
            PreviewMode::ErectedTerrain => "2.5D erected terrain",
            PreviewMode::Height => "Height",
            PreviewMode::Slope => "Slope",
            PreviewMode::MovementCost => "Movement cost",
            PreviewMode::Route => "Predicted route",
            PreviewMode::LineOfSight => "Line of sight",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewOptions {
    pub show_grid: bool,
    pub los_source: (u32, u32),
    pub los_range: u32,
    /// Pixels of vertical visual displacement per terrain height level.
    /// This is deliberately separate from simulation units.
    pub height_step_px: u32,
    /// Workbench-only visibility helper: raised faces are slightly translucent
    /// so hidden cells remain inspectable. The final runtime should make this
    /// conditional on selected units/cells, not permanently global.
    pub fade_raised_faces: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            show_grid: true,
            los_source: (8, 8),
            los_range: 18,
            height_step_px: 8,
            fade_raised_faces: true,
        }
    }
}

pub fn render_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    mode: PreviewMode,
    options: &PreviewOptions,
) -> PixelImage {
    if mode == PreviewMode::ErectedTerrain {
        return render_erected_terrain_preview(map, tileset, options);
    }

    let tile_px = tileset.recipe.tile_size;
    let width = map.width * tile_px;
    let height = map.height * tile_px;
    let mut image = PixelImage::new(width, height, Rgba8::opaque(14, 14, 16));
    let vis = if mode == PreviewMode::LineOfSight {
        Some(visibility_grid(map, options.los_source, options.los_range))
    } else {
        None
    };
    let path = if mode == PreviewMode::Route {
        Some(find_path(map, map.spawn, map.objective))
    } else {
        None
    };

    for y in 0..map.height {
        for x in 0..map.width {
            let Some(cell) = map.cell(x, y) else {
                continue;
            };
            let variant = stable_tile_variant(
                tileset.recipe.seed,
                x,
                y,
                visual_material_for_cell(cell.ground),
                tileset.recipe.variants_per_material,
            );
            let tile = &tileset
                .tile(visual_material_for_cell(cell.ground), variant)
                .image;
            let px = x * tile_px;
            let py = y * tile_px;
            image.blit(tile, px, py);

            let height_t = cell.height as f32 / 9.0;
            let height_shade = 0.18 * (height_t - 0.45);
            apply_cell_tint(
                &mut image,
                px,
                py,
                tile_px,
                Rgba8::WHITE,
                height_shade.max(0.0),
            );
            apply_cell_tint(
                &mut image,
                px,
                py,
                tile_px,
                Rgba8::BLACK,
                (-height_shade).max(0.0),
            );

            match mode {
                PreviewMode::Material | PreviewMode::ErectedTerrain => {
                    draw_height_edges(&mut image, map, x, y, tile_px);
                }
                PreviewMode::Height => {
                    let overlay = gradient_height(height_t);
                    apply_cell_overlay(&mut image, px, py, tile_px, overlay, 0.45);
                    draw_height_edges(&mut image, map, x, y, tile_px);
                }
                PreviewMode::Slope => {
                    let slope_t = clamp01(map.slope_at(x, y) / 5.0);
                    let overlay = gradient_warning(slope_t);
                    apply_cell_overlay(&mut image, px, py, tile_px, overlay, 0.45);
                }
                PreviewMode::MovementCost => {
                    let cost_t = clamp01((map.movement_cost_at(x, y) - 1.0) / 4.0);
                    let overlay = gradient_warning(cost_t);
                    apply_cell_overlay(&mut image, px, py, tile_px, overlay, 0.45);
                }
                PreviewMode::Route => {
                    let cost_t = clamp01((map.movement_cost_at(x, y) - 1.0) / 4.0);
                    apply_cell_overlay(&mut image, px, py, tile_px, gradient_warning(cost_t), 0.18);
                }
                PreviewMode::LineOfSight => {
                    if let Some(vis) = &vis {
                        match vis.get(x, y) {
                            Visibility::Visible => apply_cell_overlay(
                                &mut image,
                                px,
                                py,
                                tile_px,
                                Rgba8::opaque(112, 190, 156),
                                0.38,
                            ),
                            Visibility::Blocked => apply_cell_overlay(
                                &mut image,
                                px,
                                py,
                                tile_px,
                                Rgba8::opaque(38, 44, 52),
                                0.45,
                            ),
                        }
                    }
                }
            }

            if cell.trench_depth > 0 {
                image.outline_rect(
                    px + 2,
                    py + 2,
                    tile_px.saturating_sub(4),
                    tile_px.saturating_sub(4),
                    Rgba8::opaque(32, 19, 13),
                );
            }
            if cell.berm_height > 0 {
                image.outline_rect(
                    px + 1,
                    py + 1,
                    tile_px.saturating_sub(2),
                    tile_px.saturating_sub(2),
                    Rgba8::opaque(137, 95, 48),
                );
            }
            if options.show_grid {
                image.outline_rect(px, py, tile_px, tile_px, Rgba8::opaque(20, 21, 24));
            }
        }
    }

    draw_marker(&mut image, map.spawn, tile_px, Rgba8::opaque(99, 169, 218));
    draw_marker(
        &mut image,
        map.objective,
        tile_px,
        Rgba8::opaque(225, 196, 91),
    );
    if mode == PreviewMode::LineOfSight {
        draw_marker(
            &mut image,
            options.los_source,
            tile_px,
            Rgba8::opaque(145, 222, 165),
        );
    }

    if let Some(path) = path {
        draw_path(
            &mut image,
            &path.points,
            tile_px,
            if path.reached_goal {
                Rgba8::opaque(235, 174, 77)
            } else {
                Rgba8::opaque(230, 74, 74)
            },
        );
    }

    image
}

#[derive(Clone, Copy, Debug)]
struct RaisedProjection {
    tile_px: u32,
    height_step_px: u32,
    top_padding: i32,
    width: u32,
    height: u32,
}

impl RaisedProjection {
    fn cell_top_left(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let h = map
            .cell(x, y)
            .map(|cell| cell.effective_height())
            .unwrap_or(0.0);
        let sx = (x * self.tile_px) as i32;
        let sy = self.top_padding + (y * self.tile_px) as i32
            - (h * self.height_step_px as f32).round() as i32;
        (sx, sy)
    }
}

fn raised_projection(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> RaisedProjection {
    let tile_px = tileset.recipe.tile_size.max(1);
    let height_step_px = options.height_step_px.max(2).min(tile_px.max(2));
    let max_h = map
        .cells
        .iter()
        .map(|cell| cell.effective_height())
        .fold(0.0_f32, f32::max);
    let max_visual_lift = (max_h * height_step_px as f32).ceil() as i32;
    let top_padding = max_visual_lift + (tile_px / 2) as i32 + 6;
    let width = map.width * tile_px;
    let height = (top_padding.max(0) as u32) + map.height * tile_px + tile_px + height_step_px * 4;
    RaisedProjection {
        tile_px,
        height_step_px,
        top_padding,
        width,
        height,
    }
}

fn render_erected_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> PixelImage {
    let proj = raised_projection(map, tileset, options);
    let mut image = PixelImage::new(proj.width, proj.height, Rgba8::opaque(13, 14, 17));

    // Pass 0: cheap terrain contact shadows under high faces.
    for y in 0..map.height {
        for x in 0..map.width {
            draw_raised_shadow(&mut image, map, x, y, proj);
        }
    }

    // Pass 1: top surfaces. These remain normal generated pixel tiles, but are
    // displaced up by terrain height rather than recolored like a flat map.
    for y in 0..map.height {
        for x in 0..map.width {
            let Some(cell) = map.cell(x, y) else {
                continue;
            };
            let variant = stable_tile_variant(
                tileset.recipe.seed,
                x,
                y,
                visual_material_for_cell(cell.ground),
                tileset.recipe.variants_per_material,
            );
            let tile = &tileset
                .tile(visual_material_for_cell(cell.ground), variant)
                .image;
            let (sx, sy) = proj.cell_top_left(map, x, y);
            blit_i32(&mut image, tile, sx, sy);

            let height_t = cell.height as f32 / 9.0;
            let shade = 0.10 * (height_t - 0.45);
            apply_rect_tint_i32(
                &mut image,
                sx,
                sy,
                proj.tile_px,
                proj.tile_px,
                Rgba8::WHITE,
                shade.max(0.0),
            );
            apply_rect_tint_i32(
                &mut image,
                sx,
                sy,
                proj.tile_px,
                proj.tile_px,
                Rgba8::BLACK,
                (-shade).max(0.0),
            );

            if cell.trench_depth > 0 {
                outline_rect_i32(
                    &mut image,
                    sx + 2,
                    sy + 2,
                    proj.tile_px.saturating_sub(4),
                    proj.tile_px.saturating_sub(4),
                    Rgba8::opaque(28, 17, 12),
                );
            }
            if cell.berm_height > 0 {
                outline_rect_i32(
                    &mut image,
                    sx + 1,
                    sy + 1,
                    proj.tile_px.saturating_sub(2),
                    proj.tile_px.saturating_sub(2),
                    Rgba8::opaque(142, 96, 50),
                );
            }
        }
    }

    // Pass 2: vertical/exposed terrain faces. Drawing this after all top surfaces
    // makes raised ground read as erected terrain instead of a colored height map.
    for y in 0..map.height {
        for x in 0..map.width {
            draw_exposed_faces(
                &mut image,
                map,
                tileset,
                x,
                y,
                proj,
                options.fade_raised_faces,
            );
        }
    }

    // Pass 3: grid and editor markers. These deliberately draw on top so the
    // workbench remains usable even when faces overlap other cells.
    if options.show_grid {
        for y in 0..map.height {
            for x in 0..map.width {
                let (sx, sy) = proj.cell_top_left(map, x, y);
                outline_rect_i32(
                    &mut image,
                    sx,
                    sy,
                    proj.tile_px,
                    proj.tile_px,
                    Rgba8::opaque(22, 23, 27),
                );
            }
        }
    }

    draw_projected_marker(
        &mut image,
        map,
        proj,
        map.spawn,
        Rgba8::opaque(99, 169, 218),
    );
    draw_projected_marker(
        &mut image,
        map,
        proj,
        map.objective,
        Rgba8::opaque(225, 196, 91),
    );
    draw_projected_marker(
        &mut image,
        map,
        proj,
        options.los_source,
        Rgba8::opaque(145, 222, 165),
    );

    image
}

/// Convert a preview pixel coordinate to a terrain cell. Flat modes are direct.
/// In erected mode we test rendered cells from front to back so clicks select
/// the visible surface/face rather than the old flat map coordinate.
pub fn preview_pixel_to_cell(
    map: &TerrainMap,
    tileset: &Tileset,
    mode: PreviewMode,
    options: &PreviewOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let tile_px = tileset.recipe.tile_size.max(1);
    if mode != PreviewMode::ErectedTerrain {
        let x = px / tile_px;
        let y = py / tile_px;
        return (x < map.width && y < map.height).then_some((x, y));
    }

    let proj = raised_projection(map, tileset, options);
    let px = px as i32;
    let py = py as i32;
    for y in (0..map.height).rev() {
        for x in (0..map.width).rev() {
            let (sx, sy) = proj.cell_top_left(map, x, y);
            if point_in_rect(px, py, sx, sy, proj.tile_px, proj.tile_px) {
                return Some((x, y));
            }
            if point_in_any_face(map, x, y, px, py, proj) {
                return Some((x, y));
            }
        }
    }
    None
}

fn draw_raised_shadow(
    image: &mut PixelImage,
    map: &TerrainMap,
    x: u32,
    y: u32,
    proj: RaisedProjection,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let south = if y + 1 < map.height {
        map.cell(x, y + 1)
            .map(|c| c.effective_height())
            .unwrap_or(current)
    } else {
        current
    };
    let east = if x + 1 < map.width {
        map.cell(x + 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current)
    } else {
        current
    };
    let delta = (current - south).max(current - east).max(0.0);
    if delta <= 0.01 {
        return;
    }
    let (sx, sy) = proj.cell_top_left(map, x, y);
    let shadow_h = (delta * proj.height_step_px as f32).ceil() as u32;
    let alpha = (0.08 + delta * 0.035).min(0.24);
    blend_rect_i32(
        image,
        sx + (proj.tile_px / 10) as i32,
        sy + proj.tile_px as i32 + 1,
        proj.tile_px,
        shadow_h + proj.tile_px / 4,
        Rgba8::BLACK,
        alpha,
    );
}

fn draw_exposed_faces(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    x: u32,
    y: u32,
    proj: RaisedProjection,
    fade_faces: bool,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let (sx, sy) = proj.cell_top_left(map, x, y);

    // South/front face: the important one for a Stardew-like high-top-down view.
    if y + 1 < map.height {
        let south = map
            .cell(x, y + 1)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > south {
            let face_h = ((current - south) * proj.height_step_px as f32).ceil() as u32;
            draw_vertical_face(
                image,
                tileset,
                face_material_for_cell(cell),
                FaceRect::new(sx, sy + proj.tile_px as i32, proj.tile_px, face_h),
                FaceShade::Front,
                fade_faces,
            );
        }
    }

    // East/right side hint. This is not a full isometric side polygon yet; it is
    // a narrow readable face that communicates height deltas without requiring a
    // diamond-isometric asset set.
    if x + 1 < map.width {
        let east = map
            .cell(x + 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > east {
            let face_h = ((current - east) * proj.height_step_px as f32).ceil() as u32;
            let strip_w = (proj.tile_px / 5).max(3);
            draw_vertical_face(
                image,
                tileset,
                face_material_for_cell(cell),
                FaceRect::new(
                    sx + proj.tile_px as i32 - strip_w as i32,
                    sy + proj.tile_px as i32,
                    strip_w,
                    face_h,
                ),
                FaceShade::Right,
                fade_faces,
            );
        }
    }

    // West/left side hint.
    if x > 0 {
        let west = map
            .cell(x - 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > west {
            let face_h = ((current - west) * proj.height_step_px as f32).ceil() as u32;
            let strip_w = (proj.tile_px / 7).max(2);
            draw_vertical_face(
                image,
                tileset,
                face_material_for_cell(cell),
                FaceRect::new(sx, sy + proj.tile_px as i32, strip_w, face_h),
                FaceShade::Left,
                fade_faces,
            );
        }
    }
}

fn point_in_any_face(
    map: &TerrainMap,
    x: u32,
    y: u32,
    px: i32,
    py: i32,
    proj: RaisedProjection,
) -> bool {
    let Some(cell) = map.cell(x, y) else {
        return false;
    };
    let current = cell.effective_height();
    let (sx, sy) = proj.cell_top_left(map, x, y);

    if y + 1 < map.height {
        let south = map
            .cell(x, y + 1)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > south {
            let face_h = ((current - south) * proj.height_step_px as f32).ceil() as u32;
            if point_in_rect(px, py, sx, sy + proj.tile_px as i32, proj.tile_px, face_h) {
                return true;
            }
        }
    }

    if x + 1 < map.width {
        let east = map
            .cell(x + 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > east {
            let face_h = ((current - east) * proj.height_step_px as f32).ceil() as u32;
            let strip_w = (proj.tile_px / 5).max(3);
            if point_in_rect(
                px,
                py,
                sx + proj.tile_px as i32 - strip_w as i32,
                sy + proj.tile_px as i32,
                strip_w,
                face_h,
            ) {
                return true;
            }
        }
    }

    if x > 0 {
        let west = map
            .cell(x - 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > west {
            let face_h = ((current - west) * proj.height_step_px as f32).ceil() as u32;
            let strip_w = (proj.tile_px / 7).max(2);
            if point_in_rect(px, py, sx, sy + proj.tile_px as i32, strip_w, face_h) {
                return true;
            }
        }
    }

    false
}

#[derive(Clone, Copy)]
enum FaceShade {
    Front,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct FaceRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl FaceRect {
    fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

fn draw_vertical_face(
    image: &mut PixelImage,
    tileset: &Tileset,
    material: GroundMaterial,
    rect: FaceRect,
    shade: FaceShade,
    fade_faces: bool,
) {
    if rect.height == 0 || rect.width == 0 {
        return;
    }
    let base = tileset.palette.sample(material.ramp(), 0.42);
    let color = match shade {
        FaceShade::Front => base.darken(0.20),
        FaceShade::Left => base.darken(0.30),
        FaceShade::Right => base.darken(0.12),
    };
    let alpha = if fade_faces { 0.82 } else { 1.0 };

    for yy in 0..rect.height {
        let t = if rect.height <= 1 {
            0.0
        } else {
            yy as f32 / (rect.height - 1) as f32
        };
        let row_color = color.darken(t * 0.24);
        for xx in 0..rect.width {
            // Deterministic dither helps translucent faces preserve a pixel-art feel.
            let dither = ((rect.x + xx as i32 * 3 + rect.y + yy as i32 * 5) & 7) as f32 / 7.0;
            let row_alpha = if fade_faces && dither < 0.18 {
                alpha * 0.60
            } else {
                alpha
            };
            blend_pixel_i32(
                image,
                rect.x + xx as i32,
                rect.y + yy as i32,
                row_color,
                row_alpha,
            );
        }
    }

    // Top/lip highlight and bottom contact line.
    let lip = base.lighten(0.12);
    let foot = base.darken(0.44);
    draw_line_i32(
        image,
        rect.x,
        rect.y,
        rect.x + rect.width as i32 - 1,
        rect.y,
        lip,
    );
    draw_line_i32(
        image,
        rect.x,
        rect.y + rect.height as i32 - 1,
        rect.x + rect.width as i32 - 1,
        rect.y + rect.height as i32 - 1,
        foot,
    );
}

fn face_material_for_cell(cell: &crate::terrain::TerrainCell) -> GroundMaterial {
    if cell.trench_depth > 0 {
        GroundMaterial::TrenchWall
    } else if cell.berm_height > 0 {
        GroundMaterial::BermFace
    } else {
        match cell.ground {
            GroundMaterial::Grass
            | GroundMaterial::Dirt
            | GroundMaterial::Mud
            | GroundMaterial::Rock => cell.ground,
            GroundMaterial::TrenchFloor => GroundMaterial::TrenchWall,
            GroundMaterial::TrenchWall => GroundMaterial::TrenchWall,
            GroundMaterial::BermTop => GroundMaterial::BermFace,
            GroundMaterial::BermFace => GroundMaterial::BermFace,
        }
    }
}

fn draw_projected_marker(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: RaisedProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    if cell.0 >= map.width || cell.1 >= map.height {
        return;
    }
    let (sx, sy) = proj.cell_top_left(map, cell.0, cell.1);
    let cx = sx + proj.tile_px as i32 / 2;
    let cy = sy + proj.tile_px as i32 / 2;
    let r = (proj.tile_px / 4).max(2) as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                blend_pixel_i32(image, cx + dx, cy + dy, color, 0.92);
            }
        }
    }
}

fn blit_i32(image: &mut PixelImage, src: &PixelImage, dst_x: i32, dst_y: i32) {
    for y in 0..src.height {
        for x in 0..src.width {
            let tx = dst_x + x as i32;
            let ty = dst_y + y as i32;
            if image.in_bounds(tx, ty) {
                let s = src.get(x, y);
                if s.a == 255 {
                    image.set(tx as u32, ty as u32, s);
                } else if s.a > 0 {
                    image.blend_pixel(tx as u32, ty as u32, s, s.a as f32 / 255.0);
                }
            }
        }
    }
}

fn apply_rect_tint_i32(
    image: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Rgba8,
    alpha: f32,
) {
    if alpha <= 0.001 {
        return;
    }
    blend_rect_i32(image, x, y, width, height, color, alpha.min(0.28));
}

fn blend_rect_i32(
    image: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Rgba8,
    alpha: f32,
) {
    if alpha <= 0.001 {
        return;
    }
    for yy in 0..height {
        for xx in 0..width {
            blend_pixel_i32(image, x + xx as i32, y + yy as i32, color, alpha);
        }
    }
}

fn outline_rect_i32(image: &mut PixelImage, x: i32, y: i32, width: u32, height: u32, color: Rgba8) {
    if width == 0 || height == 0 {
        return;
    }
    draw_line_i32(image, x, y, x + width as i32 - 1, y, color);
    draw_line_i32(
        image,
        x,
        y + height as i32 - 1,
        x + width as i32 - 1,
        y + height as i32 - 1,
        color,
    );
    draw_line_i32(image, x, y, x, y + height as i32 - 1, color);
    draw_line_i32(
        image,
        x + width as i32 - 1,
        y,
        x + width as i32 - 1,
        y + height as i32 - 1,
        color,
    );
}

fn draw_line_i32(image: &mut PixelImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba8) {
    image.draw_line(x0, y0, x1, y1, color);
}

fn blend_pixel_i32(image: &mut PixelImage, x: i32, y: i32, color: Rgba8, alpha: f32) {
    if image.in_bounds(x, y) {
        image.blend_pixel(x as u32, y as u32, color, alpha);
    }
}

fn point_in_rect(px: i32, py: i32, x: i32, y: i32, width: u32, height: u32) -> bool {
    px >= x && py >= y && px < x + width as i32 && py < y + height as i32
}

fn visual_material_for_cell(material: GroundMaterial) -> GroundMaterial {
    material
}

fn apply_cell_overlay(
    image: &mut PixelImage,
    px: u32,
    py: u32,
    tile_px: u32,
    color: Rgba8,
    alpha: f32,
) {
    for y in py..(py + tile_px).min(image.height) {
        for x in px..(px + tile_px).min(image.width) {
            image.blend_pixel(x, y, color, alpha);
        }
    }
}

fn apply_cell_tint(
    image: &mut PixelImage,
    px: u32,
    py: u32,
    tile_px: u32,
    color: Rgba8,
    alpha: f32,
) {
    if alpha <= 0.001 {
        return;
    }
    apply_cell_overlay(image, px, py, tile_px, color, alpha.min(0.28));
}

fn gradient_height(t: f32) -> Rgba8 {
    let t = clamp01(t);
    if t < 0.5 {
        Rgba8::opaque(48, 92, 133).blend(Rgba8::opaque(93, 139, 98), t * 2.0)
    } else {
        Rgba8::opaque(93, 139, 98).blend(Rgba8::opaque(224, 202, 115), (t - 0.5) * 2.0)
    }
}

fn gradient_warning(t: f32) -> Rgba8 {
    let t = clamp01(t);
    if t < 0.5 {
        Rgba8::opaque(73, 132, 93).blend(Rgba8::opaque(204, 172, 75), t * 2.0)
    } else {
        Rgba8::opaque(204, 172, 75).blend(Rgba8::opaque(189, 68, 60), (t - 0.5) * 2.0)
    }
}

fn draw_height_edges(image: &mut PixelImage, map: &TerrainMap, x: u32, y: u32, tile_px: u32) {
    let h = map.height_at(x, y);
    let px = x * tile_px;
    let py = y * tile_px;
    if x > 0 && map.height_at(x - 1, y) != h {
        let color = if map.height_at(x - 1, y) < h {
            Rgba8::opaque(238, 214, 126)
        } else {
            Rgba8::opaque(38, 30, 28)
        };
        image.draw_line(
            px as i32,
            py as i32,
            px as i32,
            (py + tile_px) as i32,
            color,
        );
    }
    if y > 0 && map.height_at(x, y - 1) != h {
        let color = if map.height_at(x, y - 1) < h {
            Rgba8::opaque(238, 214, 126)
        } else {
            Rgba8::opaque(38, 30, 28)
        };
        image.draw_line(
            px as i32,
            py as i32,
            (px + tile_px) as i32,
            py as i32,
            color,
        );
    }
}

fn draw_marker(image: &mut PixelImage, cell: (u32, u32), tile_px: u32, color: Rgba8) {
    let cx = cell.0 * tile_px + tile_px / 2;
    let cy = cell.1 * tile_px + tile_px / 2;
    let r = (tile_px / 4).max(2) as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                image.set_i32(cx as i32 + dx, cy as i32 + dy, color);
            }
        }
    }
}

fn draw_path(image: &mut PixelImage, points: &[(u32, u32)], tile_px: u32, color: Rgba8) {
    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        let ax = a.0 * tile_px + tile_px / 2;
        let ay = a.1 * tile_px + tile_px / 2;
        let bx = b.0 * tile_px + tile_px / 2;
        let by = b.1 * tile_px + tile_px / 2;
        image.draw_line(ax as i32, ay as i32, bx as i32, by as i32, color);
        image.draw_line(ax as i32 + 1, ay as i32, bx as i32 + 1, by as i32, color);
    }
}
