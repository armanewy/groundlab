use crate::color::{clamp01, Rgba8};
use crate::hero_scene::HeroScene;
use crate::pathfinding::find_path;
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, ViewOrientation};
use crate::target_style::{
    StampPiece, TerrainStampDefinition, TerrainStampKind, TerrainStampResolver,
};
use crate::terrain::TerrainMap;
use crate::terrain_artkit::{TerrainArtKit, TerrainArtPieceKind, TerrainArtRepeatMode};
use crate::tileset::Tileset;

/// Milestone 4.9 renderer options.
///
/// This intentionally does not depend on `preview::PreviewOptions` so this module can remain
/// a renderer/art-composition layer rather than part of the workbench UI.
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

#[derive(Clone, Copy, Debug)]
struct TargetProjection {
    cell_w: u32,
    cell_h: u32,
    face_h: u32,
    offset_x: i32,
    offset_y: i32,
    width: u32,
    height: u32,
    orientation: ViewOrientation,
}

#[derive(Clone, Copy, Debug)]
struct TargetRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Render the editable terrain using the Milestone 4.9 target-look composition pass.
///
/// This is not a static backdrop. It resolves the current `TerrainMap` into target-style stamps,
/// composes roads/trenches/berms/stone platforms with feature-specific logic, then draws the
/// existing hero-scene dressing and optional tactical overlays.
pub fn render_target_look_scene(
    map: &TerrainMap,
    tileset: &Tileset,
    options: &TargetLookRenderOptions,
) -> PixelImage {
    let projection = target_projection(map, options);
    let artkit = TerrainArtKit::load_default_or_generate(tileset);
    let stamps = TerrainStampResolver::resolve(map);
    let mut image = PixelImage::new(
        projection.width,
        projection.height,
        Rgba8::opaque(11, 12, 15),
    );

    draw_base_field(&mut image, map, tileset, projection);
    draw_connected_target_stamps(&mut image, map, tileset, &artkit, projection, &stamps);

    let hero_scene = HeroScene::load_default_or_builtin();
    draw_hero_scene(&mut image, map, &artkit, projection, &hero_scene);

    apply_scene_lighting(&mut image, tileset, projection);

    if options.show_route {
        let path = find_path(map, map.spawn, map.objective);
        draw_route(
            &mut image,
            map,
            projection,
            &path.points,
            if path.reached_goal {
                Rgba8::opaque(235, 196, 91)
            } else {
                Rgba8::opaque(230, 74, 74)
            },
        );
    }

    if options.show_markers {
        draw_marker(
            &mut image,
            map,
            projection,
            map.spawn,
            Rgba8::opaque(99, 169, 218),
        );
        draw_marker(
            &mut image,
            map,
            projection,
            map.objective,
            Rgba8::opaque(225, 196, 91),
        );
    }

    if options.show_grid {
        for y in 0..map.height {
            for x in 0..map.width {
                draw_grid_cell(&mut image, map, projection, x, y, Rgba8::opaque(24, 28, 31));
            }
        }
    }

    if let Some(cell) = options.inspect_cell {
        draw_selection(
            &mut image,
            map,
            projection,
            cell,
            Rgba8::opaque(154, 215, 238),
        );
    }

    if options.show_debug {
        for stamp in &stamps {
            draw_stamp_debug(&mut image, map, projection, stamp);
        }
    }

    image
}

pub fn target_look_pixel_to_cell(
    map: &TerrainMap,
    _tileset: &Tileset,
    options: &TargetLookRenderOptions,
    px: u32,
    py: u32,
) -> Option<(u32, u32)> {
    let projection = target_projection(map, options);
    let px = px as i32;
    let py = py as i32;
    let mut cells = target_draw_order(map, projection.orientation);
    cells.reverse();
    for (x, y) in cells {
        let (sx, sy) = projection.cell_top_left(map, x, y);
        if px >= sx
            && py >= sy
            && px < sx + projection.cell_w as i32
            && py < sy + projection.cell_h as i32 + projection.face_h as i32
        {
            return Some((x, y));
        }
    }
    None
}

impl TargetProjection {
    fn cell_top_left(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let h = map
            .cell(x, y)
            .map(|cell| cell.effective_height())
            .unwrap_or(0.0);
        let (vx, vy) = world_to_target_view(self.orientation, map, x, y);
        let sx = vx * self.cell_w as i32 + self.offset_x;
        let sy = vy * self.cell_h as i32 + self.offset_y
            - (h * self.face_h as f32 * 0.42).round() as i32;
        (sx, sy)
    }

    fn cell_center(self, map: &TerrainMap, x: u32, y: u32) -> (i32, i32) {
        let (sx, sy) = self.cell_top_left(map, x, y);
        (sx + self.cell_w as i32 / 2, sy + self.cell_h as i32 / 2)
    }
}

fn target_projection(map: &TerrainMap, options: &TargetLookRenderOptions) -> TargetProjection {
    // These values are deliberately closer to the generated target: wider cells, lower top-down
    // compression, and a stronger front-face budget than the older faux renderer.
    let cell_w = 112;
    let cell_h = 82;
    let face_h = options.height_step_px.clamp(28, 52);
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
            let (vx, vy) = world_to_target_view(orientation, map, x, y);
            let sx = vx * cell_w as i32;
            let sy = vy * cell_h as i32 - (h * face_h as f32 * 0.42).round() as i32;
            let extra = ((h.max(0.0) + 2.0) * face_h as f32 * 0.75).ceil() as i32;
            min_x = min_x.min(sx - cell_w as i32 / 2);
            min_y = min_y.min(sy - cell_h as i32 / 2 - face_h as i32);
            max_x = max_x.max(sx + cell_w as i32 + cell_w as i32 / 2);
            max_y = max_y.max(sy + cell_h as i32 + extra + cell_h as i32 / 2);
        }
    }

    let padding = 86;
    TargetProjection {
        cell_w,
        cell_h,
        face_h,
        offset_x: -min_x + padding,
        offset_y: -min_y + padding,
        width: (max_x - min_x + padding * 2).max(cell_w as i32) as u32,
        height: (max_y - min_y + padding * 2).max(cell_h as i32) as u32,
        orientation,
    }
}

fn world_to_target_view(
    orientation: ViewOrientation,
    map: &TerrainMap,
    x: u32,
    y: u32,
) -> (i32, i32) {
    match orientation {
        ViewOrientation::SouthEast => (x as i32, y as i32),
        ViewOrientation::SouthWest => ((map.height - 1 - y) as i32, x as i32),
        ViewOrientation::NorthWest => ((map.width - 1 - x) as i32, (map.height - 1 - y) as i32),
        ViewOrientation::NorthEast => (y as i32, (map.width - 1 - x) as i32),
    }
}

fn target_draw_order(map: &TerrainMap, orientation: ViewOrientation) -> Vec<(u32, u32)> {
    let mut cells = Vec::with_capacity(map.width as usize * map.height as usize);
    for y in 0..map.height {
        for x in 0..map.width {
            let (vx, vy) = world_to_target_view(orientation, map, x, y);
            cells.push((x, y, vx + vy, vy, vx));
        }
    }
    cells.sort_by_key(|(_, _, diag, vy, vx)| (*diag, *vy, *vx));
    cells.into_iter().map(|(x, y, _, _, _)| (x, y)).collect()
}

fn draw_base_field(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
) {
    let Some(bounds) = map_bounds(map, projection) else {
        return;
    };
    let grass = tileset.palette.sample(GroundMaterial::Grass.ramp(), 0.48);
    draw_soft_shadow(
        image,
        bounds.x - 24,
        bounds.y + bounds.height as i32 - 36,
        bounds.width + 48,
        80,
        0.25,
    );
    draw_blob(
        image,
        TargetRect {
            x: bounds.x - 28,
            y: bounds.y - 24,
            width: bounds.width + 56,
            height: bounds.height + 48,
        },
        grass.darken(0.03),
        0x1111,
        1.0,
        0.10,
    );

    // Painterly grass: larger clusters and edge darkness instead of a visible grid texture.
    for i in 0..520 {
        let x = bounds.x + (hash3(i, bounds.width, 0x1201) % bounds.width.max(1)) as i32;
        let y = bounds.y + (hash3(i, bounds.height, 0x1202) % bounds.height.max(1)) as i32;
        let n = noise01(0x1203, i, bounds.width ^ bounds.height);
        let color = if n > 0.58 {
            grass.lighten(0.12)
        } else if n < 0.22 {
            grass.darken(0.13)
        } else {
            grass
        };
        let w = 2 + (hash3(i, 5, 0x1204) % 6);
        let h = 1 + (hash3(i, 3, 0x1205) % 4);
        blend_rect(image, x, y, w, h, color, 0.11);
    }

    // Border vignette in-scene, not only a post effect.
    draw_soft_shadow(
        image,
        bounds.x - 50,
        bounds.y - 48,
        bounds.width + 100,
        54,
        0.22,
    );
    draw_soft_shadow(image, bounds.x - 54, bounds.y, 54, bounds.height + 64, 0.20);
}

fn draw_connected_target_stamps(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamps: &[TerrainStampDefinition],
) {
    // Floor-like features first.
    for stamp in stamps {
        match stamp.kind {
            TerrainStampKind::GrassFieldPatch => {
                draw_grass_stamp(image, map, tileset, projection, stamp)
            }
            TerrainStampKind::DirtRoadSegment | TerrainStampKind::DirtRoadJunction => {
                draw_road_stamp(image, map, tileset, projection, stamp)
            }
            TerrainStampKind::MudPatch => draw_mud_stamp(image, map, tileset, projection, stamp),
            TerrainStampKind::StonePlatform => {
                draw_stone_platform_stamp(image, map, tileset, artkit, projection, stamp)
            }
            _ => {}
        }
    }

    // Terrain body and obstacles second.
    for stamp in stamps {
        match stamp.kind {
            TerrainStampKind::TrenchStraight
            | TerrainStampKind::TrenchCorner
            | TerrainStampKind::TrenchEndCap => {
                draw_trench_stamp(image, map, tileset, artkit, projection, stamp)
            }
            TerrainStampKind::BermStraight | TerrainStampKind::BermCorner => {
                draw_berm_stamp(image, map, tileset, artkit, projection, stamp)
            }
            TerrainStampKind::GrassTuftCluster
            | TerrainStampKind::RockScatter
            | TerrainStampKind::CastShadow => {
                draw_stamp_pieces(image, map, artkit, projection, stamp);
            }
            _ => {}
        }
    }
}

fn draw_grass_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let grass = tileset.palette.sample(GroundMaterial::Grass.ramp(), 0.62);
    for &(x, y) in &stamp.cells {
        if !hash3(x, y, 0x2121).is_multiple_of(6) {
            continue;
        }
        let (cx, cy) = projection.cell_center(map, x, y);
        draw_grass_tufts(
            image,
            TargetRect {
                x: cx - projection.cell_w as i32 / 3,
                y: cy - projection.cell_h as i32 / 3,
                width: projection.cell_w * 2 / 3,
                height: projection.cell_h * 2 / 3,
            },
            grass,
            hash3(x, y, 0x2122),
        );
    }
}

fn draw_road_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let dirt = tileset.palette.sample(GroundMaterial::Dirt.ramp(), 0.55);
    let packed = dirt.darken(0.06);

    // Connect neighboring road cells with wide organic corridors first.
    for &(x, y) in &stamp.cells {
        for (dx, dy) in [(1_i32, 0_i32), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0
                || ny < 0
                || nx >= map.width as i32
                || ny >= map.height as i32
                || !stamp.cells.contains(&(nx as u32, ny as u32))
            {
                continue;
            }
            let (cx, cy) = projection.cell_center(map, x, y);
            let (nx, ny) = projection.cell_center(map, nx as u32, ny as u32);
            let rect = TargetRect {
                x: cx.min(nx) - projection.cell_w as i32 / 2,
                y: cy.min(ny) - projection.cell_h as i32 / 2,
                width: (cx - nx).unsigned_abs() + projection.cell_w,
                height: (cy - ny).unsigned_abs() + projection.cell_h,
            };
            draw_noisy_ellipse(image, rect, packed, 0.72, hash3(x, y, 0x3131));
        }
    }

    for &(x, y) in &stamp.cells {
        let (cx, cy) = projection.cell_center(map, x, y);
        let rect = TargetRect {
            x: cx - projection.cell_w as i32 / 2,
            y: cy - projection.cell_h as i32 / 2,
            width: projection.cell_w,
            height: projection.cell_h,
        };
        draw_noisy_ellipse(image, rect, dirt, 0.88, hash3(x, y, 0x3132));
        draw_road_detail(image, rect, tileset, hash3(x, y, 0x3133));
    }

    draw_component_edge_dust(image, map, tileset, projection, stamp, GroundMaterial::Dirt);
}

fn draw_mud_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let mud = tileset.palette.sample(GroundMaterial::Mud.ramp(), 0.34);
    for &(x, y) in &stamp.cells {
        let (cx, cy) = projection.cell_center(map, x, y);
        let rect = TargetRect {
            x: cx - projection.cell_w as i32 / 2,
            y: cy - projection.cell_h as i32 / 2,
            width: projection.cell_w,
            height: projection.cell_h,
        };
        draw_noisy_ellipse(image, rect, mud, 0.78, hash3(x, y, 0x4141));
        draw_soft_shadow(
            image,
            rect.x + 10,
            rect.y + rect.height as i32 / 2,
            rect.width.saturating_sub(20),
            rect.height / 3,
            0.10,
        );
    }
}

fn draw_stone_platform_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let (x0, y0, w_cells, h_cells) = cells_bounds(&stamp.cells);
    let rect = rect_for_cells(map, projection, x0, y0, w_cells, h_cells);
    let stone_top = tileset.palette.sample(GroundMaterial::Rock.ramp(), 0.60);
    let stone_side = tileset.palette.sample(GroundMaterial::Rock.ramp(), 0.36);
    let face_h = (projection.face_h + 10).max(34);

    draw_soft_shadow(
        image,
        rect.x + 12,
        rect.y + rect.height as i32 + 12,
        rect.width,
        face_h + 16,
        0.24,
    );

    draw_blob(
        image,
        TargetRect {
            x: rect.x - 8,
            y: rect.y - 8,
            width: rect.width + 16,
            height: rect.height + 12,
        },
        stone_top,
        hash3(x0, y0, 0x5151),
        0.96,
        0.18,
    );
    blend_rect(
        image,
        rect.x,
        rect.y + rect.height as i32 - 2,
        rect.width,
        face_h,
        stone_side,
        0.94,
    );

    draw_stone_lines(
        image,
        rect.x,
        rect.y,
        rect.width,
        rect.height + face_h,
        stone_top.darken(0.34),
    );

    // Simple front steps, similar to the target image.
    let step_w = (rect.width / 4).max(64).min(rect.width);
    let step_x = rect.x + rect.width as i32 / 2 - step_w as i32 / 2;
    for i in 0..3 {
        let step_y = rect.y + rect.height as i32 + (i as i32 * (face_h as i32 / 4).max(6));
        let inset = i * 10;
        blend_rect(
            image,
            step_x - inset as i32,
            step_y,
            step_w + inset * 2,
            (face_h / 5).max(7),
            stone_top.darken(0.12 + i as f32 * 0.05),
            0.90,
        );
    }

    draw_stamp_pieces(image, map, artkit, projection, stamp);
}

fn draw_trench_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let floor = tileset
        .palette
        .sample(GroundMaterial::TrenchFloor.ramp(), 0.20);
    let wall = tileset
        .palette
        .sample(GroundMaterial::TrenchWall.ramp(), 0.45);

    for &(x, y) in &stamp.cells {
        let (cx, cy) = projection.cell_center(map, x, y);
        let outer = TargetRect {
            x: cx - projection.cell_w as i32 / 2 - 8,
            y: cy - projection.cell_h as i32 / 2 - 6,
            width: projection.cell_w + 16,
            height: projection.cell_h + 18,
        };
        draw_soft_shadow(
            image,
            outer.x + 5,
            outer.y + 8,
            outer.width,
            outer.height,
            0.30,
        );
        draw_blob(
            image,
            outer,
            wall.lighten(0.02),
            hash3(x, y, 0x6161),
            0.90,
            0.22,
        );

        let inner = TargetRect {
            x: outer.x + (projection.cell_w / 8) as i32,
            y: outer.y + (projection.cell_h / 6) as i32,
            width: outer.width.saturating_sub(projection.cell_w / 4),
            height: outer.height.saturating_sub(projection.cell_h / 3),
        };
        draw_blob(image, inner, floor, hash3(x, y, 0x6162), 0.97, 0.16);
        draw_wood_planks(image, inner, wall.darken(0.45), hash3(x, y, 0x6163));

        draw_trench_lips(image, map, tileset, projection, stamp, x, y);
    }

    draw_stamp_pieces(image, map, artkit, projection, stamp);
}

fn draw_berm_stamp(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let mound = tileset
        .palette
        .sample(GroundMaterial::BermFace.ramp(), 0.50);
    for &(x, y) in &stamp.cells {
        let (cx, cy) = projection.cell_center(map, x, y);
        let rect = TargetRect {
            x: cx - projection.cell_w as i32 / 2,
            y: cy - projection.cell_h as i32 / 3,
            width: projection.cell_w,
            height: (projection.cell_h * 2 / 3).max(38),
        };
        draw_soft_shadow(
            image,
            rect.x + 8,
            rect.y + rect.height as i32 - 8,
            rect.width,
            28,
            0.18,
        );
        draw_noisy_ellipse(image, rect, mound, 0.90, hash3(x, y, 0x7171));
        draw_berm_stones(image, rect, mound.darken(0.28), hash3(x, y, 0x7172));
    }
    draw_stamp_pieces(image, map, artkit, projection, stamp);
}

fn draw_stamp_pieces(
    image: &mut PixelImage,
    map: &TerrainMap,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    for piece in &stamp.pieces {
        draw_stamp_piece(image, map, artkit, projection, stamp, piece);
    }
}

fn draw_stamp_piece(
    image: &mut PixelImage,
    map: &TerrainMap,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
    piece: &StampPiece,
) {
    let (x0, y0, w, h) = cells_bounds(&stamp.cells);
    let rect = rect_for_cells(map, projection, x0, y0, w, h);
    let dst = TargetRect {
        x: rect.x + piece.offset_px.0,
        y: rect.y + piece.offset_px.1,
        width: piece.size_px.0.max(1),
        height: piece.size_px.1.max(1),
    };
    draw_art_piece(
        image,
        artkit,
        piece.piece_kind,
        dst,
        piece.opacity,
        piece.seed,
    );
}

fn draw_hero_scene(
    image: &mut PixelImage,
    map: &TerrainMap,
    artkit: &TerrainArtKit,
    projection: TargetProjection,
    hero: &HeroScene,
) {
    let mut placements = hero.placements.clone();
    placements.sort_by_key(|p| (p.z_bias, p.cell.1, p.cell.0, p.id.clone()));
    for placement in placements {
        if placement.cell.0 >= map.width || placement.cell.1 >= map.height {
            continue;
        }
        let (cx, cy) = projection.cell_center(map, placement.cell.0, placement.cell.1);
        let dst = TargetRect {
            x: cx - placement.size_px.0 as i32 / 2 + placement.offset_px.0,
            y: cy - placement.size_px.1 as i32 / 2 + placement.offset_px.1,
            width: placement.size_px.0,
            height: placement.size_px.1,
        };
        draw_art_piece(
            image,
            artkit,
            placement.piece_kind,
            dst,
            placement.opacity,
            placement.seed,
        );
    }
}

fn draw_art_piece(
    image: &mut PixelImage,
    artkit: &TerrainArtKit,
    kind: TerrainArtPieceKind,
    dst: TargetRect,
    alpha: f32,
    seed: u32,
) -> bool {
    let Some(piece) = artkit.piece_variant(kind, seed) else {
        return false;
    };
    let alpha = alpha * piece.definition.opacity;
    match piece.definition.repeat_mode {
        TerrainArtRepeatMode::Tile => draw_tiled(image, &piece.image, dst, alpha, seed),
        TerrainArtRepeatMode::Stretch | TerrainArtRepeatMode::StretchMiddle => {
            draw_scaled(image, &piece.image, dst, alpha)
        }
        TerrainArtRepeatMode::Stamp => {
            let w = dst.width.min(piece.image.width).max(1);
            let h = dst.height.min(piece.image.height).max(1);
            draw_scaled(
                image,
                &piece.image,
                TargetRect {
                    x: dst.x + (dst.width as i32 - w as i32) / 2,
                    y: dst.y + (dst.height as i32 - h as i32) / 2,
                    width: w,
                    height: h,
                },
                alpha,
            );
        }
    }
    true
}

fn draw_scaled(image: &mut PixelImage, src: &PixelImage, dst: TargetRect, alpha: f32) {
    if src.width == 0 || src.height == 0 || dst.width == 0 || dst.height == 0 {
        return;
    }
    for yy in 0..dst.height {
        for xx in 0..dst.width {
            let sx = (xx * src.width / dst.width).min(src.width - 1);
            let sy = (yy * src.height / dst.height).min(src.height - 1);
            let c = src.get(sx, sy);
            if c.a > 0 {
                blend_px(
                    image,
                    dst.x + xx as i32,
                    dst.y + yy as i32,
                    c,
                    alpha * c.a as f32 / 255.0,
                );
            }
        }
    }
}

fn draw_tiled(image: &mut PixelImage, src: &PixelImage, dst: TargetRect, alpha: f32, seed: u32) {
    if src.width == 0 || src.height == 0 || dst.width == 0 || dst.height == 0 {
        return;
    }
    let ox = seed % src.width;
    let oy = (seed / 17) % src.height;
    for yy in 0..dst.height {
        for xx in 0..dst.width {
            let sx = (xx + ox) % src.width;
            let sy = (yy + oy) % src.height;
            let c = src.get(sx, sy);
            if c.a > 0 {
                blend_px(
                    image,
                    dst.x + xx as i32,
                    dst.y + yy as i32,
                    c,
                    alpha * c.a as f32 / 255.0,
                );
            }
        }
    }
}

fn draw_component_edge_dust(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
    material: GroundMaterial,
) {
    let dust = tileset.palette.sample(material.ramp(), 0.58).lighten(0.07);
    for &(x, y) in &stamp.cells {
        let dirs = [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)];
        for (dx, dy) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && ny >= 0
                && nx < map.width as i32
                && ny < map.height as i32
                && stamp.cells.contains(&(nx as u32, ny as u32))
            {
                continue;
            }
            let (cx, cy) = projection.cell_center(map, x, y);
            let (ex, ey, ew, eh) = match (dx, dy) {
                (0, -1) => (
                    cx - projection.cell_w as i32 / 2,
                    cy - projection.cell_h as i32 / 2,
                    projection.cell_w,
                    14,
                ),
                (0, 1) => (
                    cx - projection.cell_w as i32 / 2,
                    cy + projection.cell_h as i32 / 2 - 14,
                    projection.cell_w,
                    16,
                ),
                (-1, 0) => (
                    cx - projection.cell_w as i32 / 2,
                    cy - projection.cell_h as i32 / 2,
                    14,
                    projection.cell_h,
                ),
                _ => (
                    cx + projection.cell_w as i32 / 2 - 14,
                    cy - projection.cell_h as i32 / 2,
                    14,
                    projection.cell_h,
                ),
            };
            draw_blob(
                image,
                TargetRect {
                    x: ex,
                    y: ey,
                    width: ew,
                    height: eh,
                },
                dust,
                hash3(x, y, 0x8282),
                0.22,
                0.28,
            );
        }
    }
}

fn draw_trench_lips(
    image: &mut PixelImage,
    map: &TerrainMap,
    tileset: &Tileset,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
    x: u32,
    y: u32,
) {
    let lip = tileset
        .palette
        .sample(GroundMaterial::TrenchWall.ramp(), 0.58)
        .lighten(0.06);
    let (cx, cy) = projection.cell_center(map, x, y);
    let dirs = [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)];
    for (dx, dy) in dirs {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        let exposed = nx < 0
            || ny < 0
            || nx >= map.width as i32
            || ny >= map.height as i32
            || !stamp.cells.contains(&(nx as u32, ny as u32));
        if !exposed {
            continue;
        }
        let rect = match (dx, dy) {
            (0, -1) => TargetRect {
                x: cx - projection.cell_w as i32 / 2,
                y: cy - projection.cell_h as i32 / 2,
                width: projection.cell_w,
                height: 10,
            },
            (0, 1) => TargetRect {
                x: cx - projection.cell_w as i32 / 2,
                y: cy + projection.cell_h as i32 / 2 - 12,
                width: projection.cell_w,
                height: 12,
            },
            (-1, 0) => TargetRect {
                x: cx - projection.cell_w as i32 / 2,
                y: cy - projection.cell_h as i32 / 3,
                width: 12,
                height: projection.cell_h * 2 / 3,
            },
            _ => TargetRect {
                x: cx + projection.cell_w as i32 / 2 - 12,
                y: cy - projection.cell_h as i32 / 3,
                width: 12,
                height: projection.cell_h * 2 / 3,
            },
        };
        draw_blob(image, rect, lip, hash3(x, y, 0x9292), 0.70, 0.12);
    }
}

fn apply_scene_lighting(image: &mut PixelImage, tileset: &Tileset, projection: TargetProjection) {
    let warm = tileset
        .palette
        .sample(GroundMaterial::Dirt.ramp(), 0.70)
        .lighten(0.12);
    let cool = Rgba8::opaque(10, 16, 18);
    let w = image.width.max(1);
    let h = image.height.max(1);
    let center_x = w as f32 * 0.47;
    let center_y = h as f32 * 0.46;
    for y in 0..h {
        for x in 0..w {
            let nx = x as f32 / w as f32;
            let ny = y as f32 / h as f32;
            let light = (1.0 - (nx * 0.65 + ny * 0.80)).clamp(0.0, 1.0);
            if light > 0.52 {
                blend_px(image, x as i32, y as i32, warm, (light - 0.52) * 0.10);
            }
            let dx = (x as f32 - center_x) / (w as f32 * 0.55);
            let dy = (y as f32 - center_y) / (h as f32 * 0.55);
            let vignette = ((dx * dx + dy * dy).sqrt() - 0.58).clamp(0.0, 1.0);
            blend_px(image, x as i32, y as i32, cool, vignette * 0.32);
        }
    }

    // A subtle south/east shadow bias anchors raised features.
    let _ = projection;
}

fn draw_route(
    image: &mut PixelImage,
    map: &TerrainMap,
    projection: TargetProjection,
    points: &[(u32, u32)],
    color: Rgba8,
) {
    for pair in points.windows(2) {
        let (x0, y0) = projection.cell_center(map, pair[0].0, pair[0].1);
        let (x1, y1) = projection.cell_center(map, pair[1].0, pair[1].1);
        image.draw_line(x0, y0, x1, y1, color.darken(0.30));
        image.draw_line(x0, y0 - 1, x1, y1 - 1, color);
    }
}

fn draw_marker(
    image: &mut PixelImage,
    map: &TerrainMap,
    projection: TargetProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    let (cx, cy) = projection.cell_center(map, cell.0, cell.1);
    draw_noisy_ellipse(
        image,
        TargetRect {
            x: cx - 11,
            y: cy - 11,
            width: 22,
            height: 22,
        },
        color,
        0.92,
        hash3(cell.0, cell.1, 0xa0a0),
    );
    draw_noisy_ellipse(
        image,
        TargetRect {
            x: cx - 15,
            y: cy - 15,
            width: 30,
            height: 30,
        },
        color.lighten(0.30),
        0.22,
        hash3(cell.0, cell.1, 0xa0a1),
    );
}

fn draw_grid_cell(
    image: &mut PixelImage,
    map: &TerrainMap,
    projection: TargetProjection,
    x: u32,
    y: u32,
    color: Rgba8,
) {
    let (sx, sy) = projection.cell_top_left(map, x, y);
    image.draw_line(sx, sy, sx + projection.cell_w as i32, sy, color);
    image.draw_line(
        sx,
        sy + projection.cell_h as i32,
        sx + projection.cell_w as i32,
        sy + projection.cell_h as i32,
        color,
    );
    image.draw_line(sx, sy, sx, sy + projection.cell_h as i32, color);
    image.draw_line(
        sx + projection.cell_w as i32,
        sy,
        sx + projection.cell_w as i32,
        sy + projection.cell_h as i32,
        color,
    );
}

fn draw_selection(
    image: &mut PixelImage,
    map: &TerrainMap,
    projection: TargetProjection,
    cell: (u32, u32),
    color: Rgba8,
) {
    draw_grid_cell(image, map, projection, cell.0, cell.1, color);
    let (cx, cy) = projection.cell_center(map, cell.0, cell.1);
    draw_noisy_ellipse(
        image,
        TargetRect {
            x: cx - 6,
            y: cy - 6,
            width: 12,
            height: 12,
        },
        color,
        0.85,
        hash3(cell.0, cell.1, 0xb0b0),
    );
}

fn draw_stamp_debug(
    image: &mut PixelImage,
    map: &TerrainMap,
    projection: TargetProjection,
    stamp: &TerrainStampDefinition,
) {
    let (x0, y0, w, h) = cells_bounds(&stamp.cells);
    let rect = rect_for_cells(map, projection, x0, y0, w, h);
    let color = match stamp.kind {
        TerrainStampKind::GrassFieldPatch => Rgba8::opaque(84, 170, 95),
        TerrainStampKind::DirtRoadSegment | TerrainStampKind::DirtRoadJunction => {
            Rgba8::opaque(218, 162, 88)
        }
        TerrainStampKind::TrenchStraight
        | TerrainStampKind::TrenchCorner
        | TerrainStampKind::TrenchEndCap => Rgba8::opaque(82, 184, 230),
        TerrainStampKind::BermStraight | TerrainStampKind::BermCorner => {
            Rgba8::opaque(228, 198, 76)
        }
        TerrainStampKind::StonePlatform => Rgba8::opaque(170, 188, 198),
        TerrainStampKind::MudPatch => Rgba8::opaque(82, 102, 134),
        _ => Rgba8::opaque(220, 220, 220),
    };
    image.draw_line(rect.x, rect.y, rect.x + rect.width as i32, rect.y, color);
    image.draw_line(
        rect.x,
        rect.y + rect.height as i32,
        rect.x + rect.width as i32,
        rect.y + rect.height as i32,
        color,
    );
    image.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height as i32, color);
    image.draw_line(
        rect.x + rect.width as i32,
        rect.y,
        rect.x + rect.width as i32,
        rect.y + rect.height as i32,
        color,
    );
}

fn map_bounds(map: &TerrainMap, projection: TargetProjection) -> Option<TargetRect> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for y in 0..map.height {
        for x in 0..map.width {
            let (sx, sy) = projection.cell_top_left(map, x, y);
            min_x = min_x.min(sx);
            min_y = min_y.min(sy);
            max_x = max_x.max(sx + projection.cell_w as i32);
            max_y = max_y.max(sy + projection.cell_h as i32);
        }
    }
    if min_x < max_x && min_y < max_y {
        Some(TargetRect {
            x: min_x,
            y: min_y,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        })
    } else {
        None
    }
}

fn cells_bounds(cells: &[(u32, u32)]) -> (u32, u32, u32, u32) {
    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_x = cells.iter().map(|(x, _)| *x).max().unwrap_or(min_x);
    let max_y = cells.iter().map(|(_, y)| *y).max().unwrap_or(min_y);
    (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

fn rect_for_cells(
    map: &TerrainMap,
    projection: TargetProjection,
    x0: u32,
    y0: u32,
    width_cells: u32,
    height_cells: u32,
) -> TargetRect {
    let x1 = (x0 + width_cells.saturating_sub(1)).min(map.width.saturating_sub(1));
    let y1 = (y0 + height_cells.saturating_sub(1)).min(map.height.saturating_sub(1));
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
        let (sx, sy) = projection.cell_top_left(map, x, y);
        min_x = min_x.min(sx);
        min_y = min_y.min(sy);
        max_x = max_x.max(sx + projection.cell_w as i32);
        max_y = max_y.max(sy + projection.cell_h as i32);
    }
    TargetRect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1) as u32,
        height: (max_y - min_y).max(1) as u32,
    }
}

fn draw_blob(
    image: &mut PixelImage,
    rect: TargetRect,
    color: Rgba8,
    seed: u32,
    alpha: f32,
    edge_noise: f32,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    for yy in 0..rect.height {
        for xx in 0..rect.width {
            let nx = (xx as f32 / rect.width.max(1) as f32) * 2.0 - 1.0;
            let ny = (yy as f32 / rect.height.max(1) as f32) * 2.0 - 1.0;
            let d = (nx * nx * 0.72 + ny * ny).sqrt();
            let n = noise_signed(seed, xx / 3, yy / 3);
            let inside = d < 1.02 + n * edge_noise;
            if !inside {
                continue;
            }
            let a = alpha * (1.0 - (d - 0.72).max(0.0) * 0.85).clamp(0.22, 1.0);
            let shade = n * 0.06;
            let c = if shade >= 0.0 {
                color.lighten(shade)
            } else {
                color.darken(-shade)
            };
            blend_px(image, rect.x + xx as i32, rect.y + yy as i32, c, a);
        }
    }
}

fn draw_noisy_ellipse(
    image: &mut PixelImage,
    rect: TargetRect,
    color: Rgba8,
    alpha: f32,
    seed: u32,
) {
    draw_blob(image, rect, color, seed, alpha, 0.18);
}

fn draw_soft_shadow(image: &mut PixelImage, x: i32, y: i32, width: u32, height: u32, alpha: f32) {
    if width == 0 || height == 0 {
        return;
    }
    for yy in 0..height {
        let ty = yy as f32 / height.max(1) as f32;
        for xx in 0..width {
            let tx = xx as f32 / width.max(1) as f32;
            let dx = (tx - 0.5).abs();
            let a = alpha * (1.0 - dx * 1.45).clamp(0.0, 1.0) * (1.0 - ty * 0.55);
            blend_px(image, x + xx as i32, y + yy as i32, Rgba8::BLACK, a);
        }
    }
}

fn draw_grass_tufts(image: &mut PixelImage, rect: TargetRect, color: Rgba8, seed: u32) {
    let count = 5 + (seed % 7);
    for i in 0..count {
        let x = rect.x + (hash3(seed, i, 0xc101) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(seed, i, 0xc102) % rect.height.max(1)) as i32;
        let c = if i % 2 == 0 {
            color
        } else {
            color.darken(0.20)
        };
        image.draw_line(x, y + 7, x - 2, y, c);
        image.draw_line(x, y + 7, x + 2, y + 1, c.darken(0.10));
    }
}

fn draw_road_detail(image: &mut PixelImage, rect: TargetRect, tileset: &Tileset, seed: u32) {
    let pebble = tileset.palette.sample(GroundMaterial::Rock.ramp(), 0.54);
    let rut = tileset.palette.sample(GroundMaterial::Dirt.ramp(), 0.34);
    for i in 0..22 {
        let x = rect.x + (hash3(seed, i, 0xd101) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(seed, i, 0xd102) % rect.height.max(1)) as i32;
        if i % 3 == 0 {
            blend_rect(image, x, y, 3, 2, pebble, 0.32);
        } else {
            image.draw_line(x, y, x + 8, y + 1, rut);
        }
    }
}

fn draw_wood_planks(image: &mut PixelImage, rect: TargetRect, color: Rgba8, seed: u32) {
    let step = 12.max((rect.width / 5) as i32) as u32;
    let mut x = 0;
    while x < rect.width {
        image.draw_line(
            rect.x + x as i32,
            rect.y,
            rect.x + x as i32,
            rect.y + rect.height as i32,
            color,
        );
        x += step;
    }
    for i in 0..10 {
        let y = rect.y + (hash3(seed, i, 0xe101) % rect.height.max(1)) as i32;
        image.draw_line(
            rect.x + 6,
            y,
            rect.x + rect.width as i32 - 8,
            y,
            color.darken(0.18),
        );
    }
}

fn draw_stone_lines(
    image: &mut PixelImage,
    sx: i32,
    sy: i32,
    width: u32,
    height: u32,
    color: Rgba8,
) {
    let block_w = 38;
    let block_h = 28;
    for y in 0..height {
        for x in 0..width {
            if x % block_w < 2 || y % block_h < 2 {
                blend_px(image, sx + x as i32, sy + y as i32, color, 0.26);
            }
        }
    }
}

fn draw_berm_stones(image: &mut PixelImage, rect: TargetRect, color: Rgba8, seed: u32) {
    for i in 0..24 {
        let x = rect.x + (hash3(seed, i, 0xf101) % rect.width.max(1)) as i32;
        let y = rect.y + (hash3(seed, i, 0xf102) % rect.height.max(1)) as i32;
        let w = 2 + (hash3(seed, i, 0xf103) % 5);
        let h = 1 + (hash3(seed, i, 0xf104) % 4);
        blend_rect(image, x, y, w, h, color, 0.45);
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
            blend_px(image, x + xx as i32, y + yy as i32, color, alpha);
        }
    }
}

fn blend_px(image: &mut PixelImage, x: i32, y: i32, color: Rgba8, alpha: f32) {
    if image.in_bounds(x, y) {
        image.blend_pixel(x as u32, y as u32, color, clamp01(alpha));
    }
}

fn hash3(a: u32, b: u32, salt: u32) -> u32 {
    let mut v = salt ^ a.wrapping_mul(0x9e37_79b1) ^ b.wrapping_mul(0x85eb_ca6b);
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^ (v >> 16)
}

fn noise01(seed: u32, x: u32, y: u32) -> f32 {
    hash3(seed ^ 0x517c_c1b7, x, y) as f32 / u32::MAX as f32
}

fn noise_signed(seed: u32, x: u32, y: u32) -> f32 {
    noise01(seed, x, y) * 2.0 - 1.0
}
