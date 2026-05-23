use crate::color::Rgba8;
use crate::pixel_image::PixelImage;
use crate::spritegen::{GeneratedTerrainSprite, TerrainSpriteKind, TerrainSpriteRecipe};

#[derive(Clone, Copy, Debug)]
struct MotifPixel {
    dx: i32,
    dy: i32,
    shade: i8,
}

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
    for mask in 0..16 {
        let kind = TerrainSpriteKind::from_path_mask(mask).expect("valid path mask");
        sprites.push(GeneratedTerrainSprite {
            id: format!("path_mask_{mask:02}"),
            kind,
            variant: 1,
            image: generate_path_mask_tile(&recipe, mask),
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

    let mut occupied = Vec::new();
    let cell = (size / 5).max(3);
    let variant_weight = if variant == 1 { 0.70 } else { 1.0 };
    let dark_chance = style.grass.dark_cluster_density * 0.95 * variant_weight;
    let light_chance = style.grass.highlight_cluster_density * 0.90 * variant_weight;
    let blade_chance = style.grass.blade_cluster_density * 0.85 * variant_weight;

    scatter_motifs(
        &mut image,
        seed ^ 0x201,
        cell,
        dark_chance,
        &[
            DARK_CLUMP_A,
            DARK_CLUMP_B,
            DARK_CLUMP_C,
            SOFT_GRASS_DARK_A,
            SHADOW_POCKET_A,
            SHADOW_POCKET_B,
        ],
        &grass_color_picker(recipe, false),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x301,
        cell,
        light_chance,
        &[
            LIGHT_LEAF_A,
            LIGHT_LEAF_B,
            LIGHT_LEAF_C,
            LIGHT_SPECK_A,
            LIGHT_SPECK_B,
            SEED_FLECK_A,
        ],
        &grass_color_picker(recipe, true),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x401,
        cell,
        blade_chance,
        &[BLADE_A, BLADE_B, BLADE_C, BLADE_D, BLADE_E, BLADE_F],
        &grass_color_picker(recipe, true),
        &mut occupied,
    );

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    let flower_count = ((size * size) as f32 * style.grass.flower_density).round() as u32;
    for i in 0..flower_count {
        if let Some((x, y)) = find_spaced_point(seed ^ 0x501, i, size, 4, &occupied) {
            draw_motif(
                &mut image,
                x,
                y,
                FLOWER_FLECK_A,
                &grass_color_picker(recipe, true),
            );
            occupied.push((x as i32, y as i32));
        }
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

    let mut occupied = Vec::new();
    let cell = (size / 6).max(2);
    let variant_weight = if variant == 1 { 0.90 } else { 1.0 };
    scatter_motifs(
        &mut image,
        seed ^ 0x701,
        cell,
        style.dirt.dust_patch_density * 1.60 * variant_weight,
        &[
            DUST_SMEAR_A,
            DUST_SMEAR_B,
            DUST_SMEAR_C,
            DUST_SMEAR_D,
            DUST_SMEAR_E,
        ],
        &dirt_color_picker(recipe, true),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x801,
        cell,
        style.dirt.compact_shadow_density * 0.90 * variant_weight,
        &[DIRT_DENT_A, DIRT_DENT_B, DIRT_DENT_C, DIRT_DENT_D],
        &dirt_color_picker(recipe, false),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x901,
        cell,
        style.dirt.rut_density * 0.25 * variant_weight,
        &[RUT_A, RUT_B, RUT_C, RUT_D, RUT_E],
        &dirt_color_picker(recipe, false),
        &mut occupied,
    );

    let pebble_count =
        ((size * size) as f32 * style.dirt.pebble_density * 0.55 * variant_weight).round() as u32;
    for i in 0..pebble_count {
        if let Some((x, y)) = find_spaced_point(seed ^ 0xa01, i, size, 3, &occupied) {
            set_wrapped(&mut image, x, y, palette.pebble);
            if hash(seed ^ i as u64 ^ 0xa02).is_multiple_of(4) {
                set_wrapped(&mut image, x + 1, y, palette.dirt_dark);
            }
            occupied.push((x as i32, y as i32));
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
    let grass = generate_grass_tile(recipe, variant);
    let dirt = generate_dirt_tile(recipe, variant);
    let mut image = grass.clone();
    let jitter = style.transition.edge_jitter_px as i32;
    let edge_width = (size as f32 * (0.14 + style.transition.edge_softness * 0.18))
        .round()
        .max(2.0) as i32;

    for y in 0..size {
        for x in 0..size {
            let along = transition_along(kind, x, y);
            let across = transition_across(kind, x, y);
            let threshold = size as i32 / 2 + edge_jitter(seed, along, size, jitter);
            let signed = across as i32 - threshold;
            let dirt_side = transition_is_dirt_side(kind, signed);
            let distance = signed.abs();
            if dirt_side {
                let color = dirt.get(x, y);
                if distance <= edge_width {
                    let grass_mix = 1.0 - distance as f32 / edge_width as f32;
                    image.set(x, y, color.blend(palette.grass_mid, grass_mix * 0.34));
                } else {
                    image.set(x, y, color);
                }
            } else if distance <= edge_width {
                let chance = hash01(seed ^ 0xc03 ^ ((x as u64) << 16) ^ y as u64);
                if chance < style.transition.dirt_speckle_density * 0.45 {
                    image.set(x, y, palette.dirt_light);
                }
            }
        }
    }

    let mut occupied = Vec::new();
    let intrusion_count =
        ((size * size) as f32 * style.transition.grass_intrusion_density / 18.0) as u32;
    for i in 0..intrusion_count.max(1) {
        let along = (hash(seed ^ 0xd01 ^ i as u64) % size as u64) as u32;
        let threshold = size as i32 / 2 + edge_jitter(seed, along, size, jitter);
        let push = 1 + (hash(seed ^ 0xd02 ^ i as u64) % edge_width.max(1) as u64) as i32;
        let across = match kind {
            TerrainSpriteKind::GrassToDirtEdgeNorth | TerrainSpriteKind::GrassToDirtEdgeWest => {
                (threshold + push).clamp(0, size as i32 - 1) as u32
            }
            TerrainSpriteKind::GrassToDirtEdgeSouth | TerrainSpriteKind::GrassToDirtEdgeEast => {
                (threshold - push).clamp(0, size as i32 - 1) as u32
            }
            _ => 0,
        };
        let (x, y) = transition_point(kind, along, across);
        if spaced(x as i32, y as i32, 3, &occupied) {
            let motif = if i % 2 == 0 { BLADE_A } else { LIGHT_LEAF_C };
            draw_motif(&mut image, x, y, motif, &grass_color_picker(recipe, true));
            occupied.push((x as i32, y as i32));
        }
    }

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    enforce_palette(&mut image, &style.palette.all_colors());
    image
}

pub fn generate_path_mask_tile(recipe: &TerrainSpriteRecipe, mask: u8) -> PixelImage {
    let size = recipe.tile_size;
    let style = &recipe.style;
    let palette = &style.palette;
    let seed = sprite_seed(recipe.seed, mask as u32 + 1, 0x1801);
    let grass_variant = mask as u32 % recipe.variant_count.max(1) + 1;
    let dirt_variant = (mask as u32 + 1) % recipe.variant_count.max(1) + 1;
    let grass = generate_grass_tile(recipe, grass_variant);
    let dirt = generate_dirt_tile(recipe, dirt_variant);
    let mut image = grass.clone();
    let soft_edge = (size as f32 * (0.06 + style.transition.edge_softness * 0.07)).max(1.4);

    for y in 0..size {
        for x in 0..size {
            let signed = path_mask_signed_distance(mask, x, y, size, seed);
            if signed <= 0.0 {
                let color = dirt.get(x, y);
                if signed.abs() <= soft_edge {
                    let grass_mix = 1.0 - signed.abs() / soft_edge;
                    image.set(x, y, color.blend(palette.grass_mid, grass_mix * 0.16));
                } else {
                    image.set(x, y, color);
                }
            } else if signed <= soft_edge {
                let dirt_mix = 1.0 - signed / soft_edge;
                image.blend_pixel(x, y, dirt.get(x, y), dirt_mix * 0.38);
                let speckle = hash01(seed ^ 0x1a01 ^ ((x as u64) << 16) ^ y as u64);
                if speckle < style.transition.dirt_speckle_density * 0.55 {
                    image.set(x, y, palette.dirt_light.blend(palette.grass_mid, 0.25));
                }
            }
        }
    }

    add_path_edge_intrusions(&mut image, recipe, mask, seed);
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

fn add_path_edge_intrusions(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    seed: u64,
) {
    let size = image.width.min(image.height);
    let style = &recipe.style;
    let count = ((size * size) as f32 * style.transition.grass_intrusion_density / 14.0)
        .round()
        .max(1.0) as u32;
    let mut occupied = Vec::new();
    for i in 0..count {
        let sample = hash(seed ^ 0x1b01 ^ (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut best = None;
        for attempt in 0..10 {
            let x = (hash(sample ^ attempt as u64 ^ 0x1b02) % size as u64) as u32;
            let y = (hash(sample ^ (attempt as u64 * 0x85eb) ^ 0x1b03) % size as u64) as u32;
            let signed = path_mask_signed_distance(mask, x, y, size, seed);
            if (-1.0..=2.5).contains(&signed) && spaced(x as i32, y as i32, 3, &occupied) {
                best = Some((x, y));
                break;
            }
        }
        if let Some((x, y)) = best {
            let motif = match hash(sample ^ 0x1b04) % 4 {
                0 => BLADE_A,
                1 => BLADE_D,
                2 => LIGHT_LEAF_C,
                _ => SOFT_GRASS_DARK_A,
            };
            draw_motif(image, x, y, motif, &grass_color_picker(recipe, true));
            occupied.push((x as i32, y as i32));
        }
    }
}

fn path_mask_signed_distance(mask: u8, x: u32, y: u32, size: u32, seed: u64) -> f32 {
    let arm_half = (size as f32 * 0.18).max(2.7);
    let core_half = (size as f32 * 0.21).max(3.2);
    let center = (size as f32 - 1.0) * 0.5;
    let xf = x as f32 + 0.5;
    let yf = y as f32 + 0.5;
    let mut distance = rect_signed_distance(
        xf,
        yf,
        center - core_half,
        center + core_half,
        center - core_half,
        center + core_half,
    );
    if mask & 1 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            center - arm_half,
            center + arm_half,
            -1.0,
            center + core_half,
        ));
    }
    if mask & 2 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            center - core_half,
            size as f32 + 1.0,
            center - arm_half,
            center + arm_half,
        ));
    }
    if mask & 4 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            center - arm_half,
            center + arm_half,
            center - core_half,
            size as f32 + 1.0,
        ));
    }
    if mask & 8 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            -1.0,
            center + core_half,
            center - arm_half,
            center + arm_half,
        ));
    }
    if mask == 0 {
        let dx = xf - center;
        let dy = yf - center;
        distance = (dx * dx + dy * dy).sqrt() - size as f32 * 0.24;
    }
    let organic = path_edge_noise(seed, x, y) * 0.85;
    distance + organic
}

fn rect_signed_distance(x: f32, y: f32, x0: f32, x1: f32, y0: f32, y1: f32) -> f32 {
    let outside_x = if x < x0 {
        x0 - x
    } else if x > x1 {
        x - x1
    } else {
        0.0
    };
    let outside_y = if y < y0 {
        y0 - y
    } else if y > y1 {
        y - y1
    } else {
        0.0
    };
    if outside_x > 0.0 || outside_y > 0.0 {
        return (outside_x * outside_x + outside_y * outside_y).sqrt();
    }
    let inside = (x - x0).min(x1 - x).min(y - y0).min(y1 - y);
    -inside
}

fn path_edge_noise(seed: u64, x: u32, y: u32) -> f32 {
    let coarse_x = x / 2;
    let coarse_y = y / 2;
    let raw = hash01(seed ^ 0x1c01 ^ ((coarse_x as u64) << 18) ^ coarse_y as u64);
    (raw - 0.5) * 2.0
}

fn scatter_motifs(
    image: &mut PixelImage,
    seed: u64,
    cell: u32,
    chance: f32,
    motifs: &[&[MotifPixel]],
    color_picker: &dyn Fn(i8) -> Rgba8,
    occupied: &mut Vec<(i32, i32)>,
) {
    let size = image.width.min(image.height);
    let count = ((size * size) as f32 * chance / 7.5).round().max(1.0) as u32;
    let min_distance = cell.max(2) as i32;
    for i in 0..count {
        if let Some((x, y)) = find_spaced_point(seed, i, size, min_distance, occupied) {
            let motif = motifs[(hash(seed ^ (i as u64 * 0x517c)) % motifs.len() as u64) as usize];
            draw_motif(image, x, y, motif, color_picker);
            occupied.push((x as i32, y as i32));
        }
    }
}

fn draw_motif(
    image: &mut PixelImage,
    origin_x: u32,
    origin_y: u32,
    motif: &[MotifPixel],
    color_picker: &dyn Fn(i8) -> Rgba8,
) {
    for pixel in motif {
        set_wrapped(
            image,
            origin_x.wrapping_add_signed(pixel.dx),
            origin_y.wrapping_add_signed(pixel.dy),
            color_picker(pixel.shade),
        );
    }
}

fn find_spaced_point(
    seed: u64,
    index: u32,
    size: u32,
    min_distance: i32,
    occupied: &[(i32, i32)],
) -> Option<(u32, u32)> {
    for attempt in 0..8 {
        let sample = hash(
            seed ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (attempt as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9),
        );
        let x = (hash(sample ^ 0xd1b5_4a32_d192_ed03) % size as u64) as u32;
        let y = (hash(sample ^ 0x94d0_49bb_1331_11eb) % size as u64) as u32;
        if spaced(x as i32, y as i32, min_distance, occupied) {
            return Some((x, y));
        }
    }
    None
}

fn spaced(x: i32, y: i32, min_distance: i32, occupied: &[(i32, i32)]) -> bool {
    occupied.iter().all(|(ox, oy)| {
        let dx = x - *ox;
        let dy = y - *oy;
        dx * dx + dy * dy >= min_distance * min_distance
    })
}

fn grass_color_picker(recipe: &TerrainSpriteRecipe, bright: bool) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match (bright, shade) {
            (_, -2) => palette.grass_shadow,
            (_, -1) => palette.grass_dark,
            (true, 1) => palette.grass_light,
            (true, 2) => palette.grass_flower,
            _ => palette.grass_mid,
        }
    }
}

fn dirt_color_picker(recipe: &TerrainSpriteRecipe, bright: bool) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match (bright, shade) {
            (_, -2) => palette.dirt_shadow,
            (_, -1) => palette.dirt_dark,
            (true, 1) => palette.dirt_light,
            (true, 2) => palette.pebble,
            _ => palette.dirt_mid,
        }
    }
}

fn transition_along(kind: TerrainSpriteKind, x: u32, y: u32) -> u32 {
    match kind {
        TerrainSpriteKind::GrassToDirtEdgeNorth | TerrainSpriteKind::GrassToDirtEdgeSouth => x,
        TerrainSpriteKind::GrassToDirtEdgeEast | TerrainSpriteKind::GrassToDirtEdgeWest => y,
        _ => 0,
    }
}

fn transition_across(kind: TerrainSpriteKind, x: u32, y: u32) -> u32 {
    match kind {
        TerrainSpriteKind::GrassToDirtEdgeNorth | TerrainSpriteKind::GrassToDirtEdgeSouth => y,
        TerrainSpriteKind::GrassToDirtEdgeEast | TerrainSpriteKind::GrassToDirtEdgeWest => x,
        _ => 0,
    }
}

fn transition_point(kind: TerrainSpriteKind, along: u32, across: u32) -> (u32, u32) {
    match kind {
        TerrainSpriteKind::GrassToDirtEdgeNorth | TerrainSpriteKind::GrassToDirtEdgeSouth => {
            (along, across)
        }
        TerrainSpriteKind::GrassToDirtEdgeEast | TerrainSpriteKind::GrassToDirtEdgeWest => {
            (across, along)
        }
        _ => (0, 0),
    }
}

fn transition_is_dirt_side(kind: TerrainSpriteKind, signed: i32) -> bool {
    match kind {
        TerrainSpriteKind::GrassToDirtEdgeNorth | TerrainSpriteKind::GrassToDirtEdgeWest => {
            signed <= 0
        }
        TerrainSpriteKind::GrassToDirtEdgeSouth | TerrainSpriteKind::GrassToDirtEdgeEast => {
            signed >= 0
        }
        _ => false,
    }
}

fn edge_jitter(seed: u64, along: u32, size: u32, jitter: i32) -> i32 {
    if jitter == 0 {
        return 0;
    }
    let coarse = (along / 3).min(size);
    let raw = hash(seed ^ 0xf01 ^ (coarse as u64 * 0x9e37)) as i32;
    raw.rem_euclid(jitter * 2 + 1) - jitter
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

fn set_wrapped(image: &mut PixelImage, x: u32, y: u32, color: Rgba8) {
    let x = x % image.width.max(1);
    let y = y % image.height.max(1);
    image.set(x, y, color);
}

fn wrap_i32(value: i32, size: u32) -> u32 {
    value.rem_euclid(size.max(1) as i32) as u32
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

const BLADE_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: -1,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: -1,
    },
];

const BLADE_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: -1,
        shade: 1,
    },
    MotifPixel {
        dx: 0,
        dy: 1,
        shade: -1,
    },
];

const BLADE_C: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 2,
        dy: -1,
        shade: 1,
    },
];

const BLADE_D: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: -1,
        dy: 1,
        shade: 1,
    },
];

const BLADE_E: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 0,
        dy: -1,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: -1,
    },
];

const BLADE_F: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: -1,
        shade: 1,
    },
    MotifPixel {
        dx: 2,
        dy: -1,
        shade: 1,
    },
];

const DARK_CLUMP_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 0,
        dy: 1,
        shade: -2,
    },
];

const DARK_CLUMP_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -2,
    },
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 0,
        dy: -1,
        shade: -1,
    },
];

const DARK_CLUMP_C: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: -1,
        dy: 1,
        shade: -2,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: -1,
    },
];

const SOFT_GRASS_DARK_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: -1,
    },
];

const SHADOW_POCKET_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -2,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: -1,
    },
];

const SHADOW_POCKET_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 0,
        dy: 1,
        shade: -2,
    },
];

const LIGHT_LEAF_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: 1,
    },
];

const LIGHT_LEAF_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 0,
        dy: -1,
        shade: 1,
    },
];

const LIGHT_LEAF_C: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 1,
        shade: 0,
    },
];

const LIGHT_SPECK_A: &[MotifPixel] = &[MotifPixel {
    dx: 0,
    dy: 0,
    shade: 1,
}];

const LIGHT_SPECK_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: 0,
    },
];

const SEED_FLECK_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: 0,
    },
];

const FLOWER_FLECK_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 2,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: 1,
    },
];

const DUST_SMEAR_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 0,
        dy: 1,
        shade: 0,
    },
];

const DUST_SMEAR_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: 0,
    },
];

const DUST_SMEAR_C: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 1,
        shade: 0,
    },
];

const DUST_SMEAR_D: &[MotifPixel] = &[
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: 0,
    },
];

const DUST_SMEAR_E: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: 1,
        dy: -1,
        shade: 1,
    },
];

const DIRT_DENT_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 0,
        shade: -1,
    },
];

const DIRT_DENT_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 0,
        dy: 1,
        shade: -1,
    },
];

const DIRT_DENT_C: &[MotifPixel] = &[MotifPixel {
    dx: 0,
    dy: 0,
    shade: -1,
}];

const DIRT_DENT_D: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: 0,
    },
];

const RUT_A: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: -1,
        shade: 0,
    },
];

const RUT_B: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: -1,
    },
];

const RUT_C: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 1,
    },
    MotifPixel {
        dx: -1,
        dy: 1,
        shade: -1,
    },
];

const RUT_D: &[MotifPixel] = &[
    MotifPixel {
        dx: -1,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: 0,
    },
    MotifPixel {
        dx: 1,
        dy: 1,
        shade: -1,
    },
];

const RUT_E: &[MotifPixel] = &[
    MotifPixel {
        dx: 0,
        dy: 0,
        shade: -1,
    },
    MotifPixel {
        dx: 1,
        dy: -1,
        shade: 0,
    },
];
