use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::feature::{CardinalDir, EdgeMask, TerrainFeatureKind, TerrainFeatureMap};
use crate::los::{visibility_grid, Visibility};
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, StructureFaceKind, TransitionEdge, ViewOrientation};
use crate::terrain::TerrainMap;
use crate::terrain_artkit::{TerrainArtKit, TerrainArtPieceKind, TerrainArtRepeatMode};
use crate::tileset::{stable_tile_variant, Tileset};
use crate::visual_scene::{VisualScene, VisualTerrainForm, VisualTerrainFormKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewMode {
    Material,
    /// New visual target: a composed 2D sprite scene. The terrain grid remains
    /// the simulation layer, but the renderer draws larger visual forms instead
    /// of one obvious square sprite per cell.
    PerspectiveSpriteScene,
    /// Debug/legacy faux-perspective terrain: screen-aligned cells whose sprite
    /// stack implies 3D height, lips, shadows, trenches, berms, and physical faces.
    FauxPerspectiveTerrain,
    /// Alternate/debug view: large-tile angled/dimetric 2.5D terrain with
    /// diamond top footprints, visible vertical faces, route overlays, and
    /// orientation-aware picking.
    AngledTerrain,
    /// Legacy software 2.5D visualization pass: top tiles are displaced upward by height,
    /// and exposed height deltas draw actual generated structure-face tiles.
    ErectedTerrain,
    Height,
    Slope,
    MovementCost,
    Route,
    LineOfSight,
}

impl PreviewMode {
    pub const ALL: [PreviewMode; 10] = [
        PreviewMode::Material,
        PreviewMode::PerspectiveSpriteScene,
        PreviewMode::FauxPerspectiveTerrain,
        PreviewMode::AngledTerrain,
        PreviewMode::ErectedTerrain,
        PreviewMode::Height,
        PreviewMode::Slope,
        PreviewMode::MovementCost,
        PreviewMode::Route,
        PreviewMode::LineOfSight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PreviewMode::Material => "Command map / flat material",
            PreviewMode::PerspectiveSpriteScene => "Perspective sprite scene",
            PreviewMode::FauxPerspectiveTerrain => "Faux-perspective 2D debug",
            PreviewMode::AngledTerrain => "Angled 2.5D terrain",
            PreviewMode::ErectedTerrain => "Legacy erected terrain",
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
    /// Workbench debug overlay for derived feature masks and coherent terrain runs.
    pub show_feature_overlay: bool,
    /// Current viewing direction for angled 2.5D projection.
    pub view_orientation: ViewOrientation,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            show_grid: false,
            los_source: (8, 8),
            los_range: 18,
            height_step_px: 24,
            fade_raised_faces: false,
            enable_local_cutaway: true,
            inspect_cell: None,
            show_projected_route: true,
            show_structure_lips: true,
            show_feature_overlay: false,
            view_orientation: ViewOrientation::SouthEast,
        }
    }
}

pub fn render_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    mode: PreviewMode,
    options: &PreviewOptions,
) -> PixelImage {
    if mode == PreviewMode::PerspectiveSpriteScene {
        return render_perspective_sprite_scene_preview(map, tileset, options);
    }
    if mode == PreviewMode::FauxPerspectiveTerrain {
        return render_faux_perspective_terrain_preview(map, tileset, options);
    }
    if mode == PreviewMode::AngledTerrain {
        return render_angled_terrain_preview(map, tileset, options);
    }
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
                PreviewMode::Material
                | PreviewMode::PerspectiveSpriteScene
                | PreviewMode::FauxPerspectiveTerrain
                | PreviewMode::AngledTerrain
                | PreviewMode::ErectedTerrain => {
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
    if mode == PreviewMode::PerspectiveSpriteScene {
        return perspective_scene_preview_pixel_to_cell(map, tileset, options, px, py);
    }
    if mode == PreviewMode::FauxPerspectiveTerrain {
        return faux_preview_pixel_to_cell(map, tileset, options, px, py);
    }
    if mode == PreviewMode::AngledTerrain {
        return angled_preview_pixel_to_cell(map, tileset, options, px, py);
    }

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

fn faux_preview_pixel_to_cell(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let proj = faux_projection(map, tileset, options);
    let px = px as i32;
    let py = py as i32;
    let mut cells = faux_draw_order(map, proj.orientation);
    cells.reverse();
    for (x, y) in cells {
        if point_in_faux_top(map, proj, x, y, px, py)
            || point_in_faux_faces(map, proj, x, y, px, py)
        {
            return Some((x, y));
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

#[derive(Clone, Copy, Debug)]
struct ImageRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn draw_scaled_image_rect(
    image: &mut PixelImage,
    src: &PixelImage,
    dst_x: i32,
    dst_y: i32,
    dst_w: u32,
    dst_h: u32,
    alpha: f32,
) {
    if dst_w == 0 || dst_h == 0 || src.width == 0 || src.height == 0 {
        return;
    }
    for yy in 0..dst_h {
        for xx in 0..dst_w {
            let src_x = (xx * src.width / dst_w).min(src.width - 1);
            let src_y = (yy * src.height / dst_h).min(src.height - 1);
            let color = src.get(src_x, src_y);
            if color.a == 0 {
                continue;
            }
            blend_pixel_i32(
                image,
                dst_x + xx as i32,
                dst_y + yy as i32,
                color,
                alpha * (color.a as f32 / 255.0),
            );
        }
    }
}

fn draw_scaled_image_rect_cropped(
    image: &mut PixelImage,
    src: &PixelImage,
    dst: ImageRect,
    alpha: f32,
    crop_px: u32,
) {
    if dst.width == 0 || dst.height == 0 || src.width == 0 || src.height == 0 {
        return;
    }
    let crop = crop_px
        .min(src.width.saturating_sub(1) / 2)
        .min(src.height.saturating_sub(1) / 2);
    let sample_w = src.width.saturating_sub(crop * 2).max(1);
    let sample_h = src.height.saturating_sub(crop * 2).max(1);
    for yy in 0..dst.height {
        for xx in 0..dst.width {
            let src_x = crop + (xx * sample_w / dst.width).min(sample_w - 1);
            let src_y = crop + (yy * sample_h / dst.height).min(sample_h - 1);
            let color = src.get(src_x, src_y);
            if color.a == 0 {
                continue;
            }
            blend_pixel_i32(
                image,
                dst.x + xx as i32,
                dst.y + yy as i32,
                color,
                alpha * (color.a as f32 / 255.0),
            );
        }
    }
}

fn draw_soft_pixel_shadow(
    image: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    alpha: f32,
) {
    if width == 0 || height == 0 {
        return;
    }
    for yy in 0..height {
        let ty = if height <= 1 {
            0.0
        } else {
            yy as f32 / (height - 1) as f32
        };
        let row_alpha = alpha * (1.0 - ty * 0.55).max(0.0);
        for xx in 0..width {
            let tx = if width <= 1 {
                0.0
            } else {
                xx as f32 / (width - 1) as f32
            };
            let edge_fade = (1.0 - ((tx - 0.5).abs() * 1.5).min(1.0) * 0.45).max(0.0);
            blend_pixel_i32(
                image,
                x + xx as i32,
                y + yy as i32,
                Rgba8::BLACK,
                row_alpha * edge_fade,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FauxProjection {
    cell_w: u32,
    cell_h: u32,
    height_step_px: u32,
    side_face_w: u32,
    offset_x: i32,
    offset_y: i32,
    width: u32,
    height: u32,
    orientation: ViewOrientation,
}

#[derive(Clone, Copy, Debug)]
struct FauxCellDraw {
    x: u32,
    y: u32,
    variant: u32,
}

impl FauxProjection {
    fn cell_top_left(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let h = map
            .cell(x, y)
            .map(|cell| cell.effective_height())
            .unwrap_or(0.0);
        let (vx, vy) = world_to_faux_view(self.orientation, map, x, y);
        let sx = vx * self.cell_w as i32 + self.offset_x;
        let sy = vy * self.cell_h as i32 + self.offset_y
            - (h * self.height_step_px as f32).round() as i32;
        (sx, sy)
    }

    fn cell_center(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let (sx, sy) = self.cell_top_left(map, x, y);
        (sx + self.cell_w as i32 / 2, sy + self.cell_h as i32 / 2)
    }
}

fn perspective_scene_projection(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> FauxProjection {
    let mut projection = tileset.recipe.projection.clone();
    projection.sanitize(tileset.recipe.tile_size);
    let cell_w = projection.faux_cell_width_px.clamp(96, 192);
    let cell_h = projection.faux_cell_height_px.clamp(80, 160);
    let height_step_px = options.height_step_px.clamp(28, 96);
    let side_face_w = projection
        .faux_side_face_width_px
        .max(18)
        .min(cell_w / 2)
        .max(2);
    let orientation = options.view_orientation;

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for y in 0..map.height {
        for x in 0..map.width {
            let h = map
                .cell(x, y)
                .map(|cell| cell.effective_height())
                .unwrap_or(0.0);
            let (vx, vy) = world_to_faux_view(orientation, map, x, y);
            let sx = vx * cell_w as i32;
            let sy = vy * cell_h as i32 - (h * height_step_px as f32).round() as i32;
            let face_extra = ((h.max(0.0) + 2.5) * height_step_px as f32).ceil() as i32;
            min_x = min_x.min(sx - side_face_w as i32 - 8);
            min_y = min_y.min(sy - 8);
            max_x = max_x.max(sx + cell_w as i32 + side_face_w as i32 + 8);
            max_y = max_y.max(sy + cell_h as i32 + face_extra + 8);
        }
    }

    let padding = 72_i32;
    FauxProjection {
        cell_w,
        cell_h,
        height_step_px,
        side_face_w,
        offset_x: -min_x + padding,
        offset_y: -min_y + padding,
        width: (max_x - min_x + padding * 2).max(cell_w as i32) as u32,
        height: (max_y - min_y + padding * 2).max(cell_h as i32) as u32,
        orientation,
    }
}

fn render_perspective_sprite_scene_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> PixelImage {
    let proj = perspective_scene_projection(map, tileset, options);
    let scene = VisualScene::from_terrain(map);
    let artkit = TerrainArtKit::generate(tileset);
    let mut image = PixelImage::new(proj.width, proj.height, Rgba8::opaque(11, 12, 15));

    // One soft ground plate keeps the scene feeling like a composed illustration instead
    // of isolated terrain tiles floating on a black editor background.
    draw_scene_base_plate(&mut image, map, tileset, &artkit, proj);

    for form in scene.forms.iter().filter(|form| form.kind.is_floor_like()) {
        draw_scene_floor_form(&mut image, map, tileset, &artkit, proj, form);
    }

    for form in scene.forms.iter().filter(|form| {
        matches!(
            form.kind,
            VisualTerrainFormKind::CliffFace
                | VisualTerrainFormKind::TrenchRun
                | VisualTerrainFormKind::BermRun
                | VisualTerrainFormKind::ShadowPatch
        )
    }) {
        match form.kind {
            VisualTerrainFormKind::CliffFace => {
                draw_scene_cliff_form(&mut image, map, tileset, &artkit, proj, form, options)
            }
            VisualTerrainFormKind::TrenchRun => {
                draw_scene_trench_form(&mut image, map, tileset, &artkit, proj, form)
            }
            VisualTerrainFormKind::BermRun => {
                draw_scene_berm_form(&mut image, map, tileset, &artkit, proj, form, options)
            }
            VisualTerrainFormKind::ShadowPatch => {
                draw_scene_shadow_form(&mut image, map, &artkit, proj, form)
            }
            _ => {}
        }
    }

    for form in scene
        .forms
        .iter()
        .filter(|form| form.kind == VisualTerrainFormKind::Dressing)
    {
        draw_scene_dressing_form(&mut image, map, tileset, &artkit, proj, form);
    }

    if options.show_projected_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_faux_path(
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

    if options.show_feature_overlay {
        for form in &scene.forms {
            draw_scene_form_debug(&mut image, map, proj, form);
        }
    }
    if options.show_grid {
        for (x, y) in faux_draw_order(map, proj.orientation) {
            draw_faux_grid_cell(&mut image, map, proj, x, y, Rgba8::opaque(24, 26, 31));
        }
    }
    if let Some(cell) = options.inspect_cell {
        draw_faux_selection(&mut image, map, proj, cell, Rgba8::opaque(154, 215, 238));
    }
    draw_faux_marker(
        &mut image,
        map,
        proj,
        map.spawn,
        Rgba8::opaque(99, 169, 218),
    );
    draw_faux_marker(
        &mut image,
        map,
        proj,
        map.objective,
        Rgba8::opaque(225, 196, 91),
    );
    image
}

fn perspective_scene_preview_pixel_to_cell(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let proj = perspective_scene_projection(map, tileset, options);
    let px = px as i32;
    let py = py as i32;
    let mut cells = faux_draw_order(map, proj.orientation);
    cells.reverse();
    for (x, y) in cells {
        if point_in_faux_top(map, proj, x, y, px, py)
            || point_in_faux_faces(map, proj, x, y, px, py)
        {
            return Some((x, y));
        }
    }
    None
}

fn draw_scene_base_plate(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
) {
    let grass = tileset
        .palette
        .sample(GroundMaterial::Grass.ramp(), 0.42)
        .darken(0.22);
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for y in 0..map.height {
        for x in 0..map.width {
            let (sx, sy) = proj.cell_top_left(map, x, y);
            min_x = min_x.min(sx);
            min_y = min_y.min(sy);
            max_x = max_x.max(sx + proj.cell_w as i32);
            max_y = max_y.max(sy + proj.cell_h as i32);
        }
    }
    if min_x < max_x && min_y < max_y {
        draw_soft_pixel_shadow(
            image,
            min_x - 16,
            max_y - (proj.cell_h / 3) as i32,
            (max_x - min_x) as u32 + 32,
            (proj.cell_h / 2).max(18),
            0.20,
        );
        let rect = ImageRect {
            x: min_x,
            y: min_y,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        };
        if !draw_art_piece_region(
            image,
            artkit,
            TerrainArtPieceKind::GrassFloorLarge,
            rect,
            0.28,
            0x3ba5,
        ) {
            blend_rect_i32(image, rect.x, rect.y, rect.width, rect.height, grass, 0.18);
        }
    }
}

fn draw_scene_floor_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    let material = visual_material_for_cell(form.material);
    let variant = stable_tile_variant(
        tileset.recipe.seed ^ 0x71d1_9b2d_47a5_c3ef,
        form.rect.x,
        form.rect.y,
        material,
        tileset.recipe.variants_per_material,
    );
    let piece_kind = match form.kind {
        VisualTerrainFormKind::RoadPatch => TerrainArtPieceKind::DirtRoadLarge,
        VisualTerrainFormKind::MudBasin => TerrainArtPieceKind::MudFloor,
        VisualTerrainFormKind::RockOutcrop => TerrainArtPieceKind::StoneFloor,
        VisualTerrainFormKind::RaisedPlatform | VisualTerrainFormKind::FloorRegion => {
            TerrainArtPieceKind::GrassFloorLarge
        }
        _ => TerrainArtPieceKind::GrassFloorLarge,
    };
    let rect = ImageRect {
        x: sx,
        y: sy,
        width,
        height,
    };
    if !draw_art_piece_region(
        image,
        artkit,
        piece_kind,
        rect,
        1.0,
        form.rect.x ^ (form.rect.y << 8),
    ) {
        let tile = &tileset.tile(material, variant).image;
        draw_tiled_image_region(image, tile, rect, 1.0, form.rect.x ^ (form.rect.y << 8));
    }

    let height_shade = 0.045 * (form.base_height - 2.0);
    if height_shade > 0.0 {
        blend_rect_i32(
            image,
            sx,
            sy,
            width,
            height,
            Rgba8::WHITE,
            height_shade.min(0.16),
        );
    } else {
        blend_rect_i32(
            image,
            sx,
            sy,
            width,
            height,
            Rgba8::BLACK,
            (-height_shade).min(0.16),
        );
    }

    match form.kind {
        VisualTerrainFormKind::RoadPatch => {
            draw_scene_road_edges(image, tileset, artkit, sx, sy, width, height)
        }
        VisualTerrainFormKind::MudBasin => {
            draw_scene_mud_basin(image, tileset, sx, sy, width, height)
        }
        VisualTerrainFormKind::RockOutcrop => {
            draw_scene_rock_mass(image, tileset, sx, sy, width, height)
        }
        VisualTerrainFormKind::RaisedPlatform => {
            draw_scene_platform_cap(image, tileset, sx, sy, width, height)
        }
        _ => {}
    }
}

fn draw_scene_cliff_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
    options: &PreviewOptions,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    let face_h = ((form.height_delta.max(0.5) * proj.height_step_px as f32 * 1.18).ceil() as u32)
        .max((proj.height_step_px / 2).max(8));
    let face_y = sy + height as i32;
    draw_soft_pixel_shadow(
        image,
        sx + 6,
        face_y + face_h as i32 / 2,
        width,
        (proj.cell_h / 3).max(12),
        0.18,
    );
    let variant = stable_tile_variant(
        tileset.recipe.seed ^ 0x93a2_bccd_51f0_aa19,
        form.rect.x,
        form.rect.y,
        form.material,
        tileset.recipe.variants_per_material,
    );
    let alpha = scene_cutaway_alpha(
        map,
        proj,
        ImageRect {
            x: sx,
            y: face_y,
            width,
            height: face_h,
        },
        tileset.recipe.cutaway_radius_px,
        tileset.recipe.cutaway_alpha,
        options,
    );
    if let Some(asset) =
        tileset.structure_face_tile(form.material, StructureFaceKind::Front, variant)
    {
        let piece_kind = match form.material {
            GroundMaterial::Rock => TerrainArtPieceKind::StoneWallFront,
            GroundMaterial::TrenchWall => TerrainArtPieceKind::TrenchWallFront,
            GroundMaterial::BermFace => TerrainArtPieceKind::BermFaceFront,
            _ => TerrainArtPieceKind::BermFaceFront,
        };
        if !draw_art_piece_region(
            image,
            artkit,
            piece_kind,
            ImageRect {
                x: sx,
                y: face_y,
                width,
                height: face_h,
            },
            alpha,
            form.rect.x ^ (form.rect.y << 10),
        ) {
            draw_scaled_image_rect(image, &asset.image, sx, face_y, width, face_h, alpha);
        }
    } else {
        draw_faux_face_fallback(
            image,
            tileset,
            form.material,
            StructureFaceKind::Front,
            ImageRect {
                x: sx,
                y: face_y,
                width,
                height: face_h,
            },
            alpha,
        );
    }
    if options.show_structure_lips {
        let lip_h = (proj.cell_h / 10).max(4).min(face_h.max(4));
        let lip_y = face_y - lip_h as i32 / 2;
        if let Some(asset) =
            tileset.structure_face_tile(form.material, StructureFaceKind::Lip, variant)
        {
            if !draw_art_piece_region(
                image,
                artkit,
                TerrainArtPieceKind::TrenchLip,
                ImageRect {
                    x: sx,
                    y: lip_y,
                    width,
                    height: lip_h,
                },
                alpha.max(0.82),
                form.rect.x,
            ) {
                draw_scaled_image_rect(
                    image,
                    &asset.image,
                    sx,
                    lip_y,
                    width,
                    lip_h,
                    alpha.max(0.82),
                );
            }
        } else {
            let lip = tileset
                .palette
                .sample(form.material.ramp(), 0.62)
                .lighten(0.05);
            blend_rect_i32(image, sx, lip_y, width, lip_h, lip, alpha.max(0.82));
        }
    }
}

fn draw_scene_trench_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    let floor = tileset
        .palette
        .sample(GroundMaterial::TrenchFloor.ramp(), 0.30);
    let wall = tileset
        .palette
        .sample(GroundMaterial::TrenchWall.ramp(), 0.46);
    let inset_x = (proj.cell_w / 8).max(6).min(width / 3).max(1);
    let inset_y = (proj.cell_h / 8).max(6).min(height / 3).max(1);
    draw_soft_pixel_shadow(
        image,
        sx + inset_x as i32,
        sy + inset_y as i32,
        width.saturating_sub(inset_x * 2),
        height.saturating_sub(inset_y),
        0.24,
    );
    let floor_rect = ImageRect {
        x: sx + inset_x as i32,
        y: sy + inset_y as i32,
        width: width.saturating_sub(inset_x * 2),
        height: height.saturating_sub(inset_y * 2),
    };
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::TrenchFloor,
        floor_rect,
        0.92,
        form.rect.x ^ (form.rect.y << 8),
    ) {
        blend_rect_i32(
            image,
            floor_rect.x,
            floor_rect.y,
            floor_rect.width,
            floor_rect.height,
            floor,
            0.82,
        );
    }
    let lip_h = (proj.cell_h / 10).max(4);
    let top_lip = ImageRect {
        x: sx,
        y: sy,
        width,
        height: lip_h,
    };
    let bottom_lip = ImageRect {
        x: sx,
        y: sy + height as i32 - lip_h as i32,
        width,
        height: lip_h,
    };
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::TrenchLip,
        top_lip,
        0.90,
        form.rect.x,
    ) {
        blend_rect_i32(
            image,
            top_lip.x,
            top_lip.y,
            top_lip.width,
            top_lip.height,
            wall.lighten(0.04),
            0.82,
        );
    }
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::TrenchWallFront,
        bottom_lip,
        0.92,
        form.rect.y,
    ) {
        blend_rect_i32(
            image,
            bottom_lip.x,
            bottom_lip.y,
            bottom_lip.width,
            bottom_lip.height,
            wall.darken(0.12),
            0.90,
        );
    }
    blend_rect_i32(
        image,
        sx,
        sy,
        (proj.cell_w / 10).max(4),
        height,
        wall.darken(0.10),
        0.58,
    );
    blend_rect_i32(
        image,
        sx + width as i32 - (proj.cell_w / 10).max(4) as i32,
        sy,
        (proj.cell_w / 10).max(4),
        height,
        wall.darken(0.02),
        0.52,
    );
}

fn draw_scene_berm_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
    options: &PreviewOptions,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    let crown = tileset.palette.sample(GroundMaterial::BermTop.ramp(), 0.68);
    let face = tileset
        .palette
        .sample(GroundMaterial::BermFace.ramp(), 0.46);
    let pad = (proj.cell_w.min(proj.cell_h) / 8)
        .max(6)
        .min(width.min(height) / 3)
        .max(1);
    draw_soft_pixel_shadow(
        image,
        sx + 4,
        sy + height as i32,
        width,
        (proj.cell_h / 3).max(12),
        0.16,
    );
    let top_rect = ImageRect {
        x: sx + pad as i32,
        y: sy + pad as i32,
        width: width.saturating_sub(pad * 2),
        height: height.saturating_sub(pad),
    };
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::BermTop,
        top_rect,
        0.72,
        form.rect.x ^ (form.rect.y << 8),
    ) {
        blend_rect_i32(
            image,
            top_rect.x,
            top_rect.y,
            top_rect.width,
            top_rect.height,
            crown,
            0.34,
        );
    }
    let face_h = (proj.height_step_px / 2).max(10);
    let face_y = sy + height as i32 - face_h as i32 / 2;
    let alpha = scene_cutaway_alpha(
        map,
        proj,
        ImageRect {
            x: sx,
            y: face_y,
            width,
            height: face_h,
        },
        tileset.recipe.cutaway_radius_px,
        tileset.recipe.cutaway_alpha,
        options,
    );
    let face_rect = ImageRect {
        x: sx,
        y: face_y,
        width,
        height: face_h,
    };
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::BermFaceFront,
        face_rect,
        alpha * 0.92,
        form.rect.x,
    ) {
        blend_rect_i32(
            image,
            face_rect.x,
            face_rect.y,
            face_rect.width,
            face_rect.height,
            face.darken(0.10),
            alpha * 0.86,
        );
    }
    let lip = crown.lighten(0.08);
    blend_rect_i32(
        image,
        sx,
        face_y - (face_h / 5).max(2) as i32,
        width,
        (face_h / 5).max(2),
        lip,
        alpha * 0.88,
    );
}

fn draw_scene_shadow_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::SoftShadow,
        ImageRect {
            x: sx,
            y: sy,
            width,
            height,
        },
        0.70,
        form.rect.x,
    ) {
        draw_soft_pixel_shadow(image, sx, sy, width, height, 0.20);
    }
}

fn draw_scene_dressing_form(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    if form.id.contains("objective") {
        draw_engineering_pad(image, tileset, sx, sy, width, height, true);
        draw_art_piece_region(
            image,
            artkit,
            TerrainArtPieceKind::CornerCap,
            ImageRect {
                x: sx + width as i32 - (width / 4).max(24) as i32,
                y: sy,
                width: (width / 4).max(24),
                height: (height / 2).max(24),
            },
            0.84,
            form.rect.x,
        );
    } else {
        draw_engineering_pad(image, tileset, sx, sy, width, height, false);
    }
    draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::PropDebris,
        ImageRect {
            x: sx + (width / 5) as i32,
            y: sy + (height / 3) as i32,
            width: (width / 3).max(32),
            height: (height / 3).max(24),
        },
        0.62,
        form.rect.y,
    );
}

fn draw_engineering_pad(
    image: &mut PixelImage,
    tileset: &Tileset,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
    defended: bool,
) {
    let timber = tileset
        .palette
        .sample(GroundMaterial::BermFace.ramp(), 0.52);
    let dark = timber.darken(0.34);
    let top = tileset
        .palette
        .sample(GroundMaterial::Dirt.ramp(), 0.58)
        .lighten(0.04);
    draw_soft_pixel_shadow(
        image,
        sx + 4,
        sy + height as i32 - 4,
        width,
        (height / 2).max(12),
        0.18,
    );
    blend_rect_i32(
        image,
        sx,
        sy,
        width,
        height,
        top,
        if defended { 0.22 } else { 0.12 },
    );
    let rail_h = (height / 8).max(4);
    blend_rect_i32(image, sx, sy, width, rail_h, timber.lighten(0.08), 0.84);
    blend_rect_i32(
        image,
        sx,
        sy + height as i32 - rail_h as i32,
        width,
        rail_h,
        dark,
        0.88,
    );
    let post_w = (width / 12).max(4);
    for px in [0, width.saturating_sub(post_w), width / 2] {
        blend_rect_i32(image, sx + px as i32, sy, post_w, height, dark, 0.58);
    }
    if defended {
        let sand = tileset
            .palette
            .sample(GroundMaterial::BermTop.ramp(), 0.70)
            .lighten(0.05);
        let bag_h = (height / 5).max(6);
        blend_rect_i32(
            image,
            sx + post_w as i32,
            sy + (height / 5) as i32,
            width.saturating_sub(post_w * 2),
            bag_h,
            sand,
            0.72,
        );
        outline_rect_i32(
            image,
            sx + post_w as i32,
            sy + (height / 5) as i32,
            width.saturating_sub(post_w * 2),
            bag_h,
            sand.darken(0.28),
        );
    }
}

fn draw_scene_road_edges(
    image: &mut PixelImage,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
) {
    let edge = tileset
        .palette
        .sample(GroundMaterial::Grass.ramp(), 0.42)
        .darken(0.06);
    let band = (height.min(width) / 12).max(4);
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::DirtRoadEdge,
        ImageRect {
            x: sx,
            y: sy,
            width,
            height: band,
        },
        0.42,
        width,
    ) {
        blend_rect_i32(image, sx, sy, width, band, edge, 0.16);
    }
    if !draw_art_piece_region(
        image,
        artkit,
        TerrainArtPieceKind::DirtRoadEdge,
        ImageRect {
            x: sx,
            y: sy + height as i32 - band as i32,
            width,
            height: band,
        },
        0.46,
        height,
    ) {
        blend_rect_i32(
            image,
            sx,
            sy + height as i32 - band as i32,
            width,
            band,
            edge.darken(0.08),
            0.18,
        );
    }
}

fn draw_scene_mud_basin(
    image: &mut PixelImage,
    tileset: &Tileset,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
) {
    let wet = tileset
        .palette
        .sample(GroundMaterial::Mud.ramp(), 0.30)
        .darken(0.10);
    draw_soft_pixel_shadow(
        image,
        sx + 8,
        sy + 8,
        width.saturating_sub(16),
        height.saturating_sub(16),
        0.18,
    );
    blend_rect_i32(
        image,
        sx + 6,
        sy + 6,
        width.saturating_sub(12),
        height.saturating_sub(12),
        wet,
        0.20,
    );
}

fn draw_scene_rock_mass(
    image: &mut PixelImage,
    tileset: &Tileset,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
) {
    let high = tileset
        .palette
        .sample(GroundMaterial::Rock.ramp(), 0.70)
        .lighten(0.04);
    let shadow = tileset
        .palette
        .sample(GroundMaterial::Rock.ramp(), 0.32)
        .darken(0.08);
    blend_rect_i32(image, sx, sy, width, (height / 5).max(5), high, 0.18);
    blend_rect_i32(
        image,
        sx,
        sy + height as i32 - (height / 4).max(6) as i32,
        width,
        (height / 4).max(6),
        shadow,
        0.16,
    );
}

fn draw_scene_platform_cap(
    image: &mut PixelImage,
    tileset: &Tileset,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
) {
    let lip = tileset
        .palette
        .sample(GroundMaterial::Dirt.ramp(), 0.60)
        .lighten(0.08);
    blend_rect_i32(image, sx, sy, width, (height / 12).max(4), lip, 0.20);
}

fn draw_scene_form_debug(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) {
    let (sx, sy, width, height) = scene_form_screen_rect(map, proj, form);
    let color = match form.kind {
        VisualTerrainFormKind::FloorRegion => Rgba8::opaque(85, 162, 95),
        VisualTerrainFormKind::RaisedPlatform => Rgba8::opaque(121, 199, 116),
        VisualTerrainFormKind::RoadPatch => Rgba8::opaque(224, 167, 88),
        VisualTerrainFormKind::MudBasin => Rgba8::opaque(83, 112, 151),
        VisualTerrainFormKind::RockOutcrop => Rgba8::opaque(157, 178, 196),
        VisualTerrainFormKind::CliffFace => Rgba8::opaque(235, 97, 72),
        VisualTerrainFormKind::TrenchRun => Rgba8::opaque(73, 178, 222),
        VisualTerrainFormKind::BermRun => Rgba8::opaque(238, 196, 83),
        VisualTerrainFormKind::ShadowPatch => Rgba8::opaque(35, 38, 46),
        VisualTerrainFormKind::Dressing => Rgba8::opaque(210, 178, 112),
    };
    outline_rect_i32(image, sx, sy, width, height, color);
    blend_rect_i32(image, sx, sy, width, height, color, 0.035);
}

fn scene_form_screen_rect(
    map: &TerrainMap,
    proj: FauxProjection,
    form: &VisualTerrainForm,
) -> (i32, i32, u32, u32) {
    let x0 = form.rect.x.min(map.width.saturating_sub(1));
    let y0 = form.rect.y.min(map.height.saturating_sub(1));
    let x1 = (form.rect.x + form.rect.width.saturating_sub(1)).min(map.width.saturating_sub(1));
    let y1 = (form.rect.y + form.rect.height.saturating_sub(1)).min(map.height.saturating_sub(1));
    let mut min_vx = i32::MAX;
    let mut min_vy = i32::MAX;
    let mut max_vx = i32::MIN;
    let mut max_vy = i32::MIN;
    for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
        let (vx, vy) = world_to_faux_view(proj.orientation, map, x, y);
        min_vx = min_vx.min(vx);
        min_vy = min_vy.min(vy);
        max_vx = max_vx.max(vx);
        max_vy = max_vy.max(vy);
    }
    let sx = min_vx * proj.cell_w as i32 + proj.offset_x;
    let sy = min_vy * proj.cell_h as i32 + proj.offset_y
        - (form.base_height * proj.height_step_px as f32).round() as i32;
    let width = ((max_vx - min_vx + 1).max(1) as u32) * proj.cell_w;
    let height = ((max_vy - min_vy + 1).max(1) as u32) * proj.cell_h;
    (sx, sy, width, height)
}

fn draw_art_piece_region(
    image: &mut PixelImage,
    artkit: &TerrainArtKit,
    kind: TerrainArtPieceKind,
    dst: ImageRect,
    alpha: f32,
    seed: u32,
) -> bool {
    let Some(piece) = artkit.piece(kind) else {
        return false;
    };
    match piece.definition.repeat_mode {
        TerrainArtRepeatMode::Tile => {
            draw_tiled_image_region(image, &piece.image, dst, alpha, seed);
        }
        TerrainArtRepeatMode::Stretch | TerrainArtRepeatMode::StretchMiddle => {
            draw_scaled_image_rect(
                image,
                &piece.image,
                dst.x,
                dst.y,
                dst.width,
                dst.height,
                alpha,
            );
        }
        TerrainArtRepeatMode::Stamp => {
            let stamp_w = dst.width.min(piece.image.width).max(1);
            let stamp_h = dst.height.min(piece.image.height).max(1);
            draw_scaled_image_rect(
                image,
                &piece.image,
                dst.x + (dst.width as i32 - stamp_w as i32) / 2,
                dst.y + (dst.height as i32 - stamp_h as i32) / 2,
                stamp_w,
                stamp_h,
                alpha,
            );
        }
    }
    true
}

fn draw_tiled_image_region(
    image: &mut PixelImage,
    src: &PixelImage,
    dst: ImageRect,
    alpha: f32,
    seed: u32,
) {
    if dst.width == 0 || dst.height == 0 || src.width == 0 || src.height == 0 {
        return;
    }
    let ox = seed % src.width;
    let oy = (seed / 17) % src.height;
    for yy in 0..dst.height {
        for xx in 0..dst.width {
            let src_x = (xx + ox) % src.width;
            let src_y = (yy + oy) % src.height;
            let color = src.get(src_x, src_y);
            blend_pixel_i32(
                image,
                dst.x + xx as i32,
                dst.y + yy as i32,
                color,
                alpha * (color.a as f32 / 255.0),
            );
        }
    }
}

fn scene_cutaway_alpha(
    map: &TerrainMap,
    proj: FauxProjection,
    rect: ImageRect,
    cutaway_radius_px: u32,
    cutaway_alpha: f32,
    options: &PreviewOptions,
) -> f32 {
    let mut alpha: f32 = if options.fade_raised_faces { 0.84 } else { 1.0 };
    if options.enable_local_cutaway {
        if let Some((cx, cy)) = options.inspect_cell {
            if cx < map.width && cy < map.height {
                let (focus_x, focus_y) = proj.cell_center(map, cx, cy);
                let center_x = rect.x + rect.width as i32 / 2;
                let center_y = rect.y + rect.height as i32 / 2;
                let dx = (center_x - focus_x) as f32;
                let dy = (center_y - focus_y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = cutaway_radius_px.max(1) as f32;
                if dist < radius {
                    let t = clamp01(dist / radius);
                    alpha = alpha.min(cutaway_alpha + (1.0 - cutaway_alpha) * t);
                }
            }
        }
    }
    alpha.clamp(0.16, 1.0)
}

fn faux_projection(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> FauxProjection {
    let mut projection = tileset.recipe.projection.clone();
    projection.sanitize(tileset.recipe.tile_size);
    let cell_w = projection.faux_cell_width_px.max(8);
    let cell_h = projection.faux_cell_height_px.max(8);
    let height_step_px = options.height_step_px.clamp(4, 96);
    let side_face_w = projection.faux_side_face_width_px.min(cell_w / 2).max(2);
    let orientation = options.view_orientation;

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for y in 0..map.height {
        for x in 0..map.width {
            let h = map
                .cell(x, y)
                .map(|cell| cell.effective_height())
                .unwrap_or(0.0);
            let (vx, vy) = world_to_faux_view(orientation, map, x, y);
            let sx = vx * cell_w as i32;
            let sy = vy * cell_h as i32 - (h * height_step_px as f32).round() as i32;
            let face_extra = ((h.max(0.0) + 2.0) * height_step_px as f32).ceil() as i32;
            min_x = min_x.min(sx - side_face_w as i32 - 2);
            min_y = min_y.min(sy - 2);
            max_x = max_x.max(sx + cell_w as i32 + side_face_w as i32 + 2);
            max_y = max_y.max(sy + cell_h as i32 + face_extra + 4);
        }
    }

    let padding = 48_i32;
    FauxProjection {
        cell_w,
        cell_h,
        height_step_px,
        side_face_w,
        offset_x: -min_x + padding,
        offset_y: -min_y + padding,
        width: (max_x - min_x + padding * 2).max(cell_w as i32) as u32,
        height: (max_y - min_y + padding * 2).max(cell_h as i32) as u32,
        orientation,
    }
}

fn render_faux_perspective_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> PixelImage {
    let proj = faux_projection(map, tileset, options);
    let mut image = PixelImage::new(proj.width, proj.height, Rgba8::opaque(12, 13, 16));
    let draw_order = faux_draw_order(map, proj.orientation);
    let features = TerrainFeatureMap::from_terrain(map);

    for &(x, y) in &draw_order {
        draw_faux_contact_shadow(
            &mut image,
            map,
            &features,
            x,
            y,
            proj,
            tileset.recipe.shadow_strength,
        );
    }
    for &(x, y) in &draw_order {
        draw_faux_top_surface(&mut image, map, tileset, &features, x, y, proj);
    }
    for &(x, y) in &draw_order {
        draw_faux_feature_surface_details(&mut image, map, tileset, &features, x, y, proj);
    }
    for &(x, y) in &draw_order {
        draw_faux_exposed_faces(&mut image, map, tileset, &features, (x, y), proj, options);
    }

    if options.show_projected_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_faux_path(
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
    if options.show_feature_overlay {
        for &(x, y) in &draw_order {
            draw_faux_feature_overlay(&mut image, map, &features, proj, x, y);
        }
    }
    if options.show_grid {
        for &(x, y) in &draw_order {
            draw_faux_grid_cell(&mut image, map, proj, x, y, Rgba8::opaque(24, 26, 31));
        }
    }
    if let Some(cell) = options.inspect_cell {
        draw_faux_selection(&mut image, map, proj, cell, Rgba8::opaque(154, 215, 238));
    }
    draw_faux_marker(
        &mut image,
        map,
        proj,
        map.spawn,
        Rgba8::opaque(99, 169, 218),
    );
    draw_faux_marker(
        &mut image,
        map,
        proj,
        map.objective,
        Rgba8::opaque(225, 196, 91),
    );
    draw_faux_marker(
        &mut image,
        map,
        proj,
        options.los_source,
        Rgba8::opaque(145, 222, 165),
    );
    image
}

fn faux_draw_order(map: &TerrainMap, orientation: ViewOrientation) -> Vec<(u32, u32)> {
    let mut cells = Vec::with_capacity(map.cells.len());
    for y in 0..map.height {
        for x in 0..map.width {
            cells.push((x, y));
        }
    }
    cells.sort_by_key(|&(x, y)| {
        let (vx, vy) = world_to_faux_view(orientation, map, x, y);
        (vy, vx)
    });
    cells
}

fn draw_faux_top_surface(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    features: &TerrainFeatureMap,
    x: u32,
    y: u32,
    proj: FauxProjection,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let Some(feature) = features.cell(x, y) else {
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

    // Use the inner portion of generated surface tiles for map rendering. The
    // full tiles still keep their edge pixels in the contact sheet/atlas, but
    // the art preview no longer looks like every cell is outlined by a grid.
    draw_scaled_image_rect_cropped(
        image,
        tile,
        ImageRect {
            x: sx,
            y: sy,
            width: proj.cell_w,
            height: proj.cell_h,
        },
        1.0,
        2,
    );
    draw_faux_material_transitions(
        image,
        map,
        tileset,
        features,
        FauxCellDraw { x, y, variant },
        proj,
    );

    let height_t = cell.height as f32 / 9.0;
    let shade = 0.07 * (height_t - 0.45);
    if shade > 0.0 {
        blend_rect_i32(image, sx, sy, proj.cell_w, proj.cell_h, Rgba8::WHITE, shade);
    } else {
        blend_rect_i32(
            image,
            sx,
            sy,
            proj.cell_w,
            proj.cell_h,
            Rgba8::BLACK,
            -shade,
        );
    }

    match feature.kind {
        TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch => {
            draw_faux_trench_top(image, tileset, feature, sx, sy, proj);
        }
        TerrainFeatureKind::Berm => {
            draw_faux_berm_top(image, tileset, feature, sx, sy, proj);
        }
        TerrainFeatureKind::Ledge => {
            draw_faux_subtle_lip_hints(image, tileset, feature, sx, sy, proj, 0.22);
        }
        TerrainFeatureKind::Open | TerrainFeatureKind::Plateau => {}
    }
}

fn draw_faux_material_transitions(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    features: &TerrainFeatureMap,
    cell: FauxCellDraw,
    proj: FauxProjection,
) {
    let x = cell.x;
    let y = cell.y;
    let Some(feature) = features.cell(x, y) else {
        return;
    };
    if !feature.material_edges.any() {
        return;
    }
    let current = visual_material_for_cell(feature.material);
    let (sx, sy) = proj.cell_top_left(map, x, y);
    for (view_dx, view_dy, transition_edge) in [
        (0, -1, TransitionEdge::North),
        (0, 1, TransitionEdge::South),
        (1, 0, TransitionEdge::East),
        (-1, 0, TransitionEdge::West),
    ] {
        let (world_dx, world_dy) = faux_view_delta_to_world(proj.orientation, view_dx, view_dy);
        let Some(dir) = CardinalDir::from_delta(world_dx, world_dy) else {
            continue;
        };
        if !feature.material_edges.get(dir) {
            continue;
        }
        let nx = x as i32 + world_dx;
        let ny = y as i32 + world_dy;
        if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
            continue;
        }
        let Some(neighbor) = map.cell(nx as u32, ny as u32) else {
            continue;
        };
        let neighbor_material = visual_material_for_cell(neighbor.ground);
        if current == neighbor_material {
            continue;
        }
        if let Some(asset) =
            tileset.transition_tile(current, neighbor_material, transition_edge, cell.variant)
        {
            draw_scaled_image_rect_cropped(
                image,
                &asset.image,
                ImageRect {
                    x: sx,
                    y: sy,
                    width: proj.cell_w,
                    height: proj.cell_h,
                },
                0.92,
                1,
            );
        } else {
            draw_faux_transition_fallback(
                image,
                tileset,
                neighbor_material,
                sx,
                sy,
                proj,
                transition_edge,
            );
        }
    }
}

fn draw_faux_transition_fallback(
    image: &mut PixelImage,
    tileset: &Tileset,
    material: GroundMaterial,
    sx: i32,
    sy: i32,
    proj: FauxProjection,
    edge: TransitionEdge,
) {
    let color = tileset.palette.sample(material.ramp(), 0.48);
    let band = (proj.cell_w.min(proj.cell_h) / 7).max(4);
    match edge {
        TransitionEdge::North => blend_rect_i32(image, sx, sy, proj.cell_w, band, color, 0.32),
        TransitionEdge::South => blend_rect_i32(
            image,
            sx,
            sy + proj.cell_h as i32 - band as i32,
            proj.cell_w,
            band,
            color,
            0.32,
        ),
        TransitionEdge::East => blend_rect_i32(
            image,
            sx + proj.cell_w as i32 - band as i32,
            sy,
            band,
            proj.cell_h,
            color,
            0.28,
        ),
        TransitionEdge::West => blend_rect_i32(image, sx, sy, band, proj.cell_h, color, 0.28),
    }
}

fn draw_faux_trench_top(
    image: &mut PixelImage,
    tileset: &Tileset,
    feature: &crate::feature::TerrainFeatureCell,
    sx: i32,
    sy: i32,
    proj: FauxProjection,
) {
    let inset_x = (proj.cell_w / 7).max(4);
    let inset_y = (proj.cell_h / 7).max(4);
    let floor = tileset
        .palette
        .sample(GroundMaterial::TrenchFloor.ramp(), 0.32);
    let wall = tileset
        .palette
        .sample(GroundMaterial::TrenchWall.ramp(), 0.42);
    blend_rect_i32(
        image,
        sx + inset_x as i32,
        sy + inset_y as i32,
        proj.cell_w.saturating_sub(inset_x * 2),
        proj.cell_h.saturating_sub(inset_y * 2),
        floor,
        0.42,
    );
    draw_faux_edge_lips(
        image,
        edge_mask_to_faux_view(feature.trench_edges, proj.orientation),
        (sx, sy),
        proj,
        wall,
        0.72,
        true,
    );
}

fn draw_faux_berm_top(
    image: &mut PixelImage,
    tileset: &Tileset,
    feature: &crate::feature::TerrainFeatureCell,
    sx: i32,
    sy: i32,
    proj: FauxProjection,
) {
    let crown = tileset.palette.sample(GroundMaterial::BermTop.ramp(), 0.68);
    let edge = tileset
        .palette
        .sample(GroundMaterial::BermFace.ramp(), 0.46);
    let pad = (proj.cell_w.min(proj.cell_h) / 8).max(4);
    blend_rect_i32(
        image,
        sx + pad as i32,
        sy + pad as i32,
        proj.cell_w.saturating_sub(pad * 2),
        proj.cell_h.saturating_sub(pad * 2),
        crown,
        0.18,
    );
    draw_faux_edge_lips(
        image,
        edge_mask_to_faux_view(feature.berm_edges, proj.orientation),
        (sx, sy),
        proj,
        edge,
        0.64,
        false,
    );
}

fn draw_faux_subtle_lip_hints(
    image: &mut PixelImage,
    tileset: &Tileset,
    feature: &crate::feature::TerrainFeatureCell,
    sx: i32,
    sy: i32,
    proj: FauxProjection,
    alpha: f32,
) {
    let color = tileset
        .palette
        .sample(feature.visual_material.ramp(), 0.66)
        .lighten(0.04);
    draw_faux_edge_lips(
        image,
        edge_mask_to_faux_view(feature.ledge_edges, proj.orientation),
        (sx, sy),
        proj,
        color,
        alpha,
        false,
    );
}

fn draw_faux_edge_lips(
    image: &mut PixelImage,
    mask: crate::feature::EdgeMask,
    origin: (i32, i32),
    proj: FauxProjection,
    color: Rgba8,
    alpha: f32,
    dark_inside: bool,
) {
    let (sx, sy) = origin;
    let lip_h = (proj.cell_h / 13).max(3);
    let lip_w = (proj.cell_w / 13).max(3);
    if mask.north {
        blend_rect_i32(image, sx, sy, proj.cell_w, lip_h, color, alpha);
        if dark_inside {
            blend_rect_i32(
                image,
                sx,
                sy + lip_h as i32,
                proj.cell_w,
                lip_h,
                Rgba8::BLACK,
                alpha * 0.18,
            );
        }
    }
    if mask.south {
        blend_rect_i32(
            image,
            sx,
            sy + proj.cell_h as i32 - lip_h as i32,
            proj.cell_w,
            lip_h,
            color.darken(0.10),
            alpha,
        );
        if dark_inside {
            blend_rect_i32(
                image,
                sx,
                sy + proj.cell_h as i32 - (lip_h * 2) as i32,
                proj.cell_w,
                lip_h,
                Rgba8::BLACK,
                alpha * 0.20,
            );
        }
    }
    if mask.east {
        blend_rect_i32(
            image,
            sx + proj.cell_w as i32 - lip_w as i32,
            sy,
            lip_w,
            proj.cell_h,
            color.darken(0.08),
            alpha * 0.85,
        );
    }
    if mask.west {
        blend_rect_i32(
            image,
            sx,
            sy,
            lip_w,
            proj.cell_h,
            color.darken(0.16),
            alpha * 0.78,
        );
    }
}

fn draw_faux_feature_surface_details(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    features: &TerrainFeatureMap,
    x: u32,
    y: u32,
    proj: FauxProjection,
) {
    let Some(feature) = features.cell(x, y) else {
        return;
    };
    let (sx, sy) = proj.cell_top_left(map, x, y);
    match feature.kind {
        TerrainFeatureKind::Trench => {
            let shadow_h = (proj.cell_h / 5).max(6);
            draw_soft_pixel_shadow(
                image,
                sx + (proj.cell_w / 8) as i32,
                sy + (proj.cell_h / 2) as i32,
                proj.cell_w.saturating_sub(proj.cell_w / 4),
                shadow_h,
                0.18,
            );
        }
        TerrainFeatureKind::Berm => {
            let highlight = tileset.palette.sample(GroundMaterial::BermTop.ramp(), 0.72);
            blend_rect_i32(
                image,
                sx + (proj.cell_w / 6) as i32,
                sy + (proj.cell_h / 7) as i32,
                proj.cell_w.saturating_sub(proj.cell_w / 3),
                (proj.cell_h / 8).max(3),
                highlight,
                0.16,
            );
        }
        TerrainFeatureKind::Ditch => {
            draw_soft_pixel_shadow(
                image,
                sx + (proj.cell_w / 5) as i32,
                sy + (proj.cell_h / 3) as i32,
                proj.cell_w.saturating_sub(proj.cell_w / 3),
                (proj.cell_h / 4).max(5),
                0.12,
            );
        }
        TerrainFeatureKind::Open | TerrainFeatureKind::Plateau | TerrainFeatureKind::Ledge => {}
    }
}

fn draw_faux_exposed_faces(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    features: &TerrainFeatureMap,
    cell: (u32, u32),
    proj: FauxProjection,
    options: &PreviewOptions,
) {
    let (x, y) = cell;
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let Some(feature) = features.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let material = face_material_for_cell(cell);
    let variant = stable_tile_variant(
        tileset.recipe.seed ^ 0x9e37_79b9_7f4a_7c15,
        x,
        y,
        material,
        tileset.recipe.variants_per_material,
    );

    for (view_dx, view_dy, face) in [
        (0, 1, StructureFaceKind::Front),
        (1, 0, StructureFaceKind::Right),
        (-1, 0, StructureFaceKind::Left),
    ] {
        let neighbor =
            faux_neighbor_height_in_view_direction(map, proj.orientation, x, y, view_dx, view_dy)
                .unwrap_or(0.0);
        let delta = current - neighbor;
        if delta <= 0.01 {
            continue;
        }
        let structural_boost = match feature.kind {
            TerrainFeatureKind::Berm => 1.35,
            TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch => 1.18,
            TerrainFeatureKind::Ledge => 1.22,
            TerrainFeatureKind::Open | TerrainFeatureKind::Plateau => 1.10,
        };
        let face_h = ((delta * proj.height_step_px as f32 * structural_boost).ceil() as u32)
            .max((proj.height_step_px / 2).max(3));
        let (x_offset, width) = match face {
            StructureFaceKind::Front | StructureFaceKind::Lip => (0, proj.cell_w),
            StructureFaceKind::Right => (
                proj.cell_w as i32 - proj.side_face_w as i32,
                proj.side_face_w,
            ),
            StructureFaceKind::Left => (0, proj.side_face_w),
        };
        draw_faux_structure_face(
            image,
            map,
            tileset,
            proj,
            FauxFaceDraw {
                material,
                face,
                variant,
                x,
                y,
                screen_x_offset: x_offset,
                screen_y_offset: proj.cell_h as i32,
                width,
                height: face_h,
            },
            options,
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct FauxFaceDraw {
    material: GroundMaterial,
    face: StructureFaceKind,
    variant: u32,
    x: u32,
    y: u32,
    screen_x_offset: i32,
    screen_y_offset: i32,
    width: u32,
    height: u32,
}

fn draw_faux_structure_face(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    proj: FauxProjection,
    draw: FauxFaceDraw,
    options: &PreviewOptions,
) {
    if draw.width == 0 || draw.height == 0 {
        return;
    }
    let (sx, sy) = proj.cell_top_left(map, draw.x, draw.y);
    let face_x = sx + draw.screen_x_offset;
    let face_y = sy + draw.screen_y_offset;
    let face_rect = ImageRect {
        x: face_x,
        y: face_y,
        width: draw.width,
        height: draw.height,
    };
    let alpha = faux_face_alpha(
        map,
        proj,
        face_rect,
        tileset.recipe.cutaway_radius_px,
        tileset.recipe.cutaway_alpha,
        options,
    );
    if let Some(asset) = tileset.structure_face_tile(draw.material, draw.face, draw.variant) {
        draw_scaled_image_rect(
            image,
            &asset.image,
            face_x,
            face_y,
            draw.width,
            draw.height,
            alpha,
        );
    } else {
        draw_faux_face_fallback(image, tileset, draw.material, draw.face, face_rect, alpha);
    }

    if options.show_structure_lips && draw.face == StructureFaceKind::Front {
        let lip_h = (proj.cell_h / 9).max(2).min(draw.height.max(2));
        let lip_y = face_y - lip_h as i32 / 2;
        if let Some(asset) =
            tileset.structure_face_tile(draw.material, StructureFaceKind::Lip, draw.variant)
        {
            draw_scaled_image_rect(
                image,
                &asset.image,
                face_x,
                lip_y,
                draw.width,
                lip_h,
                alpha.max(0.72),
            );
        } else {
            let base = tileset
                .palette
                .sample(draw.material.ramp(), 0.60)
                .lighten(0.06);
            blend_rect_i32(
                image,
                face_x,
                lip_y,
                draw.width,
                lip_h,
                base,
                alpha.max(0.72),
            );
        }
    }
}

fn draw_faux_face_fallback(
    image: &mut PixelImage,
    tileset: &Tileset,
    material: GroundMaterial,
    face: StructureFaceKind,
    rect: ImageRect,
    alpha: f32,
) {
    let base = tileset.palette.sample(material.ramp(), 0.42);
    let color = match face {
        StructureFaceKind::Front => base.darken(0.24),
        StructureFaceKind::Left => base.darken(0.34),
        StructureFaceKind::Right => base.darken(0.14),
        StructureFaceKind::Lip => base.lighten(0.06),
    };
    for yy in 0..rect.height {
        let t = if rect.height <= 1 {
            0.0
        } else {
            yy as f32 / (rect.height - 1) as f32
        };
        let row = color.darken(t * 0.20);
        for xx in 0..rect.width {
            blend_pixel_i32(image, rect.x + xx as i32, rect.y + yy as i32, row, alpha);
        }
    }
    outline_rect_i32(
        image,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        Rgba8::opaque(7, 8, 10),
    );
}

fn draw_faux_contact_shadow(
    image: &mut PixelImage,
    map: &TerrainMap,
    features: &TerrainFeatureMap,
    x: u32,
    y: u32,
    proj: FauxProjection,
    shadow_strength: f32,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let Some(feature) = features.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let front =
        faux_neighbor_height_in_view_direction(map, proj.orientation, x, y, 0, 1).unwrap_or(0.0);
    let right =
        faux_neighbor_height_in_view_direction(map, proj.orientation, x, y, 1, 0).unwrap_or(0.0);
    let delta = (current - front).max(current - right).max(0.0);
    if delta <= 0.01 {
        return;
    }
    let (sx, sy) = proj.cell_top_left(map, x, y);
    let shadow_h = ((delta * proj.height_step_px as f32).ceil() as u32).max(2);
    let feature_boost = match feature.kind {
        TerrainFeatureKind::Berm => 1.30,
        TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch => 0.85,
        TerrainFeatureKind::Ledge => 1.15,
        TerrainFeatureKind::Open | TerrainFeatureKind::Plateau => 1.0,
    };
    draw_soft_pixel_shadow(
        image,
        sx + (proj.cell_w / 12) as i32,
        sy + proj.cell_h as i32 + shadow_h as i32 / 3,
        proj.cell_w + proj.side_face_w,
        (proj.cell_h / 4).max(5) + shadow_h / 2,
        (0.09 + delta * 0.040).min(0.30) * shadow_strength.max(0.15) * feature_boost,
    );
}

fn faux_face_alpha(
    map: &TerrainMap,
    proj: FauxProjection,
    rect: ImageRect,
    cutaway_radius_px: u32,
    cutaway_alpha: f32,
    options: &PreviewOptions,
) -> f32 {
    let mut alpha: f32 = if options.fade_raised_faces { 0.82 } else { 1.0 };
    if options.enable_local_cutaway {
        if let Some((cx, cy)) = options.inspect_cell {
            if cx < map.width && cy < map.height {
                let (focus_x, focus_y) = proj.cell_center(map, cx, cy);
                let face_cx = rect.x + rect.width as i32 / 2;
                let face_cy = rect.y + rect.height as i32 / 2;
                let dx = (face_cx - focus_x) as f32;
                let dy = (face_cy - focus_y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = cutaway_radius_px.max(1) as f32;
                if dist < radius {
                    let t = clamp01(dist / radius);
                    let cutaway = cutaway_alpha + (1.0 - cutaway_alpha) * t;
                    alpha = alpha.min(cutaway);
                }
            }
        }
    }
    alpha.clamp(0.15, 1.0)
}

fn draw_faux_selection(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: FauxProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    if cell.0 >= map.width || cell.1 >= map.height {
        return;
    }
    let (sx, sy) = proj.cell_top_left(map, cell.0, cell.1);
    blend_rect_i32(image, sx, sy, proj.cell_w, proj.cell_h, color, 0.14);
    outline_rect_i32(image, sx, sy, proj.cell_w, proj.cell_h, color);
    outline_rect_i32(
        image,
        sx - 1,
        sy - 1,
        proj.cell_w + 2,
        proj.cell_h + 2,
        color,
    );
}

fn draw_faux_marker(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: FauxProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    if cell.0 >= map.width || cell.1 >= map.height {
        return;
    }
    let (cx, cy) = proj.cell_center(map, cell.0, cell.1);
    let r = (proj.cell_w.min(proj.cell_h) / 5).max(5) as i32;
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

fn draw_faux_path(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: FauxProjection,
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

fn draw_faux_feature_overlay(
    image: &mut PixelImage,
    map: &TerrainMap,
    features: &TerrainFeatureMap,
    proj: FauxProjection,
    x: u32,
    y: u32,
) {
    let Some(feature) = features.cell(x, y) else {
        return;
    };
    let (sx, sy) = proj.cell_top_left(map, x, y);
    let feature_color = match feature.kind {
        TerrainFeatureKind::Trench => Rgba8::opaque(78, 178, 223),
        TerrainFeatureKind::Ditch => Rgba8::opaque(76, 136, 190),
        TerrainFeatureKind::Berm => Rgba8::opaque(236, 183, 86),
        TerrainFeatureKind::Ledge => Rgba8::opaque(229, 115, 93),
        TerrainFeatureKind::Plateau => Rgba8::opaque(115, 198, 118),
        TerrainFeatureKind::Open => Rgba8::opaque(164, 164, 164),
    };
    if feature.has_structural_edge() {
        blend_rect_i32(
            image,
            sx,
            sy,
            proj.cell_w,
            proj.cell_h,
            feature_color,
            0.055,
        );
    }
    draw_faux_mask_lines(
        image,
        edge_mask_to_faux_view(feature.material_edges, proj.orientation),
        sx,
        sy,
        proj,
        Rgba8::opaque(42, 49, 56),
        0.45,
    );
    draw_faux_mask_lines(
        image,
        edge_mask_to_faux_view(feature.ledge_edges, proj.orientation),
        sx,
        sy,
        proj,
        Rgba8::opaque(230, 84, 62),
        0.72,
    );
    draw_faux_mask_lines(
        image,
        edge_mask_to_faux_view(feature.trench_edges, proj.orientation),
        sx,
        sy,
        proj,
        Rgba8::opaque(80, 184, 224),
        0.78,
    );
    draw_faux_mask_lines(
        image,
        edge_mask_to_faux_view(feature.berm_edges, proj.orientation),
        sx,
        sy,
        proj,
        Rgba8::opaque(235, 184, 86),
        0.78,
    );
}

fn draw_faux_mask_lines(
    image: &mut PixelImage,
    mask: EdgeMask,
    sx: i32,
    sy: i32,
    proj: FauxProjection,
    color: Rgba8,
    alpha: f32,
) {
    let line = 2;
    if mask.north {
        blend_rect_i32(image, sx, sy, proj.cell_w, line, color, alpha);
    }
    if mask.south {
        blend_rect_i32(
            image,
            sx,
            sy + proj.cell_h as i32 - line as i32,
            proj.cell_w,
            line,
            color,
            alpha,
        );
    }
    if mask.east {
        blend_rect_i32(
            image,
            sx + proj.cell_w as i32 - line as i32,
            sy,
            line,
            proj.cell_h,
            color,
            alpha,
        );
    }
    if mask.west {
        blend_rect_i32(image, sx, sy, line, proj.cell_h, color, alpha);
    }
}

fn draw_faux_grid_cell(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: FauxProjection,
    x: u32,
    y: u32,
    color: Rgba8,
) {
    let (sx, sy) = proj.cell_top_left(map, x, y);
    outline_rect_i32(image, sx, sy, proj.cell_w, proj.cell_h, color);
}

fn point_in_faux_top(
    map: &TerrainMap,
    proj: FauxProjection,
    x: u32,
    y: u32,
    px: i32,
    py: i32,
) -> bool {
    let (sx, sy) = proj.cell_top_left(map, x, y);
    point_in_rect(px, py, sx, sy, proj.cell_w, proj.cell_h)
}

fn point_in_faux_faces(
    map: &TerrainMap,
    proj: FauxProjection,
    x: u32,
    y: u32,
    px: i32,
    py: i32,
) -> bool {
    let Some(cell) = map.cell(x, y) else {
        return false;
    };
    let current = cell.effective_height();
    let (sx, sy) = proj.cell_top_left(map, x, y);
    for (view_dx, view_dy, face) in [
        (0, 1, StructureFaceKind::Front),
        (1, 0, StructureFaceKind::Right),
        (-1, 0, StructureFaceKind::Left),
    ] {
        let neighbor =
            faux_neighbor_height_in_view_direction(map, proj.orientation, x, y, view_dx, view_dy)
                .unwrap_or(0.0);
        let delta = current - neighbor;
        if delta <= 0.01 {
            continue;
        }
        let face_h = ((delta * proj.height_step_px as f32 * 1.15).ceil() as u32)
            .max((proj.height_step_px / 2).max(3));
        let (x_offset, width) = match face {
            StructureFaceKind::Front | StructureFaceKind::Lip => (0, proj.cell_w),
            StructureFaceKind::Right => (
                proj.cell_w as i32 - proj.side_face_w as i32,
                proj.side_face_w,
            ),
            StructureFaceKind::Left => (0, proj.side_face_w),
        };
        if point_in_rect(
            px,
            py,
            sx + x_offset,
            sy + proj.cell_h as i32,
            width,
            face_h,
        ) {
            return true;
        }
    }
    false
}

fn world_to_faux_view(
    orientation: ViewOrientation,
    map: &TerrainMap,
    x: u32,
    y: u32,
) -> (i32, i32) {
    match orientation {
        ViewOrientation::SouthEast => (x as i32, y as i32),
        ViewOrientation::SouthWest => (y as i32, map.width as i32 - 1 - x as i32),
        ViewOrientation::NorthWest => (
            map.width as i32 - 1 - x as i32,
            map.height as i32 - 1 - y as i32,
        ),
        ViewOrientation::NorthEast => (map.height as i32 - 1 - y as i32, x as i32),
    }
}

fn faux_view_delta_to_world(
    orientation: ViewOrientation,
    view_dx: i32,
    view_dy: i32,
) -> (i32, i32) {
    match orientation {
        ViewOrientation::SouthEast => (view_dx, view_dy),
        ViewOrientation::SouthWest => (-view_dy, view_dx),
        ViewOrientation::NorthWest => (-view_dx, -view_dy),
        ViewOrientation::NorthEast => (view_dy, -view_dx),
    }
}

fn world_delta_to_faux_view(
    orientation: ViewOrientation,
    world_dx: i32,
    world_dy: i32,
) -> (i32, i32) {
    match orientation {
        ViewOrientation::SouthEast => (world_dx, world_dy),
        ViewOrientation::SouthWest => (world_dy, -world_dx),
        ViewOrientation::NorthWest => (-world_dx, -world_dy),
        ViewOrientation::NorthEast => (-world_dy, world_dx),
    }
}

fn edge_mask_to_faux_view(mask: EdgeMask, orientation: ViewOrientation) -> EdgeMask {
    let mut out = EdgeMask::empty();
    for dir in CardinalDir::ALL {
        if !mask.get(dir) {
            continue;
        }
        let (world_dx, world_dy) = dir.delta();
        let (view_dx, view_dy) = world_delta_to_faux_view(orientation, world_dx, world_dy);
        if let Some(view_dir) = CardinalDir::from_delta(view_dx, view_dy) {
            out.set(view_dir, true);
        }
    }
    out
}

fn faux_neighbor_height_in_view_direction(
    map: &TerrainMap,
    orientation: ViewOrientation,
    x: u32,
    y: u32,
    view_dx: i32,
    view_dy: i32,
) -> Option<f32> {
    let (world_dx, world_dy) = faux_view_delta_to_world(orientation, view_dx, view_dy);
    let nx = x as i32 + world_dx;
    let ny = y as i32 + world_dy;
    if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
        return None;
    }
    map.cell(nx as u32, ny as u32)
        .map(|cell| cell.effective_height())
}

#[derive(Clone, Copy, Debug)]
struct AngledProjection {
    half_w: i32,
    half_h: i32,
    tile_w: u32,
    tile_h: u32,
    height_step_px: u32,
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
    orientation: ViewOrientation,
}

type AngledEdgePoints = ((i32, i32), (i32, i32), StructureFaceKind, bool);

impl AngledProjection {
    fn cell_center(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let (u, v) = world_to_oriented(self.orientation, map.width, map.height, x, y);
        let lift = map
            .cell(x, y)
            .map(|cell| (cell.effective_height() * self.height_step_px as f32).round() as i32)
            .unwrap_or(0);
        (
            self.origin_x + (u as i32 - v as i32) * self.half_w,
            self.origin_y + (u as i32 + v as i32) * self.half_h - lift,
        )
    }

    fn diamond(self, map: &TerrainMap, x: u32, y: u32) -> [(i32, i32); 4] {
        let (cx, cy) = self.cell_center(map, x, y);
        [
            (cx, cy - self.half_h),
            (cx + self.half_w, cy),
            (cx, cy + self.half_h),
            (cx - self.half_w, cy),
        ]
    }

    fn face_height(self, delta: f32) -> u32 {
        (delta.max(0.0) * self.height_step_px as f32)
            .ceil()
            .max(1.0) as u32
    }

    fn ordered_cells(self, map: &TerrainMap) -> Vec<(u32, u32, u32, u32)> {
        let mut cells = Vec::with_capacity((map.width * map.height) as usize);
        for y in 0..map.height {
            for x in 0..map.width {
                let (u, v) = world_to_oriented(self.orientation, map.width, map.height, x, y);
                cells.push((u + v, u, x, y));
            }
        }
        cells.sort_by_key(|(depth, u, _, _)| (*depth, *u));
        cells
    }
}

fn angled_projection(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> AngledProjection {
    let mut spec = tileset.recipe.projection.clone();
    spec.sanitize(tileset.recipe.tile_size);
    let orientation = if spec.supports_four_way_rotation {
        options.view_orientation
    } else {
        spec.default_orientation
    };
    let (oriented_width, oriented_height) = oriented_dimensions(orientation, map.width, map.height);
    let half_w = (spec.tile_screen_width_px / 2).max(8) as i32;
    let half_h = (spec.tile_screen_height_px / 2).max(4) as i32;
    let max_h = map
        .cells
        .iter()
        .map(|cell| cell.effective_height())
        .fold(0.0_f32, f32::max);
    let max_lift = (max_h * spec.height_step_px as f32).ceil() as i32;
    let margin = spec.tile_screen_width_px as i32 + 16;
    let width = ((oriented_width + oriented_height) as i32 * half_w + margin * 2 + half_w * 2)
        .max(1) as u32;
    let height = ((oriented_width + oriented_height) as i32 * half_h
        + max_lift
        + spec.tile_screen_height_px as i32
        + spec.height_step_px as i32 * 6
        + margin * 2)
        .max(1) as u32;
    let origin_x = margin + oriented_height as i32 * half_w + half_w;
    let origin_y = margin + max_lift + half_h;

    AngledProjection {
        half_w,
        half_h,
        tile_w: spec.tile_screen_width_px,
        tile_h: spec.tile_screen_height_px,
        height_step_px: spec.height_step_px,
        origin_x,
        origin_y,
        width,
        height,
        orientation,
    }
}

fn render_angled_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
) -> PixelImage {
    let proj = angled_projection(map, tileset, options);
    let mut image = PixelImage::new(proj.width, proj.height, Rgba8::opaque(11, 12, 15));
    let cells = proj.ordered_cells(map);

    for &(_, _, x, y) in &cells {
        draw_angled_contact_shadow(&mut image, map, x, y, proj, tileset.recipe.shadow_strength);
    }

    for &(_, _, x, y) in &cells {
        draw_angled_exposed_faces(&mut image, map, tileset, x, y, proj, options);
        draw_angled_top_surface(&mut image, map, tileset, x, y, proj);
    }

    if options.show_projected_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_angled_path(
            &mut image,
            map,
            proj,
            &path.points,
            if path.reached_goal {
                Rgba8::opaque(245, 184, 77)
            } else {
                Rgba8::opaque(230, 74, 74)
            },
        );
    }

    if options.show_grid {
        for &(_, _, x, y) in &cells {
            draw_angled_grid_outline(&mut image, map, proj, x, y, Rgba8::opaque(24, 26, 30));
        }
    }

    if let Some(cell) = options.inspect_cell {
        draw_angled_selection(&mut image, map, proj, cell, Rgba8::opaque(154, 215, 238));
    }

    draw_angled_marker(
        &mut image,
        map,
        proj,
        map.spawn,
        Rgba8::opaque(99, 169, 218),
    );
    draw_angled_marker(
        &mut image,
        map,
        proj,
        map.objective,
        Rgba8::opaque(225, 196, 91),
    );
    draw_angled_marker(
        &mut image,
        map,
        proj,
        options.los_source,
        Rgba8::opaque(145, 222, 165),
    );

    image
}

fn draw_angled_top_surface(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    x: u32,
    y: u32,
    proj: AngledProjection,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let material = visual_material_for_cell(cell.ground);
    let variant = stable_tile_variant(
        tileset.recipe.seed,
        x,
        y,
        material,
        tileset.recipe.variants_per_material,
    );
    let tile = &tileset.tile(material, variant).image;
    let (cx, cy) = proj.cell_center(map, x, y);
    draw_diamond_textured(image, tile, cx, cy, proj, 1.0);

    let height_t = cell.height as f32 / 9.0;
    let shade = 0.12 * (height_t - 0.45);
    if shade > 0.001 {
        draw_diamond_overlay(image, cx, cy, proj, Rgba8::WHITE, shade.min(0.16));
    } else if shade < -0.001 {
        draw_diamond_overlay(image, cx, cy, proj, Rgba8::BLACK, (-shade).min(0.18));
    }

    if cell.trench_depth > 0 {
        draw_diamond_overlay(
            image,
            cx,
            cy + proj.half_h / 6,
            proj,
            Rgba8::opaque(15, 10, 9),
            0.24,
        );
    }
    if cell.berm_height > 0 {
        draw_diamond_outline(image, cx, cy, proj, Rgba8::opaque(172, 125, 62));
    }
}

fn draw_diamond_textured(
    image: &mut PixelImage,
    src: &PixelImage,
    cx: i32,
    cy: i32,
    proj: AngledProjection,
    alpha: f32,
) {
    let left = cx - proj.half_w;
    let right = cx + proj.half_w;
    let top = cy - proj.half_h;
    let bottom = cy + proj.half_h;
    for py in top..=bottom {
        for px in left..=right {
            let dx = (px - cx) as f32 / proj.half_w.max(1) as f32;
            let dy = (py - cy) as f32 / proj.half_h.max(1) as f32;
            let u = (dx - dy) * 0.5 + 0.5;
            let v = (dx + dy) * 0.5 + 0.5;
            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                continue;
            }
            let sx = (u * (src.width.saturating_sub(1)) as f32).round() as u32;
            let sy = (v * (src.height.saturating_sub(1)) as f32).round() as u32;
            blend_pixel_i32(image, px, py, src.get(sx, sy), alpha);
        }
    }
}

fn draw_diamond_overlay(
    image: &mut PixelImage,
    cx: i32,
    cy: i32,
    proj: AngledProjection,
    color: Rgba8,
    alpha: f32,
) {
    if alpha <= 0.001 {
        return;
    }
    let left = cx - proj.half_w;
    let right = cx + proj.half_w;
    let top = cy - proj.half_h;
    let bottom = cy + proj.half_h;
    for py in top..=bottom {
        for px in left..=right {
            if point_in_diamond(px, py, cx, cy, proj) {
                blend_pixel_i32(image, px, py, color, alpha);
            }
        }
    }
}

fn draw_angled_contact_shadow(
    image: &mut PixelImage,
    map: &TerrainMap,
    x: u32,
    y: u32,
    proj: AngledProjection,
    shadow_strength: f32,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    let mut strongest: f32 = 0.0;
    for (nx, ny) in cardinal_neighbors(x, y, map.width, map.height) {
        let neighbor = map
            .cell(nx, ny)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        strongest = strongest.max(current - neighbor);
    }
    if strongest <= 0.01 {
        return;
    }
    let (cx, cy) = proj.cell_center(map, x, y);
    let alpha = (0.06 + strongest * 0.05).min(0.26) * shadow_strength.max(0.1);
    draw_diamond_overlay(
        image,
        cx + proj.half_w / 5,
        cy + proj.half_h + 2,
        proj,
        Rgba8::BLACK,
        alpha,
    );
}

fn draw_angled_exposed_faces(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    x: u32,
    y: u32,
    proj: AngledProjection,
    options: &PreviewOptions,
) {
    let Some(cell) = map.cell(x, y) else {
        return;
    };
    let current = cell.effective_height();
    for (nx, ny) in cardinal_neighbors(x, y, map.width, map.height) {
        let neighbor = map
            .cell(nx, ny)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current <= neighbor + 0.01 {
            continue;
        }
        let delta = current - neighbor;
        let (du, dv) =
            oriented_delta_between(proj.orientation, map.width, map.height, x, y, nx, ny);
        let Some((p0, p1, face_kind, near_face)) =
            angled_edge_points_for_delta(map, proj, x, y, du, dv)
        else {
            continue;
        };
        let material = face_material_for_cell(cell);
        let variant = stable_tile_variant(
            tileset.recipe.seed ^ 0x73cf_519d_21e4_917b,
            x,
            y,
            material,
            tileset.recipe.variants_per_material,
        );
        let face_h = proj.face_height(delta);
        let alpha = angled_face_alpha(map, proj, options, p0, p1, near_face);
        draw_angled_face(
            image,
            tileset,
            AngledFaceDraw {
                material,
                face: face_kind,
                variant,
                p0,
                p1,
                height: face_h,
                alpha,
            },
        );
        if options.show_structure_lips {
            draw_angled_lip(image, tileset, material, variant, p0, p1, alpha.max(0.72));
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct AngledFaceDraw {
    material: GroundMaterial,
    face: StructureFaceKind,
    variant: u32,
    p0: (i32, i32),
    p1: (i32, i32),
    height: u32,
    alpha: f32,
}

fn draw_angled_face(image: &mut PixelImage, tileset: &Tileset, draw: AngledFaceDraw) {
    let p2 = (draw.p1.0, draw.p1.1 + draw.height as i32);
    let p3 = (draw.p0.0, draw.p0.1 + draw.height as i32);
    let min_x = draw.p0.0.min(draw.p1.0).min(p2.0).min(p3.0);
    let max_x = draw.p0.0.max(draw.p1.0).max(p2.0).max(p3.0);
    let min_y = draw.p0.1.min(draw.p1.1).min(p2.1).min(p3.1);
    let max_y = draw.p0.1.max(draw.p1.1).max(p2.1).max(p3.1);
    let src = tileset
        .structure_face_tile(draw.material, draw.face, draw.variant)
        .map(|asset| &asset.image);
    for py in min_y..=max_y {
        for px in min_x..=max_x {
            if !point_in_quad(px, py, draw.p0, draw.p1, p2, p3) {
                continue;
            }
            let u = if max_x == min_x {
                0.0
            } else {
                (px - min_x) as f32 / (max_x - min_x) as f32
            };
            let v = if max_y == min_y {
                0.0
            } else {
                (py - min_y) as f32 / (max_y - min_y) as f32
            };
            let color = if let Some(src) = src {
                let sx = (u * (src.width.saturating_sub(1)) as f32).round() as u32;
                let sy = (v * (src.height.saturating_sub(1)) as f32).round() as u32;
                src.get(sx, sy)
            } else {
                fallback_face_color(tileset, draw.material, draw.face, v)
            };
            blend_pixel_i32(image, px, py, color, draw.alpha);
        }
    }
    draw_line_i32(
        image,
        draw.p0.0,
        draw.p0.1,
        draw.p1.0,
        draw.p1.1,
        Rgba8::opaque(34, 27, 24),
    );
    draw_line_i32(image, p3.0, p3.1, p2.0, p2.1, Rgba8::opaque(17, 15, 14));
}

fn draw_angled_lip(
    image: &mut PixelImage,
    tileset: &Tileset,
    material: GroundMaterial,
    variant: u32,
    p0: (i32, i32),
    p1: (i32, i32),
    alpha: f32,
) {
    let src = tileset
        .structure_face_tile(material, StructureFaceKind::Lip, variant)
        .map(|asset| &asset.image);
    let lip_h = (tileset.recipe.projection.height_step_px / 5).max(2) as i32;
    let min_x = p0.0.min(p1.0) - 1;
    let max_x = p0.0.max(p1.0) + 1;
    let min_y = p0.1.min(p1.1) - lip_h;
    let max_y = p0.1.max(p1.1) + 1;
    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dist = distance_to_segment(px as f32, py as f32, p0, p1);
            if dist > lip_h as f32 {
                continue;
            }
            let along = segment_t(px as f32, py as f32, p0, p1);
            let color = if let Some(src) = src {
                let sx = (along * (src.width.saturating_sub(1)) as f32).round() as u32;
                let sy =
                    ((dist / lip_h as f32) * (src.height.saturating_sub(1)) as f32).round() as u32;
                src.get(sx, sy)
            } else {
                tileset.palette.sample(material.ramp(), 0.62).lighten(0.06)
            };
            blend_pixel_i32(
                image,
                px,
                py,
                color,
                alpha * (1.0 - dist / lip_h as f32 * 0.5),
            );
        }
    }
}

fn fallback_face_color(
    tileset: &Tileset,
    material: GroundMaterial,
    face: StructureFaceKind,
    v: f32,
) -> Rgba8 {
    let base = tileset.palette.sample(material.ramp(), 0.45);
    match face {
        StructureFaceKind::Front => base.darken(0.20 + v * 0.12),
        StructureFaceKind::Left => base.darken(0.30 + v * 0.12),
        StructureFaceKind::Right => base.darken(0.12 + v * 0.08),
        StructureFaceKind::Lip => base.lighten(0.06),
    }
}

fn angled_face_alpha(
    map: &TerrainMap,
    proj: AngledProjection,
    options: &PreviewOptions,
    p0: (i32, i32),
    p1: (i32, i32),
    near_face: bool,
) -> f32 {
    let mut alpha: f32 = if options.fade_raised_faces { 0.80 } else { 1.0 };
    if !near_face {
        alpha = alpha.min(0.88);
    }
    if options.enable_local_cutaway {
        if let Some((cx, cy)) = options.inspect_cell {
            if cx < map.width && cy < map.height {
                let (fx, fy) = proj.cell_center(map, cx, cy);
                let mx = (p0.0 + p1.0) / 2;
                let my = (p0.1 + p1.1) / 2;
                let dx = (mx - fx) as f32;
                let dy = (my - fy) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = 160.0_f32.max((proj.tile_w + proj.tile_h) as f32 * 0.7);
                if dist < radius {
                    let t = clamp01(dist / radius);
                    alpha = alpha.min(0.30 + 0.70 * t);
                }
            }
        }
    }
    alpha.clamp(0.18, 1.0)
}

fn angled_preview_pixel_to_cell(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &PreviewOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let proj = angled_projection(map, tileset, options);
    let px = px as i32;
    let py = py as i32;
    let cells = proj.ordered_cells(map);
    for &(_, _, x, y) in cells.iter().rev() {
        let (cx, cy) = proj.cell_center(map, x, y);
        if point_in_diamond(px, py, cx, cy, proj) {
            return Some((x, y));
        }
        if point_in_angled_any_face(map, x, y, px, py, proj) {
            return Some((x, y));
        }
    }
    None
}

fn point_in_angled_any_face(
    map: &TerrainMap,
    x: u32,
    y: u32,
    px: i32,
    py: i32,
    proj: AngledProjection,
) -> bool {
    let Some(cell) = map.cell(x, y) else {
        return false;
    };
    let current = cell.effective_height();
    for (nx, ny) in cardinal_neighbors(x, y, map.width, map.height) {
        let neighbor = map
            .cell(nx, ny)
            .map(|c| c.effective_height())
            .unwrap_or(current);
        if current <= neighbor + 0.01 {
            continue;
        }
        let (du, dv) =
            oriented_delta_between(proj.orientation, map.width, map.height, x, y, nx, ny);
        let Some((p0, p1, _, _)) = angled_edge_points_for_delta(map, proj, x, y, du, dv) else {
            continue;
        };
        let p2 = (p1.0, p1.1 + proj.face_height(current - neighbor) as i32);
        let p3 = (p0.0, p0.1 + proj.face_height(current - neighbor) as i32);
        if point_in_quad(px, py, p0, p1, p2, p3) {
            return true;
        }
    }
    false
}

fn angled_edge_points_for_delta(
    map: &TerrainMap,
    proj: AngledProjection,
    x: u32,
    y: u32,
    du: i32,
    dv: i32,
) -> Option<AngledEdgePoints> {
    let [top, right, bottom, left] = proj.diamond(map, x, y);
    match (du, dv) {
        (1, 0) => Some((right, bottom, StructureFaceKind::Right, true)),
        (0, 1) => Some((bottom, left, StructureFaceKind::Front, true)),
        (-1, 0) => Some((left, top, StructureFaceKind::Left, false)),
        (0, -1) => Some((top, right, StructureFaceKind::Right, false)),
        _ => None,
    }
}

fn draw_angled_grid_outline(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: AngledProjection,
    x: u32,
    y: u32,
    color: Rgba8,
) {
    let [top, right, bottom, left] = proj.diamond(map, x, y);
    draw_line_i32(image, top.0, top.1, right.0, right.1, color);
    draw_line_i32(image, right.0, right.1, bottom.0, bottom.1, color);
    draw_line_i32(image, bottom.0, bottom.1, left.0, left.1, color);
    draw_line_i32(image, left.0, left.1, top.0, top.1, color);
}

fn draw_diamond_outline(
    image: &mut PixelImage,
    cx: i32,
    cy: i32,
    proj: AngledProjection,
    color: Rgba8,
) {
    let top = (cx, cy - proj.half_h);
    let right = (cx + proj.half_w, cy);
    let bottom = (cx, cy + proj.half_h);
    let left = (cx - proj.half_w, cy);
    draw_line_i32(image, top.0, top.1, right.0, right.1, color);
    draw_line_i32(image, right.0, right.1, bottom.0, bottom.1, color);
    draw_line_i32(image, bottom.0, bottom.1, left.0, left.1, color);
    draw_line_i32(image, left.0, left.1, top.0, top.1, color);
}

fn draw_angled_selection(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: AngledProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    if cell.0 >= map.width || cell.1 >= map.height {
        return;
    }
    let (cx, cy) = proj.cell_center(map, cell.0, cell.1);
    draw_diamond_outline(image, cx, cy, proj, color);
    draw_diamond_overlay(image, cx, cy, proj, color, 0.12);
}

fn draw_angled_marker(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: AngledProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    if cell.0 >= map.width || cell.1 >= map.height {
        return;
    }
    let (cx, cy) = proj.cell_center(map, cell.0, cell.1);
    let r = (proj.half_h / 2).max(4);
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                blend_pixel_i32(image, cx + dx, cy + dy, color, 0.94);
            }
        }
    }
    draw_diamond_outline(image, cx, cy, proj, Rgba8::opaque(8, 9, 10));
}

fn draw_angled_path(
    image: &mut PixelImage,
    map: &TerrainMap,
    proj: AngledProjection,
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
        draw_line_i32(image, ax, ay, bx, by, Rgba8::opaque(9, 8, 7));
        draw_line_i32(image, ax + 1, ay, bx + 1, by, Rgba8::opaque(9, 8, 7));
        draw_line_i32(image, ax, ay - 2, bx, by - 2, color);
        draw_line_i32(image, ax + 1, ay - 2, bx + 1, by - 2, color);
    }
}

fn oriented_dimensions(orientation: ViewOrientation, width: u32, height: u32) -> (u32, u32) {
    match orientation {
        ViewOrientation::SouthEast | ViewOrientation::NorthWest => (width, height),
        ViewOrientation::NorthEast | ViewOrientation::SouthWest => (height, width),
    }
}

fn world_to_oriented(
    orientation: ViewOrientation,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> (u32, u32) {
    match orientation {
        ViewOrientation::SouthEast => (x, y),
        ViewOrientation::SouthWest => (height.saturating_sub(1).saturating_sub(y), x),
        ViewOrientation::NorthWest => (
            width.saturating_sub(1).saturating_sub(x),
            height.saturating_sub(1).saturating_sub(y),
        ),
        ViewOrientation::NorthEast => (y, width.saturating_sub(1).saturating_sub(x)),
    }
}

fn oriented_delta_between(
    orientation: ViewOrientation,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    nx: u32,
    ny: u32,
) -> (i32, i32) {
    let (u0, v0) = world_to_oriented(orientation, width, height, x, y);
    let (u1, v1) = world_to_oriented(orientation, width, height, nx, ny);
    (u1 as i32 - u0 as i32, v1 as i32 - v0 as i32)
}

fn cardinal_neighbors(x: u32, y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if x + 1 < width {
        out.push((x + 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    if y + 1 < height {
        out.push((x, y + 1));
    }
    out
}

fn point_in_diamond(px: i32, py: i32, cx: i32, cy: i32, proj: AngledProjection) -> bool {
    let dx = (px - cx).abs() as f32 / proj.half_w.max(1) as f32;
    let dy = (py - cy).abs() as f32 / proj.half_h.max(1) as f32;
    dx + dy <= 1.0
}

fn point_in_quad(
    px: i32,
    py: i32,
    a: (i32, i32),
    b: (i32, i32),
    c: (i32, i32),
    d: (i32, i32),
) -> bool {
    let p = (px as f32, py as f32);
    point_in_triangle(p, a, b, c) || point_in_triangle(p, a, c, d)
}

fn point_in_triangle(p: (f32, f32), a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let a = (a.0 as f32, a.1 as f32);
    let b = (b.0 as f32, b.1 as f32);
    let c = (c.0 as f32, c.1 as f32);
    let area = signed_area(a, b, c);
    if area.abs() < 0.001 {
        return false;
    }
    let s = signed_area(p, b, c) / area;
    let t = signed_area(a, p, c) / area;
    let u = signed_area(a, b, p) / area;
    s >= -0.001 && t >= -0.001 && u >= -0.001
}

fn signed_area(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

fn distance_to_segment(px: f32, py: f32, a: (i32, i32), b: (i32, i32)) -> f32 {
    let t = segment_t(px, py, a, b);
    let ax = a.0 as f32;
    let ay = a.1 as f32;
    let bx = b.0 as f32;
    let by = b.1 as f32;
    let qx = ax + (bx - ax) * t;
    let qy = ay + (by - ay) * t;
    let dx = px - qx;
    let dy = py - qy;
    (dx * dx + dy * dy).sqrt()
}

fn segment_t(px: f32, py: f32, a: (i32, i32), b: (i32, i32)) -> f32 {
    let ax = a.0 as f32;
    let ay = a.1 as f32;
    let bx = b.0 as f32;
    let by = b.1 as f32;
    let dx = bx - ax;
    let dy = by - ay;
    let denom = dx * dx + dy * dy;
    if denom <= 0.001 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / denom).clamp(0.0, 1.0)
    }
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
