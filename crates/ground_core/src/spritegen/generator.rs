use crate::color::Rgba8;
use crate::pixel_image::PixelImage;
use crate::spritegen::{
    GeneratedTerrainSprite, TerrainMotif, TerrainSpriteKind, TerrainSpriteRecipe,
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
            metadata: TerrainSpriteKind::GrassTile.default_piece_metadata(),
            image: generate_grass_tile(&recipe, variant),
        });
    }
    for variant in 1..=recipe.variant_count {
        sprites.push(GeneratedTerrainSprite {
            id: format!("dirt_tile_{variant:02}"),
            kind: TerrainSpriteKind::DirtTile,
            variant,
            metadata: TerrainSpriteKind::DirtTile.default_piece_metadata(),
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
            metadata: kind.default_piece_metadata(),
            image: generate_transition_tile(&recipe, kind, 1),
        });
    }
    for mask in 0..16 {
        let kind = TerrainSpriteKind::from_path_mask(mask).expect("valid path mask");
        sprites.push(GeneratedTerrainSprite {
            id: format!("path_mask_{mask:02}"),
            kind,
            variant: 1,
            metadata: kind.default_piece_metadata(),
            image: generate_path_mask_tile(&recipe, mask),
        });
    }
    for variant in 1..=2 {
        for kind in [
            TerrainSpriteKind::TrenchFloorTop,
            TerrainSpriteKind::TrenchWallFront,
        ] {
            sprites.push(GeneratedTerrainSprite {
                id: format!("{}_{variant:02}", kind.id()),
                kind,
                variant,
                metadata: kind.default_piece_metadata(),
                image: generate_trench_piece(&recipe, kind, variant),
            });
        }
    }
    for kind in [
        TerrainSpriteKind::TrenchLipFront,
        TerrainSpriteKind::TrenchLipBack,
        TerrainSpriteKind::TrenchEndCapLeft,
        TerrainSpriteKind::TrenchEndCapRight,
        TerrainSpriteKind::TrenchCornerInner,
        TerrainSpriteKind::TrenchCornerOuter,
        TerrainSpriteKind::TrenchContactShadow,
        TerrainSpriteKind::TrenchSpoilPile,
    ] {
        sprites.push(GeneratedTerrainSprite {
            id: format!("{}_01", kind.id()),
            kind,
            variant: 1,
            metadata: kind.default_piece_metadata(),
            image: generate_trench_piece(&recipe, kind, 1),
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
        &recipe.motifs.grass_dark,
        &grass_color_picker(recipe, false),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x301,
        cell,
        light_chance,
        &recipe.motifs.grass_light,
        &grass_color_picker(recipe, true),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x401,
        cell,
        blade_chance,
        &recipe.motifs.grass_blades,
        &grass_color_picker(recipe, true),
        &mut occupied,
    );

    if style.pixel.avoid_single_pixel_noise {
        soften_isolated_pixels(&mut image);
    }
    let flower_count = ((size * size) as f32 * style.grass.flower_density).round() as u32;
    for i in 0..flower_count {
        if let Some((x, y)) = find_spaced_point(seed ^ 0x501, i, size, 4, &occupied) {
            draw_motif_from_set(
                &mut image,
                x,
                y,
                &recipe.motifs.grass_flowers,
                seed ^ 0x511 ^ i as u64,
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
        &recipe.motifs.dirt_dust,
        &dirt_color_picker(recipe, true),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x801,
        cell,
        style.dirt.compact_shadow_density * 0.90 * variant_weight,
        &recipe.motifs.dirt_dents,
        &dirt_color_picker(recipe, false),
        &mut occupied,
    );
    scatter_motifs(
        &mut image,
        seed ^ 0x901,
        cell,
        style.dirt.rut_density * 0.25 * variant_weight,
        &recipe.motifs.dirt_ruts,
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
            draw_motif_from_set(
                &mut image,
                x,
                y,
                &recipe.motifs.transition_intrusion,
                seed ^ 0xd11 ^ i as u64,
                &grass_color_picker(recipe, true),
            );
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
            let signed = path_mask_signed_distance(recipe, mask, x, y, size, seed);
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

pub fn generate_trench_piece(
    recipe: &TerrainSpriteRecipe,
    kind: TerrainSpriteKind,
    variant: u32,
) -> PixelImage {
    debug_assert!(kind.is_trench());
    match kind {
        TerrainSpriteKind::TrenchFloorTop => generate_trench_floor_top(recipe, variant),
        TerrainSpriteKind::TrenchWallFront => generate_trench_wall_front(recipe, variant),
        TerrainSpriteKind::TrenchLipFront => generate_trench_lip(recipe, variant, true),
        TerrainSpriteKind::TrenchLipBack => generate_trench_lip(recipe, variant, false),
        TerrainSpriteKind::TrenchEndCapLeft => generate_trench_end_cap(recipe, variant, true),
        TerrainSpriteKind::TrenchEndCapRight => generate_trench_end_cap(recipe, variant, false),
        TerrainSpriteKind::TrenchCornerInner => generate_trench_corner(recipe, variant, true),
        TerrainSpriteKind::TrenchCornerOuter => generate_trench_corner(recipe, variant, false),
        TerrainSpriteKind::TrenchContactShadow => generate_trench_contact_shadow(recipe, variant),
        TerrainSpriteKind::TrenchSpoilPile => generate_trench_spoil_pile(recipe, variant),
        _ => PixelImage::transparent(1, 1),
    }
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
            let signed = path_mask_signed_distance(recipe, mask, x, y, size, seed);
            if (-1.0..=2.5).contains(&signed) && spaced(x as i32, y as i32, 3, &occupied) {
                best = Some((x, y));
                break;
            }
        }
        if let Some((x, y)) = best {
            draw_motif_from_set(
                image,
                x,
                y,
                &recipe.motifs.transition_intrusion,
                sample ^ 0x1b04,
                &grass_color_picker(recipe, true),
            );
            occupied.push((x as i32, y as i32));
        }
    }
}

fn generate_trench_floor_top(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.cell_height_px * 7 / 16).max(28);
    let seed = sprite_seed(recipe.seed, variant, 0x2101);
    let base = palette.dirt_shadow.darken(rules.floor_darkness * 0.72);
    let plank = palette.dirt_dark.darken(rules.floor_darkness * 0.38);
    let seam = palette
        .dirt_shadow
        .darken(rules.inner_shadow_strength * 0.42);
    let mut image = PixelImage::new(width, height, base);

    for y in (5..height).step_by(8) {
        image.draw_line(1, y as i32, width as i32 - 2, y as i32, seam);
    }
    for x in (12..width).step_by(22) {
        let offset = (hash(seed ^ x as u64) % 5) as i32 - 2;
        image.draw_line(
            x as i32 + offset,
            2,
            x as i32 + offset + 3,
            height as i32 - 3,
            seam.blend(plank, 0.28),
        );
    }

    let detail_count = ((width * height) as f32 * rules.floor_detail_density / 28.0)
        .round()
        .max(2.0) as u32;
    for i in 0..detail_count {
        let x = (hash(seed ^ 0x2102 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2103 ^ (i as u64 * 11)) % height as u64) as u32;
        let color = if i % 3 == 0 {
            palette.dirt_light.darken(0.35)
        } else {
            palette.dirt_shadow.darken(0.18)
        };
        image.blend_pixel(x, y, color, 0.55);
        if x + 1 < width {
            image.blend_pixel(x + 1, y, color, 0.35);
        }
    }
    image
}

fn generate_trench_wall_front(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 18).max(32);
    let seed = sprite_seed(recipe.seed, variant, 0x2201);
    let base = palette.dirt_dark.darken(rules.wall_shadow_strength * 0.30);
    let mut image = PixelImage::new(width, height, base);

    blend_rect(
        &mut image,
        0,
        height * 2 / 3,
        width,
        height / 3,
        Rgba8::BLACK,
        0.18,
    );
    for y in (6..height.saturating_sub(2)).step_by(9) {
        let color = palette.dirt_shadow.darken(0.10);
        image.draw_line(0, y as i32, width as i32 - 1, y as i32, color);
    }
    for x in (10..width).step_by(24) {
        let post = palette.dirt_shadow.darken(0.22);
        blend_rect(&mut image, x, 0, 3, height, post, 0.58);
        image.draw_line(
            x as i32 + 3,
            1,
            x as i32 + 3,
            height as i32 - 2,
            palette.dirt_light.darken(0.25),
        );
    }

    let motif_count = ((width * height) as f32 * rules.wall_detail_density / 38.0)
        .round()
        .max(3.0) as u32;
    for i in 0..motif_count {
        let x = (hash(seed ^ 0x2202 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2203 ^ (i as u64 * 17)) % height as u64) as u32;
        let motifs = if hash(seed ^ i as u64).is_multiple_of(3) {
            &recipe.motifs.trench_wall_shadow
        } else {
            &recipe.motifs.trench_wood
        };
        draw_motif_from_set(
            &mut image,
            x,
            y,
            motifs,
            seed ^ 0x2204 ^ i as u64,
            &trench_wood_color_picker(recipe),
        );
    }

    let knot_count = ((width * height) as f32 * rules.wood_knot_density / 80.0)
        .round()
        .max(1.0) as u32;
    for i in 0..knot_count {
        let x = (hash(seed ^ 0x2205 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2206 ^ (i as u64 * 19)) % height as u64) as u32;
        image.blend_pixel(x, y, palette.dirt_shadow.darken(0.30), 0.75);
        if x + 1 < width {
            image.blend_pixel(x + 1, y, palette.dirt_shadow, 0.45);
        }
    }
    image
}

fn generate_trench_lip(recipe: &TerrainSpriteRecipe, variant: u32, front: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px / 2).max(14);
    let seed = sprite_seed(recipe.seed, variant, if front { 0x2301 } else { 0x2302 });
    let mut image = PixelImage::transparent(width, height);
    let band_y = if front { height / 3 } else { height / 5 };
    let base = palette
        .dirt_mid
        .blend(palette.dirt_light, rules.lip_highlight_strength);

    for x in 0..width {
        let jitter = (hash(seed ^ x as u64 ^ 0x2303)
            % (rules.lip_irregularity_px.max(1) * 2 + 1) as u64) as i32
            - rules.lip_irregularity_px as i32;
        let y0 = (band_y as i32 + jitter).clamp(0, height as i32 - 1) as u32;
        let lip_h = 4 + (hash(seed ^ x as u64 ^ 0x2304) % 4) as u32;
        for y in y0..(y0 + lip_h).min(height) {
            let shade = if y == y0 {
                base.lighten(0.08)
            } else if y > y0 + lip_h / 2 {
                palette.dirt_dark
            } else {
                base
            };
            image.set(x, y, shade);
        }
    }
    let motif_count = ((width * height) as f32 * rules.spoil_density / 22.0)
        .round()
        .max(2.0) as u32;
    for i in 0..motif_count {
        let x = (hash(seed ^ 0x2305 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2306 ^ (i as u64 * 13)) % height as u64) as u32;
        let motifs = if i % 4 == 0 {
            &recipe.motifs.trench_grass_overhang
        } else {
            &recipe.motifs.trench_lip
        };
        if i % 4 == 0 {
            draw_motif_from_set(
                &mut image,
                x,
                y,
                motifs,
                seed ^ i as u64,
                &trench_grass_color_picker(recipe),
            );
        } else {
            draw_motif_from_set(
                &mut image,
                x,
                y,
                motifs,
                seed ^ i as u64,
                &trench_dirt_color_picker(recipe),
            );
        }
    }
    image
}

fn generate_trench_end_cap(recipe: &TerrainSpriteRecipe, variant: u32, left: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = (projection.cell_width_px / 2).max(32);
    let height = (projection.cell_height_px / 2 + projection.face_height_px / 2).max(42);
    let seed = sprite_seed(recipe.seed, variant, if left { 0x2401 } else { 0x2402 });
    let mut image = PixelImage::transparent(width, height);
    let palette = &recipe.style.palette;
    let wall = palette.dirt_dark.darken(0.18);
    let lip = palette.dirt_mid.lighten(0.06);
    let edge_x = if left { width / 3 } else { width * 2 / 3 };

    for y in 4..height.saturating_sub(4) {
        let spread = (height - y).min(height / 2) / 2;
        let x0 = if left {
            edge_x.saturating_sub(spread)
        } else {
            edge_x
        };
        let x1 = if left {
            edge_x
        } else {
            (edge_x + spread).min(width - 1)
        };
        for x in x0..=x1 {
            image.set(x, y, if y < height / 3 { lip } else { wall });
        }
    }
    for i in 0..9 {
        let x = (hash(seed ^ i) % width as u64) as u32;
        let y = (hash(seed ^ 0x2403 ^ i) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_lip,
            seed ^ 0x2404 ^ i,
            &trench_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_trench_corner(recipe: &TerrainSpriteRecipe, variant: u32, inner: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let size = (projection.cell_width_px / 2).max(40);
    let seed = sprite_seed(recipe.seed, variant, if inner { 0x2501 } else { 0x2502 });
    let mut image = PixelImage::transparent(size, size);
    let palette = &recipe.style.palette;
    let lip = palette.dirt_mid.lighten(0.08);
    let wall = palette.dirt_dark.darken(0.12);
    let radius = size as f32 * if inner { 0.44 } else { 0.58 };

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - size as f32 * 0.52;
            let dy = y as f32 - size as f32 * 0.50;
            let d = (dx * dx + dy * dy).sqrt();
            if (d - radius).abs() < 5.5 || (inner && d < radius && x > size / 2 && y > size / 2) {
                let color = if y < size / 3 { lip } else { wall };
                image.set(x, y, color);
            }
        }
    }
    for i in 0..10 {
        let x = (hash(seed ^ 0x2503 ^ i) % size as u64) as u32;
        let y = (hash(seed ^ 0x2504 ^ (i * 7)) % size as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_lip,
            seed ^ i,
            &trench_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_trench_contact_shadow(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 16).max(36);
    let seed = sprite_seed(recipe.seed, variant, 0x2601);
    let mut image = PixelImage::transparent(width, height);
    let strength = recipe.style.trench.contact_shadow_strength;
    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let organic = hash01(seed ^ (((x / 3) as u64) << 8) ^ (y / 2) as u64) * 0.10;
            let alpha =
                ((1.0 - xf * 1.7).max(0.0) * (1.0 - yf).powf(1.4) + organic) * strength * 0.72;
            if alpha > 0.02 {
                image.set(
                    x,
                    y,
                    Rgba8::BLACK.with_alpha((alpha.clamp(0.0, 0.65) * 255.0) as u8),
                );
            }
        }
    }
    image
}

fn blend_rect(
    target: &mut PixelImage,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    color: Rgba8,
    alpha: f32,
) {
    let x1 = (x0 + width).min(target.width);
    let y1 = (y0 + height).min(target.height);
    for y in y0..y1 {
        for x in x0..x1 {
            target.blend_pixel(x, y, color, alpha);
        }
    }
}

fn generate_trench_spoil_pile(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = projection.cell_width_px.max(48);
    let height = (projection.cell_height_px / 3).max(24);
    let seed = sprite_seed(recipe.seed, variant, 0x2701);
    let mut image = PixelImage::transparent(width, height);
    let rules = &recipe.style.trench;
    let count = ((width * height) as f32 * rules.spoil_density / 10.0)
        .round()
        .max(6.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x2702 ^ i as u64) % width as u64) as u32;
        let center_y = height as i32 / 2
            + (hash(seed ^ 0x2703 ^ (i as u64 * 13)) % (height / 2).max(1) as u64) as i32
            - height as i32 / 4;
        let y = center_y.clamp(0, height as i32 - 1) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_spoil,
            seed ^ 0x2704 ^ i as u64,
            &trench_dirt_color_picker(recipe),
        );
    }
    let grass_count = ((width * height) as f32 * rules.grass_intrusion_density / 42.0)
        .round()
        .max(1.0) as u32;
    for i in 0..grass_count {
        let x = (hash(seed ^ 0x2705 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2706 ^ (i as u64 * 5)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_grass_overhang,
            seed ^ 0x2707 ^ i as u64,
            &trench_grass_color_picker(recipe),
        );
    }
    image
}

fn path_mask_signed_distance(
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    x: u32,
    y: u32,
    size: u32,
    seed: u64,
) -> f32 {
    let path = &recipe.style.path;
    let arm_half = (path.width_px * 0.5).max(1.5);
    let core_half = (path.core_width_px * 0.5 + path.corner_rounding * 0.35).max(arm_half);
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
    let organic = path_edge_noise(seed, x, y) * path.edge_noise;
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
    motifs: &[TerrainMotif],
    color_picker: &dyn Fn(i8) -> Rgba8,
    occupied: &mut Vec<(i32, i32)>,
) {
    let size = image.width.min(image.height);
    let count = ((size * size) as f32 * chance / 7.5).round().max(1.0) as u32;
    let min_distance = cell.max(2) as i32;
    for i in 0..count {
        if let Some((x, y)) = find_spaced_point(seed, i, size, min_distance, occupied) {
            draw_motif_from_set(
                image,
                x,
                y,
                motifs,
                seed ^ (i as u64 * 0x517c),
                color_picker,
            );
            occupied.push((x as i32, y as i32));
        }
    }
}

fn draw_motif_from_set(
    image: &mut PixelImage,
    origin_x: u32,
    origin_y: u32,
    motifs: &[TerrainMotif],
    seed: u64,
    color_picker: &dyn Fn(i8) -> Rgba8,
) {
    if motifs.is_empty() {
        return;
    }
    let motif = choose_motif(motifs, seed);
    let flip_x = motif.allow_flip_x && hash(seed ^ 0x51f1).is_multiple_of(2);
    let flip_y = motif.allow_flip_y && hash(seed ^ 0x71f1).is_multiple_of(2);
    for pixel in &motif.pixels {
        let dx = if flip_x { -pixel.dx } else { pixel.dx };
        let dy = if flip_y { -pixel.dy } else { pixel.dy };
        set_wrapped(
            image,
            origin_x.wrapping_add_signed(dx),
            origin_y.wrapping_add_signed(dy),
            color_picker(pixel.shade),
        );
    }
}

fn choose_motif(motifs: &[TerrainMotif], seed: u64) -> &TerrainMotif {
    let total = motifs
        .iter()
        .map(|motif| motif.weight.max(0.0))
        .sum::<f32>();
    if total <= f32::EPSILON {
        return &motifs[(hash(seed) % motifs.len() as u64) as usize];
    }
    let mut target = hash01(seed) * total;
    for motif in motifs {
        target -= motif.weight.max(0.0);
        if target <= 0.0 {
            return motif;
        }
    }
    &motifs[motifs.len() - 1]
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

fn trench_dirt_color_picker(recipe: &TerrainSpriteRecipe) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match shade {
            -2 => palette.dirt_shadow.darken(0.30),
            -1 => palette.dirt_dark.darken(0.12),
            1 => palette.dirt_light,
            2 => palette.pebble,
            _ => palette.dirt_mid,
        }
    }
}

fn trench_wood_color_picker(recipe: &TerrainSpriteRecipe) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match shade {
            -2 => palette.dirt_shadow.darken(0.36),
            -1 => palette.dirt_shadow,
            1 => palette.dirt_light.darken(0.20),
            2 => palette.dirt_light,
            _ => palette.dirt_dark,
        }
    }
}

fn trench_grass_color_picker(recipe: &TerrainSpriteRecipe) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match shade {
            -2 => palette.grass_shadow,
            -1 => palette.grass_dark,
            1 => palette.grass_light,
            2 => palette.grass_flower,
            _ => palette.grass_mid,
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
