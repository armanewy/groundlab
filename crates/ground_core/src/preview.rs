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
    Height,
    Slope,
    MovementCost,
    Route,
    LineOfSight,
}

impl PreviewMode {
    pub const ALL: [PreviewMode; 6] = [
        PreviewMode::Material,
        PreviewMode::Height,
        PreviewMode::Slope,
        PreviewMode::MovementCost,
        PreviewMode::Route,
        PreviewMode::LineOfSight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PreviewMode::Material => "Material",
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
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            show_grid: true,
            los_source: (8, 8),
            los_range: 18,
        }
    }
}

pub fn render_terrain_preview(
    map: &TerrainMap,
    tileset: &Tileset,
    mode: PreviewMode,
    options: &PreviewOptions,
) -> PixelImage {
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
                PreviewMode::Material => {
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
