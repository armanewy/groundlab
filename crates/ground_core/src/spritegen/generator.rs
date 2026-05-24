use crate::color::Rgba8;
use crate::pixel_image::PixelImage;
use crate::spritegen::{
    GeneratedTerrainSprite, TerrainMotif, TerrainSpriteKind, TerrainSpriteRecipe,
    TerrainSpriteSource,
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
            source: TerrainSpriteSource::Generated,
            metadata: TerrainSpriteKind::GrassTile.default_piece_metadata(),
            image: generate_grass_tile(&recipe, variant),
        });
    }
    for variant in 1..=recipe.variant_count {
        sprites.push(GeneratedTerrainSprite {
            id: format!("dirt_tile_{variant:02}"),
            kind: TerrainSpriteKind::DirtTile,
            variant,
            source: TerrainSpriteSource::Generated,
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
            source: TerrainSpriteSource::Generated,
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
            source: TerrainSpriteSource::Generated,
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
                source: TerrainSpriteSource::Generated,
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
            source: TerrainSpriteSource::Generated,
            metadata: kind.default_piece_metadata(),
            image: generate_trench_piece(&recipe, kind, 1),
        });
    }
    for variant in 1..=2 {
        for kind in [TerrainSpriteKind::BermTop, TerrainSpriteKind::BermFaceFront] {
            sprites.push(GeneratedTerrainSprite {
                id: format!("{}_{variant:02}", kind.id()),
                kind,
                variant,
                source: TerrainSpriteSource::Generated,
                metadata: kind.default_piece_metadata(),
                image: generate_berm_piece(&recipe, kind, variant),
            });
        }
    }
    for kind in [
        TerrainSpriteKind::BermLipFront,
        TerrainSpriteKind::BermLipBack,
        TerrainSpriteKind::BermEndCapLeft,
        TerrainSpriteKind::BermEndCapRight,
        TerrainSpriteKind::BermCornerInner,
        TerrainSpriteKind::BermCornerOuter,
        TerrainSpriteKind::BermContactShadow,
        TerrainSpriteKind::BermSpoilPile,
        TerrainSpriteKind::BermGrassFringe,
    ] {
        sprites.push(GeneratedTerrainSprite {
            id: format!("{}_01", kind.id()),
            kind,
            variant: 1,
            source: TerrainSpriteSource::Generated,
            metadata: kind.default_piece_metadata(),
            image: generate_berm_piece(&recipe, kind, 1),
        });
    }
    for mask in 0..16 {
        let kind = TerrainSpriteKind::from_trench_mask(mask).expect("valid trench mask");
        sprites.push(GeneratedTerrainSprite {
            id: format!("trench_mask_{mask:02}"),
            kind,
            variant: 1,
            source: TerrainSpriteSource::Generated,
            metadata: kind.default_piece_metadata(),
            image: generate_trench_mask_tile(&recipe, mask),
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

pub fn generate_berm_piece(
    recipe: &TerrainSpriteRecipe,
    kind: TerrainSpriteKind,
    variant: u32,
) -> PixelImage {
    debug_assert!(kind.is_berm());
    match kind {
        TerrainSpriteKind::BermTop => generate_berm_top(recipe, variant),
        TerrainSpriteKind::BermFaceFront => generate_berm_face_front(recipe, variant),
        TerrainSpriteKind::BermLipFront => generate_berm_lip(recipe, variant, true),
        TerrainSpriteKind::BermLipBack => generate_berm_lip(recipe, variant, false),
        TerrainSpriteKind::BermEndCapLeft => generate_berm_end_cap(recipe, variant, true),
        TerrainSpriteKind::BermEndCapRight => generate_berm_end_cap(recipe, variant, false),
        TerrainSpriteKind::BermCornerInner => generate_berm_corner(recipe, variant, true),
        TerrainSpriteKind::BermCornerOuter => generate_berm_corner(recipe, variant, false),
        TerrainSpriteKind::BermContactShadow => generate_berm_contact_shadow(recipe, variant),
        TerrainSpriteKind::BermSpoilPile => generate_berm_spoil_pile(recipe, variant),
        TerrainSpriteKind::BermGrassFringe => generate_berm_grass_fringe(recipe, variant),
        _ => PixelImage::transparent(1, 1),
    }
}

pub fn generate_trench_mask_tile(recipe: &TerrainSpriteRecipe, mask: u8) -> PixelImage {
    let projection = &recipe.style.projection;
    let palette = &recipe.style.palette;
    let rules = &recipe.style.trench;
    let width = projection.cell_width_px.max(recipe.tile_size * 2);
    let surface_h = projection.cell_height_px.max(recipe.tile_size * 2);
    let face_h = projection.face_height_px.max(12);
    let height = surface_h + face_h + 12;
    let seed = sprite_seed(recipe.seed, mask as u32 + 1, 0x3401);
    let grass = generate_grass_tile(recipe, mask as u32 % recipe.variant_count.max(1) + 1);
    let mut image = PixelImage::transparent(width, height);

    for y in 0..surface_h {
        for x in 0..width {
            let sx = x * grass.width / width.max(1);
            let sy = y * grass.height / surface_h.max(1);
            image.set(x, y, grass.get(sx, sy));
        }
    }

    let soft_edge = (surface_h as f32 * 0.055).clamp(2.0, 5.0);
    let lip_band = (surface_h as f32 * 0.075).clamp(3.0, 7.0);
    let floor = palette.dirt_shadow.darken(rules.floor_darkness * 0.66);
    let floor_hi = palette.dirt_dark.darken(rules.floor_darkness * 0.36);
    let wall_top = palette.dirt_mid.darken(rules.wall_shadow_strength * 0.12);
    let wall_mid = palette.dirt_dark.darken(rules.wall_shadow_strength * 0.10);
    let wall_bottom = palette.dirt_shadow.darken(0.24);
    let lip = palette
        .dirt_mid
        .lighten(rules.lip_highlight_strength * 0.40);
    let lip_shadow = palette.dirt_dark.darken(0.12);

    let mut bottom_edge = vec![None; width as usize];
    let mut top_edge = vec![None; width as usize];
    let mut left_edge = vec![None; surface_h as usize];
    let mut right_edge = vec![None; surface_h as usize];

    for y in 0..surface_h {
        for x in 0..width {
            let signed = trench_mask_signed_distance(recipe, mask, x, y, width, surface_h, seed);
            if signed <= -lip_band {
                let floor_noise = trench_noise(seed ^ 0x3402, x / 4, y / 4);
                let mut color = if y as f32 > surface_h as f32 * 0.58 {
                    floor.darken(0.09)
                } else if floor_noise > 0.45 {
                    floor_hi
                } else {
                    floor
                };
                if (x / 18 + y / 8 + mask as u32).is_multiple_of(3) && floor_noise > 0.38 {
                    color = color.blend(palette.dirt_light.darken(0.46), 0.18);
                }
                image.set(x, y, color.with_alpha(245));
            } else if signed <= 0.0 {
                let t = 1.0 - signed.abs() / lip_band;
                let color = lip_shadow.blend(lip, t * 0.72);
                image.set(x, y, color.with_alpha(248));
            } else if signed <= soft_edge {
                let t = 1.0 - signed / soft_edge;
                image.blend_pixel(x, y, palette.dirt_mid, 0.35 * t);
                image.blend_pixel(x, y, Rgba8::BLACK, 0.10 * t * rules.contact_shadow_strength);
            } else if signed <= soft_edge * 2.8 {
                let chance = hash01(seed ^ 0x3403 ^ ((x as u64) << 18) ^ y as u64);
                if chance < rules.spoil_density * 0.30 {
                    image.blend_pixel(x, y, palette.dirt_mid, 0.18);
                }
            }

            if signed <= 0.0 {
                let xi = x as usize;
                let yi = y as usize;
                bottom_edge[xi] = Some(bottom_edge[xi].map_or(y, |old: u32| old.max(y)));
                top_edge[xi] = Some(top_edge[xi].map_or(y, |old: u32| old.min(y)));
                left_edge[yi] = Some(left_edge[yi].map_or(x, |old: u32| old.min(x)));
                right_edge[yi] = Some(right_edge[yi].map_or(x, |old: u32| old.max(x)));
            }
        }
    }
    normalize_trench_opening_bands(&mut image, recipe, mask, width, surface_h, floor);
    suppress_trench_openings(
        mask,
        width,
        surface_h,
        &mut top_edge,
        &mut bottom_edge,
        &mut left_edge,
        &mut right_edge,
    );

    for x in 0..width {
        let Some(edge_y) = bottom_edge[x as usize] else {
            continue;
        };
        let open_south = mask & 4 != 0 && edge_y + 4 >= surface_h;
        if open_south {
            continue;
        }
        let wall_h = if edge_y > surface_h / 2 {
            face_h * 3 / 4
        } else {
            face_h / 3
        }
        .max(5);
        for dy in 0..wall_h {
            let ty = edge_y + dy + 1;
            if ty >= height {
                break;
            }
            let t = dy as f32 / wall_h.max(1) as f32;
            let color = if t > 0.68 {
                wall_bottom
            } else if t < 0.18 {
                wall_top
            } else {
                wall_mid
            };
            image.set(x, ty, color.with_alpha((236.0 - t * 28.0) as u8));
        }
        if edge_y + wall_h + 1 < height {
            image.blend_pixel(
                x,
                edge_y + wall_h + 1,
                Rgba8::BLACK,
                rules.contact_shadow_strength * 0.32,
            );
        }
    }

    draw_trench_mask_lip_segments(&mut image, recipe, seed ^ 0x3404, &top_edge, true);
    draw_trench_mask_lip_segments(&mut image, recipe, seed ^ 0x3405, &bottom_edge, false);
    draw_trench_mask_side_lips(&mut image, recipe, seed ^ 0x3406, &left_edge, true);
    draw_trench_mask_side_lips(&mut image, recipe, seed ^ 0x3407, &right_edge, false);
    draw_trench_mask_caps(&mut image, recipe, mask, seed ^ 0x3408);
    draw_trench_mask_spoil(&mut image, recipe, mask, seed ^ 0x3409);
    draw_trench_mask_center_resolver(&mut image, recipe, mask, seed ^ 0x340a);
    image
}

fn normalize_trench_opening_bands(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    width: u32,
    surface_h: u32,
    floor: Rgba8,
) {
    let cx = width / 2;
    let cy = surface_h / 2;
    let open_w = (width as f32 * 0.34).round() as u32;
    let open_h = (surface_h as f32 * 0.34).round() as u32;
    let band = (recipe.style.projection.face_height_px / 5).clamp(4, 9);
    if mask & 1 != 0 {
        fill_opening_band(image, cx.saturating_sub(open_w / 2), 0, open_w, band, floor);
    }
    if mask & 4 != 0 {
        fill_opening_band(
            image,
            cx.saturating_sub(open_w / 2),
            surface_h.saturating_sub(band),
            open_w,
            band,
            floor,
        );
    }
    if mask & 8 != 0 {
        fill_opening_band(image, 0, cy.saturating_sub(open_h / 2), band, open_h, floor);
    }
    if mask & 2 != 0 {
        fill_opening_band(
            image,
            width.saturating_sub(band),
            cy.saturating_sub(open_h / 2),
            band,
            open_h,
            floor,
        );
    }
}

fn fill_opening_band(
    image: &mut PixelImage,
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    color: Rgba8,
) {
    for y in y0..(y0 + height).min(image.height) {
        for x in x0..(x0 + width).min(image.width) {
            image.set(x, y, color.with_alpha(245));
        }
    }
}

fn suppress_trench_openings(
    mask: u8,
    width: u32,
    surface_h: u32,
    top_edge: &mut [Option<u32>],
    bottom_edge: &mut [Option<u32>],
    left_edge: &mut [Option<u32>],
    right_edge: &mut [Option<u32>],
) {
    let cx = width / 2;
    let cy = surface_h / 2;
    let open_w = (width as f32 * 0.34).round() as u32;
    let open_h = (surface_h as f32 * 0.34).round() as u32;
    if mask & 1 != 0 {
        clear_edge_span(top_edge, cx.saturating_sub(open_w / 2), cx + open_w / 2);
    }
    if mask & 4 != 0 {
        clear_edge_span(bottom_edge, cx.saturating_sub(open_w / 2), cx + open_w / 2);
    }
    if mask & 8 != 0 {
        clear_edge_span(left_edge, cy.saturating_sub(open_h / 2), cy + open_h / 2);
    }
    if mask & 2 != 0 {
        clear_edge_span(right_edge, cy.saturating_sub(open_h / 2), cy + open_h / 2);
    }
}

fn clear_edge_span(edges: &mut [Option<u32>], start: u32, end: u32) {
    let end = end.min(edges.len().saturating_sub(1) as u32);
    for index in start.min(end)..=end {
        if let Some(edge) = edges.get_mut(index as usize) {
            *edge = None;
        }
    }
}

fn draw_trench_mask_lip_segments(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    seed: u64,
    edges: &[Option<u32>],
    back_lip: bool,
) {
    let palette = &recipe.style.palette;
    let lip = if back_lip {
        palette.dirt_mid.lighten(0.12)
    } else {
        palette.dirt_dark.lighten(0.05)
    };
    let shadow = palette.dirt_shadow.darken(0.14);
    let step = (image.width / 11).max(7);
    for x0 in (0..image.width).step_by(step as usize) {
        let x1 = (x0 + step + (hash(seed ^ x0 as u64) % 5) as u32).min(image.width);
        let mut samples = Vec::new();
        for x in x0..x1 {
            if let Some(y) = edges.get(x as usize).and_then(|edge| *edge) {
                samples.push(y);
            }
        }
        if samples.is_empty() {
            continue;
        }
        let y = samples.iter().sum::<u32>() / samples.len() as u32;
        let jitter = (hash(seed ^ 0x11 ^ x0 as u64) % 5) as i32 - 2;
        let ty = y as i32 + if back_lip { -2 } else { 1 } + jitter.signum();
        let color = if back_lip {
            lip
        } else {
            shadow.blend(lip, 0.45)
        };
        let h = if back_lip { 3 } else { 4 };
        for dy in 0..h {
            for x in x0..x1 {
                let tx = x as i32;
                let py = ty + dy;
                if image.in_bounds(tx, py) {
                    image.blend_pixel(tx as u32, py as u32, color, 0.62);
                }
            }
        }
    }
}

fn draw_trench_mask_side_lips(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    seed: u64,
    edges: &[Option<u32>],
    left: bool,
) {
    let palette = &recipe.style.palette;
    let lip = palette.dirt_mid.lighten(0.05);
    let step = (image.height / 12).max(6);
    for y0 in (0..image.height.min(edges.len() as u32)).step_by(step as usize) {
        let y1 = (y0 + step).min(edges.len() as u32);
        let mut samples = Vec::new();
        for y in y0..y1 {
            if let Some(x) = edges.get(y as usize).and_then(|edge| *edge) {
                samples.push(x);
            }
        }
        if samples.is_empty() {
            continue;
        }
        let x = samples.iter().sum::<u32>() / samples.len() as u32;
        let jitter = (hash(seed ^ y0 as u64) % 3) as i32 - 1;
        let tx = x as i32 + if left { -1 } else { 1 } + jitter;
        for y in y0..y1 {
            for dx in 0..3 {
                let px = tx + if left { -dx } else { dx };
                if image.in_bounds(px, y as i32) {
                    image.blend_pixel(px as u32, y, lip, 0.36);
                }
            }
        }
    }
}

fn draw_trench_mask_caps(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    seed: u64,
) {
    let palette = &recipe.style.palette;
    let width = image.width;
    let surface_h = recipe
        .style
        .projection
        .cell_height_px
        .max(recipe.tile_size * 2);
    let cx = width as i32 / 2;
    let cy = surface_h as i32 / 2;
    let cap_w = (width / 5).max(13) as i32;
    let cap_h = (surface_h / 5).max(10) as i32;
    let cap_color = palette.dirt_mid.lighten(0.08);
    let shadow = palette.dirt_shadow.darken(0.20);
    let degree = mask.count_ones();
    if degree != 1 {
        return;
    }
    let cap_crosses_vertical_trench = mask == 1 || mask == 4;
    let rx = if cap_crosses_vertical_trench {
        cap_w
    } else {
        cap_w / 2
    };
    let ry = if cap_crosses_vertical_trench {
        cap_h / 2
    } else {
        cap_h
    };
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            if !image.in_bounds(x, y) {
                continue;
            }
            let dx = (x - cx).abs() as f32 / rx.max(1) as f32;
            let dy = (y - cy).abs() as f32 / ry.max(1) as f32;
            if dx * dx + dy * dy > 1.08 {
                continue;
            }
            let noise = trench_noise(seed ^ mask as u64, x.max(0) as u32 / 3, y.max(0) as u32 / 3);
            let color = if dy > 0.55 || noise < -0.45 {
                shadow.blend(cap_color, 0.32)
            } else {
                cap_color
            };
            let alpha = if dx * dx + dy * dy > 0.72 { 0.28 } else { 0.54 };
            image.blend_pixel(x as u32, y as u32, color, alpha);
        }
    }
}

fn draw_trench_mask_center_resolver(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    seed: u64,
) {
    let degree = mask.count_ones();
    if degree < 2 {
        return;
    }
    let projection = &recipe.style.projection;
    let palette = &recipe.style.palette;
    let surface_h = projection.cell_height_px.max(recipe.tile_size * 2);
    let cx = image.width as i32 / 2;
    let cy = surface_h as i32 / 2;
    let floor = palette
        .dirt_shadow
        .darken(recipe.style.trench.floor_darkness * 0.64);
    let floor_hi = palette
        .dirt_dark
        .darken(recipe.style.trench.floor_darkness * 0.34);
    let lip_shadow = palette.dirt_dark.darken(0.10);
    let radius_x = if degree >= 3 {
        (image.width as f32 * 0.23) as i32
    } else {
        (image.width as f32 * 0.17) as i32
    }
    .max(12);
    let radius_y = if degree >= 3 {
        (surface_h as f32 * 0.23) as i32
    } else {
        (surface_h as f32 * 0.17) as i32
    }
    .max(10);

    for y in cy - radius_y..=cy + radius_y {
        for x in cx - radius_x..=cx + radius_x {
            if !image.in_bounds(x, y) {
                continue;
            }
            let dx = (x - cx) as f32 / radius_x.max(1) as f32;
            let dy = (y - cy) as f32 / radius_y.max(1) as f32;
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            let noise = trench_noise(seed ^ 0x41, x.max(0) as u32 / 4, y.max(0) as u32 / 4);
            let color = if noise > 0.40 { floor_hi } else { floor };
            let alpha = if degree >= 3 { 0.72 } else { 0.54 };
            image.blend_pixel(x as u32, y as u32, color, alpha);
        }
    }

    let outer_alpha = if degree >= 3 { 0.30 } else { 0.22 };
    for y in cy - radius_y - 2..=cy + radius_y + 2 {
        for x in cx - radius_x - 2..=cx + radius_x + 2 {
            if !image.in_bounds(x, y) {
                continue;
            }
            let dx = (x - cx) as f32 / radius_x.max(1) as f32;
            let dy = (y - cy) as f32 / radius_y.max(1) as f32;
            let d = dx * dx + dy * dy;
            if (0.92..=1.22).contains(&d) {
                image.blend_pixel(x as u32, y as u32, lip_shadow, outer_alpha);
            }
        }
    }
}

fn draw_trench_mask_spoil(
    image: &mut PixelImage,
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    seed: u64,
) {
    let palette = &recipe.style.palette;
    let rules = &recipe.style.trench;
    let count = ((image.width * recipe.style.projection.cell_height_px) as f32
        * rules.spoil_density
        / 310.0)
        .round()
        .max(3.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x31 ^ i as u64) % image.width as u64) as u32;
        let y = (hash(seed ^ 0x32 ^ (i as u64 * 17))
            % recipe.style.projection.cell_height_px.max(1) as u64) as u32;
        let signed = trench_mask_signed_distance(
            recipe,
            mask,
            x,
            y,
            image.width,
            recipe
                .style
                .projection
                .cell_height_px
                .max(recipe.tile_size * 2),
            seed,
        );
        if !(1.0..=11.0).contains(&signed) {
            continue;
        }
        let color = if i % 3 == 0 {
            palette.dirt_light.blend(palette.dirt_mid, 0.38)
        } else {
            palette.dirt_mid
        };
        let w = 3 + (hash(seed ^ i as u64 ^ 0x33) % 5) as u32;
        let h = 2 + (hash(seed ^ i as u64 ^ 0x34) % 3) as u32;
        for dy in 0..h {
            for dx in 0..w {
                if dx + dy > w {
                    continue;
                }
                let tx = x + dx;
                let ty = y + dy;
                if tx < image.width && ty < image.height {
                    image.blend_pixel(tx, ty, color, 0.36);
                }
            }
        }
    }
}

fn trench_mask_signed_distance(
    recipe: &TerrainSpriteRecipe,
    mask: u8,
    x: u32,
    y: u32,
    width: u32,
    surface_h: u32,
    seed: u64,
) -> f32 {
    let cx = (width as f32 - 1.0) * 0.5;
    let cy = (surface_h as f32 - 1.0) * 0.50;
    let xf = x as f32 + 0.5;
    let yf = y as f32 + 0.5;
    let trench_half_w = (width as f32 * 0.145).max(8.0);
    let trench_half_h = (surface_h as f32 * 0.145).max(7.0);
    let core_half_w = (width as f32 * 0.19).max(trench_half_w);
    let core_half_h = (surface_h as f32 * 0.18).max(trench_half_h);
    let mut distance = rect_signed_distance(
        xf,
        yf,
        cx - core_half_w,
        cx + core_half_w,
        cy - core_half_h,
        cy + core_half_h,
    );
    if mask & 1 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            cx - trench_half_w,
            cx + trench_half_w,
            -3.0,
            cy + core_half_h,
        ));
    }
    if mask & 2 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            cx - core_half_w,
            width as f32 + 3.0,
            cy - trench_half_h,
            cy + trench_half_h,
        ));
    }
    if mask & 4 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            cx - trench_half_w,
            cx + trench_half_w,
            cy - core_half_h,
            surface_h as f32 + 3.0,
        ));
    }
    if mask & 8 != 0 {
        distance = distance.min(rect_signed_distance(
            xf,
            yf,
            -3.0,
            cx + core_half_w,
            cy - trench_half_h,
            cy + trench_half_h,
        ));
    }
    if mask == 0 {
        let dx = (xf - cx) / (width as f32 * 0.22).max(1.0);
        let dy = (yf - cy) / (surface_h as f32 * 0.20).max(1.0);
        distance = (dx * dx + dy * dy).sqrt() * surface_h as f32 * 0.15 - surface_h as f32 * 0.13;
    }
    let organic = trench_noise(seed ^ 0x3410, x / 3, y / 3)
        * recipe.style.trench.lip_irregularity_px.max(1) as f32
        * 0.45;
    distance + organic
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
    let height = (projection.cell_height_px * 7 / 16).max(30);
    let seed = sprite_seed(recipe.seed, variant, 0x2b01);

    let mut image = PixelImage::transparent(width, height);
    let floor = palette.dirt_shadow.darken(rules.floor_darkness * 0.62);
    let floor_hi = palette.dirt_dark.darken(rules.floor_darkness * 0.34);
    let seam = palette
        .dirt_shadow
        .darken(rules.inner_shadow_strength * 0.52);

    for y in 0..height {
        let t_y = y as f32 / height.max(1) as f32;
        let edge_y = (height / 3).saturating_sub(y);
        for x in 0..width {
            let left_noise = trench_noise(seed ^ 0x11, x / 3, y / 3);
            let right_noise = trench_noise(seed ^ 0x12, x / 3, y / 3);
            let left_margin = 8 + (left_noise * 5.0).round() as i32;
            let right_margin = 8 + (right_noise * 5.0).round() as i32;
            if x as i32 <= left_margin || x as i32 >= width as i32 - right_margin {
                continue;
            }
            let shade = if t_y > 0.70 {
                floor.darken(0.10)
            } else if t_y < 0.18 {
                floor_hi
            } else {
                floor
            };
            let alpha = (0.92 - edge_y as f32 * 0.018).clamp(0.65, 1.0);
            image.set(x, y, shade.with_alpha((alpha * 255.0) as u8));
        }
    }

    for y in (7..height.saturating_sub(3)).step_by(9) {
        image.draw_line(10, y as i32, width as i32 - 12, y as i32, seam);
    }
    for x in (18..width.saturating_sub(10)).step_by(28) {
        let offset = (hash(seed ^ x as u64) % 5) as i32 - 2;
        image.draw_line(
            x as i32 + offset,
            4,
            x as i32 + offset + 3,
            height as i32 - 4,
            seam.blend(floor_hi, 0.30),
        );
    }

    let detail_count = ((width * height) as f32 * rules.floor_detail_density / 48.0)
        .round()
        .max(3.0) as u32;
    for i in 0..detail_count {
        let x = (hash(seed ^ 0x2b02 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2b03 ^ (i as u64 * 11)) % height as u64) as u32;
        if x < 12 || x + 12 >= width {
            continue;
        }
        let color = if i % 3 == 0 {
            palette.dirt_light.darken(0.42)
        } else {
            palette.dirt_shadow.darken(0.22)
        };
        image.blend_pixel(x, y, color, 0.45);
        if x + 1 < width {
            image.blend_pixel(x + 1, y, color, 0.25);
        }
    }
    image
}

fn generate_trench_wall_front(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 20).max(34);
    let seed = sprite_seed(recipe.seed, variant, 0x2b21);
    let mut image = PixelImage::transparent(width, height);

    let top = palette.dirt_mid.darken(rules.wall_shadow_strength * 0.14);
    let mid = palette.dirt_dark.darken(rules.wall_shadow_strength * 0.08);
    let bottom = palette.dirt_shadow.darken(0.18);

    for y in 0..height {
        let t = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let edge = trench_noise(seed ^ 0x21, x / 4, 0);
            if y < 2 && edge > 0.38 {
                continue;
            }
            let color = if t > 0.68 {
                bottom
            } else if t < 0.18 {
                top.lighten(0.06)
            } else {
                mid
            };
            image.set(x, y, color.with_alpha(238));
        }
    }

    for y in (7..height.saturating_sub(3)).step_by(10) {
        let line = palette.dirt_shadow.darken(0.22);
        image.draw_line(0, y as i32, width as i32 - 1, y as i32, line);
        if y + 1 < height {
            image.draw_line(
                0,
                y as i32 + 1,
                width as i32 - 1,
                y as i32 + 1,
                palette.dirt_light.darken(0.36),
            );
        }
    }
    for x in (14..width).step_by(30) {
        let post_w = 4 + (hash(seed ^ x as u64) % 2) as u32;
        let post = palette.dirt_shadow.darken(0.30);
        blend_rect(&mut image, x, 0, post_w, height, post, 0.70);
        if x + post_w < width {
            image.draw_line(
                (x + post_w) as i32,
                1,
                (x + post_w) as i32,
                height as i32 - 2,
                palette.dirt_light.darken(0.33),
            );
        }
    }
    blend_rect(
        &mut image,
        0,
        height * 3 / 4,
        width,
        height / 4,
        Rgba8::BLACK,
        0.22,
    );

    let motif_count = ((width * height) as f32 * rules.wall_detail_density / 54.0)
        .round()
        .max(4.0) as u32;
    for i in 0..motif_count {
        let x = (hash(seed ^ 0x2b22 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2b23 ^ (i as u64 * 17)) % height as u64) as u32;
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
            seed ^ 0x2b24 ^ i as u64,
            &trench_wood_color_picker(recipe),
        );
    }
    image
}

fn generate_trench_lip(recipe: &TerrainSpriteRecipe, variant: u32, front: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px / 2).max(16);
    let seed = sprite_seed(recipe.seed, variant, if front { 0x2b31 } else { 0x2b32 });
    let mut image = PixelImage::transparent(width, height);

    let band_y = if front { height / 3 } else { height / 4 };
    let lip = palette
        .dirt_mid
        .blend(palette.dirt_light, rules.lip_highlight_strength)
        .lighten(if front { 0.03 } else { 0.00 });
    let shadow = palette.dirt_dark.darken(0.16);

    let mut x = 0;
    while x < width {
        let seg = 7 + (hash(seed ^ x as u64 ^ 0x2b33) % 17) as u32;
        let gap = 1 + (hash(seed ^ x as u64 ^ 0x2b34) % 5) as u32;
        let jitter = (hash(seed ^ x as u64 ^ 0x2b35)
            % (rules.lip_irregularity_px.max(1) * 2 + 1) as u64) as i32
            - rules.lip_irregularity_px as i32;
        let y0 = (band_y as i32 + jitter).clamp(0, height as i32 - 1) as u32;
        let lip_h = 4 + (hash(seed ^ x as u64 ^ 0x2b36) % 4) as u32;
        for xx in x..(x + seg).min(width) {
            for yy in y0..(y0 + lip_h).min(height) {
                let color = if yy == y0 {
                    lip.lighten(0.08)
                } else if yy > y0 + lip_h / 2 {
                    shadow
                } else {
                    lip
                };
                image.set(xx, yy, color.with_alpha(242));
            }
        }
        x = x.saturating_add(seg + gap);
    }

    let clump_count = ((width * height) as f32 * rules.spoil_density / 42.0)
        .round()
        .max(2.0) as u32;
    for i in 0..clump_count {
        let x = (hash(seed ^ 0x2b37 ^ i as u64) % width as u64) as u32;
        let y = (band_y + (hash(seed ^ 0x2b38 ^ i as u64) % height.max(1) as u64) as u32 / 2)
            .min(height - 1);
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_lip,
            seed ^ i as u64,
            &trench_dirt_color_picker(recipe),
        );
    }

    let grass_count = ((width * height) as f32 * rules.grass_intrusion_density / 58.0)
        .round()
        .max(1.0) as u32;
    for i in 0..grass_count {
        let x = (hash(seed ^ 0x2b39 ^ i as u64) % width as u64) as u32;
        let y = (band_y.saturating_sub(2) + (hash(seed ^ 0x2b3a ^ i as u64) % 4) as u32)
            .min(height - 1);
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_grass_overhang,
            seed ^ 0x2b3b ^ i as u64,
            &trench_grass_color_picker(recipe),
        );
    }
    image
}

fn generate_trench_end_cap(recipe: &TerrainSpriteRecipe, variant: u32, left: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = (projection.cell_width_px / 2).max(38);
    let height = (projection.cell_height_px / 2 + projection.face_height_px / 2).max(48);
    let seed = sprite_seed(recipe.seed, variant, if left { 0x2b41 } else { 0x2b42 });
    let mut image = PixelImage::transparent(width, height);
    let palette = &recipe.style.palette;

    let wall = palette.dirt_dark.darken(0.20);
    let floor = palette.dirt_shadow.darken(0.28);
    let lip = palette.dirt_mid.lighten(0.08);
    let open_side = if left { width - 1 } else { 0 };

    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = if left {
                x as f32 / width.max(1) as f32
            } else {
                1.0 - x as f32 / width.max(1) as f32
            };
            let center = 0.56 + (trench_noise(seed ^ 0x2b43, y / 4, 0) * 0.07);
            let taper = (xf - 0.16).clamp(0.0, 1.0);
            let half = 0.08 + taper * 0.38;
            if (yf - center).abs() > half {
                continue;
            }
            let color = if yf < center - half * 0.28 {
                lip
            } else if yf < center + half * 0.18 {
                floor
            } else {
                wall
            };
            let edge_fade = if xf < 0.24 { xf / 0.24 } else { 1.0 };
            image.set(
                x,
                y,
                color.with_alpha((edge_fade.clamp(0.35, 1.0) * 245.0) as u8),
            );
        }
    }

    let edge_x = open_side.min(width - 1);
    for y in height / 3..height.saturating_sub(4) {
        image.set(edge_x, y, wall.darken(0.22).with_alpha(230));
    }

    for i in 0..10 {
        let x = (hash(seed ^ 0x2b44 ^ i) % width as u64) as u32;
        let y = (hash(seed ^ 0x2b45 ^ i) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_lip,
            seed ^ 0x2b46 ^ i,
            &trench_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_trench_corner(recipe: &TerrainSpriteRecipe, variant: u32, inner: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let size = (projection.cell_width_px / 2).max(46);
    let seed = sprite_seed(recipe.seed, variant, if inner { 0x2b51 } else { 0x2b52 });
    let mut image = PixelImage::transparent(size, size);
    let palette = &recipe.style.palette;
    let lip = palette.dirt_mid.lighten(0.08);
    let wall = palette.dirt_dark.darken(0.17);
    let floor = palette.dirt_shadow.darken(0.28);

    let arm = (size / 3).max(14);
    let center = size / 2;
    for y in 0..size {
        for x in 0..size {
            let horizontal =
                y >= center.saturating_sub(arm / 2) && y <= center + arm / 2 && x <= center + arm;
            let vertical =
                x >= center.saturating_sub(arm / 2) && x <= center + arm / 2 && y <= center + arm;
            let l_shape = if inner {
                horizontal || vertical
            } else {
                (horizontal || vertical) && !(x > center && y > center)
            };
            if !l_shape {
                continue;
            }
            let noise = trench_noise(seed ^ 0x2b53, x / 3, y / 3);
            let color = if y < center.saturating_sub(arm / 2) + 4
                || x < center.saturating_sub(arm / 2) + 4
            {
                lip
            } else if y > center + arm / 3 || x > center + arm / 3 {
                wall
            } else {
                floor
            };
            let alpha = (0.86 + noise * 0.12).clamp(0.62, 1.0);
            image.set(x, y, color.with_alpha((alpha * 255.0) as u8));
        }
    }

    for y in center.saturating_sub(arm / 3)..(center + arm).min(size) {
        for x in center.saturating_sub(arm / 3)..(center + arm).min(size) {
            image.blend_pixel(x, y, Rgba8::BLACK, if inner { 0.22 } else { 0.12 });
        }
    }

    let motif_count = 8 + (hash(seed ^ 0x2b54) % 5) as u32;
    for i in 0..motif_count {
        let x = (hash(seed ^ 0x2b55 ^ i as u64) % size as u64) as u32;
        let y = (hash(seed ^ 0x2b56 ^ (i as u64 * 7)) % size as u64) as u32;
        if i % 3 == 0 {
            draw_motif_from_set(
                &mut image,
                x,
                y,
                &recipe.motifs.trench_grass_overhang,
                seed ^ i as u64,
                &trench_grass_color_picker(recipe),
            );
        } else {
            draw_motif_from_set(
                &mut image,
                x,
                y,
                &recipe.motifs.trench_lip,
                seed ^ i as u64,
                &trench_dirt_color_picker(recipe),
            );
        }
    }
    image
}

fn generate_trench_contact_shadow(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 18).max(38);
    let seed = sprite_seed(recipe.seed, variant, 0x2b61);
    let mut image = PixelImage::transparent(width, height);
    let strength = recipe.style.trench.contact_shadow_strength;
    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let organic = trench_noise(seed ^ 0x2b62, x / 4, y / 3) * 0.08;
            let alpha = ((1.0 - xf * 1.65).max(0.0) * (1.0 - yf * 0.92).powf(1.55) + organic)
                * strength
                * 0.78;
            if alpha > 0.018 {
                image.set(
                    x,
                    y,
                    Rgba8::BLACK.with_alpha((alpha.clamp(0.0, 0.58) * 255.0) as u8),
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
    let width = projection.cell_width_px.max(54);
    let height = (projection.cell_height_px / 3).max(26);
    let seed = sprite_seed(recipe.seed, variant, 0x2b71);
    let mut image = PixelImage::transparent(width, height);
    let rules = &recipe.style.trench;
    let palette = &recipe.style.palette;

    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let n = trench_noise(seed ^ 0x2b72, x / 3, y / 3);
            if yf < 0.20 + n * 0.12 || yf > 0.82 - xf * 0.16 + n * 0.08 {
                continue;
            }
            let color = if yf < 0.42 {
                palette.dirt_light.blend(palette.dirt_mid, 0.40)
            } else {
                palette.dirt_mid.darken(0.08)
            };
            let alpha = (0.72 + n * 0.12).clamp(0.45, 0.90);
            image.set(x, y, color.with_alpha((alpha * 255.0) as u8));
        }
    }

    let count = ((width * height) as f32 * rules.spoil_density / 24.0)
        .round()
        .max(5.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x2b73 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2b74 ^ (i as u64 * 13)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_spoil,
            seed ^ 0x2b75 ^ i as u64,
            &trench_dirt_color_picker(recipe),
        );
    }

    let grass_count = ((width * height) as f32 * rules.grass_intrusion_density / 58.0)
        .round()
        .max(1.0) as u32;
    for i in 0..grass_count {
        let x = (hash(seed ^ 0x2b76 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x2b77 ^ (i as u64 * 5)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.trench_grass_overhang,
            seed ^ 0x2b78 ^ i as u64,
            &trench_grass_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_top(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.cell_height_px * 7 / 16).max(34);
    let seed = sprite_seed(recipe.seed, variant, 0x3b01);
    let mut image = PixelImage::transparent(width, height);
    let dirt = palette
        .dirt_mid
        .lighten(0.04)
        .blend(palette.grass_mid, rules.top_grass_blend * 0.34);
    let crown = palette
        .dirt_light
        .lighten(0.05)
        .blend(palette.grass_light, rules.top_grass_blend * 0.45);

    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let n = trench_noise(seed ^ 0x3b02, x / 4, y / 3);
            let upper = 0.18 + xf * 0.18 + n * 0.08;
            let lower = 0.88 - xf * 0.14 + n * 0.06;
            if yf < upper || yf > lower {
                continue;
            }
            let ridge = (1.0 - (yf - 0.40).abs() * 2.4).clamp(0.0, 1.0);
            let side_shadow = (xf * 0.30 + yf * 0.10).clamp(0.0, 0.32);
            let color = dirt
                .blend(crown, ridge * rules.mound_height_strength)
                .darken(side_shadow);
            let edge_alpha = ((yf - upper).min(lower - yf) * 12.0).clamp(0.50, 1.0);
            image.set(x, y, color.with_alpha((edge_alpha * 245.0) as u8));
        }
    }

    let clumps = ((width * height) as f32 * rules.spoil_density / 48.0)
        .round()
        .max(5.0) as u32;
    for i in 0..clumps {
        let x = (hash(seed ^ 0x3b03 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b04 ^ (i as u64 * 13)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_soil_clump,
            seed ^ i as u64,
            &berm_dirt_color_picker(recipe),
        );
    }
    let grass = ((width * height) as f32 * rules.grass_intrusion_density / 64.0)
        .round()
        .max(2.0) as u32;
    for i in 0..grass {
        let x = (hash(seed ^ 0x3b05 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b06 ^ (i as u64 * 7)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_grass_overhang,
            seed ^ 0x3b07 ^ i as u64,
            &trench_grass_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_face_front(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 28).max(42);
    let seed = sprite_seed(recipe.seed, variant, 0x3b21);
    let mut image = PixelImage::transparent(width, height);
    let top = palette
        .dirt_mid
        .blend(palette.grass_dark, rules.top_grass_blend * 0.20)
        .lighten(0.03)
        .darken(rules.face_shadow_strength * 0.05);
    let mid = palette
        .dirt_dark
        .blend(palette.dirt_mid, 0.24)
        .darken(0.04 + rules.face_shadow_strength * 0.14);
    let bottom = palette
        .dirt_shadow
        .darken(0.18 + rules.face_shadow_strength * 0.42);
    let crevice = palette
        .dirt_shadow
        .darken(0.24 + rules.face_shadow_strength * 0.28);

    for y in 0..height {
        for x in 0..width {
            let xf = x as f32 / width.max(1) as f32;
            let side = (xf - 0.5).abs();
            let shoulder = side.powf(1.55);
            let top_noise = trench_noise(seed ^ 0x3b22, x / 5, 0);
            let bottom_noise = trench_noise(seed ^ 0x3b26, x / 4, 1);
            let long_top_noise = trench_noise(seed ^ 0x3b2a, x / 11, 0);
            let long_bottom_noise = trench_noise(seed ^ 0x3b2b, x / 13, 1);
            let dent_noise = trench_noise(seed ^ 0x3b27, x / 9, y / 4);
            let top_edge = 3.0
                + shoulder * 8.0
                + top_noise * (rules.edge_irregularity_px + 2).max(1) as f32
                + long_top_noise * 3.5
                - dent_noise.max(0.0) * 2.0;
            let bottom_edge = height as f32 - 3.0 - shoulder * 11.0
                + bottom_noise * (rules.edge_irregularity_px + 4).max(1) as f32
                + long_bottom_noise * 4.5
                + dent_noise.min(0.0) * 2.5;
            let top_edge = top_edge.clamp(0.0, height as f32 - 9.0);
            let bottom_edge = bottom_edge.clamp(top_edge + 12.0, height as f32 - 1.0);
            let yf = y as f32 + 0.5;
            if yf < top_edge || yf > bottom_edge {
                continue;
            }
            let local_t = ((yf - top_edge) / (bottom_edge - top_edge).max(1.0)).clamp(0.0, 1.0);
            let shoulder_shadow = (shoulder * 0.16).clamp(0.0, 0.18);
            let mound_rounding = (1.0 - (local_t - 0.42).abs() * 2.2).clamp(0.0, 1.0);
            let color = if local_t < 0.16 {
                top
            } else if local_t > 0.70 {
                bottom
            } else {
                mid.lighten(mound_rounding * rules.mound_height_strength * 0.04)
                    .darken(shoulder_shadow)
            };
            let edge_fade = (yf - top_edge)
                .min(bottom_edge - yf)
                .mul_add(0.24, 0.62)
                .clamp(0.48, 1.0);
            let alpha = if local_t > 0.84 {
                (edge_fade * 216.0) as u8
            } else {
                (edge_fade * 244.0) as u8
            };
            image.set(x, y, color.with_alpha(alpha));
            if local_t < 0.09 && !(x + y + variant).is_multiple_of(5) {
                image.blend_pixel(x, y, crevice, 0.28);
            }
            if local_t > 0.78 {
                image.blend_pixel(
                    x,
                    y,
                    Rgba8::BLACK,
                    (0.10 + rules.face_shadow_strength * 0.16) * local_t,
                );
            }
        }
    }
    for x in 0..width {
        let ground_noise = trench_noise(seed ^ 0x3b28, x / 4, 0);
        let y0 = (height as f32 * 0.78 + ground_noise * 3.0) as i32;
        let shadow_h = 4 + (hash(seed ^ 0x3b29 ^ x as u64) % 4) as i32;
        for y in y0..(y0 + shadow_h) {
            if image.in_bounds(x as i32, y) {
                image.blend_pixel(x, y as u32, Rgba8::BLACK, 0.18);
            }
        }
    }

    let count = ((width * height) as f32 * rules.spoil_density / 44.0)
        .round()
        .max(4.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x3b23 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b24 ^ (i as u64 * 17)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_face_shadow,
            seed ^ 0x3b25 ^ i as u64,
            &berm_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_lip(recipe: &TerrainSpriteRecipe, variant: u32, front: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let palette = &recipe.style.palette;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px / 2).max(16);
    let seed = sprite_seed(recipe.seed, variant, if front { 0x3b31 } else { 0x3b32 });
    let mut image = PixelImage::transparent(width, height);
    let band_y = if front { height / 3 } else { height / 4 };
    let lip = palette
        .dirt_light
        .blend(palette.grass_light, rules.top_grass_blend * 0.30)
        .lighten(rules.lip_highlight_strength * 0.16);
    let shadow = palette.dirt_dark.darken(0.10);

    let mut x = 0;
    while x < width {
        let seg = 8 + (hash(seed ^ x as u64 ^ 0x3b33) % 20) as u32;
        let gap = 1 + (hash(seed ^ x as u64 ^ 0x3b34) % 6) as u32;
        let jitter = (hash(seed ^ x as u64 ^ 0x3b35)
            % (rules.edge_irregularity_px.max(1) * 2 + 1) as u64) as i32
            - rules.edge_irregularity_px as i32;
        let y0 = (band_y as i32 + jitter).clamp(0, height as i32 - 1) as u32;
        let h = 4 + (hash(seed ^ x as u64 ^ 0x3b36) % 5) as u32;
        for xx in x..(x + seg).min(width) {
            for yy in y0..(y0 + h).min(height) {
                let taper = ((xx - x).min(x + seg - xx) as f32 / seg.max(1) as f32).clamp(0.0, 0.5);
                let color = if yy == y0 {
                    lip
                } else if yy > y0 + h / 2 {
                    shadow
                } else {
                    palette.dirt_mid
                };
                let alpha = (190.0 + taper * 110.0).clamp(150.0, 246.0) as u8;
                image.set(xx, yy, color.with_alpha(alpha));
            }
        }
        if front && seg > 12 {
            let drip_x = x + seg / 2;
            for dy in 0..(2 + seg % 3) {
                if drip_x < width && y0 + h + dy < height {
                    image.blend_pixel(drip_x, y0 + h + dy, shadow, 0.32);
                }
            }
        }
        x = x.saturating_add(seg + gap);
    }

    let highlights = ((width * height) as f32 * rules.spoil_density / 50.0)
        .round()
        .max(2.0) as u32;
    for i in 0..highlights {
        let x = (hash(seed ^ 0x3b37 ^ i as u64) % width as u64) as u32;
        let y = (band_y + (hash(seed ^ 0x3b38 ^ i as u64) % height.max(1) as u64) as u32 / 2)
            .min(height - 1);
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_edge_highlight,
            seed ^ i as u64,
            &berm_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_end_cap(recipe: &TerrainSpriteRecipe, variant: u32, left: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let width = (projection.cell_width_px / 2).max(38);
    let height = (projection.cell_height_px / 2 + projection.face_height_px / 2).max(48);
    let seed = sprite_seed(recipe.seed, variant, if left { 0x3b41 } else { 0x3b42 });
    let mut image = PixelImage::transparent(width, height);
    let palette = &recipe.style.palette;
    let top = palette
        .dirt_light
        .blend(palette.grass_light, rules.top_grass_blend * 0.36);
    let body = palette.dirt_mid;
    let shadow = palette.dirt_dark.darken(0.22);
    let base = palette.dirt_shadow.darken(0.10);

    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = if left {
                x as f32 / width.max(1) as f32
            } else {
                1.0 - x as f32 / width.max(1) as f32
            };
            let n = trench_noise(seed ^ 0x3b43, y / 4, x / 4);
            let center = 0.52 + n * 0.05;
            let taper = xf.powf(0.74);
            let half_top = 0.08 + taper * 0.24;
            let half_bottom = 0.13 + taper * 0.32;
            let top_edge = center - half_top + n * 0.035;
            let bottom_edge = center + half_bottom - (1.0 - taper) * 0.08 + n * 0.04;
            if yf < top_edge || yf > bottom_edge {
                continue;
            }
            let local_t = ((yf - top_edge) / (bottom_edge - top_edge).max(0.01)).clamp(0.0, 1.0);
            let color = if local_t < 0.28 {
                top
            } else if local_t > 0.70 {
                base.blend(shadow, 0.44)
            } else if local_t > 0.52 {
                shadow
            } else {
                body
            };
            let edge_fade = (taper * 0.70 + 0.26).clamp(0.20, 1.0);
            image.set(x, y, color.with_alpha((edge_fade * 242.0) as u8));
        }
    }
    for i in 0..8 {
        let x = (hash(seed ^ 0x3b44 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b45 ^ i as u64) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_soil_clump,
            seed ^ 0x3b46 ^ i as u64,
            &berm_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_corner(recipe: &TerrainSpriteRecipe, variant: u32, inner: bool) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let size = (projection.cell_width_px / 2).max(46);
    let seed = sprite_seed(recipe.seed, variant, if inner { 0x3b51 } else { 0x3b52 });
    let mut image = PixelImage::transparent(size, size);
    let palette = &recipe.style.palette;
    let top = palette
        .dirt_light
        .blend(palette.grass_light, rules.top_grass_blend * 0.34);
    let body = palette
        .dirt_mid
        .blend(palette.grass_mid, rules.top_grass_blend * 0.12);
    let shadow = palette.dirt_dark.darken(0.18);
    let arm = (size / 3).max(14);
    let center = size / 2;
    for y in 0..size {
        for x in 0..size {
            let n = trench_noise(seed ^ 0x3b53, x / 3, y / 3);
            let horizontal = y >= center.saturating_sub(arm / 2)
                && y <= center + arm / 2 + (n * 3.0) as u32
                && x <= center + arm;
            let vertical = x >= center.saturating_sub(arm / 2)
                && x <= center + arm / 2 + (n * 3.0) as u32
                && y <= center + arm;
            let l_shape = if inner {
                horizontal || vertical
            } else {
                (horizontal || vertical) && !(x > center && y > center)
            };
            if !l_shape {
                continue;
            }
            let distal = x.max(y) as f32 / size.max(1) as f32;
            let color = if y < center.saturating_sub(arm / 2) + 5
                || x < center.saturating_sub(arm / 2) + 5
            {
                top
            } else if y > center + arm / 3 || x > center + arm / 3 || distal > 0.78 {
                shadow.blend(body, 0.18)
            } else {
                body
            };
            image.set(
                x,
                y,
                color.with_alpha(((0.82 + n * 0.10).clamp(0.58, 1.0) * 255.0) as u8),
            );
        }
    }
    image
}

fn generate_berm_contact_shadow(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px + 18).max(38);
    let seed = sprite_seed(recipe.seed, variant, 0x3b61);
    let mut image = PixelImage::transparent(width, height);
    let strength = recipe.style.berm.contact_shadow_strength;
    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let organic = trench_noise(seed ^ 0x3b62, x / 4, y / 3) * 0.07;
            let alpha = ((1.0 - xf * 1.35).max(0.0) * (1.0 - yf * 0.68).powf(1.18) + organic)
                * strength
                * 0.94;
            if alpha > 0.016 {
                image.set(
                    x,
                    y,
                    Rgba8::BLACK.with_alpha((alpha.clamp(0.0, 0.48) * 255.0) as u8),
                );
            }
        }
    }
    image
}

fn generate_berm_spoil_pile(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let width = projection.cell_width_px.max(54);
    let height = (projection.cell_height_px / 3).max(26);
    let seed = sprite_seed(recipe.seed, variant, 0x3b71);
    let mut image = PixelImage::transparent(width, height);
    let palette = &recipe.style.palette;
    for y in 0..height {
        let yf = y as f32 / height.max(1) as f32;
        for x in 0..width {
            let xf = (x as f32 / width.max(1) as f32 - 0.5).abs();
            let n = trench_noise(seed ^ 0x3b72, x / 3, y / 3);
            if yf < 0.22 + n * 0.08 || yf > 0.80 - xf * 0.12 + n * 0.06 {
                continue;
            }
            let color = if yf < 0.42 {
                palette
                    .dirt_light
                    .blend(palette.grass_light, rules.top_grass_blend * 0.18)
            } else {
                palette.dirt_mid.darken(0.06)
            };
            image.set(
                x,
                y,
                color.with_alpha(((0.68 + n * 0.12).clamp(0.42, 0.88) * 255.0) as u8),
            );
        }
    }
    let count = ((width * height) as f32 * rules.spoil_density / 24.0)
        .round()
        .max(5.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x3b73 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b74 ^ (i as u64 * 13)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_spoil,
            seed ^ 0x3b75 ^ i as u64,
            &berm_dirt_color_picker(recipe),
        );
    }
    image
}

fn generate_berm_grass_fringe(recipe: &TerrainSpriteRecipe, variant: u32) -> PixelImage {
    let projection = &recipe.style.projection;
    let rules = &recipe.style.berm;
    let width = projection.cell_width_px * 2;
    let height = (projection.face_height_px / 2).max(16);
    let seed = sprite_seed(recipe.seed, variant, 0x3b81);
    let mut image = PixelImage::transparent(width, height);
    let count = ((width * height) as f32 * rules.grass_intrusion_density / 18.0)
        .round()
        .max(8.0) as u32;
    for i in 0..count {
        let x = (hash(seed ^ 0x3b82 ^ i as u64) % width as u64) as u32;
        let y = (hash(seed ^ 0x3b83 ^ (i as u64 * 9)) % height as u64) as u32;
        draw_motif_from_set(
            &mut image,
            x,
            y,
            &recipe.motifs.berm_grass_overhang,
            seed ^ 0x3b84 ^ i as u64,
            &trench_grass_color_picker(recipe),
        );
    }
    image
}

fn trench_noise(seed: u64, x: u32, y: u32) -> f32 {
    hash01(seed ^ ((x as u64) << 18) ^ (y as u64 * 0x9e37)) * 2.0 - 1.0
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

fn berm_dirt_color_picker(recipe: &TerrainSpriteRecipe) -> impl Fn(i8) -> Rgba8 + '_ {
    move |shade| {
        let palette = &recipe.style.palette;
        match shade {
            -2 => palette.dirt_shadow.darken(0.18),
            -1 => palette.dirt_dark.darken(0.06),
            1 => palette.dirt_light.blend(
                palette.grass_light,
                recipe.style.berm.top_grass_blend * 0.20,
            ),
            2 => palette.pebble,
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
