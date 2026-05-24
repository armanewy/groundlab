use crate::color::Rgba8;
use crate::pixel_image::PixelImage;
use crate::spritegen::{
    scale_nearest, GeneratedTerrainSprite, TerrainSpriteKind, TerrainSpriteRecipe,
};

pub fn build_sprite_contact_sheet(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let preview_size = recipe.tile_size * scale;
    let columns = 6;
    let rows = (sprites.len() as u32).div_ceil(columns).max(1);
    let padding = 10;
    let cell = preview_size + padding * 2;
    let mut sheet = PixelImage::new(
        cell * columns + padding,
        cell * rows + padding,
        Rgba8::opaque(13, 15, 17),
    );

    for (i, sprite) in sprites.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        let x = padding + col * cell;
        let y = padding + row * cell;
        sheet.fill_rect(
            x,
            y,
            cell - padding,
            cell - padding,
            Rgba8::opaque(28, 31, 28),
        );
        sheet.outline_rect(
            x,
            y,
            cell - padding,
            cell - padding,
            Rgba8::opaque(55, 60, 54),
        );
        let scaled = scale_nearest(&sprite.image, scale);
        sheet.blit(&scaled, x + padding / 2, y + padding / 2);
    }

    sheet
}

pub fn build_repeat_preview(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_variant_repeat_preview(sprites, kind, recipe, 4)
}

pub fn build_single_repeat_preview(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
    recipe: &TerrainSpriteRecipe,
    repeats: u32,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let Some(sprite) = sprites.iter().find(|sprite| sprite.kind == kind) else {
        return PixelImage::new(1, 1, Rgba8::BLACK);
    };
    let mut preview = PixelImage::transparent(tile * repeats, tile * repeats);
    for y in 0..repeats {
        for x in 0..repeats {
            preview.blit(&sprite.image, x * tile, y * tile);
        }
    }
    scale_nearest(&preview, scale)
}

pub fn build_variant_repeat_preview(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
    recipe: &TerrainSpriteRecipe,
    repeats: u32,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut preview = PixelImage::transparent(tile * repeats, tile * repeats);
    let variants = sprites
        .iter()
        .filter(|sprite| sprite.kind == kind)
        .collect::<Vec<_>>();
    if variants.is_empty() {
        return PixelImage::new(1, 1, Rgba8::BLACK);
    }
    for y in 0..repeats {
        for x in 0..repeats {
            let index = variant_index(x, y, variants.len());
            let sprite = variants[index];
            preview.blit(&sprite.image, x * tile, y * tile);
        }
    }
    scale_nearest(&preview, scale)
}

pub fn build_transition_repeat_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut preview = PixelImage::transparent(tile * 4, tile * 4);
    let grass = sprites
        .iter()
        .find(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .map(|sprite| &sprite.image);
    let dirt = sprites
        .iter()
        .find(|sprite| sprite.kind == TerrainSpriteKind::DirtTile)
        .map(|sprite| &sprite.image);
    if let Some(grass) = grass {
        for y in 0..4 {
            for x in 0..4 {
                preview.blit(grass, x * tile, y * tile);
            }
        }
    }
    if let Some(dirt) = dirt {
        for y in 1..3 {
            for x in 1..3 {
                preview.blit(dirt, x * tile, y * tile);
            }
        }
    }
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        tile,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        tile,
        tile * 2,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeWest,
        tile,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        tile * 2,
        tile,
    );
    scale_nearest(&preview, scale)
}

pub fn build_transition_edges_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut preview = PixelImage::transparent(tile * 6, tile * 3);
    let grass = sprites
        .iter()
        .find(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .map(|sprite| &sprite.image);
    if let Some(grass) = grass {
        for y in 0..3 {
            for x in 0..6 {
                preview.blit(grass, x * tile, y * tile);
            }
        }
    }
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        0,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        tile,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        tile * 2,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeWest,
        tile * 3,
        tile,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        tile * 4,
        0,
    );
    blit_kind(
        &mut preview,
        sprites,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        tile * 4,
        tile * 2,
    );
    scale_nearest(&preview, scale)
}

pub fn build_path_autotile_sheet(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut sheet = PixelImage::new(tile * 4, tile * 4, Rgba8::opaque(13, 15, 17));
    for mask in 0..16 {
        let x = mask as u32 % 4;
        let y = mask as u32 / 4;
        if let Some(sprite) = path_mask_sprite(sprites, mask) {
            sheet.blit(&sprite.image, x * tile, y * tile);
        }
    }
    scale_nearest(&sheet, scale)
}

pub fn build_path_preview_random(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_path_preview_for_pattern(sprites, recipe, PathPreviewPattern::Random)
}

pub fn build_path_preview_sparse(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_path_preview_for_pattern(sprites, recipe, PathPreviewPattern::Sparse)
}

pub fn build_path_preview_dense(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_path_preview_for_pattern(sprites, recipe, PathPreviewPattern::Dense)
}

pub fn build_path_preview_loop(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_path_preview_for_pattern(sprites, recipe, PathPreviewPattern::Loop)
}

pub fn build_path_preview_junctions(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_path_preview_for_pattern(sprites, recipe, PathPreviewPattern::Junctions)
}

pub fn build_oblique_material_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let projection = &recipe.style.projection;
    let cell_w = projection.cell_width_px.max(recipe.tile_size);
    let cell_h = projection.cell_height_px.max(recipe.tile_size);
    let face_h = projection.face_height_px.max(1);
    let step_y = (cell_h * 7 / 12).max(recipe.tile_size);
    let width = 7;
    let height = 5;
    let margin = 24 + face_h + projection.shadow_offset_px.1.unsigned_abs();
    let canvas_w =
        margin * 2 + width * cell_w + projection.shadow_offset_px.0.unsigned_abs() + cell_w / 3;
    let canvas_h = margin * 2
        + (height - 1) * step_y
        + cell_h
        + face_h * 2
        + projection.shadow_offset_px.1.unsigned_abs();
    let mut preview = PixelImage::new(canvas_w, canvas_h, Rgba8::opaque(25, 30, 24));
    let map = sample_path_map(width, height, PathPreviewPattern::Random);
    let grass = sprites
        .iter()
        .filter(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .collect::<Vec<_>>();
    let palette = &recipe.style.palette;

    for y in 0..height {
        for x in 0..width {
            let h = oblique_height(x, y);
            if h == 0 {
                continue;
            }
            let (sx, sy) = oblique_cell_position(margin, step_y, cell_w, face_h, x, y, h);
            blend_rect_i32(
                &mut preview,
                sx + projection.shadow_offset_px.0,
                sy + projection.shadow_offset_px.1,
                cell_w,
                cell_h / 3,
                Rgba8::BLACK,
                0.24,
            );
        }
    }

    for y in 0..height {
        for x in 0..width {
            let h = oblique_height(x, y);
            let (sx, sy) = oblique_cell_position(margin, step_y, cell_w, face_h, x, y, h);
            let path = map[(y * width + x) as usize];
            if path {
                let mask = path_neighbor_mask(&map, width, height, x, y);
                if let Some(sprite) = path_mask_sprite(sprites, mask) {
                    blit_scaled_i32(&mut preview, &sprite.image, sx, sy, cell_w, cell_h);
                }
            } else if !grass.is_empty() {
                let sprite = grass[variant_index(x, y, grass.len())];
                blit_scaled_i32(&mut preview, &sprite.image, sx, sy, cell_w, cell_h);
            }
            if h > 0 {
                preview.draw_line(
                    sx,
                    sy + cell_h as i32 - 1,
                    sx + cell_w as i32 - 1,
                    sy + cell_h as i32 - 1,
                    palette.grass_dark,
                );
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            let h = oblique_height(x, y);
            let south = if y + 1 < height {
                oblique_height(x, y + 1)
            } else {
                0
            };
            let east = if x + 1 < width {
                oblique_height(x + 1, y)
            } else {
                0
            };
            let (sx, sy) = oblique_cell_position(margin, step_y, cell_w, face_h, x, y, h);
            if h > south {
                draw_oblique_front_face(
                    &mut preview,
                    sx,
                    sy + cell_h as i32 - 1,
                    cell_w,
                    face_h * (h - south),
                    palette,
                );
                preview.draw_line(
                    sx,
                    sy + cell_h as i32 - 1,
                    sx + cell_w as i32 - 1,
                    sy + cell_h as i32 - 1,
                    palette.grass_dark,
                );
            }
            if h > east {
                draw_oblique_side_face(
                    &mut preview,
                    sx + cell_w as i32 - cell_w as i32 / 8,
                    sy + cell_h as i32 / 6,
                    cell_w / 8,
                    cell_h * 2 / 3,
                    face_h * (h - east),
                    palette,
                );
            }
        }
    }

    preview
}

pub fn build_trench_contact_sheet(
    sprites: &[GeneratedTerrainSprite],
    _recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let trench = sprites
        .iter()
        .filter(|sprite| sprite.kind.is_trench())
        .collect::<Vec<_>>();
    let columns = 3;
    let padding = 12;
    let cell_w = trench
        .iter()
        .map(|sprite| sprite.image.width)
        .max()
        .unwrap_or(64)
        + padding * 2;
    let cell_h = trench
        .iter()
        .map(|sprite| sprite.image.height)
        .max()
        .unwrap_or(32)
        + padding * 2;
    let rows = (trench.len() as u32).div_ceil(columns).max(1);
    let mut sheet = PixelImage::new(
        cell_w * columns + padding,
        cell_h * rows + padding,
        Rgba8::opaque(13, 15, 17),
    );

    for (i, sprite) in trench.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        let x = padding + col * cell_w;
        let y = padding + row * cell_h;
        sheet.fill_rect(
            x,
            y,
            cell_w - padding,
            cell_h - padding,
            Rgba8::opaque(28, 31, 28),
        );
        sheet.outline_rect(
            x,
            y,
            cell_w - padding,
            cell_h - padding,
            Rgba8::opaque(55, 60, 54),
        );
        sheet.blit(&sprite.image, x + padding / 2, y + padding / 2);
    }
    sheet
}

pub fn build_berm_contact_sheet(
    sprites: &[GeneratedTerrainSprite],
    _recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let berm = sprites
        .iter()
        .filter(|sprite| sprite.kind.is_berm())
        .collect::<Vec<_>>();
    let columns = 3;
    let padding = 12;
    let cell_w = berm
        .iter()
        .map(|sprite| sprite.image.width)
        .max()
        .unwrap_or(64)
        + padding * 2;
    let cell_h = berm
        .iter()
        .map(|sprite| sprite.image.height)
        .max()
        .unwrap_or(32)
        + padding * 2;
    let rows = (berm.len() as u32).div_ceil(columns).max(1);
    let mut sheet = PixelImage::new(
        cell_w * columns + padding,
        cell_h * rows + padding,
        Rgba8::opaque(13, 15, 17),
    );

    for (i, sprite) in berm.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        let x = padding + col * cell_w;
        let y = padding + row * cell_h;
        sheet.fill_rect(
            x,
            y,
            cell_w - padding,
            cell_h - padding,
            Rgba8::opaque(28, 31, 28),
        );
        sheet.outline_rect(
            x,
            y,
            cell_w - padding,
            cell_h - padding,
            Rgba8::opaque(55, 60, 54),
        );
        sheet.blit(&sprite.image, x + padding / 2, y + padding / 2);
    }
    sheet
}

pub fn build_trench_oblique_straight_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    draw_straight_trench(&mut preview, sprites, 138, 98, 1.0);
    preview
}

pub fn build_trench_oblique_caps_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    draw_straight_trench(&mut preview, sprites, 138, 98, 0.78);
    if let Some(cap) = sprite_image(sprites, TerrainSpriteKind::TrenchEndCapLeft, 1) {
        blit_scaled_i32(&mut preview, cap, 122, 91, 54, 78);
    }
    if let Some(cap) = sprite_image(sprites, TerrainSpriteKind::TrenchEndCapRight, 1) {
        blit_scaled_i32(&mut preview, cap, 324, 91, 54, 78);
    }
    preview
}

pub fn build_trench_oblique_corner_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 4);
    draw_straight_trench(&mut preview, sprites, 104, 126, 0.72);
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::TrenchContactShadow, 1) {
        blit_scaled_i32(&mut preview, shadow, 232, 80, 62, 150);
    }
    if let Some(floor) = sprite_image(sprites, TerrainSpriteKind::TrenchFloorTop, 1) {
        blit_scaled_i32(&mut preview, floor, 234, 70, 58, 146);
    }
    if let Some(wall) = sprite_image(sprites, TerrainSpriteKind::TrenchWallFront, 1) {
        blit_scaled_i32(&mut preview, wall, 280, 106, 46, 120);
    }
    if let Some(lip) = sprite_image(sprites, TerrainSpriteKind::TrenchLipFront, 1) {
        blit_scaled_i32(&mut preview, lip, 224, 82, 76, 20);
        blit_scaled_i32(&mut preview, lip, 246, 204, 70, 18);
    }
    if let Some(corner) = sprite_image(sprites, TerrainSpriteKind::TrenchCornerInner, 1) {
        blit_scaled_i32(&mut preview, corner, 224, 118, 72, 72);
    }
    if let Some(corner) = sprite_image(sprites, TerrainSpriteKind::TrenchCornerOuter, 1) {
        blit_scaled_i32(&mut preview, corner, 292, 118, 64, 64);
    }
    preview
}

pub fn build_trench_oblique_shadow_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::TrenchContactShadow, 1) {
        blit_scaled_i32(&mut preview, shadow, 110, 94, 270, 70);
        blit_scaled_i32(&mut preview, shadow, 134, 130, 220, 48);
    }
    if let Some(spoil) = sprite_image(sprites, TerrainSpriteKind::TrenchSpoilPile, 1) {
        blit_scaled_i32(&mut preview, spoil, 128, 62, 144, 38);
        blit_scaled_i32(&mut preview, spoil, 238, 152, 120, 34);
    }
    preview
}

pub fn build_berm_oblique_straight_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    draw_straight_berm(&mut preview, sprites, 128, 102, 1.0);
    preview
}

pub fn build_berm_oblique_caps_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    draw_straight_berm(&mut preview, sprites, 138, 102, 0.78);
    if let Some(cap) = sprite_image(sprites, TerrainSpriteKind::BermEndCapLeft, 1) {
        blit_scaled_i32(&mut preview, cap, 98, 101, 58, 56);
    }
    if let Some(cap) = sprite_image(sprites, TerrainSpriteKind::BermEndCapRight, 1) {
        blit_scaled_i32(&mut preview, cap, 332, 101, 58, 56);
    }
    preview
}

pub fn build_berm_oblique_corner_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 4);
    draw_straight_berm(&mut preview, sprites, 102, 148, 0.66);
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::BermContactShadow, 1) {
        blit_scaled_i32(&mut preview, shadow, 213, 108, 78, 76);
    }
    if let Some(face) = sprite_image(sprites, TerrainSpriteKind::BermFaceFront, 1) {
        blit_scaled_i32(&mut preview, face, 218, 73, 52, 168);
    }
    if let Some(top) = sprite_image(sprites, TerrainSpriteKind::BermTop, 1) {
        blit_scaled_i32(&mut preview, top, 194, 73, 88, 116);
    }
    if let Some(corner) = sprite_image(sprites, TerrainSpriteKind::BermCornerInner, 1) {
        preview.blit(corner, 205, 130);
    }
    if let Some(corner) = sprite_image(sprites, TerrainSpriteKind::BermCornerOuter, 1) {
        preview.blit(corner, 250, 168);
    }
    preview
}

pub fn build_berm_oblique_shadow_preview(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let mut preview = trench_preview_base(sprites, recipe, 5, 3);
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::BermContactShadow, 1) {
        blit_scaled_i32(&mut preview, shadow, 104, 148, 286, 72);
    }
    if let Some(spoil) = sprite_image(sprites, TerrainSpriteKind::BermSpoilPile, 1) {
        blit_scaled_i32(&mut preview, spoil, 138, 92, 120, 34);
        blit_scaled_i32(&mut preview, spoil, 248, 174, 110, 32);
    }
    if let Some(fringe) = sprite_image(sprites, TerrainSpriteKind::BermGrassFringe, 1) {
        blit_scaled_i32(&mut preview, fringe, 118, 84, 270, 28);
    }
    draw_straight_berm(&mut preview, sprites, 128, 102, 1.0);
    preview
}

pub fn build_berm_mask_debug_preview(recipe: &TerrainSpriteRecipe) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let projection = &recipe.style.projection;
    let cell_w = projection.cell_width_px;
    let cell_h = projection.cell_height_px;
    let face_h = projection.face_height_px.max(8);
    let mut preview = PixelImage::new(cell_w * 3, cell_h * 2, Rgba8::opaque(36, 43, 31));
    let top = Rgba8::opaque(150, 112, 65);
    let face = Rgba8::opaque(104, 70, 43);
    let lip = Rgba8::opaque(202, 151, 87);
    let shadow = Rgba8::opaque(20, 22, 18);
    preview.fill_rect(cell_w / 2, cell_h / 2, cell_w * 2, face_h / 2, shadow);
    preview.fill_rect(
        cell_w / 2,
        cell_h / 2 - face_h / 2,
        cell_w * 2,
        face_h,
        face,
    );
    preview.fill_rect(cell_w / 2, cell_h / 2 - face_h, cell_w * 2, face_h / 2, top);
    preview.fill_rect(cell_w / 2, cell_h / 2 - face_h - 3, cell_w * 2, 4, lip);
    preview.outline_rect(
        cell_w / 2,
        cell_h / 2 - face_h,
        cell_w * 2,
        face_h,
        Rgba8::WHITE,
    );
    scale_nearest(&preview, scale)
}

pub fn build_trench_mask_debug_preview(recipe: &TerrainSpriteRecipe) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = (recipe.tile_size * 2).max(28);
    let mut preview = PixelImage::new(tile * 4, tile * 4, Rgba8::opaque(25, 34, 26));
    let trench = Rgba8::opaque(45, 32, 26);
    let lip = Rgba8::opaque(174, 116, 69);
    let edge = Rgba8::opaque(98, 125, 64);
    let center = tile / 2;
    let arm = (tile / 5).max(5);
    for mask in 0..16 {
        let ox = mask as u32 % 4 * tile;
        let oy = mask as u32 / 4 * tile;
        preview.outline_rect(ox, oy, tile, tile, edge);
        preview.fill_rect(
            ox + center - arm,
            oy + center - arm,
            arm * 2,
            arm * 2,
            trench,
        );
        if mask & 1 != 0 {
            preview.fill_rect(ox + center - arm / 2, oy, arm, center, trench);
        }
        if mask & 2 != 0 {
            preview.fill_rect(ox + center, oy + center - arm / 2, center, arm, trench);
        }
        if mask & 4 != 0 {
            preview.fill_rect(ox + center - arm / 2, oy + center, arm, center, trench);
        }
        if mask & 8 != 0 {
            preview.fill_rect(ox, oy + center - arm / 2, center, arm, trench);
        }
        preview.outline_rect(ox + center - arm, oy + center - arm, arm * 2, arm * 2, lip);
    }
    scale_nearest(&preview, scale)
}

pub fn build_trench_autotile_sheet(
    sprites: &[GeneratedTerrainSprite],
    _recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let trench_masks = (0..16)
        .filter_map(|mask| trench_mask_sprite(sprites, mask))
        .collect::<Vec<_>>();
    let cell_w = trench_masks
        .iter()
        .map(|sprite| sprite.image.width)
        .max()
        .unwrap_or(96)
        + 12;
    let cell_h = trench_masks
        .iter()
        .map(|sprite| sprite.image.height)
        .max()
        .unwrap_or(96)
        + 12;
    let mut sheet = PixelImage::new(cell_w * 4 + 8, cell_h * 4 + 8, Rgba8::opaque(13, 15, 17));
    for mask in 0..16 {
        let x = 8 + mask as u32 % 4 * cell_w;
        let y = 8 + mask as u32 / 4 * cell_h;
        sheet.fill_rect(x, y, cell_w - 6, cell_h - 6, Rgba8::opaque(28, 31, 28));
        sheet.outline_rect(x, y, cell_w - 6, cell_h - 6, Rgba8::opaque(55, 60, 54));
        if let Some(sprite) = trench_mask_sprite(sprites, mask) {
            sheet.blit(&sprite.image, x + 3, y + 3);
        }
    }
    sheet
}

pub fn build_trench_preview_sparse(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_preview_for_pattern(sprites, recipe, PathPreviewPattern::Sparse)
}

pub fn build_trench_preview_dense(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_preview_for_pattern(sprites, recipe, PathPreviewPattern::Dense)
}

pub fn build_trench_preview_loop(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_preview_for_pattern(sprites, recipe, PathPreviewPattern::Loop)
}

pub fn build_trench_preview_junctions(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_preview_for_pattern(sprites, recipe, PathPreviewPattern::Junctions)
}

pub fn build_trench_preview_single_masks(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_autotile_sheet(sprites, recipe)
}

pub fn build_trench_preview_dead_ends(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let width = 8;
    let height = 5;
    let mut map = vec![false; (width * height) as usize];
    for x in 1..4 {
        set_path(&mut map, width, x, 1);
    }
    for y in 1..4 {
        set_path(&mut map, width, 5, y);
    }
    for x in 1..7 {
        set_path(&mut map, width, x, 3);
    }
    build_trench_preview_from_map(sprites, recipe, width, height, &map)
}

pub fn build_trench_preview_corners(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let width = 8;
    let height = 5;
    let mut map = vec![false; (width * height) as usize];
    for x in 1..4 {
        set_path(&mut map, width, x, 1);
    }
    for y in 1..4 {
        set_path(&mut map, width, 3, y);
    }
    for x in 4..7 {
        set_path(&mut map, width, x, 3);
    }
    for y in 1..=3 {
        set_path(&mut map, width, 6, y);
    }
    build_trench_preview_from_map(sprites, recipe, width, height, &map)
}

pub fn build_trench_preview_dense_clean(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let width = 10;
    let height = 6;
    let mut map = vec![false; (width * height) as usize];
    for x in 1..9 {
        set_path(&mut map, width, x, 2);
    }
    for y in 1..5 {
        set_path(&mut map, width, 2, y);
        set_path(&mut map, width, 7, y);
    }
    for x in 2..8 {
        set_path(&mut map, width, x, 4);
    }
    build_trench_preview_from_map(sprites, recipe, width, height, &map)
}

pub fn build_trench_neighbor_seam_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Neighbor)
}

pub fn build_trench_lip_continuity_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Lip)
}

pub fn build_trench_floor_continuity_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Floor)
}

pub fn build_trench_neighbor_seam_edge_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Neighbor)
}

pub fn build_trench_lip_continuity_edge_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Lip)
}

pub fn build_trench_floor_continuity_edge_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    build_trench_continuity_heatmap(sprites, recipe, TrenchContinuityMode::Floor)
}

fn build_path_preview_for_pattern(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
    pattern: PathPreviewPattern,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let width = 12;
    let height = 8;
    let map = sample_path_map(width, height, pattern);
    let mut preview = PixelImage::transparent(tile * width, tile * height);
    let grass = sprites
        .iter()
        .filter(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .collect::<Vec<_>>();
    for y in 0..height {
        for x in 0..width {
            if map[(y * width + x) as usize] {
                let mask = path_neighbor_mask(&map, width, height, x, y);
                if let Some(sprite) = path_mask_sprite(sprites, mask) {
                    preview.blit(&sprite.image, x * tile, y * tile);
                }
            } else if !grass.is_empty() {
                let sprite = grass[variant_index(x, y, grass.len())];
                preview.blit(&sprite.image, x * tile, y * tile);
            }
        }
    }
    scale_nearest(&preview, scale)
}

fn build_trench_preview_for_pattern(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
    pattern: PathPreviewPattern,
) -> PixelImage {
    let width = 10;
    let height = 6;
    let map = sample_path_map(width, height, pattern);
    build_trench_preview_from_map(sprites, recipe, width, height, &map)
}

fn build_trench_preview_from_map(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
    width: u32,
    height: u32,
    map: &[bool],
) -> PixelImage {
    let projection = &recipe.style.projection;
    let cell_w = projection.cell_width_px;
    let cell_h = projection.cell_height_px;
    let mask_h = (projection.cell_height_px + projection.face_height_px + 12).max(cell_h);
    let mut preview = PixelImage::new(
        width * cell_w,
        height * cell_h + mask_h.saturating_sub(cell_h),
        Rgba8::opaque(40, 48, 34),
    );
    let grass = sprites
        .iter()
        .filter(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .collect::<Vec<_>>();
    for y in 0..height {
        for x in 0..width {
            if let Some(sprite) = grass.get(variant_index(x, y, grass.len().max(1))) {
                blit_scaled_i32(
                    &mut preview,
                    &sprite.image,
                    (x * cell_w) as i32,
                    (y * cell_h) as i32,
                    cell_w,
                    cell_h,
                );
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            if !map[(y * width + x) as usize] {
                continue;
            }
            let mask = path_neighbor_mask(map, width, height, x, y);
            if let Some(sprite) = trench_mask_sprite(sprites, mask) {
                preview.blit(&sprite.image, x * cell_w, y * cell_h);
            }
        }
    }
    preview
}

#[derive(Clone, Copy)]
enum TrenchContinuityMode {
    Neighbor,
    Lip,
    Floor,
}

fn build_trench_continuity_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
    mode: TrenchContinuityMode,
) -> PixelImage {
    let cell_w = recipe.style.projection.cell_width_px;
    let cell_h = recipe.style.projection.cell_height_px;
    let mut image = PixelImage::new(cell_w * 4, cell_h * 4, Rgba8::opaque(15, 18, 16));
    for mask in 0..16 {
        let ox = mask as u32 % 4 * cell_w;
        let oy = mask as u32 / 4 * cell_h;
        let Some(sprite) = trench_mask_sprite(sprites, mask) else {
            continue;
        };
        for x in 0..cell_w.min(sprite.image.width) {
            let v = if mask & 1 != 0 {
                trench_connected_edge_score(sprite, sprites, mask, 1, x, mode, recipe)
            } else {
                0.0
            };
            paint_heat_block(&mut image, ox + x, oy, 1, 4, v);
            let v = if mask & 4 != 0 {
                trench_connected_edge_score(sprite, sprites, mask, 4, x, mode, recipe)
            } else {
                0.0
            };
            paint_heat_block(&mut image, ox + x, oy + cell_h.saturating_sub(4), 1, 4, v);
        }
        for y in 0..cell_h.min(sprite.image.height) {
            let v = if mask & 8 != 0 {
                trench_connected_edge_score(sprite, sprites, mask, 8, y, mode, recipe)
            } else {
                0.0
            };
            paint_heat_block(&mut image, ox, oy + y, 4, 1, v);
            let v = if mask & 2 != 0 {
                trench_connected_edge_score(sprite, sprites, mask, 2, y, mode, recipe)
            } else {
                0.0
            };
            paint_heat_block(&mut image, ox + cell_w.saturating_sub(4), oy + y, 4, 1, v);
        }
    }
    scale_nearest(&image, recipe.style.display_scale.max(1))
}

pub fn build_path_neighbor_seam_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut image = PixelImage::new(tile * 4, tile * 4, Rgba8::opaque(15, 18, 16));
    for mask in 0..16 {
        let ox = mask as u32 % 4 * tile;
        let oy = mask as u32 / 4 * tile;
        let Some(sprite) = path_mask_sprite(sprites, mask) else {
            continue;
        };
        for x in 0..tile {
            let v = if mask & 1 != 0 {
                connected_edge_score(sprite, sprites, mask, 1, x)
            } else {
                0.0
            };
            image.set(ox + x, oy, heat(v));
            let v = if mask & 4 != 0 {
                connected_edge_score(sprite, sprites, mask, 4, x)
            } else {
                0.0
            };
            image.set(ox + x, oy + tile - 1, heat(v));
        }
        for y in 0..tile {
            let v = if mask & 8 != 0 {
                connected_edge_score(sprite, sprites, mask, 8, y)
            } else {
                0.0
            };
            image.set(ox, oy + y, heat(v));
            let v = if mask & 2 != 0 {
                connected_edge_score(sprite, sprites, mask, 2, y)
            } else {
                0.0
            };
            image.set(ox + tile - 1, oy + y, heat(v));
        }
    }
    scale_nearest(&image, scale)
}

pub fn build_path_mask_debug_preview(recipe: &TerrainSpriteRecipe) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut preview = PixelImage::new(tile * 4, tile * 4, Rgba8::opaque(25, 34, 26));
    let path = Rgba8::opaque(188, 131, 82);
    let edge = Rgba8::opaque(98, 125, 64);
    let center = tile / 2;
    let arm = (tile / 5).max(2);
    for mask in 0..16 {
        let ox = mask as u32 % 4 * tile;
        let oy = mask as u32 / 4 * tile;
        preview.outline_rect(ox, oy, tile, tile, edge);
        preview.fill_rect(ox + center - arm / 2, oy + center - arm / 2, arm, arm, path);
        if mask & 1 != 0 {
            preview.fill_rect(ox + center - arm / 2, oy, arm, center, path);
        }
        if mask & 2 != 0 {
            preview.fill_rect(ox + center, oy + center - arm / 2, center, arm, path);
        }
        if mask & 4 != 0 {
            preview.fill_rect(ox + center - arm / 2, oy + center, arm, center, path);
        }
        if mask & 8 != 0 {
            preview.fill_rect(ox, oy + center - arm / 2, center, arm, path);
        }
    }
    scale_nearest(&preview, scale)
}

pub fn build_seam_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let surfaces = sprites
        .iter()
        .filter(|sprite| {
            matches!(
                sprite.kind,
                TerrainSpriteKind::GrassTile | TerrainSpriteKind::DirtTile
            )
        })
        .collect::<Vec<_>>();
    let mut image = PixelImage::new(tile * surfaces.len().max(1) as u32, tile, Rgba8::BLACK);
    for (i, sprite) in surfaces.iter().enumerate() {
        let x0 = i as u32 * tile;
        for y in 0..tile {
            let left = sprite.image.get(0, y);
            let right = sprite.image.get(tile - 1, y);
            let score = left.rgb_distance(right).min(80.0) / 80.0;
            image.set(x0, y, heat(score));
            image.set(x0 + tile - 1, y, heat(score));
        }
        for x in 0..tile {
            let top = sprite.image.get(x, 0);
            let bottom = sprite.image.get(x, tile - 1);
            let score = top.rgb_distance(bottom).min(80.0) / 80.0;
            image.set(x0 + x, 0, heat(score));
            image.set(x0 + x, tile - 1, heat(score));
        }
    }
    scale_nearest(&image, scale)
}

pub fn build_motif_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut counts = vec![0u32; (tile * tile) as usize];
    for sprite in sprites.iter().filter(|sprite| {
        matches!(
            sprite.kind,
            TerrainSpriteKind::GrassTile | TerrainSpriteKind::DirtTile
        )
    }) {
        let base = dominant_color(&sprite.image);
        for y in 0..tile {
            for x in 0..tile {
                if sprite.image.get(x, y) != base {
                    counts[(y * tile + x) as usize] += 1;
                }
            }
        }
    }
    let max = counts.iter().copied().max().unwrap_or(1).max(1);
    let mut image = PixelImage::new(tile, tile, Rgba8::opaque(15, 18, 16));
    for y in 0..tile {
        for x in 0..tile {
            let v = counts[(y * tile + x) as usize] as f32 / max as f32;
            image.set(x, y, heat(v));
        }
    }
    scale_nearest(&image, scale)
}

pub fn build_palette_preview(recipe: &TerrainSpriteRecipe) -> PixelImage {
    let colors = recipe.style.palette.all_colors();
    let scale = recipe.style.display_scale.max(1);
    let swatch = 8 * scale;
    let margin = 3 * scale;
    let mut image = PixelImage::new(
        margin * 2 + colors.len() as u32 * (swatch + scale),
        margin * 2 + swatch,
        Rgba8::opaque(13, 15, 17),
    );
    for (i, color) in colors.iter().enumerate() {
        let x = margin + i as u32 * (swatch + scale);
        image.fill_rect(x, margin, swatch, swatch, *color);
        image.outline_rect(x, margin, swatch, swatch, Rgba8::opaque(55, 60, 54));
    }
    image
}

fn blit_kind(
    target: &mut PixelImage,
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
    x: u32,
    y: u32,
) {
    if let Some(sprite) = sprites.iter().find(|sprite| sprite.kind == kind) {
        target.blit(&sprite.image, x, y);
    }
}

fn trench_preview_base(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
    width_cells: u32,
    height_cells: u32,
) -> PixelImage {
    let projection = &recipe.style.projection;
    let cell_w = projection.cell_width_px;
    let cell_h = projection.cell_height_px;
    let mut preview = PixelImage::new(
        cell_w * width_cells,
        cell_h * height_cells,
        Rgba8::opaque(40, 48, 34),
    );
    let grass = sprites
        .iter()
        .filter(|sprite| sprite.kind == TerrainSpriteKind::GrassTile)
        .collect::<Vec<_>>();
    for y in 0..height_cells {
        for x in 0..width_cells {
            if let Some(sprite) = grass.get(variant_index(x, y, grass.len().max(1))) {
                blit_scaled_i32(
                    &mut preview,
                    &sprite.image,
                    (x * cell_w) as i32,
                    (y * cell_h) as i32,
                    cell_w,
                    cell_h,
                );
            }
        }
    }
    preview
}

fn draw_straight_trench(
    target: &mut PixelImage,
    sprites: &[GeneratedTerrainSprite],
    x: i32,
    y: i32,
    width_factor: f32,
) {
    let trench_w = (250.0 * width_factor).round() as u32;
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::TrenchContactShadow, 1) {
        blit_scaled_i32(target, shadow, x - 14, y + 56, trench_w + 34, 58);
    }

    if let Some(spoil) = sprite_image(sprites, TerrainSpriteKind::TrenchSpoilPile, 1) {
        blit_scaled_i32(target, spoil, x - 2, y - 24, trench_w / 2, 36);
        blit_scaled_i32(
            target,
            spoil,
            x + trench_w as i32 / 2 - 8,
            y + 92,
            trench_w / 2,
            34,
        );
    }

    if let Some(lip) = sprite_image(sprites, TerrainSpriteKind::TrenchLipBack, 1) {
        blit_scaled_i32(target, lip, x - 8, y + 22, trench_w + 16, 22);
    }
    if let Some(floor) = sprite_image(sprites, TerrainSpriteKind::TrenchFloorTop, 1) {
        blit_scaled_i32(target, floor, x + 10, y + 40, trench_w - 20, 52);
    }
    if let Some(wall) = sprite_image(sprites, TerrainSpriteKind::TrenchWallFront, 1) {
        blit_scaled_i32(target, wall, x + 10, y + 86, trench_w - 20, 56);
    }
    if let Some(lip) = sprite_image(sprites, TerrainSpriteKind::TrenchLipFront, 1) {
        blit_scaled_i32(target, lip, x - 8, y + 79, trench_w + 16, 24);
    }
}

fn draw_straight_berm(
    target: &mut PixelImage,
    sprites: &[GeneratedTerrainSprite],
    x: i32,
    y: i32,
    width_factor: f32,
) {
    let berm_w = (260.0 * width_factor).round() as u32;
    if let Some(shadow) = sprite_image(sprites, TerrainSpriteKind::BermContactShadow, 1) {
        blit_scaled_i32(target, shadow, x - 16, y + 76, berm_w + 36, 58);
    }
    if let Some(spoil) = sprite_image(sprites, TerrainSpriteKind::BermSpoilPile, 1) {
        blit_scaled_i32(target, spoil, x + 6, y + 96, berm_w / 2, 34);
        blit_scaled_i32(target, spoil, x + berm_w as i32 / 2, y - 12, berm_w / 2, 32);
    }
    if let Some(fringe) = sprite_image(sprites, TerrainSpriteKind::BermGrassFringe, 1) {
        blit_scaled_i32(target, fringe, x - 6, y - 8, berm_w + 16, 22);
    }
    if let Some(lip) = sprite_image(sprites, TerrainSpriteKind::BermLipBack, 1) {
        blit_scaled_i32(target, lip, x - 8, y + 8, berm_w + 16, 22);
    }
    if let Some(top) = sprite_image(sprites, TerrainSpriteKind::BermTop, 1) {
        blit_scaled_i32(target, top, x, y + 22, berm_w, 64);
    }
    if let Some(face) = sprite_image(sprites, TerrainSpriteKind::BermFaceFront, 1) {
        blit_scaled_i32(target, face, x + 8, y + 70, berm_w - 16, 48);
    }
    if let Some(lip) = sprite_image(sprites, TerrainSpriteKind::BermLipFront, 1) {
        blit_scaled_i32(target, lip, x - 6, y + 60, berm_w + 12, 24);
    }
}

fn sprite_image(
    sprites: &[GeneratedTerrainSprite],
    kind: TerrainSpriteKind,
    variant: u32,
) -> Option<&PixelImage> {
    sprites
        .iter()
        .find(|sprite| sprite.kind == kind && sprite.variant == variant)
        .or_else(|| sprites.iter().find(|sprite| sprite.kind == kind))
        .map(|sprite| &sprite.image)
}

fn path_mask_sprite(
    sprites: &[GeneratedTerrainSprite],
    mask: u8,
) -> Option<&GeneratedTerrainSprite> {
    sprites
        .iter()
        .find(|sprite| sprite.kind.path_mask() == Some(mask))
}

fn trench_mask_sprite(
    sprites: &[GeneratedTerrainSprite],
    mask: u8,
) -> Option<&GeneratedTerrainSprite> {
    sprites
        .iter()
        .find(|sprite| sprite.kind.trench_mask() == Some(mask))
}

#[derive(Clone, Copy)]
enum PathPreviewPattern {
    Random,
    Sparse,
    Dense,
    Loop,
    Junctions,
}

fn sample_path_map(width: u32, height: u32, pattern: PathPreviewPattern) -> Vec<bool> {
    let mut map = vec![false; (width * height) as usize];
    match pattern {
        PathPreviewPattern::Random => sample_random_path_map(&mut map, width, height),
        PathPreviewPattern::Sparse => sample_sparse_path_map(&mut map, width, height),
        PathPreviewPattern::Dense => sample_dense_path_map(&mut map, width, height),
        PathPreviewPattern::Loop => sample_loop_path_map(&mut map, width, height),
        PathPreviewPattern::Junctions => sample_junction_path_map(&mut map, width, height),
    }
    map
}

fn sample_random_path_map(map: &mut [bool], width: u32, height: u32) {
    let mut y = height / 2;
    for x in 0..width {
        if x == 3 {
            y = y.saturating_sub(1);
        }
        if x == 7 {
            y = (y + 1).min(height - 2);
        }
        set_path(map, width, x, y);
        if x == 2 || x == 8 {
            set_path(map, width, x, y.saturating_sub(1));
        }
        if x == 5 {
            for by in y..height.saturating_sub(1) {
                set_path(map, width, x, by);
            }
        }
        if x == 9 {
            for by in 1..=y {
                set_path(map, width, x, by);
            }
        }
    }
    set_path(map, width, 1, height.saturating_sub(2));
    set_path(map, width, 2, height.saturating_sub(2));
    set_path(map, width, 2, height.saturating_sub(3));
}

fn sample_sparse_path_map(map: &mut [bool], width: u32, height: u32) {
    for x in 1..width.saturating_sub(1) {
        let y = if x < width / 3 {
            height / 2
        } else if x < width * 2 / 3 {
            height / 2 - 1
        } else {
            height / 2
        };
        set_path(map, width, x, y);
    }
    for y in 1..height / 2 {
        set_path(map, width, width.saturating_sub(3), y);
    }
}

fn sample_dense_path_map(map: &mut [bool], width: u32, height: u32) {
    sample_random_path_map(map, width, height);
    for x in 2..width.saturating_sub(2) {
        set_path(map, width, x, 1);
    }
    for y in 2..height.saturating_sub(1) {
        set_path(map, width, 3, y);
        set_path(map, width, width.saturating_sub(4), y);
    }
}

fn sample_loop_path_map(map: &mut [bool], width: u32, height: u32) {
    for x in 2..width.saturating_sub(2) {
        set_path(map, width, x, 2);
        set_path(map, width, x, height.saturating_sub(3));
    }
    for y in 2..height.saturating_sub(2) {
        set_path(map, width, 2, y);
        set_path(map, width, width.saturating_sub(3), y);
    }
    for x in 0..=2 {
        set_path(map, width, x, height / 2);
    }
}

fn sample_junction_path_map(map: &mut [bool], width: u32, height: u32) {
    let cy = height / 2;
    for x in 1..width.saturating_sub(1) {
        set_path(map, width, x, cy);
    }
    for y in 1..height.saturating_sub(1) {
        set_path(map, width, width / 3, y);
        set_path(map, width, width * 2 / 3, y);
    }
    for x in 2..5 {
        set_path(map, width, x, 1);
    }
}

fn set_path(map: &mut [bool], width: u32, x: u32, y: u32) {
    let idx = (y * width + x) as usize;
    if idx < map.len() {
        map[idx] = true;
    }
}

fn path_neighbor_mask(map: &[bool], width: u32, height: u32, x: u32, y: u32) -> u8 {
    let mut mask = 0;
    if y > 0 && map[((y - 1) * width + x) as usize] {
        mask |= 1;
    }
    if x + 1 < width && map[(y * width + x + 1) as usize] {
        mask |= 2;
    }
    if y + 1 < height && map[((y + 1) * width + x) as usize] {
        mask |= 4;
    }
    if x > 0 && map[(y * width + x - 1) as usize] {
        mask |= 8;
    }
    mask
}

fn oblique_height(x: u32, y: u32) -> u32 {
    u32::from(x >= 4 && y <= 2)
}

fn oblique_cell_position(
    margin: u32,
    step_y: u32,
    cell_w: u32,
    face_h: u32,
    x: u32,
    y: u32,
    height: u32,
) -> (i32, i32) {
    (
        (margin + x * cell_w) as i32,
        (margin + y * step_y) as i32 - (height * face_h) as i32,
    )
}

fn blit_scaled_i32(
    target: &mut PixelImage,
    src: &PixelImage,
    dst_x: i32,
    dst_y: i32,
    width: u32,
    height: u32,
) {
    for y in 0..height {
        for x in 0..width {
            let sx = x * src.width / width.max(1);
            let sy = y * src.height / height.max(1);
            let tx = dst_x + x as i32;
            let ty = dst_y + y as i32;
            if target.in_bounds(tx, ty) {
                let color = src.get(sx, sy);
                if color.a == 255 {
                    target.set(tx as u32, ty as u32, color);
                } else if color.a > 0 {
                    let base = target.get(tx as u32, ty as u32);
                    target.set(
                        tx as u32,
                        ty as u32,
                        base.blend(color, color.a as f32 / 255.0),
                    );
                }
            }
        }
    }
}

fn blend_rect_i32(
    target: &mut PixelImage,
    x0: i32,
    y0: i32,
    width: u32,
    height: u32,
    color: Rgba8,
    alpha: f32,
) {
    for y in 0..height {
        for x in 0..width {
            let tx = x0 + x as i32;
            let ty = y0 + y as i32;
            if target.in_bounds(tx, ty) {
                let base = target.get(tx as u32, ty as u32);
                target.set(tx as u32, ty as u32, base.blend(color, alpha));
            }
        }
    }
}

fn draw_oblique_front_face(
    target: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    palette: &crate::spritegen::CozyTerrainPalette,
) {
    blend_rect_i32(target, x, y, width, height, palette.dirt_dark, 0.82);
    blend_rect_i32(
        target,
        x,
        y + height as i32 / 2,
        width,
        height / 2,
        Rgba8::BLACK,
        0.16,
    );
    for line_x in (4..width).step_by(11) {
        target.draw_line(
            x + line_x as i32,
            y + 2,
            x + line_x as i32 + 5,
            y + height as i32 - 3,
            palette.dirt_shadow,
        );
    }
}

fn draw_oblique_side_face(
    target: &mut PixelImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    face_height: u32,
    palette: &crate::spritegen::CozyTerrainPalette,
) {
    blend_rect_i32(
        target,
        x,
        y + height as i32 - 1,
        width,
        face_height,
        palette.dirt_shadow,
        0.74,
    );
}

fn connected_edge_score(
    sprite: &GeneratedTerrainSprite,
    sprites: &[GeneratedTerrainSprite],
    mask: u8,
    direction: u8,
    offset: u32,
) -> f32 {
    let opposite = match direction {
        1 => 4,
        2 => 8,
        4 => 1,
        8 => 2,
        _ => return 0.0,
    };
    let Some(neighbor) = path_mask_sprite(sprites, opposite) else {
        return 1.0;
    };
    let tile = sprite.image.width;
    let (a, b) = match direction {
        1 => (
            sprite.image.get(offset, 0),
            neighbor.image.get(offset, tile - 1),
        ),
        4 => (
            sprite.image.get(offset, tile - 1),
            neighbor.image.get(offset, 0),
        ),
        8 => (
            sprite.image.get(0, offset),
            neighbor.image.get(tile - 1, offset),
        ),
        2 => (
            sprite.image.get(tile - 1, offset),
            neighbor.image.get(0, offset),
        ),
        _ => return 0.0,
    };
    let connection_weight = if mask & direction != 0 { 1.0 } else { 0.25 };
    (a.rgb_distance(b).min(120.0) / 120.0) * connection_weight
}

fn trench_connected_edge_score(
    sprite: &GeneratedTerrainSprite,
    sprites: &[GeneratedTerrainSprite],
    _mask: u8,
    direction: u8,
    offset: u32,
    mode: TrenchContinuityMode,
    recipe: &TerrainSpriteRecipe,
) -> f32 {
    let opposite = match direction {
        1 => 4,
        2 => 8,
        4 => 1,
        8 => 2,
        _ => return 0.0,
    };
    let Some(neighbor) = trench_mask_sprite(sprites, opposite) else {
        return 1.0;
    };
    let surface_h = recipe
        .style
        .projection
        .cell_height_px
        .min(sprite.image.height)
        .min(neighbor.image.height)
        .max(1);
    let width = sprite.image.width.min(neighbor.image.width).max(1);
    let center_x = width / 2;
    let center_y = surface_h / 2;
    let open_w = (width as f32 * 0.34).round() as u32;
    let open_h = (surface_h as f32 * 0.34).round() as u32;
    let (a, b) = match direction {
        1 => {
            let x = offset.min(width - 1);
            if x < center_x.saturating_sub(open_w / 2) || x > center_x + open_w / 2 {
                return 0.0;
            }
            let band = match mode {
                TrenchContinuityMode::Neighbor => 0,
                TrenchContinuityMode::Lip => 2.min(surface_h - 1),
                TrenchContinuityMode::Floor => (surface_h / 2).min(surface_h - 1),
            };
            (
                sprite.image.get(x, band),
                neighbor
                    .image
                    .get(x, surface_h - 1 - band.min(surface_h - 1)),
            )
        }
        4 => {
            let x = offset.min(width - 1);
            if x < center_x.saturating_sub(open_w / 2) || x > center_x + open_w / 2 {
                return 0.0;
            }
            let band = match mode {
                TrenchContinuityMode::Neighbor => 0,
                TrenchContinuityMode::Lip => 2.min(surface_h - 1),
                TrenchContinuityMode::Floor => (surface_h / 2).min(surface_h - 1),
            };
            (
                sprite.image.get(x, surface_h - 1 - band.min(surface_h - 1)),
                neighbor.image.get(x, band),
            )
        }
        8 => {
            let y = offset.min(surface_h - 1);
            if y < center_y.saturating_sub(open_h / 2) || y > center_y + open_h / 2 {
                return 0.0;
            }
            let band = match mode {
                TrenchContinuityMode::Neighbor => 0,
                TrenchContinuityMode::Lip => 2.min(width - 1),
                TrenchContinuityMode::Floor => (width / 2).min(width - 1),
            };
            (
                sprite.image.get(band, y),
                neighbor.image.get(width - 1 - band.min(width - 1), y),
            )
        }
        2 => {
            let y = offset.min(surface_h - 1);
            if y < center_y.saturating_sub(open_h / 2) || y > center_y + open_h / 2 {
                return 0.0;
            }
            let band = match mode {
                TrenchContinuityMode::Neighbor => 0,
                TrenchContinuityMode::Lip => 2.min(width - 1),
                TrenchContinuityMode::Floor => (width / 2).min(width - 1),
            };
            (
                sprite.image.get(width - 1 - band.min(width - 1), y),
                neighbor.image.get(band, y),
            )
        }
        _ => return 0.0,
    };
    a.rgb_distance(b).min(140.0) / 140.0
}

fn variant_index(x: u32, y: u32, len: usize) -> usize {
    let mut value = (x as u64 * 0x9e37_79b9) ^ (y as u64 * 0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    (value as usize) % len
}

fn dominant_color(image: &PixelImage) -> Rgba8 {
    let mut colors = image.pixels.clone();
    colors.sort_by_key(|color| {
        ((color.r as u32) << 24)
            | ((color.g as u32) << 16)
            | ((color.b as u32) << 8)
            | color.a as u32
    });
    let mut best = colors[0];
    let mut best_count = 0;
    let mut current = colors[0];
    let mut current_count = 0;
    for color in colors {
        if color == current {
            current_count += 1;
        } else {
            if current_count > best_count {
                best = current;
                best_count = current_count;
            }
            current = color;
            current_count = 1;
        }
    }
    if current_count > best_count {
        best = current;
    }
    best
}

fn paint_heat_block(image: &mut PixelImage, x0: u32, y0: u32, width: u32, height: u32, value: f32) {
    let color = heat((value * 1.65).clamp(0.0, 1.0));
    for y in y0..(y0 + height).min(image.height) {
        for x in x0..(x0 + width).min(image.width) {
            image.set(x, y, color);
        }
    }
}

fn heat(v: f32) -> Rgba8 {
    let v = v.clamp(0.0, 1.0);
    if v < 0.34 {
        Rgba8::opaque(34, (80.0 + v * 180.0) as u8, 46)
    } else if v < 0.67 {
        Rgba8::opaque((120.0 + v * 110.0) as u8, 112, 42)
    } else {
        Rgba8::opaque(190, (90.0 - v * 45.0) as u8, 48)
    }
}
