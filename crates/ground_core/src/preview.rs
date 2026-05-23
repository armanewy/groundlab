use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::los::{visibility_grid, Visibility};
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, StructureFaceKind};
use crate::terrain::TerrainMap;
use crate::tileset::{stable_tile_variant, Tileset};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewMode {
    Material,
    /// Software 2.5D visualization pass: top tiles are displaced upward by height,
    /// and exposed height deltas draw actual generated structure-face tiles.
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
    /// Workbench global helper: raised faces become slightly translucent.
    /// The final runtime should use local/conditional fading most of the time.
    pub fade_raised_faces: bool,
    /// Workbench-only inspection helper. When true and `inspect_cell` is set,
    /// faces near that projected cell fade to reveal terrain/objects behind them.
    pub enable_local_cutaway: bool,
    pub inspect_cell: Option<(u32, u32)>,
    /// Draw a projected route over the 2.5D terrain. Planning overlays should win
    /// over occluders so pathing remains inspectable.
    pub show_projected_route: bool,
    /// Draw thin generated lip strips on exposed terrain cuts.
    pub show_structure_lips: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            show_grid: false,
            los_source: (8, 8),
            los_range: 18,
            height_step_px: 8,
            fade_raised_faces: false,
            enable_local_cutaway: true,
            inspect_cell: None,
            show_projected_route: true,
            show_structure_lips: true,
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

    fn cell_center(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let (sx, sy) = self.cell_top_left(map, x, y);
        (sx + self.tile_px as i32 / 2, sy + self.tile_px as i32 / 2)
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

    // Pass 0: terrain contact shadows under high faces.
    for y in 0..map.height {
        for x in 0..map.width {
            draw_raised_shadow(&mut image, map, x, y, proj, tileset.recipe.shadow_strength);
        }
    }

    // Pass 1: top surfaces. These remain generated pixel tiles, but are displaced
    // by terrain height rather than recolored like a flat map.
    for y in 0..map.height {
        for x in 0..map.width {
            draw_projected_top_surface(&mut image, map, tileset, x, y, proj);
        }
    }

    // Pass 2: generated vertical/exposed terrain faces. Drawing this after all
    // top surfaces makes raised ground visibly occupy space and overlap lower cells.
    for y in 0..map.height {
        for x in 0..map.width {
            draw_exposed_faces(&mut image, map, tileset, x, y, proj, options);
        }
    }

    if options.show_projected_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_projected_path(
            &mut image,
            map,
            proj,
            &path.points,
            if path.reached_goal {
                Rgba8::opaque(235, 174, 77)
            } else {
                Rgba8::opaque(230, 74, 74)
            },
        );
    }

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
                    Rgba8::opaque(27, 29, 34),
                );
            }
        }
    }

    if let Some(cell) = options.inspect_cell {
        draw_projected_selection(&mut image, map, proj, cell, Rgba8::opaque(154, 215, 238));
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

fn draw_projected_top_surface(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    x: u32,
    y: u32,
    proj: RaisedProjection,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let visual_material = visual_material_for_cell(cell.ground);
    let variant = stable_tile_variant(
        tileset.recipe.seed,
        x,
        y,
        visual_material,
        tileset.recipe.variants_per_material,
    );
    let tile = &tileset.tile(visual_material, variant).image;
    let (sx, sy) = proj.cell_top_left(map, x, y);
    blit_i32(image, tile, sx, sy);

    let height_t = cell.height as f32 / 9.0;
    let shade = 0.10 * (height_t - 0.45);
    apply_rect_tint_i32(
        image,
        sx,
        sy,
        proj.tile_px,
        proj.tile_px,
        Rgba8::WHITE,
        shade.max(0.0),
    );
    apply_rect_tint_i32(
        image,
        sx,
        sy,
        proj.tile_px,
        proj.tile_px,
        Rgba8::BLACK,
        (-shade).max(0.0),
    );

    if cell.trench_depth > 0 {
        draw_trench_floor_detail(image, sx, sy, proj.tile_px);
    }
    if cell.berm_height > 0 {
        draw_berm_top_detail(image, sx, sy, proj.tile_px);
    }
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
    shadow_strength: f32,
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
    let alpha = (0.08 + delta * 0.045).min(0.32) * shadow_strength.max(0.15);
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
    options: &PreviewOptions,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let (sx, sy) = proj.cell_top_left(map, x, y);
    let material = face_material_for_cell(cell);
    let variant = stable_tile_variant(
        tileset.recipe.seed ^ 0x5f37_59df_6c8d_1b29,
        x,
        y,
        material,
        tileset.recipe.variants_per_material,
    );

    // South/front face: the important one for a high-top-down tactical view.
    if y + 1 < map.height {
        let south = map
            .cell(x, y + 1)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > south {
            let face_h = ((current - south) * proj.height_step_px as f32).ceil() as u32;
            draw_structure_face(
                image,
                map,
                tileset,
                proj,
                FaceDraw {
                    material,
                    face: StructureFaceKind::Front,
                    variant,
                    x: sx,
                    y: sy + proj.tile_px as i32,
                    width: proj.tile_px,
                    height: face_h,
                },
                options,
            );
        }
    }

    // East/right side hint. This keeps the projection square/readable while still
    // making height deltas feel like physical side walls.
    if x + 1 < map.width {
        let east = map
            .cell(x + 1, y)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current > east {
            let face_h = ((current - east) * proj.height_step_px as f32).ceil() as u32;
            let strip_w = (proj.tile_px / 5).max(3);
            draw_structure_face(
                image,
                map,
                tileset,
                proj,
                FaceDraw {
                    material,
                    face: StructureFaceKind::Right,
                    variant,
                    x: sx + proj.tile_px as i32 - strip_w as i32,
                    y: sy + proj.tile_px as i32,
                    width: strip_w,
                    height: face_h,
                },
                options,
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
            draw_structure_face(
                image,
                map,
                tileset,
                proj,
                FaceDraw {
                    material,
                    face: StructureFaceKind::Left,
                    variant,
                    x: sx,
                    y: sy + proj.tile_px as i32,
                    width: strip_w,
                    height: face_h,
                },
                options,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FaceDraw {
    material: GroundMaterial,
    face: StructureFaceKind,
    variant: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn draw_structure_face(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    proj: RaisedProjection,
    draw: FaceDraw,
    options: &PreviewOptions,
) {
    if draw.width == 0 || draw.height == 0 {
        return;
    }

    let alpha = face_alpha(map, tileset, proj, draw, options);
    if let Some(asset) = tileset.structure_face_tile(draw.material, draw.face, draw.variant) {
        draw_tiled_face_image(image, &asset.image, draw, alpha);
    } else {
        draw_vertical_face_fallback(image, tileset, draw, alpha);
    }

    if options.show_structure_lips {
        draw_structure_lip(image, tileset, draw, alpha.max(0.72));
    }
}

fn draw_structure_lip(image: &mut PixelImage, tileset: &Tileset, draw: FaceDraw, alpha: f32) {
    let lip_h = (tileset.recipe.tile_size / 6)
        .max(2)
        .min(draw.height.max(2));
    let lip_y = draw.y - lip_h as i32 + 1;
    if let Some(asset) =
        tileset.structure_face_tile(draw.material, StructureFaceKind::Lip, draw.variant)
    {
        let lip_draw = FaceDraw {
            face: StructureFaceKind::Lip,
            y: lip_y,
            height: lip_h,
            ..draw
        };
        draw_tiled_face_image(image, &asset.image, lip_draw, alpha);
    } else {
        let base = tileset
            .palette
            .sample(draw.material.ramp(), 0.58)
            .lighten(0.08);
        blend_rect_i32(image, draw.x, lip_y, draw.width, lip_h, base, alpha);
    }
}

fn draw_tiled_face_image(image: &mut PixelImage, src: &PixelImage, draw: FaceDraw, alpha: f32) {
    for yy in 0..draw.height {
        for xx in 0..draw.width {
            let src_x = if draw.width == 0 {
                0
            } else if draw.width >= src.width {
                xx % src.width
            } else {
                (xx * src.width / draw.width).min(src.width - 1)
            };
            let src_y = if draw.height >= src.height {
                yy % src.height
            } else {
                (yy * src.height / draw.height).min(src.height - 1)
            };
            let color = src.get(src_x, src_y);
            let dither = ((draw.x + xx as i32 * 3 + draw.y + yy as i32 * 5) & 7) as f32 / 7.0;
            let row_alpha = if alpha < 0.95 && dither < 0.14 {
                alpha * 0.62
            } else {
                alpha
            };
            blend_pixel_i32(
                image,
                draw.x + xx as i32,
                draw.y + yy as i32,
                color,
                row_alpha,
            );
        }
    }

    let edge = Rgba8::BLACK.with_alpha(255);
    draw_line_i32(
        image,
        draw.x,
        draw.y,
        draw.x + draw.width as i32 - 1,
        draw.y,
        edge.darken(0.10),
    );
    draw_line_i32(
        image,
        draw.x,
        draw.y + draw.height as i32 - 1,
        draw.x + draw.width as i32 - 1,
        draw.y + draw.height as i32 - 1,
        edge,
    );
}

fn draw_vertical_face_fallback(
    image: &mut PixelImage,
    tileset: &Tileset,
    draw: FaceDraw,
    alpha: f32,
) {
    let base = tileset.palette.sample(draw.material.ramp(), 0.42);
    let color = match draw.face {
        StructureFaceKind::Front => base.darken(0.20),
        StructureFaceKind::Left => base.darken(0.30),
        StructureFaceKind::Right => base.darken(0.12),
        StructureFaceKind::Lip => base.lighten(0.06),
    };
    for yy in 0..draw.height {
        let t = if draw.height <= 1 {
            0.0
        } else {
            yy as f32 / (draw.height - 1) as f32
        };
        let row_color = color.darken(t * 0.24);
        for xx in 0..draw.width {
            blend_pixel_i32(
                image,
                draw.x + xx as i32,
                draw.y + yy as i32,
                row_color,
                alpha,
            );
        }
    }
}

fn face_alpha(
    map: &TerrainMap,
    tileset: &Tileset,
    proj: RaisedProjection,
    draw: FaceDraw,
    options: &PreviewOptions,
) -> f32 {
    let mut alpha: f32 = if options.fade_raised_faces { 0.82 } else { 1.0 };
    if options.enable_local_cutaway {
        if let Some((cx, cy)) = options.inspect_cell {
            if cx < map.width && cy < map.height {
                let (focus_x, focus_y) = proj.cell_center(map, cx, cy);
                let face_cx = draw.x + draw.width as i32 / 2;
                let face_cy = draw.y + draw.height as i32 / 2;
                let dx = (face_cx - focus_x) as f32;
                let dy = (face_cy - focus_y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = tileset.recipe.cutaway_radius_px.max(1) as f32;
                if dist < radius {
                    let t = clamp01(dist / radius);
                    let cutaway =
                        tileset.recipe.cutaway_alpha + (1.0 - tileset.recipe.cutaway_alpha) * t;
                    alpha = alpha.min(cutaway);
                }
            }
        }
    }
    alpha.clamp(0.15, 1.0)
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

fn face_material_for_cell(cell: &crate::terrain::TerrainCell) -> GroundMaterial {
    if cell.trench_depth > 0 {
        GroundMaterial::TrenchWall
    } else if cell.berm_height > 0 {
        GroundMaterial::BermFace
    } else {
        match cell.ground {
            GroundMaterial::Grass | GroundMaterial::Dirt => GroundMaterial::Dirt,
            GroundMaterial::Mud => GroundMaterial::Mud,
            GroundMaterial::Rock => GroundMaterial::Rock,
            GroundMaterial::TrenchFloor => GroundMaterial::TrenchWall,
            GroundMaterial::TrenchWall => GroundMaterial::TrenchWall,
            GroundMaterial::BermTop => GroundMaterial::BermFace,
            GroundMaterial::BermFace => GroundMaterial::BermFace,
        }
    }
}

fn draw_trench_floor_detail(image: &mut PixelImage, sx: i32, sy: i32, tile_px: u32) {
    let inset = (tile_px / 6).max(2);
    blend_rect_i32(
        image,
        sx + inset as i32,
        sy + inset as i32,
        tile_px.saturating_sub(inset * 2),
        tile_px.saturating_sub(inset * 2),
        Rgba8::opaque(13, 10, 9),
        0.24,
    );
    outline_rect_i32(
        image,
        sx + 2,
        sy + 2,
        tile_px.saturating_sub(4),
        tile_px.saturating_sub(4),
        Rgba8::opaque(37, 25, 18),
    );
}

fn draw_berm_top_detail(image: &mut PixelImage, sx: i32, sy: i32, tile_px: u32) {
    let inset = (tile_px / 8).max(2);
    outline_rect_i32(
        image,
        sx + inset as i32,
        sy + inset as i32,
        tile_px.saturating_sub(inset * 2),
        tile_px.saturating_sub(inset * 2),
        Rgba8::opaque(158, 111, 57),
    );
    blend_rect_i32(
        image,
        sx,
        sy,
        tile_px,
        (tile_px / 5).max(2),
        Rgba8::WHITE,
        0.04,
    );
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
    let (cx, cy) = proj.cell_center(map, cell.0, cell.1);
    let r = (proj.tile_px / 4).max(2) as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                blend_pixel_i32(image, cx + dx, cy + dy, color, 0.92);
            }
        }
    }
    for dy in -r - 1..=r + 1 {
        for dx in -r - 1..=r + 1 {
            let d = dx * dx + dy * dy;
            if d <= (r + 1) * (r + 1) && d > r * r {
                blend_pixel_i32(image, cx + dx, cy + dy, Rgba8::opaque(9, 10, 12), 0.85);
            }
        }
    }
}

fn draw_projected_selection(
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
    outline_rect_i32(
        image,
        sx - 1,
        sy - 1,
        proj.tile_px + 2,
        proj.tile_px + 2,
        color,
    );
    blend_rect_i32(image, sx, sy, proj.tile_px, proj.tile_px, color, 0.12);
}

fn draw_projected_path(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: RaisedProjection,
    points: &[(u32, u32)],
    color: Rgba8,
) {
    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        if a.0 >= map.width || a.1 >= map.height || b.0 >= map.width || b.1 >= map.height {
            continue;
        }
        let (ax, ay) = proj.cell_center(map, a.0, a.1);
        let (bx, by) = proj.cell_center(map, b.0, b.1);
        draw_line_i32(image, ax, ay, bx, by, Rgba8::opaque(12, 11, 9));
        draw_line_i32(image, ax + 1, ay, bx + 1, by, Rgba8::opaque(12, 11, 9));
        draw_line_i32(image, ax, ay - 1, bx, by - 1, color);
        draw_line_i32(image, ax + 1, ay - 1, bx + 1, by - 1, color);
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
