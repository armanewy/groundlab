use std::fs;
use std::path::Path;

use anyhow::Result;
use ron::ser::PrettyConfig;

use crate::recipe::{GroundMaterial, TransitionEdge};
use crate::spritegen::{
    build_motif_heatmap, build_palette_preview, build_path_autotile_sheet,
    build_path_mask_debug_preview, build_path_neighbor_seam_heatmap, build_path_preview_dense,
    build_path_preview_junctions, build_path_preview_loop, build_path_preview_random,
    build_path_preview_sparse, build_repeat_preview, build_seam_heatmap,
    build_single_repeat_preview, build_sprite_contact_sheet, build_transition_edges_preview,
    build_transition_repeat_preview, build_variant_repeat_preview, generate_terrain_sprites,
    validate_terrain_sprites, GeneratedTerrainSprite, TerrainSpriteKind, TerrainSpriteRecipe,
    DEFAULT_SPRITEGEN_EXPORT_DIR,
};
use crate::terrain_artkit::{
    TerrainArtKitFile, TerrainArtOcclusion, TerrainArtOrientationSupport, TerrainArtPiece,
    TerrainArtPieceFile, TerrainArtPieceKind, TerrainArtRepeatMode,
};

#[derive(Clone, Debug)]
pub struct TerrainSpriteExportSummary {
    pub out_dir: String,
    pub sprite_count: usize,
    pub validation_issue_count: usize,
}

pub fn export_terrain_sprite_bundle(
    out_dir: impl AsRef<Path>,
    recipe: &TerrainSpriteRecipe,
) -> Result<TerrainSpriteExportSummary> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir.join("pieces"))?;

    let mut recipe = recipe.clone();
    recipe.sanitize();
    let sprites = generate_terrain_sprites(&recipe);
    for sprite in &sprites {
        sprite
            .image
            .save_png(out_dir.join("pieces").join(format!("{}.png", sprite.id)))?;
    }

    let manifest = TerrainArtKitFile {
        id: recipe.id.clone(),
        pieces: sprites
            .iter()
            .map(|sprite| TerrainArtPieceFile {
                piece: art_piece_for_sprite(sprite),
                file: format!("pieces/{}.png", sprite.id),
            })
            .collect(),
    };
    fs::write(
        out_dir.join("manifest.ron"),
        ron::ser::to_string_pretty(&manifest, PrettyConfig::new())?,
    )?;
    fs::write(
        out_dir.join("recipe.ron"),
        ron::ser::to_string_pretty(&recipe, PrettyConfig::new())?,
    )?;

    build_sprite_contact_sheet(&sprites, &recipe).save_png(out_dir.join("contact_sheet.png"))?;
    build_single_repeat_preview(&sprites, TerrainSpriteKind::GrassTile, &recipe, 5)
        .save_png(out_dir.join("repeat_preview_grass_single.png"))?;
    build_variant_repeat_preview(&sprites, TerrainSpriteKind::GrassTile, &recipe, 5)
        .save_png(out_dir.join("repeat_preview_grass_variants.png"))?;
    build_single_repeat_preview(&sprites, TerrainSpriteKind::DirtTile, &recipe, 5)
        .save_png(out_dir.join("repeat_preview_dirt_single.png"))?;
    build_variant_repeat_preview(&sprites, TerrainSpriteKind::DirtTile, &recipe, 5)
        .save_png(out_dir.join("repeat_preview_dirt_variants.png"))?;
    build_transition_repeat_preview(&sprites, &recipe)
        .save_png(out_dir.join("repeat_preview_transition.png"))?;
    build_transition_edges_preview(&sprites, &recipe)
        .save_png(out_dir.join("repeat_preview_transition_edges.png"))?;
    build_path_autotile_sheet(&sprites, &recipe)
        .save_png(out_dir.join("path_autotile_sheet.png"))?;
    build_path_preview_random(&sprites, &recipe)
        .save_png(out_dir.join("path_preview_random.png"))?;
    build_path_preview_sparse(&sprites, &recipe)
        .save_png(out_dir.join("path_preview_random_sparse.png"))?;
    build_path_preview_dense(&sprites, &recipe)
        .save_png(out_dir.join("path_preview_random_dense.png"))?;
    build_path_preview_loop(&sprites, &recipe).save_png(out_dir.join("path_preview_loop.png"))?;
    build_path_preview_junctions(&sprites, &recipe)
        .save_png(out_dir.join("path_preview_junctions.png"))?;
    build_path_mask_debug_preview(&recipe).save_png(out_dir.join("path_preview_mask_debug.png"))?;
    build_path_neighbor_seam_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("path_neighbor_seam_heatmap.png"))?;
    build_seam_heatmap(&sprites, &recipe).save_png(out_dir.join("seam_heatmap.png"))?;
    build_motif_heatmap(&sprites, &recipe).save_png(out_dir.join("motif_heatmap.png"))?;
    build_repeat_preview(&sprites, TerrainSpriteKind::GrassTile, &recipe)
        .save_png(out_dir.join("repeat_preview_grass.png"))?;
    build_repeat_preview(&sprites, TerrainSpriteKind::DirtTile, &recipe)
        .save_png(out_dir.join("repeat_preview_dirt.png"))?;
    build_palette_preview(&recipe).save_png(out_dir.join("palette_preview.png"))?;

    let validation = validate_terrain_sprites(&sprites);
    fs::write(
        out_dir.join("validation.json"),
        serde_json::to_string_pretty(&validation)?,
    )?;

    Ok(TerrainSpriteExportSummary {
        out_dir: out_dir.to_string_lossy().to_string(),
        sprite_count: sprites.len(),
        validation_issue_count: validation.issues.len(),
    })
}

pub fn export_default_terrain_sprite_bundle() -> Result<TerrainSpriteExportSummary> {
    export_terrain_sprite_bundle(
        DEFAULT_SPRITEGEN_EXPORT_DIR,
        &TerrainSpriteRecipe::from_default_style_profile(),
    )
}

fn art_piece_for_sprite(sprite: &GeneratedTerrainSprite) -> TerrainArtPiece {
    let (kind, material, repeat_mode, tags) = match sprite.kind {
        TerrainSpriteKind::GrassTile => (
            TerrainArtPieceKind::GrassFloorLarge,
            Some(GroundMaterial::Grass),
            TerrainArtRepeatMode::Tile,
            vec!["spritegen", "cozy", "grass", "tile"],
        ),
        TerrainSpriteKind::DirtTile => (
            TerrainArtPieceKind::DirtRoadLarge,
            Some(GroundMaterial::Dirt),
            TerrainArtRepeatMode::Tile,
            vec!["spritegen", "cozy", "dirt", "tile"],
        ),
        TerrainSpriteKind::GrassToDirtEdgeNorth
        | TerrainSpriteKind::GrassToDirtEdgeSouth
        | TerrainSpriteKind::GrassToDirtEdgeEast
        | TerrainSpriteKind::GrassToDirtEdgeWest => (
            TerrainArtPieceKind::DirtRoadEdge,
            Some(GroundMaterial::Dirt),
            TerrainArtRepeatMode::Tile,
            vec!["spritegen", "cozy", "grass-dirt-transition"],
        ),
        TerrainSpriteKind::PathMask00
        | TerrainSpriteKind::PathMask01
        | TerrainSpriteKind::PathMask02
        | TerrainSpriteKind::PathMask03
        | TerrainSpriteKind::PathMask04
        | TerrainSpriteKind::PathMask05
        | TerrainSpriteKind::PathMask06
        | TerrainSpriteKind::PathMask07
        | TerrainSpriteKind::PathMask08
        | TerrainSpriteKind::PathMask09
        | TerrainSpriteKind::PathMask10
        | TerrainSpriteKind::PathMask11
        | TerrainSpriteKind::PathMask12
        | TerrainSpriteKind::PathMask13
        | TerrainSpriteKind::PathMask14
        | TerrainSpriteKind::PathMask15 => (
            TerrainArtPieceKind::DirtRoadLarge,
            Some(GroundMaterial::Dirt),
            TerrainArtRepeatMode::Tile,
            vec!["spritegen", "cozy", "path-mask"],
        ),
    };
    TerrainArtPiece {
        id: sprite.id.clone(),
        kind,
        material,
        width_px: sprite.image.width,
        height_px: sprite.image.height,
        anchor_px: (0, 0),
        footprint_cells: (1, 1),
        repeat_mode,
        orientation: orientation_for_sprite(sprite.kind),
        z_bias: 0,
        opacity: 1.0,
        occlusion: TerrainArtOcclusion::None,
        tags: tags.into_iter().map(str::to_string).collect(),
    }
}

fn orientation_for_sprite(kind: TerrainSpriteKind) -> TerrainArtOrientationSupport {
    match transition_edge(kind) {
        Some(_) => TerrainArtOrientationSupport::FourWay,
        None => TerrainArtOrientationSupport::Any,
    }
}

fn transition_edge(kind: TerrainSpriteKind) -> Option<TransitionEdge> {
    match kind {
        TerrainSpriteKind::GrassToDirtEdgeNorth => Some(TransitionEdge::North),
        TerrainSpriteKind::GrassToDirtEdgeSouth => Some(TransitionEdge::South),
        TerrainSpriteKind::GrassToDirtEdgeEast => Some(TransitionEdge::East),
        TerrainSpriteKind::GrassToDirtEdgeWest => Some(TransitionEdge::West),
        _ => None,
    }
}
