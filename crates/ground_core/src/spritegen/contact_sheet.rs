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

pub fn build_seam_heatmap(
    sprites: &[GeneratedTerrainSprite],
    recipe: &TerrainSpriteRecipe,
) -> PixelImage {
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let surfaces = sprites
        .iter()
        .filter(|sprite| !sprite.kind.is_transition())
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
    for sprite in sprites.iter().filter(|sprite| !sprite.kind.is_transition()) {
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
