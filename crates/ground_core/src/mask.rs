use crate::color::{clamp_u8, Rgba8};
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, StructureFaceKind, TileRole, TilesetRecipe, TransitionEdge};
use crate::tileset::{hash01, TileAsset, Tileset};

pub fn build_height_mask_atlas(tileset: &Tileset, columns: u32, padding: u32) -> PixelImage {
    build_mask_atlas(tileset, columns, padding, |asset, recipe| {
        build_height_mask_tile(asset, recipe)
    })
}

pub fn build_normal_map_atlas(tileset: &Tileset, columns: u32, padding: u32) -> PixelImage {
    let height = build_height_mask_atlas(tileset, columns, padding);
    build_normal_map_from_height(&height, tileset.recipe.mask_strength)
}

pub fn build_shadow_mask_atlas(tileset: &Tileset, columns: u32, padding: u32) -> PixelImage {
    build_mask_atlas(tileset, columns, padding, |asset, recipe| {
        build_shadow_mask_tile(asset, recipe)
    })
}

pub fn build_occlusion_mask_atlas(tileset: &Tileset, columns: u32, padding: u32) -> PixelImage {
    build_mask_atlas(tileset, columns, padding, |asset, recipe| {
        build_occlusion_mask_tile(asset, recipe)
    })
}

fn build_mask_atlas<F>(
    tileset: &Tileset,
    columns: u32,
    padding: u32,
    mut build_tile: F,
) -> PixelImage
where
    F: FnMut(&TileAsset, &TilesetRecipe) -> PixelImage,
{
    let columns = columns.max(1);
    let tile_size = tileset.recipe.tile_size;
    let rows = (tileset.tiles.len() as u32).div_ceil(columns);
    let width = columns * tile_size + (columns + 1) * padding;
    let height = rows * tile_size + (rows + 1) * padding;
    let mut image = PixelImage::new(width, height, Rgba8::opaque(0, 0, 0));

    for (i, asset) in tileset.tiles.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        let x = padding + col * (tile_size + padding);
        let y = padding + row * (tile_size + padding);
        let tile = build_tile(asset, &tileset.recipe);
        image.blit(&tile, x, y);
    }

    image
}

pub fn build_height_mask_tile(asset: &TileAsset, recipe: &TilesetRecipe) -> PixelImage {
    let size = recipe.tile_size;
    let mut image = PixelImage::transparent(size, size);
    for y in 0..size {
        for x in 0..size {
            let nx = if size <= 1 {
                0.0
            } else {
                x as f32 / (size - 1) as f32
            };
            let ny = if size <= 1 {
                0.0
            } else {
                y as f32 / (size - 1) as f32
            };
            let base = material_height_signal(asset.meta.material, nx, ny);
            let signal = match asset.meta.role {
                TileRole::Surface => base,
                TileRole::Transition => {
                    let to = asset.meta.transition_to.unwrap_or(asset.meta.material);
                    let edge = asset.meta.transition_edge.unwrap_or(TransitionEdge::North);
                    let to_signal = material_height_signal(to, nx, ny);
                    let alpha = transition_alpha(edge, nx, ny, asset.meta.variant, recipe);
                    base * (1.0 - alpha) + to_signal * alpha
                }
                TileRole::StructureFace => {
                    let face = asset
                        .meta
                        .structure_face
                        .unwrap_or(StructureFaceKind::Front);
                    structure_face_height_signal(asset.meta.material, face, nx, ny)
                }
            };
            let grain = (hash01(recipe.seed, x, y, asset.meta.variant.wrapping_add(9001)) - 0.5)
                * 16.0
                * if asset.meta.role == TileRole::StructureFace {
                    recipe.face_detail_density
                } else {
                    recipe.detail_density
                };
            let value = clamp_u8((signal * 255.0) + grain);
            image.set(x, y, Rgba8::opaque(value, value, value));
        }
    }
    image
}

pub fn build_shadow_mask_tile(asset: &TileAsset, recipe: &TilesetRecipe) -> PixelImage {
    let size = recipe.tile_size;
    let mut image = PixelImage::transparent(size, size);
    let (lx, ly) = recipe.light_direction.vector();
    for y in 0..size {
        for x in 0..size {
            let nx = if size <= 1 {
                0.0
            } else {
                x as f32 / (size - 1) as f32
            };
            let ny = if size <= 1 {
                0.0
            } else {
                y as f32 / (size - 1) as f32
            };
            let directional = ((nx - 0.5) * lx + (ny - 0.5) * ly).max(0.0);
            let mut role_shadow = match asset.meta.material {
                GroundMaterial::TrenchFloor => 0.48,
                GroundMaterial::TrenchWall => 0.62,
                GroundMaterial::BermFace => 0.58,
                GroundMaterial::BermTop => 0.26,
                GroundMaterial::Mud => 0.22,
                GroundMaterial::Rock => 0.18,
                GroundMaterial::Grass | GroundMaterial::Dirt => 0.12,
            };
            if asset.meta.role == TileRole::StructureFace {
                let face = asset
                    .meta
                    .structure_face
                    .unwrap_or(StructureFaceKind::Front);
                role_shadow += match face {
                    StructureFaceKind::Front => 0.22 + ny * 0.18,
                    StructureFaceKind::Left => 0.34 + ny * 0.20,
                    StructureFaceKind::Right => 0.16 + ny * 0.12,
                    StructureFaceKind::Lip => 0.06,
                };
            }
            let edge_shadow = if nx < 0.05 || ny < 0.05 || nx > 0.95 || ny > 0.95 {
                0.22
            } else {
                0.0
            };
            let strength = if asset.meta.role == TileRole::StructureFace {
                recipe.face_shadow_strength.max(recipe.shadow_strength)
            } else {
                recipe.shadow_strength
            };
            let value =
                clamp_u8((role_shadow + directional * 0.34 + edge_shadow) * 255.0 * strength);
            image.set(x, y, Rgba8::opaque(value, value, value));
        }
    }
    image
}

pub fn build_occlusion_mask_tile(asset: &TileAsset, recipe: &TilesetRecipe) -> PixelImage {
    let size = recipe.tile_size;
    let mut image = PixelImage::transparent(size, size);
    let base = match asset.meta.material {
        GroundMaterial::TrenchWall | GroundMaterial::BermFace => 210,
        GroundMaterial::BermTop => 120,
        GroundMaterial::Rock => 110,
        GroundMaterial::TrenchFloor => 70,
        GroundMaterial::Mud => 35,
        GroundMaterial::Grass | GroundMaterial::Dirt => 20,
    };
    let role_bonus = match asset.meta.role {
        TileRole::Transition => 18,
        TileRole::StructureFace => 90,
        TileRole::Surface => 0,
    };
    let value = (base + role_bonus).min(255) as u8;
    for y in 0..size {
        for x in 0..size {
            image.set(x, y, Rgba8::opaque(value, value, value));
        }
    }
    image
}

pub fn build_normal_map_from_height(height: &PixelImage, strength: f32) -> PixelImage {
    let mut normal = PixelImage::new(height.width, height.height, Rgba8::opaque(128, 128, 255));
    if height.width == 0 || height.height == 0 {
        return normal;
    }

    let strength = strength.max(0.001);
    for y in 0..height.height {
        for x in 0..height.width {
            let x0 = x.saturating_sub(1);
            let x1 = (x + 1).min(height.width - 1);
            let y0 = y.saturating_sub(1);
            let y1 = (y + 1).min(height.height - 1);

            let left = height.get(x0, y).luma() as f32 / 255.0;
            let right = height.get(x1, y).luma() as f32 / 255.0;
            let up = height.get(x, y0).luma() as f32 / 255.0;
            let down = height.get(x, y1).luma() as f32 / 255.0;

            let dx = (right - left) * strength;
            let dy = (down - up) * strength;
            let mut nx = -dx;
            let mut ny = -dy;
            let mut nz = 1.0_f32;
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            nx /= len;
            ny /= len;
            nz /= len;

            normal.set(
                x,
                y,
                Rgba8::opaque(
                    clamp_u8((nx * 0.5 + 0.5) * 255.0),
                    clamp_u8((ny * 0.5 + 0.5) * 255.0),
                    clamp_u8((nz * 0.5 + 0.5) * 255.0),
                ),
            );
        }
    }
    normal
}

fn material_height_signal(material: GroundMaterial, nx: f32, ny: f32) -> f32 {
    match material {
        GroundMaterial::Grass => 0.50 + soft_center_crown(nx, ny) * 0.04,
        GroundMaterial::Dirt => 0.49 + soft_center_crown(nx, ny) * 0.03,
        GroundMaterial::Mud => 0.42 - soft_center_crown(nx, ny) * 0.04,
        GroundMaterial::Rock => 0.58 + fractured_height(nx, ny) * 0.10,
        GroundMaterial::TrenchFloor => 0.28 - soft_center_crown(nx, ny) * 0.08,
        GroundMaterial::TrenchWall => 0.42 + ny * 0.28,
        GroundMaterial::BermTop => 0.66 + soft_center_crown(nx, ny) * 0.10,
        GroundMaterial::BermFace => 0.36 + (1.0 - ny) * 0.34,
    }
    .clamp(0.0, 1.0)
}

fn structure_face_height_signal(
    material: GroundMaterial,
    face: StructureFaceKind,
    nx: f32,
    ny: f32,
) -> f32 {
    let material_base = match material {
        GroundMaterial::Rock => 0.60,
        GroundMaterial::TrenchWall => 0.44,
        GroundMaterial::BermFace => 0.50,
        GroundMaterial::Mud => 0.38,
        GroundMaterial::Dirt
        | GroundMaterial::Grass
        | GroundMaterial::TrenchFloor
        | GroundMaterial::BermTop => 0.46,
    };
    let directional = match face {
        StructureFaceKind::Front => 0.22 + ny * 0.36,
        StructureFaceKind::Left => 0.18 + ny * 0.42 - nx * 0.10,
        StructureFaceKind::Right => 0.30 + ny * 0.28 + nx * 0.08,
        StructureFaceKind::Lip => 0.62 + soft_center_crown(nx, ny) * 0.10,
    };
    (material_base + directional * 0.45).clamp(0.0, 1.0)
}

fn soft_center_crown(nx: f32, ny: f32) -> f32 {
    let d = ((nx - 0.5).abs() + (ny - 0.5).abs()).min(1.0);
    1.0 - d
}

fn fractured_height(nx: f32, ny: f32) -> f32 {
    ((nx * 6.0).sin() * 0.5 + (ny * 7.0).cos() * 0.5).abs()
}

fn transition_alpha(
    edge: TransitionEdge,
    nx: f32,
    ny: f32,
    variant: u32,
    recipe: &TilesetRecipe,
) -> f32 {
    let edge_t = match edge {
        TransitionEdge::North => ny,
        TransitionEdge::South => 1.0 - ny,
        TransitionEdge::West => nx,
        TransitionEdge::East => 1.0 - nx,
    };
    let noise = (hash01(
        recipe.seed,
        (nx * 255.0) as u32,
        (ny * 255.0) as u32,
        variant + 313,
    ) - 0.5)
        * 0.10;
    let boundary = 0.36 + noise;
    ((boundary - edge_t) / recipe.transition_feather + 0.5).clamp(0.0, 1.0)
}
