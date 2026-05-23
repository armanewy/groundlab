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

fn path_mask_sprite(
    sprites: &[GeneratedTerrainSprite],
    mask: u8,
) -> Option<&GeneratedTerrainSprite> {
    sprites
        .iter()
        .find(|sprite| sprite.kind.path_mask() == Some(mask))
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
