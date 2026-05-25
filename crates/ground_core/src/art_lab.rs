use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{clamp_u8, PixelImage, Rgba8};

pub const ART_VARIANT_MAX_COUNT: u32 = 64;
pub const ART_VARIANT_MIN_SIZE: u32 = 32;
pub const ART_VARIANT_MAX_SIZE: u32 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtSpriteFamily {
    TerrainBase,
    Path,
    Trench,
    Berm,
    Tree,
    Log,
    Rock,
    Wall,
    Stakes,
    Wire,
    ObjectiveMarker,
    SpawnMarker,
}

impl ArtSpriteFamily {
    pub const ALL: [ArtSpriteFamily; 12] = [
        ArtSpriteFamily::TerrainBase,
        ArtSpriteFamily::Path,
        ArtSpriteFamily::Trench,
        ArtSpriteFamily::Berm,
        ArtSpriteFamily::Tree,
        ArtSpriteFamily::Log,
        ArtSpriteFamily::Rock,
        ArtSpriteFamily::Wall,
        ArtSpriteFamily::Stakes,
        ArtSpriteFamily::Wire,
        ArtSpriteFamily::ObjectiveMarker,
        ArtSpriteFamily::SpawnMarker,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ArtSpriteFamily::TerrainBase => "Terrain base",
            ArtSpriteFamily::Path => "Path",
            ArtSpriteFamily::Trench => "Trench",
            ArtSpriteFamily::Berm => "Berm",
            ArtSpriteFamily::Tree => "Tree",
            ArtSpriteFamily::Log => "Log",
            ArtSpriteFamily::Rock => "Rock",
            ArtSpriteFamily::Wall => "Wall",
            ArtSpriteFamily::Stakes => "Stakes",
            ArtSpriteFamily::Wire => "Wire",
            ArtSpriteFamily::ObjectiveMarker => "Objective marker",
            ArtSpriteFamily::SpawnMarker => "Spawn marker",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            ArtSpriteFamily::TerrainBase => "terrain_base",
            ArtSpriteFamily::Path => "path",
            ArtSpriteFamily::Trench => "trench",
            ArtSpriteFamily::Berm => "berm",
            ArtSpriteFamily::Tree => "tree",
            ArtSpriteFamily::Log => "log",
            ArtSpriteFamily::Rock => "rock",
            ArtSpriteFamily::Wall => "wall",
            ArtSpriteFamily::Stakes => "stakes",
            ArtSpriteFamily::Wire => "wire",
            ArtSpriteFamily::ObjectiveMarker => "objective_marker",
            ArtSpriteFamily::SpawnMarker => "spawn_marker",
        }
    }
}

impl fmt::Display for ArtSpriteFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for ArtSpriteFamily {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        for family in Self::ALL {
            if normalized == family.slug() || normalized == family.label().to_ascii_lowercase() {
                return Ok(family);
            }
        }
        Err(format!("unknown art sprite family '{value}'"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantRequest {
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub count: u32,
    pub width: u32,
    pub height: u32,
}

impl ArtVariantRequest {
    pub fn sanitized(&self) -> Self {
        Self {
            family: self.family,
            seed: self.seed,
            count: self.count.clamp(1, ART_VARIANT_MAX_COUNT),
            width: self.width.clamp(ART_VARIANT_MIN_SIZE, ART_VARIANT_MAX_SIZE),
            height: self
                .height
                .clamp(ART_VARIANT_MIN_SIZE, ART_VARIANT_MAX_SIZE),
        }
    }
}

impl Default for ArtVariantRequest {
    fn default() -> Self {
        Self {
            family: ArtSpriteFamily::Trench,
            seed: 99_418_113,
            count: 12,
            width: 32,
            height: 32,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariant {
    pub id: String,
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub variant_index: u32,
    pub image: PixelImage,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantBatch {
    pub request: ArtVariantRequest,
    pub variants: Vec<ArtVariant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtVariantMetadata {
    pub id: String,
    pub family: ArtSpriteFamily,
    pub seed: u64,
    pub variant_index: u32,
    pub width: u32,
    pub height: u32,
    pub notes: Vec<String>,
}

impl From<&ArtVariant> for ArtVariantMetadata {
    fn from(variant: &ArtVariant) -> Self {
        Self {
            id: variant.id.clone(),
            family: variant.family,
            seed: variant.seed,
            variant_index: variant.variant_index,
            width: variant.image.width,
            height: variant.image.height,
            notes: variant.notes.clone(),
        }
    }
}

pub fn generate_art_variants(request: &ArtVariantRequest) -> ArtVariantBatch {
    let request = request.sanitized();
    let variants = (0..request.count)
        .map(|variant_index| {
            let seed = derive_variant_seed(request.seed, request.family, variant_index);
            let mut rng = TinyRng::new(seed);
            let image = generate_family_image(&request, variant_index, &mut rng);
            ArtVariant {
                id: format!(
                    "{}_seed_{}_variant_{:02}",
                    request.family.slug(),
                    request.seed,
                    variant_index
                ),
                family: request.family,
                seed: request.seed,
                variant_index,
                image,
                notes: vec![
                    format!("family: {}", request.family.label()),
                    format!("deterministic seed: {seed}"),
                    "Art Lab procedural sprite".to_string(),
                ],
            }
        })
        .collect();
    ArtVariantBatch { request, variants }
}

pub fn export_art_variant_approved(
    variant: &ArtVariant,
    root_dir: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf)> {
    let dir = root_dir
        .as_ref()
        .join("approved")
        .join(variant.family.slug());
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let png_path = dir.join(format!("{}.png", variant.id));
    let json_path = dir.join(format!("{}.json", variant.id));
    variant
        .image
        .save_png(&png_path)
        .with_context(|| format!("failed to save {}", png_path.display()))?;
    let metadata = ArtVariantMetadata::from(variant);
    std::fs::write(&json_path, serde_json::to_string_pretty(&metadata)?)
        .with_context(|| format!("failed to write {}", json_path.display()))?;
    Ok((png_path, json_path))
}

pub fn export_art_variant_batch(
    batch: &ArtVariantBatch,
    out_dir: impl AsRef<Path>,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let out_dir = out_dir.as_ref();
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let mut exported = Vec::new();
    for variant in &batch.variants {
        let png_path = out_dir.join(format!("{}.png", variant.id));
        let json_path = out_dir.join(format!("{}.json", variant.id));
        variant
            .image
            .save_png(&png_path)
            .with_context(|| format!("failed to save {}", png_path.display()))?;
        let metadata = ArtVariantMetadata::from(variant);
        std::fs::write(&json_path, serde_json::to_string_pretty(&metadata)?)
            .with_context(|| format!("failed to write {}", json_path.display()))?;
        exported.push((png_path, json_path));
    }
    Ok(exported)
}

pub fn build_art_variant_contact_sheet(batch: &ArtVariantBatch) -> PixelImage {
    let scale = 3;
    let gap = 4;
    let border = 1;
    let count = batch.variants.len() as u32;
    let columns = (count as f32).sqrt().ceil().max(1.0) as u32;
    let rows = count.div_ceil(columns).max(1);
    let cell_w = batch.request.width * scale + border * 2;
    let cell_h = batch.request.height * scale + border * 2;
    let width = columns * cell_w + (columns + 1) * gap;
    let height = rows * cell_h + (rows + 1) * gap;
    let mut sheet = PixelImage::new(width, height, Rgba8::opaque(18, 21, 19));
    for (i, variant) in batch.variants.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        let x0 = gap + col * (cell_w + gap);
        let y0 = gap + row * (cell_h + gap);
        sheet.fill_rect(x0, y0, cell_w, cell_h, Rgba8::opaque(34, 38, 34));
        sheet.outline_rect(x0, y0, cell_w, cell_h, family_color(batch.request.family));
        blit_scaled_nearest(&mut sheet, &variant.image, x0 + border, y0 + border, scale);
    }
    sheet
}

pub fn export_art_contact_sheet(
    batch: &ArtVariantBatch,
    root_dir: impl AsRef<Path>,
) -> Result<PathBuf> {
    let dir = root_dir.as_ref().join("contact_sheets");
    std::fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = dir.join(format!(
        "{}_{}_{}.png",
        batch.request.family.slug(),
        batch.request.seed,
        batch.variants.len()
    ));
    build_art_variant_contact_sheet(batch)
        .save_png(&path)
        .with_context(|| format!("failed to save {}", path.display()))?;
    Ok(path)
}

fn generate_family_image(
    request: &ArtVariantRequest,
    variant_index: u32,
    rng: &mut TinyRng,
) -> PixelImage {
    let mut image = PixelImage::transparent(request.width, request.height);
    match request.family {
        ArtSpriteFamily::TerrainBase => draw_terrain_base(&mut image, rng),
        ArtSpriteFamily::Path => draw_path(&mut image, variant_index, rng),
        ArtSpriteFamily::Trench => draw_trench(&mut image, variant_index, rng),
        ArtSpriteFamily::Berm => draw_berm(&mut image, variant_index, rng),
        ArtSpriteFamily::Tree => draw_tree(&mut image, rng),
        ArtSpriteFamily::Log => draw_log(&mut image, rng),
        ArtSpriteFamily::Rock => draw_rock(&mut image, rng),
        ArtSpriteFamily::Wall => draw_wall(&mut image, rng),
        ArtSpriteFamily::Stakes => draw_stakes(&mut image, rng),
        ArtSpriteFamily::Wire => draw_wire(&mut image, rng),
        ArtSpriteFamily::ObjectiveMarker => draw_marker(&mut image, true, rng),
        ArtSpriteFamily::SpawnMarker => draw_marker(&mut image, false, rng),
    }
    image
}

fn draw_terrain_base(image: &mut PixelImage, rng: &mut TinyRng) {
    let base = Rgba8::opaque(91, 126, 61);
    fill(image, base);
    speckles(image, rng, 92, Rgba8::opaque(119, 151, 76), 0.10);
    speckles(image, rng, 53, Rgba8::opaque(65, 96, 48), 0.07);
}

fn draw_path(image: &mut PixelImage, variant_index: u32, rng: &mut TinyRng) {
    fill(image, Rgba8::opaque(88, 125, 62));
    let axis = art_band_axis(variant_index);
    let dirt = Rgba8::opaque(166, 107, 63);
    let compact = Rgba8::opaque(128, 78, 47);
    let dust = Rgba8::opaque(198, 145, 85);
    let grass_intrusion = Rgba8::opaque(80, 125, 64);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.48 + (rng.next_f32() - 0.5) * 0.14);
    let half_width = (max_lane * (0.17 + rng.next_f32() * 0.06)).max(5.0);
    let phase = rng.next_f32() * 9.0;
    let wave_amp = 1.6 + rng.next_f32() * 2.4;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let edge_noise = (rng.hash_xy(x, y) - 0.5) * 4.8;
            let dist = (lane - centerline + edge_noise).abs();
            if dist < half_width {
                let t = dist / half_width;
                let noise = rng.hash_xy(x.wrapping_add(19), y.wrapping_add(31));
                let color = dirt
                    .blend(compact, 0.22 * (1.0 - t))
                    .blend(dust, noise * 0.12);
                image.set(x, y, color);
            } else if dist < half_width + 4.5 && rng.hash_xy(x, y) > 0.22 {
                let t = ((dist - half_width) / 4.5).clamp(0.0, 1.0);
                image.set(x, y, dirt.blend(grass_intrusion, 0.38 + t * 0.42));
            }
        }
    }
    speckles(image, rng, scaled_count(image, 90), dust, 0.13);
    speckles(image, rng, scaled_count(image, 35), compact, 0.10);
    draw_path_ruts(image, axis, center, half_width, phase, rng);
}

fn draw_trench(image: &mut PixelImage, variant_index: u32, rng: &mut TinyRng) {
    fill(image, Rgba8::opaque(83, 120, 61));
    let axis = art_band_axis(variant_index);
    let floor_dark = Rgba8::opaque(38, 29, 24);
    let floor_warm = Rgba8::opaque(67, 45, 31);
    let wall_lit = Rgba8::opaque(126, 79, 44);
    let wall_shadow = Rgba8::opaque(84, 53, 35);
    let lip = Rgba8::opaque(178, 117, 66);
    let spoil = Rgba8::opaque(138, 88, 51);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.50 + (rng.next_f32() - 0.5) * 0.10);
    let half = (max_lane * (0.18 + rng.next_f32() * 0.05)).max(5.5);
    let floor_half = half * (0.42 + rng.next_f32() * 0.10);
    let phase = rng.next_f32() * 11.0;
    let wave_amp = 1.2 + rng.next_f32() * 2.0;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let signed = lane - centerline + (rng.hash_xy(x, y) - 0.5) * 3.4;
            let dist = signed.abs();
            if dist < floor_half {
                let center_t = 1.0 - (dist / floor_half).clamp(0.0, 1.0);
                let grain = rng.hash_xy(x.wrapping_add(71), y.wrapping_add(13));
                image.set(
                    x,
                    y,
                    floor_warm
                        .blend(floor_dark, 0.56 * center_t)
                        .blend(Rgba8::opaque(83, 56, 38), grain * 0.16),
                );
            } else if dist < half {
                let wall_t = ((dist - floor_half) / (half - floor_half)).clamp(0.0, 1.0);
                let side_light = if signed < 0.0 { 0.20 } else { 0.0 };
                image.set(
                    x,
                    y,
                    wall_shadow.blend(wall_lit, side_light + wall_t * 0.34),
                );
            } else if dist < half + 3.8 {
                let edge_t = ((dist - half) / 3.8).clamp(0.0, 1.0);
                let dirt = lip.blend(spoil, rng.hash_xy(x, y) * 0.18);
                image.set(x, y, dirt.blend(Rgba8::opaque(91, 124, 65), edge_t * 0.36));
            } else if dist < half + 6.0 && rng.hash_xy(x, y) > 0.62 {
                image.blend_pixel(x, y, spoil, 0.34);
            }
        }
    }
    speckles(
        image,
        rng,
        scaled_count(image, 58),
        Rgba8::opaque(204, 146, 83),
        0.10,
    );
    speckles(
        image,
        rng,
        scaled_count(image, 42),
        Rgba8::opaque(27, 23, 22),
        0.09,
    );
    draw_trench_cross_details(image, axis, center, floor_half, phase, rng);
}

fn draw_berm(image: &mut PixelImage, variant_index: u32, rng: &mut TinyRng) {
    fill(image, Rgba8::opaque(82, 119, 61));
    let axis = art_band_axis(variant_index);
    let top = Rgba8::opaque(149, 101, 56);
    let crest = Rgba8::opaque(187, 133, 75);
    let face = Rgba8::opaque(101, 65, 40);
    let base_shadow = Rgba8::opaque(49, 47, 34);
    let grass = Rgba8::opaque(82, 122, 64);
    let max_lane = art_band_max_lane(axis, image);
    let center = max_lane * (0.50 + (rng.next_f32() - 0.5) * 0.12);
    let half = (max_lane * (0.16 + rng.next_f32() * 0.07)).max(5.0);
    let crest_half = half * (0.34 + rng.next_f32() * 0.12);
    let phase = rng.next_f32() * 13.0;
    let wave_amp = 1.4 + rng.next_f32() * 2.4;
    for y in 0..image.height {
        for x in 0..image.width {
            let (lane, along) = art_band_coords(axis, x, y, image);
            let centerline = center + art_band_wave(along, phase, wave_amp);
            let signed = lane - centerline + (rng.hash_xy(x, y) - 0.5) * 3.8;
            let dist = signed.abs();
            if dist < crest_half {
                let crown = 1.0 - (dist / crest_half).clamp(0.0, 1.0);
                image.set(
                    x,
                    y,
                    top.blend(crest, 0.22 + crown * 0.28)
                        .blend(grass, rng.hash_xy(x, y) * 0.12),
                );
            } else if dist < half {
                let face_t = ((dist - crest_half) / (half - crest_half)).clamp(0.0, 1.0);
                let color = face.blend(top, (1.0 - face_t) * 0.22);
                image.set(x, y, color);
            } else if dist < half + 3.4 && rng.hash_xy(x, y) > 0.18 {
                let t = ((dist - half) / 3.4).clamp(0.0, 1.0);
                image.set(x, y, top.blend(grass, 0.35 + t * 0.42));
            } else if signed > 0.0 && dist < half + 5.2 {
                image.blend_pixel(x, y, base_shadow, 0.08 + rng.hash_xy(x, y) * 0.06);
            }
        }
    }
    speckles(image, rng, scaled_count(image, 62), crest, 0.09);
    speckles(
        image,
        rng,
        scaled_count(image, 38),
        Rgba8::opaque(67, 47, 34),
        0.08,
    );
    draw_mound_strata(image, axis, center, crest_half, half, phase, rng);
}

#[derive(Clone, Copy, Debug)]
enum ArtBandAxis {
    Horizontal,
    Vertical,
    DiagonalDown,
    DiagonalUp,
}

fn art_band_axis(variant_index: u32) -> ArtBandAxis {
    match variant_index % 6 {
        1 => ArtBandAxis::Vertical,
        2 => ArtBandAxis::DiagonalDown,
        3 => ArtBandAxis::Horizontal,
        4 => ArtBandAxis::DiagonalUp,
        _ => ArtBandAxis::Horizontal,
    }
}

fn art_band_max_lane(axis: ArtBandAxis, image: &PixelImage) -> f32 {
    match axis {
        ArtBandAxis::Horizontal | ArtBandAxis::DiagonalDown | ArtBandAxis::DiagonalUp => {
            image.height as f32
        }
        ArtBandAxis::Vertical => image.width as f32,
    }
}

fn art_band_coords(axis: ArtBandAxis, x: u32, y: u32, image: &PixelImage) -> (f32, f32) {
    let xf = x as f32;
    let yf = y as f32;
    let w = image.width.max(1) as f32;
    let h = image.height.max(1) as f32;
    match axis {
        ArtBandAxis::Horizontal => (yf, xf),
        ArtBandAxis::Vertical => (xf, yf),
        ArtBandAxis::DiagonalDown => (yf - xf * (h / w) + h * 0.50, xf),
        ArtBandAxis::DiagonalUp => (yf + xf * (h / w) - h * 0.50, xf),
    }
}

fn art_band_wave(along: f32, phase: f32, amplitude: f32) -> f32 {
    (along * 0.34 + phase).sin() * amplitude + (along * 0.17 + phase * 1.7).sin() * amplitude * 0.42
}

fn scaled_count(image: &PixelImage, base_count: u32) -> u32 {
    let area = image.width.max(1) * image.height.max(1);
    let scale = area as f32 / (32.0 * 32.0);
    (base_count as f32 * scale).round().max(1.0) as u32
}

fn draw_path_ruts(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    half_width: f32,
    phase: f32,
    rng: &mut TinyRng,
) {
    let color = Rgba8::opaque(101, 67, 43).with_alpha(220);
    let marks = scaled_count(image, 8);
    for _ in 0..marks {
        let along = rng.next_f32() * image.width.max(image.height) as f32;
        let side = if rng.next_f32() > 0.5 { 1.0 } else { -1.0 };
        let lane = center + art_band_wave(along, phase, 1.0) + side * half_width * 0.34;
        let (x, y) = art_band_point(axis, along, lane, image);
        match axis {
            ArtBandAxis::Horizontal => {
                image.draw_line(x - 4, y, x + 5, y + rng.range_i32(-1, 2), color)
            }
            ArtBandAxis::Vertical => {
                image.draw_line(x, y - 4, x + rng.range_i32(-1, 2), y + 5, color)
            }
            ArtBandAxis::DiagonalDown => image.draw_line(x - 3, y - 3, x + 4, y + 4, color),
            ArtBandAxis::DiagonalUp => image.draw_line(x - 3, y + 3, x + 4, y - 4, color),
        }
    }
}

fn draw_trench_cross_details(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    floor_half: f32,
    phase: f32,
    rng: &mut TinyRng,
) {
    let detail = Rgba8::opaque(104, 66, 39).with_alpha(215);
    let marks = scaled_count(image, 5);
    for _ in 0..marks {
        let along = rng.next_f32() * image.width.max(image.height) as f32;
        let lane = center + art_band_wave(along, phase, 1.0);
        let (x, y) = art_band_point(axis, along, lane, image);
        let half = floor_half.max(3.0) as i32;
        match axis {
            ArtBandAxis::Horizontal => image.draw_line(x, y - half / 2, x, y + half / 2, detail),
            ArtBandAxis::Vertical => image.draw_line(x - half / 2, y, x + half / 2, y, detail),
            ArtBandAxis::DiagonalDown => image.draw_line(x - 2, y + 2, x + 2, y - 2, detail),
            ArtBandAxis::DiagonalUp => image.draw_line(x - 2, y - 2, x + 2, y + 2, detail),
        }
    }
}

fn draw_mound_strata(
    image: &mut PixelImage,
    axis: ArtBandAxis,
    center: f32,
    crest_half: f32,
    half: f32,
    phase: f32,
    rng: &mut TinyRng,
) {
    let color = Rgba8::opaque(78, 52, 36).with_alpha(190);
    let lines = 2 + (rng.next_u32() % 3);
    for i in 0..lines {
        let lane_offset = crest_half + (half - crest_half) * (i as f32 + 0.45) / lines as f32;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let mut prev = None;
        for step in 0..image.width.max(image.height) {
            let along = step as f32;
            let lane = center + art_band_wave(along, phase, 0.9) + side * lane_offset;
            let point = art_band_point(axis, along, lane, image);
            if let Some((px, py)) = prev {
                if rng.hash_xy(step, i) > 0.32 {
                    image.draw_line(px, py, point.0, point.1, color);
                }
            }
            prev = Some(point);
        }
    }
}

fn art_band_point(axis: ArtBandAxis, along: f32, lane: f32, image: &PixelImage) -> (i32, i32) {
    let w = image.width.max(1) as f32;
    let h = image.height.max(1) as f32;
    match axis {
        ArtBandAxis::Horizontal => (along.round() as i32, lane.round() as i32),
        ArtBandAxis::Vertical => (lane.round() as i32, along.round() as i32),
        ArtBandAxis::DiagonalDown => {
            let x = along;
            let y = lane + x * (h / w) - h * 0.50;
            (x.round() as i32, y.round() as i32)
        }
        ArtBandAxis::DiagonalUp => {
            let x = along;
            let y = lane - x * (h / w) + h * 0.50;
            (x.round() as i32, y.round() as i32)
        }
    }
}

fn draw_tree(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.58);
    let trunk = Rgba8::opaque(99, 63, 35);
    let dark = Rgba8::opaque(39, 83, 44);
    let mid = Rgba8::opaque(54, 119, 55);
    let light = Rgba8::opaque(82, 146, 66);
    let cx = image.width as i32 / 2 + rng.range_i32(-2, 3);
    let cy = image.height as i32 / 2 + 5;
    rect_i32(image, cx - 2, cy, 4, 10, trunk);
    for layer in 0..3 {
        let radius = 10 - layer * 2;
        let y = cy - 7 - layer * 6;
        ellipse(image, cx, y, radius, 6, dark);
        ellipse(image, cx - 2, y - 1, radius - 2, 4, mid);
        if layer == 0 {
            ellipse(image, cx - 4, y - 3, 4, 2, light);
        }
    }
}

fn draw_log(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.45);
    let y = image.height as i32 / 2 + rng.range_i32(1, 4);
    let x0 = image.width as i32 / 2 - 12;
    for i in 0..25 {
        let x = x0 + i;
        rect_i32(image, x, y - 2 + i / 10, 1, 6, Rgba8::opaque(126, 72, 37));
        if i % 6 == 0 {
            rect_i32(image, x, y - 3 + i / 10, 1, 7, Rgba8::opaque(78, 48, 31));
        }
    }
    ellipse(image, x0, y, 3, 4, Rgba8::opaque(88, 54, 32));
    ellipse(image, x0 + 24, y + 2, 3, 4, Rgba8::opaque(169, 103, 55));
}

fn draw_rock(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.40);
    let cx = image.width as i32 / 2 + rng.range_i32(-2, 2);
    let cy = image.height as i32 / 2 + rng.range_i32(1, 4);
    ellipse(image, cx, cy, 10, 7, Rgba8::opaque(99, 104, 96));
    ellipse(image, cx - 2, cy - 2, 7, 4, Rgba8::opaque(139, 143, 131));
    image.draw_line(cx - 8, cy + 1, cx + 2, cy + 5, Rgba8::opaque(70, 74, 70));
}

fn draw_wall(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.40);
    let y = image.height / 2;
    for i in 0..4 {
        let x = 5 + i * 6;
        let color = if i % 2 == 0 {
            Rgba8::opaque(119, 117, 105)
        } else {
            Rgba8::opaque(92, 92, 84)
        };
        image.fill_rect(x, y - 5 + (rng.next_u32() % 3), 7, 10, color);
        image.outline_rect(x, y - 5, 7, 10, Rgba8::opaque(53, 54, 51));
    }
}

fn draw_stakes(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.34);
    for i in 0..5 {
        let x = 8 + i * 4 + rng.range_i32(-1, 2) as u32;
        let y = 19 + rng.range_i32(-2, 2) as u32;
        image.draw_line(
            x as i32,
            y as i32,
            x as i32,
            y as i32 - 12,
            Rgba8::opaque(117, 73, 38),
        );
        image.draw_line(
            x as i32 - 2,
            y as i32 - 7,
            x as i32 + 2,
            y as i32 - 9,
            Rgba8::opaque(184, 126, 63),
        );
    }
}

fn draw_wire(image: &mut PixelImage, rng: &mut TinyRng) {
    draw_shadow(image, 0.20);
    for strand in 0..3 {
        let y = 13 + strand * 4;
        for x in 4..image.width.saturating_sub(4) {
            let wobble = if (x + strand) % 4 < 2 { 1 } else { -1 };
            image.set_i32(x as i32, y as i32 + wobble, Rgba8::opaque(143, 144, 128));
            if x % 7 == (rng.next_u32() % 7) {
                image.set_i32(x as i32, y as i32 - 2, Rgba8::opaque(219, 208, 126));
            }
        }
    }
}

fn draw_marker(image: &mut PixelImage, objective: bool, _rng: &mut TinyRng) {
    draw_shadow(image, 0.32);
    let pole = if objective {
        Rgba8::opaque(222, 194, 91)
    } else {
        Rgba8::opaque(92, 143, 207)
    };
    let flag = if objective {
        Rgba8::opaque(210, 63, 50)
    } else {
        Rgba8::opaque(233, 186, 73)
    };
    let cx = image.width as i32 / 2;
    let base_y = image.height as i32 - 9;
    rect_i32(image, cx - 7, base_y, 14, 4, Rgba8::opaque(119, 91, 45));
    image.draw_line(cx, base_y, cx, base_y - 17, pole);
    rect_i32(image, cx + 1, base_y - 17, 10, 7, flag);
    rect_i32(image, cx + 1, base_y - 10, 6, 3, flag.darken(0.20));
}

fn fill(image: &mut PixelImage, color: Rgba8) {
    image.fill_rect(0, 0, image.width, image.height, color);
}

fn draw_shadow(image: &mut PixelImage, alpha: f32) {
    let cx = image.width as i32 / 2;
    let cy = image.height as i32 - 8;
    ellipse(
        image,
        cx,
        cy,
        (image.width as i32 / 3).max(4),
        4,
        Rgba8::BLACK.with_alpha(clamp_u8(alpha * 150.0)),
    );
}

fn speckles(image: &mut PixelImage, rng: &mut TinyRng, count: u32, color: Rgba8, alpha: f32) {
    for _ in 0..count {
        let x = rng.next_u32() % image.width.max(1);
        let y = rng.next_u32() % image.height.max(1);
        image.blend_pixel(x, y, color, alpha);
    }
}

fn rect_i32(image: &mut PixelImage, x: i32, y: i32, width: i32, height: i32, color: Rgba8) {
    for yy in y..y + height {
        for xx in x..x + width {
            image.set_i32(xx, yy, color);
        }
    }
}

fn ellipse(image: &mut PixelImage, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba8) {
    let rx = rx.max(1);
    let ry = ry.max(1);
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let dx = (x - cx) as f32 / rx as f32;
            let dy = (y - cy) as f32 / ry as f32;
            if dx * dx + dy * dy <= 1.0 {
                if color.a == 255 {
                    image.set_i32(x, y, color);
                } else if image.in_bounds(x, y) {
                    let current = image.get(x as u32, y as u32);
                    image.set_i32(x, y, current.blend(color, color.a as f32 / 255.0));
                }
            }
        }
    }
}

fn blit_scaled_nearest(target: &mut PixelImage, source: &PixelImage, x0: u32, y0: u32, scale: u32) {
    for y in 0..source.height {
        for x in 0..source.width {
            let color = source.get(x, y);
            for sy in 0..scale {
                for sx in 0..scale {
                    target.set(x0 + x * scale + sx, y0 + y * scale + sy, color);
                }
            }
        }
    }
}

fn family_color(family: ArtSpriteFamily) -> Rgba8 {
    match family {
        ArtSpriteFamily::TerrainBase => Rgba8::opaque(104, 151, 82),
        ArtSpriteFamily::Path => Rgba8::opaque(190, 134, 80),
        ArtSpriteFamily::Trench => Rgba8::opaque(78, 128, 164),
        ArtSpriteFamily::Berm => Rgba8::opaque(190, 139, 76),
        ArtSpriteFamily::Tree => Rgba8::opaque(82, 162, 86),
        ArtSpriteFamily::Log => Rgba8::opaque(175, 101, 52),
        ArtSpriteFamily::Rock | ArtSpriteFamily::Wall => Rgba8::opaque(156, 156, 144),
        ArtSpriteFamily::Stakes | ArtSpriteFamily::Wire => Rgba8::opaque(217, 181, 91),
        ArtSpriteFamily::ObjectiveMarker => Rgba8::opaque(218, 80, 62),
        ArtSpriteFamily::SpawnMarker => Rgba8::opaque(90, 146, 220),
    }
}

fn derive_variant_seed(seed: u64, family: ArtSpriteFamily, variant_index: u32) -> u64 {
    let mut hash = seed ^ 0x9e37_79b9_7f4a_7c15;
    for b in family.slug().bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash ^ (variant_index as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
}

#[derive(Clone, Debug)]
struct TinyRng {
    state: u64,
}

impl TinyRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xa076_1d64_78bd_642f,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(0x5851_f42d_4c95_7f2d)
            .wrapping_add(0x1405_7b7e_f767_814f);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_u32() % (max - min) as u32) as i32
    }

    fn hash_xy(&self, x: u32, y: u32) -> f32 {
        let mut h = self.state ^ ((x as u64) << 32) ^ y as u64;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        ((h >> 40) as f32) / ((1_u64 << 24) as f32)
    }
}

pub fn parse_art_variant_cli(family: &str, seed: &str, count: &str) -> Result<ArtVariantRequest> {
    let family = family
        .parse::<ArtSpriteFamily>()
        .map_err(|err| anyhow::anyhow!(err))?;
    let seed = seed.parse::<u64>().context("invalid art variant seed")?;
    let count = count.parse::<u32>().context("invalid art variant count")?;
    Ok(ArtVariantRequest {
        family,
        seed,
        count,
        width: 32,
        height: 32,
    }
    .sanitized())
}

pub fn ensure_art_batch_not_empty(batch: &ArtVariantBatch) -> Result<()> {
    if batch.variants.is_empty() {
        bail!("art variant batch is empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_variants_are_deterministic() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Trench,
            seed: 123,
            count: 4,
            width: 32,
            height: 32,
        };
        let a = generate_art_variants(&request);
        let b = generate_art_variants(&request);
        assert_eq!(
            a.variants[2].image.to_rgba_bytes(),
            b.variants[2].image.to_rgba_bytes()
        );
        assert_eq!(a.variants[2].id, "trench_seed_123_variant_02");
    }

    #[test]
    fn art_variant_count_is_clamped() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Path,
            seed: 55,
            count: 128,
            width: 8,
            height: 512,
        };
        let batch = generate_art_variants(&request);
        assert_eq!(batch.variants.len(), ART_VARIANT_MAX_COUNT as usize);
        assert_eq!(batch.request.width, ART_VARIANT_MIN_SIZE);
        assert_eq!(batch.request.height, ART_VARIANT_MAX_SIZE);
    }

    #[test]
    fn art_variant_metadata_serializes_and_contact_sheet_has_size() {
        let request = ArtVariantRequest {
            family: ArtSpriteFamily::Berm,
            seed: 77,
            count: 3,
            width: 32,
            height: 32,
        };
        let batch = generate_art_variants(&request);
        let metadata = ArtVariantMetadata::from(&batch.variants[0]);
        let json = serde_json::to_string(&metadata).expect("metadata should serialize");
        assert!(json.contains("berm_seed_77_variant_00"));

        let sheet = build_art_variant_contact_sheet(&batch);
        assert!(sheet.width > batch.request.width);
        assert!(sheet.height > batch.request.height);
    }

    #[test]
    fn terrain_art_families_produce_distinct_variants() {
        for family in [
            ArtSpriteFamily::Path,
            ArtSpriteFamily::Trench,
            ArtSpriteFamily::Berm,
        ] {
            let request = ArtVariantRequest {
                family,
                seed: 99_418_113,
                count: 2,
                width: 32,
                height: 32,
            };
            let batch = generate_art_variants(&request);
            assert_ne!(
                batch.variants[0].image.to_rgba_bytes(),
                batch.variants[1].image.to_rgba_bytes(),
                "{family:?} variants should differ"
            );
        }
    }
}
