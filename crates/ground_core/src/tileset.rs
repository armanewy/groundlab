use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};
use crate::palette::{muted_field_32, Palette};
use crate::pixel_image::PixelImage;
use crate::recipe::{GroundMaterial, TileRole, TilesetRecipe, TransitionEdge};

const TRANSITION_PAIRS: [(GroundMaterial, GroundMaterial); 8] = [
    (GroundMaterial::Grass, GroundMaterial::Dirt),
    (GroundMaterial::Grass, GroundMaterial::Mud),
    (GroundMaterial::Dirt, GroundMaterial::Mud),
    (GroundMaterial::Dirt, GroundMaterial::Rock),
    (GroundMaterial::Grass, GroundMaterial::Rock),
    (GroundMaterial::Grass, GroundMaterial::BermTop),
    (GroundMaterial::Dirt, GroundMaterial::TrenchFloor),
    (GroundMaterial::Grass, GroundMaterial::TrenchFloor),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileMetadata {
    pub id: String,
    pub role: TileRole,
    pub material: GroundMaterial,
    pub transition_to: Option<GroundMaterial>,
    pub transition_edge: Option<TransitionEdge>,
    pub variant: u32,
    pub movement_cost: f32,
    pub cover_hint: f32,
    pub blocks_sight_hint: bool,
    pub height_role: String,
}

#[derive(Clone, Debug)]
pub struct TileAsset {
    pub meta: TileMetadata,
    pub image: PixelImage,
}

#[derive(Clone, Debug)]
pub struct Tileset {
    pub recipe: TilesetRecipe,
    pub palette: Palette,
    pub tiles: Vec<TileAsset>,
}

impl Tileset {
    pub fn generate(recipe: &TilesetRecipe) -> Self {
        Self::generate_with_palette(recipe, &muted_field_32())
    }

    pub fn generate_with_palette(recipe: &TilesetRecipe, palette: &Palette) -> Self {
        let mut recipe = recipe.clone();
        recipe.sanitize();
        let palette = palette.clone();
        let mut tiles = Vec::new();

        for material in GroundMaterial::ALL {
            for variant in 0..recipe.variants_per_material {
                let image = generate_tile_image(&recipe, &palette, material, variant);
                let meta = TileMetadata {
                    id: format!("{}_{}", material.id(), variant),
                    role: TileRole::Surface,
                    material,
                    transition_to: None,
                    transition_edge: None,
                    variant,
                    movement_cost: material.base_movement_cost(),
                    cover_hint: cover_hint(material),
                    blocks_sight_hint: matches!(
                        material,
                        GroundMaterial::TrenchWall | GroundMaterial::BermFace
                    ),
                    height_role: height_role(material).to_string(),
                };
                tiles.push(TileAsset { meta, image });
            }
        }

        if recipe.generate_transitions {
            for (from, to) in TRANSITION_PAIRS {
                for edge in TransitionEdge::ALL {
                    for variant in 0..recipe.variants_per_material {
                        let image = generate_transition_tile_image(
                            &recipe, &palette, from, to, edge, variant,
                        );
                        let meta = TileMetadata {
                            id: format!(
                                "transition_{}_to_{}_{}_{}",
                                from.id(),
                                to.id(),
                                edge.id(),
                                variant
                            ),
                            role: TileRole::Transition,
                            material: from,
                            transition_to: Some(to),
                            transition_edge: Some(edge),
                            variant,
                            movement_cost: (from.base_movement_cost() + to.base_movement_cost())
                                * 0.5,
                            cover_hint: cover_hint(from).max(cover_hint(to)) * 0.5,
                            blocks_sight_hint: false,
                            height_role: "material_transition".to_string(),
                        };
                        tiles.push(TileAsset { meta, image });
                    }
                }
            }
        }

        Self {
            recipe,
            palette,
            tiles,
        }
    }

    pub fn tile(&self, material: GroundMaterial, variant: u32) -> &TileAsset {
        let variants = self.recipe.variants_per_material.max(1);
        let wanted = variant % variants;
        self.tiles
            .iter()
            .find(|t| {
                t.meta.role == TileRole::Surface
                    && t.meta.material == material
                    && t.meta.variant == wanted
            })
            .unwrap_or_else(|| &self.tiles[0])
    }

    pub fn transition_tile(
        &self,
        from: GroundMaterial,
        to: GroundMaterial,
        edge: TransitionEdge,
        variant: u32,
    ) -> Option<&TileAsset> {
        let variants = self.recipe.variants_per_material.max(1);
        let wanted = variant % variants;
        self.tiles.iter().find(|t| {
            t.meta.role == TileRole::Transition
                && t.meta.material == from
                && t.meta.transition_to == Some(to)
                && t.meta.transition_edge == Some(edge)
                && t.meta.variant == wanted
        })
    }

    pub fn surface_tiles_for(&self, material: GroundMaterial) -> Vec<&TileAsset> {
        self.tiles
            .iter()
            .filter(|tile| tile.meta.role == TileRole::Surface && tile.meta.material == material)
            .collect()
    }

    pub fn transition_tiles(&self) -> Vec<&TileAsset> {
        self.tiles
            .iter()
            .filter(|tile| tile.meta.role == TileRole::Transition)
            .collect()
    }

    pub fn surface_tile_count(&self) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile.meta.role == TileRole::Surface)
            .count()
    }

    pub fn transition_tile_count(&self) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile.meta.role == TileRole::Transition)
            .count()
    }

    pub fn build_contact_sheet(&self, columns: u32, padding: u32) -> PixelImage {
        let columns = columns.max(1);
        let tile = self.recipe.tile_size;
        let rows = (self.tiles.len() as u32).div_ceil(columns);
        let width = columns * tile + (columns + 1) * padding;
        let height = rows * tile + (rows + 1) * padding;
        let mut image = PixelImage::new(width, height, Rgba8::opaque(18, 18, 20));

        for (i, asset) in self.tiles.iter().enumerate() {
            let col = i as u32 % columns;
            let row = i as u32 / columns;
            let x = padding + col * (tile + padding);
            let y = padding + row * (tile + padding);
            image.blit(&asset.image, x, y);
            image.outline_rect(x, y, tile, tile, role_outline(asset.meta.role));
        }
        image
    }

    pub fn build_atlas(&self, columns: u32, padding: u32) -> PixelImage {
        self.build_contact_sheet(columns, padding)
    }
}

fn role_outline(role: TileRole) -> Rgba8 {
    match role {
        TileRole::Surface => Rgba8::opaque(38, 38, 42),
        TileRole::Transition => Rgba8::opaque(60, 78, 68),
        TileRole::StructureFace => Rgba8::opaque(82, 58, 42),
    }
}

fn generate_tile_image(
    recipe: &TilesetRecipe,
    palette: &Palette,
    material: GroundMaterial,
    variant: u32,
) -> PixelImage {
    let size = recipe.tile_size;
    let mut image = PixelImage::transparent(size, size);
    let seed = recipe.seed ^ (material as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ variant as u64;
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
            let coarse = value_noise(seed, x / 3, y / 3, 11);
            let fine = value_noise(seed, x, y, 31);
            let flecks = value_noise(seed, x.wrapping_mul(5), y.wrapping_mul(7), 47);
            let centered_noise = (coarse - 0.5) * recipe.detail_density * 0.38
                + (fine - 0.5) * recipe.detail_density * 0.16;

            let light_gradient =
                ((0.5 - nx) * lx + (0.5 - ny) * ly) * recipe.highlight_strength * 0.36;
            let mut t = material_base_t(material) + centered_noise + light_gradient;

            t += material_shape_bias(material, nx, ny, flecks, recipe);

            let mut color = palette.sample(material.ramp(), t);
            color = apply_material_marks(color, material, nx, ny, flecks, recipe);

            if is_edge_pixel(size, x, y) {
                color = color.darken(recipe.outline_strength * edge_strength(material));
            }

            image.set(x, y, color);
        }
    }

    add_material_detail(&mut image, recipe, material, seed);
    image
}

fn generate_transition_tile_image(
    recipe: &TilesetRecipe,
    palette: &Palette,
    from: GroundMaterial,
    to: GroundMaterial,
    edge: TransitionEdge,
    variant: u32,
) -> PixelImage {
    let size = recipe.tile_size;
    let from_img = generate_tile_image(recipe, palette, from, variant);
    let to_img = generate_tile_image(recipe, palette, to, variant.wrapping_add(23));
    let mut image = PixelImage::transparent(size, size);
    let seed = recipe.seed
        ^ (from as u64).wrapping_mul(0x51ed_270b_ee99_21b5)
        ^ (to as u64).wrapping_mul(0x94d0_49bb_1331_11eb)
        ^ (variant as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ edge as u64;

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
            let noise = value_noise(seed, x / 2, y / 2, 219) - 0.5;
            let edge_t = match edge {
                TransitionEdge::North => ny,
                TransitionEdge::South => 1.0 - ny,
                TransitionEdge::West => nx,
                TransitionEdge::East => 1.0 - nx,
            };
            let boundary = 0.36 + noise * 0.18;
            let alpha = clamp01((boundary - edge_t) / recipe.transition_feather + 0.5);
            let mut color = from_img.get(x, y).blend(to_img.get(x, y), alpha);
            if (alpha - 0.5).abs() < 0.10 {
                color = color.darken(recipe.outline_strength * 0.12);
            }
            image.set(x, y, color);
        }
    }

    image
}

fn material_base_t(material: GroundMaterial) -> f32 {
    match material {
        GroundMaterial::Grass => 0.54,
        GroundMaterial::Dirt => 0.55,
        GroundMaterial::Mud => 0.45,
        GroundMaterial::Rock => 0.50,
        GroundMaterial::TrenchFloor => 0.40,
        GroundMaterial::TrenchWall => 0.46,
        GroundMaterial::BermTop => 0.56,
        GroundMaterial::BermFace => 0.45,
    }
}

fn material_shape_bias(
    material: GroundMaterial,
    nx: f32,
    ny: f32,
    flecks: f32,
    recipe: &TilesetRecipe,
) -> f32 {
    match material {
        GroundMaterial::TrenchFloor => {
            let center = ((nx - 0.5).abs() + (ny - 0.5).abs()) * 0.5;
            -recipe.shadow_strength * (0.16 + center * 0.18)
        }
        GroundMaterial::TrenchWall => {
            let strata = ((ny * 6.0).floor() % 2.0) * 0.04;
            -recipe.shadow_strength * 0.16 + strata
        }
        GroundMaterial::BermFace => {
            let lower_shadow = ny * recipe.shadow_strength * -0.22;
            let ledge = if ny < 0.18 { 0.10 } else { 0.0 };
            lower_shadow + ledge
        }
        GroundMaterial::BermTop => {
            let crown = 1.0 - ((nx - 0.5).abs() + (ny - 0.5).abs()).min(1.0);
            crown * 0.08 - (flecks - 0.5) * 0.03
        }
        GroundMaterial::Mud => -recipe.shadow_strength * 0.05,
        GroundMaterial::Rock => (flecks - 0.5) * 0.08,
        GroundMaterial::Grass | GroundMaterial::Dirt => 0.0,
    }
}

fn apply_material_marks(
    color: Rgba8,
    material: GroundMaterial,
    nx: f32,
    ny: f32,
    flecks: f32,
    recipe: &TilesetRecipe,
) -> Rgba8 {
    match material {
        GroundMaterial::Grass if flecks > 0.82 => color.lighten(0.10 * recipe.detail_density),
        GroundMaterial::Dirt if flecks > 0.86 => color.lighten(0.08 * recipe.detail_density),
        GroundMaterial::Mud if flecks > 0.76 => color.darken(0.12 * recipe.shadow_strength),
        GroundMaterial::Rock if flecks > 0.66 => color.lighten(0.18 * recipe.detail_density),
        GroundMaterial::TrenchWall if (ny * 9.0 + nx * 2.0).fract() < 0.08 => {
            color.darken(0.10 * recipe.shadow_strength)
        }
        GroundMaterial::BermFace if (ny * 7.0).fract() < 0.08 => {
            color.darken(0.12 * recipe.shadow_strength)
        }
        _ => color,
    }
}

fn add_material_detail(
    image: &mut PixelImage,
    recipe: &TilesetRecipe,
    material: GroundMaterial,
    seed: u64,
) {
    let size = recipe.tile_size;
    let count =
        ((size * size) as f32 * recipe.detail_density * detail_density_factor(material)) as u32;
    for i in 0..count {
        let x = hash_u32(seed, i, 101) % size.max(1);
        let y = hash_u32(seed, i, 211) % size.max(1);
        let roll = hash01(seed, x, y, i);
        let px = image.get(x, y);
        let marked = if roll > 0.55 {
            px.lighten(0.08)
        } else {
            px.darken(0.08)
        };
        image.set(x, y, marked);

        if matches!(
            material,
            GroundMaterial::Rock | GroundMaterial::TrenchWall | GroundMaterial::BermFace
        ) && x + 1 < size
            && y + 1 < size
            && roll > 0.72
        {
            image.set(x + 1, y, marked.darken(0.05));
            image.set(x, y + 1, marked.darken(0.05));
        }
    }
}

fn detail_density_factor(material: GroundMaterial) -> f32 {
    match material {
        GroundMaterial::Grass => 0.09,
        GroundMaterial::Dirt => 0.08,
        GroundMaterial::Mud => 0.04,
        GroundMaterial::Rock => 0.12,
        GroundMaterial::TrenchFloor => 0.06,
        GroundMaterial::TrenchWall => 0.10,
        GroundMaterial::BermTop => 0.09,
        GroundMaterial::BermFace => 0.11,
    }
}

fn edge_strength(material: GroundMaterial) -> f32 {
    match material {
        GroundMaterial::Grass => 0.10,
        GroundMaterial::Dirt => 0.16,
        GroundMaterial::Mud => 0.22,
        GroundMaterial::Rock => 0.25,
        GroundMaterial::TrenchFloor => 0.32,
        GroundMaterial::TrenchWall => 0.38,
        GroundMaterial::BermTop => 0.20,
        GroundMaterial::BermFace => 0.36,
    }
}

pub fn cover_hint(material: GroundMaterial) -> f32 {
    match material {
        GroundMaterial::TrenchFloor => 0.65,
        GroundMaterial::TrenchWall | GroundMaterial::BermFace => 0.45,
        GroundMaterial::BermTop => 0.25,
        _ => 0.0,
    }
}

pub fn height_role(material: GroundMaterial) -> &'static str {
    match material {
        GroundMaterial::Grass
        | GroundMaterial::Dirt
        | GroundMaterial::Mud
        | GroundMaterial::Rock => "surface",
        GroundMaterial::TrenchFloor => "depression_floor",
        GroundMaterial::TrenchWall => "depression_edge",
        GroundMaterial::BermTop => "raised_surface",
        GroundMaterial::BermFace => "raised_edge",
    }
}

fn is_edge_pixel(size: u32, x: u32, y: u32) -> bool {
    x == 0 || y == 0 || x + 1 == size || y + 1 == size
}

pub fn stable_tile_variant(
    seed: u64,
    x: u32,
    y: u32,
    material: GroundMaterial,
    variants: u32,
) -> u32 {
    if variants == 0 {
        return 0;
    }
    hash_u32(seed ^ material as u64, x, y) % variants
}

pub fn hash01(seed: u64, x: u32, y: u32, salt: u32) -> f32 {
    (hash_u32(seed, x ^ salt, y.wrapping_add(salt)) as f32) / (u32::MAX as f32)
}

pub fn value_noise(seed: u64, x: u32, y: u32, salt: u32) -> f32 {
    let a = hash01(seed, x, y, salt);
    let b = hash01(seed, x.wrapping_add(1), y, salt);
    let c = hash01(seed, x, y.wrapping_add(1), salt);
    let d = hash01(seed, x.wrapping_add(1), y.wrapping_add(1), salt);
    clamp01((a + b + c + d) * 0.25)
}

pub fn hash_u32(seed: u64, x: u32, y: u32) -> u32 {
    let mut z = seed ^ ((x as u64) << 32) ^ y as u64;
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as u32
}
