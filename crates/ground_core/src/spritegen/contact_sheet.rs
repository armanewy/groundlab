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
    let scale = recipe.style.display_scale.max(1);
    let tile = recipe.tile_size;
    let mut preview = PixelImage::transparent(tile * 4, tile * 4);
    let variants = sprites
        .iter()
        .filter(|sprite| sprite.kind == kind)
        .collect::<Vec<_>>();
    if variants.is_empty() {
        return PixelImage::new(1, 1, Rgba8::BLACK);
    }
    for y in 0..4 {
        for x in 0..4 {
            let sprite = variants[(x as usize + y as usize) % variants.len()];
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
