use crate::color::{clamp01, lerp_f32, Rgba8};
use crate::pixel_image::PixelImage;
use crate::spritegen::{
    GeneratedTerrainSprite, TerrainSpriteKind, TerrainSpriteRecipe, TerrainSpriteStyle,
};

pub fn generate_terrain_sprites(recipe: &TerrainSpriteRecipe) -> Vec<GeneratedTerrainSprite> {
    let mut recipe = recipe.clone();
    recipe.sanitize();
    let mut sprites = Vec::new();

    for variant in 1..=recipe.variant_count {
        sprites.push(GeneratedTerrainSprite {
            id: format!("grass_tile_{variant:02}"),
            kind: TerrainSpriteKind::GrassTile,
            variant,
            image: generate_grass_tile(&recipe, variant),
        });
    }
    for variant in 1..=recipe.variant_count {
        sprites.push(GeneratedTerrainSprite {
            id: format!("dirt_tile_{variant:02}"),
            kind: TerrainSpriteKind::DirtTile,
            variant,
            image: generate_dirt_tile(&recipe, variant),
        });
    }
    for kind in [
        TerrainSpriteKind::GrassToDirtEdgeNorth,
        TerrainSpriteKind::GrassToDirtEdgeSouth,
        TerrainSpriteKind::GrassToDirtEdgeEast,
        TerrainSpriteKind::GrassToDirtEdgeWest,
    ] {
        sprites.push(GeneratedTerrainSprite {
            id: format!("{}_01", kind.id()),
            kind,
            variant: 1,
            image: generate_transition_tile(&recipe, kind, 1),
        });
    }

    sprites
}

pub fn generate_grass_tile(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let size = recipe.tile_size;
    let style = &recipe.style;
    let palette = &style.palette;
    let seed = sprite_seed(recipe.seed, variant, 0x101);
    let mut image = PixelImage::new(size, size, palette.grass_mid);

    fill_soft_base(
        &mut image,
        seed,
        &[palette.grass_dark, palette.grass_mid, palette.grass_light],
        style,
    );

    let dark_count = density_count(size, style.grass.dark_cluster_density);
    for i in 0..dark_count {
        let (x, y) = random_tile_point(seed, i, 0x201, size);
        let rx = random_range(seed, i, 0x202, 1, style.pixel.max_cluster_size);
        let ry = random_range(seed, i, 0x203, 1, style.pixel.max_cluster_size);
        draw_wrapped_blob(
            &mut image,
            x,
            y,
            rx,
            ry,
            palette.grass_dark,
            seed ^ i as u64,
        );
    }

    let light_count = density_count(size, style.grass.highlight_cluster_density);
    for i in 0..light_count {
        let (x, y) = random_tile_point(seed, i, 0x301, size);
        draw_wrapped_blade_cluster(
            &mut image,
            x,
            y,
            palette.grass_light,
            seed ^ 0x3310 ^ i as u64,
            2,
        );
    }

    let blade_count = density_count(size, style.grass.blade_cluster_density);
    for i in 0..blade_count {
        let (x, y) = random_tile_point(seed, i, 0x401, size);
        let color = if i % 3 == 0 {
            palette.grass_shadow
        } else {
            palette.grass_light
        };
        draw_wrapped_blade_cluster(&mut image, x, y, color, seed ^ i as u64, 2);
    }

    let flower_count = ((size * size) as f32 * style.grass.flower_density).round() as u32;
    for i in 0..flower_count {
        let (x, y) = random_tile_point(seed, i, 0x501, size);
        set_wrapped(&mut image, x, y, palette.grass_flower);
        if i % 3 == 0 {
            set_wrapped(&mut image, x + 1, y, palette.grass_flower.darken(0.12));
        }
    }

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    enforce_palette(&mut image, &style.palette.all_colors());
    image
}

pub fn generate_dirt_tile(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let size = recipe.tile_size;
    let style = &recipe.style;
    let palette = &style.palette;
    let seed = sprite_seed(recipe.seed, variant, 0x601);
    let mut image = PixelImage::new(size, size, palette.dirt_mid);

    fill_soft_base(
        &mut image,
        seed,
        &[palette.dirt_dark, palette.dirt_mid, palette.dirt_light],
        style,
    );

    let dust_count = density_count(size, style.dirt.dust_patch_density);
    for i in 0..dust_count {
        let (x, y) = random_tile_point(seed, i, 0x701, size);
        let rx = random_range(seed, i, 0x702, 1, style.pixel.max_cluster_size + 1);
        let ry = random_range(seed, i, 0x703, 1, style.pixel.max_cluster_size.max(2));
        draw_wrapped_blob(
            &mut image,
            x,
            y,
            rx,
            ry,
            palette.dirt_light,
            seed ^ i as u64,
        );
    }

    let dent_count = density_count(size, style.dirt.compact_shadow_density);
    for i in 0..dent_count {
        let (x, y) = random_tile_point(seed, i, 0x801, size);
        draw_wrapped_blob(
            &mut image,
            x,
            y,
            1 + (i % 2),
            1,
            palette.dirt_dark,
            seed ^ 0x8877 ^ i as u64,
        );
    }

    let rut_count = ((size * size) as f32 * style.dirt.rut_density / 10.0).round() as u32;
    for i in 0..rut_count {
        let (x, y) = random_tile_point(seed, i, 0x901, size);
        let len = random_range(seed, i, 0x902, 2, 5);
        let color = if i % 2 == 0 {
            palette.dirt_shadow
        } else {
            palette.dirt_light
        };
        for dx in 0..len {
            set_wrapped(&mut image, x + dx, y + (dx % 2), color);
        }
    }

    let pebble_count = ((size * size) as f32 * style.dirt.pebble_density).round() as u32;
    for i in 0..pebble_count {
        let (x, y) = random_tile_point(seed, i, 0xa01, size);
        set_wrapped(&mut image, x, y, palette.pebble);
        if i % 5 == 0 {
            set_wrapped(&mut image, x + 1, y, palette.pebble.darken(0.20));
        }
    }

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    enforce_palette(&mut image, &style.palette.all_colors());
    image
}

pub fn generate_transition_tile(
    recipe: &TerrainSpriteRecipe,
    kind: TerrainSpriteKind,
    variant: u32,
) -> PixelImage {
    debug_assert!(kind.is_transition());
    let size = recipe.tile_size;
    let style = &recipe.style;
    let palette = &style.palette;
    let seed = sprite_seed(recipe.seed, variant, 0xb01 ^ kind as u64);
    let mut image = generate_grass_tile(recipe, variant);
    let jitter = style.transition.edge_jitter_px as f32;

    for y in 0..size {
        for x in 0..size {
            let along = match kind {
                TerrainSpriteKind::GrassToDirtEdgeNorth
                | TerrainSpriteKind::GrassToDirtEdgeSouth => x,
                TerrainSpriteKind::GrassToDirtEdgeEast | TerrainSpriteKind::GrassToDirtEdgeWest => {
                    y
                }
                _ => 0,
            };
            let across = match kind {
                TerrainSpriteKind::GrassToDirtEdgeNorth
                | TerrainSpriteKind::GrassToDirtEdgeSouth => y,
                TerrainSpriteKind::GrassToDirtEdgeEast | TerrainSpriteKind::GrassToDirtEdgeWest => {
                    x
                }
                _ => 0,
            };
            let wave = (tile_noise(seed, along as i32, 0, 0xc01, size) - 0.5) * jitter * 2.0;
            let threshold = size as f32 * 0.50 + wave;
            let dirt_side = match kind {
                TerrainSpriteKind::GrassToDirtEdgeNorth => across as f32 <= threshold,
                TerrainSpriteKind::GrassToDirtEdgeSouth => across as f32 >= threshold,
                TerrainSpriteKind::GrassToDirtEdgeWest => across as f32 <= threshold,
                TerrainSpriteKind::GrassToDirtEdgeEast => across as f32 >= threshold,
                _ => false,
            };
            let distance = (across as f32 - threshold).abs();
            let edge_zone = distance < size as f32 * 0.25;
            if dirt_side {
                let n = tile_noise(seed, x as i32, y as i32, 0xc02, size);
                let dirt = sample_ramp(&palette.dirt_ramp(), 0.38 + (n - 0.5) * 0.35);
                let grass_blend = if edge_zone {
                    clamp01(1.0 - distance / (size as f32 * 0.25)) * style.transition.edge_softness
                } else {
                    0.0
                };
                image.set(x, y, dirt.blend(palette.grass_mid, grass_blend * 0.45));
            } else if edge_zone {
                let chance = tile_noise(seed, x as i32, y as i32, 0xc03, size);
                if chance < style.transition.dirt_speckle_density {
                    image.set(x, y, palette.dirt_light);
                }
            }
        }
    }

    let intrusion_count = density_count(size, style.transition.grass_intrusion_density);
    for i in 0..intrusion_count {
        let (x, y) = random_tile_point(seed, i, 0xd01, size);
        draw_wrapped_blade_cluster(
            &mut image,
            x,
            y,
            if i % 2 == 0 {
                palette.grass_light
            } else {
                palette.grass_dark
            },
            seed ^ i as u64,
            2,
        );
    }

    let pebble_count = ((size * size) as f32 * style.dirt.pebble_density * 0.55).round() as u32;
    for i in 0..pebble_count {
        let (x, y) = random_tile_point(seed, i, 0xe01, size);
        if image.get(x, y).rgb_distance(palette.dirt_mid) < 70.0 {
            image.set(x, y, palette.pebble);
        }
    }

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    enforce_palette(&mut image, &style.palette.all_colors());
    image
}

pub fn scale_nearest(image: &PixelImage, scale: u32) -> PixelImage {
    let scale = scale.max(1);
    let mut out = PixelImage::transparent(image.width * scale, image.height * scale);
    for y in 0..image.height {
        for x in 0..image.width {
            let color = image.get(x, y);
            out.fill_rect(x * scale, y * scale, scale, scale, color);
        }
    }
    out
}

fn fill_soft_base(
    image: &mut PixelImage,
    seed: u64,
    colors: &[Rgba8; 3],
    style: &TerrainSpriteStyle,
) {
    let size = image.width.min(image.height).max(1);
    let cluster = style.pixel.min_cluster_size.max(1);
    for y in 0..image.height {
        for x in 0..image.width {
            let n = tile_noise(
                seed,
                (x / cluster) as i32,
                (y / cluster) as i32,
                0x1101,
                size,
            );
            let color = if n < 0.10 {
                colors[0]
            } else if n > 0.92 {
                colors[2]
            } else {
                colors[1]
            };
            image.set(x, y, color);
        }
    }
}

fn density_count(size: u32, density: f32) -> u32 {
    ((size * size) as f32 * density / 10.0).round().max(1.0) as u32
}

fn draw_wrapped_blob(
    image: &mut PixelImage,
    cx: u32,
    cy: u32,
    rx: u32,
    ry: u32,
    color: Rgba8,
    seed: u64,
) {
    let rx = rx.max(1) as i32;
    let ry = ry.max(1) as i32;
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            let nx = dx as f32 / rx as f32;
            let ny = dy as f32 / ry as f32;
            let noise = hash01(seed ^ ((dx as u64) << 32) ^ dy as u64);
            if nx * nx + ny * ny <= 1.0 + (noise - 0.5) * 0.35 {
                set_wrapped(
                    image,
                    cx.wrapping_add_signed(dx),
                    cy.wrapping_add_signed(dy),
                    color,
                );
            }
        }
    }
}

fn draw_wrapped_blade_cluster(
    image: &mut PixelImage,
    x: u32,
    y: u32,
    color: Rgba8,
    seed: u64,
    size: u32,
) {
    let count = 2 + (hash(seed) % size.max(1) as u64) as u32;
    for i in 0..count {
        let dx = (hash(seed ^ i as u64 ^ 0x71) % 3) as i32 - 1;
        let len = 1 + (hash(seed ^ i as u64 ^ 0x91) % 3) as i32;
        for dy in 0..len {
            set_wrapped(
                image,
                x.wrapping_add_signed(dx + dy / 2),
                y.wrapping_add_signed(-dy),
                color,
            );
        }
    }
}

fn soften_isolated_pixels(image: &mut PixelImage) {
    let original = image.clone();
    for y in 0..image.height {
        for x in 0..image.width {
            let center = original.get(x, y);
            let mut same = 0;
            let mut neighbors = Vec::with_capacity(8);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = wrap_i32(x as i32 + dx, image.width);
                    let ny = wrap_i32(y as i32 + dy, image.height);
                    let candidate = original.get(nx, ny);
                    neighbors.push(candidate);
                    if candidate == center {
                        same += 1;
                    }
                }
            }
            if same == 0 {
                neighbors.sort_by_key(|color| color.luma());
                image.set(x, y, neighbors[neighbors.len() / 2]);
            }
        }
    }
}

fn enforce_palette(image: &mut PixelImage, palette: &[Rgba8]) {
    for y in 0..image.height {
        for x in 0..image.width {
            let color = image.get(x, y);
            let nearest = nearest_color(color, palette);
            image.set(x, y, nearest.with_alpha(color.a));
        }
    }
}

fn nearest_color(color: Rgba8, palette: &[Rgba8]) -> Rgba8 {
    palette
        .iter()
        .copied()
        .min_by(|a, b| {
            a.rgb_distance(color)
                .partial_cmp(&b.rgb_distance(color))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(color)
}

fn sample_ramp(ramp: &[Rgba8; 4], t: f32) -> Rgba8 {
    let t = clamp01(t);
    let scaled = t * 3.0;
    let i0 = scaled.floor() as usize;
    let i1 = (i0 + 1).min(3);
    let frac = scaled - i0 as f32;
    ramp[i0].blend(ramp[i1], frac)
}

fn random_tile_point(seed: u64, index: u32, salt: u64, size: u32) -> (u32, u32) {
    let x = (hash(seed ^ salt ^ (index as u64 * 0x9e37_79b9)) % size as u64) as u32;
    let y = (hash(seed ^ (salt << 1) ^ (index as u64 * 0x85eb_ca6b)) % size as u64) as u32;
    (x, y)
}

fn random_range(seed: u64, index: u32, salt: u64, min: u32, max: u32) -> u32 {
    if max <= min {
        return min;
    }
    min + (hash(seed ^ salt ^ (index as u64 * 0xc2b2_ae35)) % (max - min + 1) as u64) as u32
}

fn set_wrapped(image: &mut PixelImage, x: u32, y: u32, color: Rgba8) {
    let x = x % image.width.max(1);
    let y = y % image.height.max(1);
    image.set(x, y, color);
}

fn wrap_i32(value: i32, size: u32) -> u32 {
    value.rem_euclid(size.max(1) as i32) as u32
}

fn tile_noise(seed: u64, x: i32, y: i32, salt: u64, period: u32) -> f32 {
    let period = period.max(1) as i32;
    let x = x.rem_euclid(period) as u64;
    let y = y.rem_euclid(period) as u64;
    let a = hash01(seed ^ salt ^ x.wrapping_mul(0x9e37_79b9) ^ y.wrapping_mul(0x85eb_ca6b));
    let b = hash01(seed ^ salt ^ ((period as u64 - x) * 0xc2b2_ae35) ^ (y * 0x27d4_eb2f));
    let c = hash01(seed ^ salt ^ (x * 0x1656_67b1) ^ ((period as u64 - y) * 0xd3a2_646c));
    lerp_f32(a, (a + b + c) / 3.0, 0.45)
}

fn sprite_seed(seed: u64, variant: u32, salt: u64) -> u64 {
    hash(seed ^ salt ^ (variant as u64 * 0x9e37_79b9))
}

fn hash01(value: u64) -> f32 {
    hash(value) as f32 / u64::MAX as f32
}

fn hash(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
