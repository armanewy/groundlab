use std::fs;
use std::path::Path;

use anyhow::Result;
use ron::ser::PrettyConfig;

use crate::recipe::{GroundMaterial, TransitionEdge};
use crate::spritegen::{
    build_berm_contact_sheet, build_berm_mask_debug_preview, build_berm_oblique_caps_preview,
    build_berm_oblique_corner_preview, build_berm_oblique_shadow_preview,
    build_berm_oblique_straight_preview, build_motif_heatmap, build_oblique_material_preview,
    build_override_contact_sheet, build_override_diff_sheet, build_palette_preview,
    build_path_autotile_sheet, build_path_mask_debug_preview, build_path_neighbor_seam_heatmap,
    build_path_preview_dense, build_path_preview_junctions, build_path_preview_loop,
    build_path_preview_random, build_path_preview_sparse, build_repeat_preview, build_seam_heatmap,
    build_single_repeat_preview, build_sprite_contact_sheet, build_transition_edges_preview,
    build_transition_repeat_preview, build_trench_autotile_sheet, build_trench_contact_sheet,
    build_trench_floor_continuity_edge_heatmap, build_trench_floor_continuity_heatmap,
    build_trench_lip_continuity_edge_heatmap, build_trench_lip_continuity_heatmap,
    build_trench_mask_debug_preview, build_trench_neighbor_seam_edge_heatmap,
    build_trench_neighbor_seam_heatmap, build_trench_oblique_caps_preview,
    build_trench_oblique_corner_preview, build_trench_oblique_shadow_preview,
    build_trench_oblique_straight_preview, build_trench_preview_corners,
    build_trench_preview_dead_ends, build_trench_preview_dense, build_trench_preview_dense_clean,
    build_trench_preview_junctions, build_trench_preview_loop, build_trench_preview_single_masks,
    build_trench_preview_sparse, build_variant_repeat_preview, generate_effective_terrain_sprites,
    promote_generated_sprites_to_overrides, validate_terrain_sprites, GeneratedTerrainSprite,
    TerrainSpriteBundleManifest, TerrainSpriteKind, TerrainSpritePieceManifest,
    TerrainSpriteRecipe, DEFAULT_SPRITEGEN_EXPORT_DIR,
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
    pub overridden_count: usize,
    pub override_issue_count: usize,
}

pub fn export_terrain_sprite_bundle(
    out_dir: impl AsRef<Path>,
    recipe: &TerrainSpriteRecipe,
) -> Result<TerrainSpriteExportSummary> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir.join("pieces"))?;
    fs::create_dir_all(out_dir.join("generated_pieces"))?;

    let mut recipe = recipe.clone();
    recipe.sanitize();
    let effective_bundle = generate_effective_terrain_sprites(&recipe);
    let generated_sprites = effective_bundle.generated;
    let sprites = effective_bundle.effective;
    let override_report = effective_bundle.report;
    for sprite in &generated_sprites {
        sprite.image.save_png(
            out_dir
                .join("generated_pieces")
                .join(format!("{}.png", sprite.id)),
        )?;
    }
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
    let sprite_manifest = TerrainSpriteBundleManifest {
        id: recipe.id.clone(),
        projection: recipe.style.projection.clone(),
        pieces: sprites
            .iter()
            .map(|sprite| TerrainSpritePieceManifest {
                id: sprite.id.clone(),
                kind: sprite.kind,
                role: sprite.metadata.role,
                source: sprite.source,
                file: format!("pieces/{}.png", sprite.id),
                width_px: sprite.image.width,
                height_px: sprite.image.height,
                anchor_px: sprite.metadata.anchor_px,
                footprint_cells: sprite.metadata.footprint_cells,
                z_bias: sprite.metadata.z_bias,
                occludes: sprite.metadata.occludes,
            })
            .collect(),
    };
    fs::write(
        out_dir.join("sprite_manifest.ron"),
        ron::ser::to_string_pretty(&sprite_manifest, PrettyConfig::new())?,
    )?;
    fs::write(
        out_dir.join("sprite_manifest.json"),
        serde_json::to_string_pretty(&sprite_manifest)?,
    )?;
    fs::write(
        out_dir.join("recipe.ron"),
        ron::ser::to_string_pretty(&recipe, PrettyConfig::new())?,
    )?;

    build_sprite_contact_sheet(&sprites, &recipe).save_png(out_dir.join("contact_sheet.png"))?;
    build_sprite_contact_sheet(&generated_sprites, &recipe)
        .save_png(out_dir.join("generated_contact_sheet.png"))?;
    build_sprite_contact_sheet(&sprites, &recipe)
        .save_png(out_dir.join("effective_contact_sheet.png"))?;
    build_override_contact_sheet(&generated_sprites, &sprites, &recipe)
        .save_png(out_dir.join("override_contact_sheet.png"))?;
    build_override_diff_sheet(&generated_sprites, &sprites, &recipe)
        .save_png(out_dir.join("override_diff_sheet.png"))?;
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
    build_oblique_material_preview(&sprites, &recipe)
        .save_png(out_dir.join("oblique_material_preview.png"))?;
    build_trench_contact_sheet(&sprites, &recipe)
        .save_png(out_dir.join("trench_contact_sheet.png"))?;
    build_trench_oblique_straight_preview(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_oblique_straight.png"))?;
    build_trench_oblique_caps_preview(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_oblique_caps.png"))?;
    build_trench_oblique_corner_preview(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_oblique_corner.png"))?;
    build_trench_oblique_shadow_preview(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_oblique_shadow.png"))?;
    build_berm_contact_sheet(&sprites, &recipe).save_png(out_dir.join("berm_contact_sheet.png"))?;
    build_berm_oblique_straight_preview(&sprites, &recipe)
        .save_png(out_dir.join("berm_preview_oblique_straight.png"))?;
    build_berm_oblique_caps_preview(&sprites, &recipe)
        .save_png(out_dir.join("berm_preview_oblique_caps.png"))?;
    build_berm_oblique_corner_preview(&sprites, &recipe)
        .save_png(out_dir.join("berm_preview_oblique_corner.png"))?;
    build_berm_oblique_shadow_preview(&sprites, &recipe)
        .save_png(out_dir.join("berm_preview_oblique_shadow.png"))?;
    build_berm_mask_debug_preview(&recipe).save_png(out_dir.join("berm_mask_debug.png"))?;
    build_trench_mask_debug_preview(&recipe).save_png(out_dir.join("trench_mask_debug.png"))?;
    build_trench_autotile_sheet(&sprites, &recipe)
        .save_png(out_dir.join("trench_autotile_sheet.png"))?;
    build_trench_preview_single_masks(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_single_masks.png"))?;
    build_trench_preview_sparse(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_sparse.png"))?;
    build_trench_preview_dense(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_dense.png"))?;
    build_trench_preview_dense_clean(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_dense_clean.png"))?;
    build_trench_preview_dead_ends(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_dead_ends.png"))?;
    build_trench_preview_corners(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_corners.png"))?;
    build_trench_preview_loop(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_loop.png"))?;
    build_trench_preview_junctions(&sprites, &recipe)
        .save_png(out_dir.join("trench_preview_junctions.png"))?;
    build_trench_neighbor_seam_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_neighbor_seam_heatmap.png"))?;
    build_trench_lip_continuity_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_lip_continuity_heatmap.png"))?;
    build_trench_floor_continuity_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_floor_continuity_heatmap.png"))?;
    build_trench_neighbor_seam_edge_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_neighbor_seam_heatmap_edges.png"))?;
    build_trench_lip_continuity_edge_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_lip_continuity_heatmap_edges.png"))?;
    build_trench_floor_continuity_edge_heatmap(&sprites, &recipe)
        .save_png(out_dir.join("trench_floor_continuity_heatmap_edges.png"))?;
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
        out_dir.join("override_report.json"),
        serde_json::to_string_pretty(&override_report)?,
    )?;
    fs::write(
        out_dir.join("trench_neighbor_pairs.json"),
        serde_json::to_string_pretty(&validation.trench.worst_trench_neighbor_pairs)?,
    )?;
    fs::write(
        out_dir.join("validation.json"),
        serde_json::to_string_pretty(&validation)?,
    )?;

    Ok(TerrainSpriteExportSummary {
        out_dir: out_dir.to_string_lossy().to_string(),
        sprite_count: sprites.len(),
        validation_issue_count: validation.issues.len() + override_report.issue_count(),
        overridden_count: override_report.overridden_count,
        override_issue_count: override_report.issue_count(),
    })
}

pub fn promote_terrain_sprite_overrides(
    profile_path: impl AsRef<Path>,
) -> Result<TerrainSpriteExportSummary> {
    let report = promote_generated_sprites_to_overrides(profile_path.as_ref())?;
    Ok(TerrainSpriteExportSummary {
        out_dir: report.override_dir.clone().unwrap_or_default(),
        sprite_count: report.generated_count,
        validation_issue_count: report.issue_count(),
        overridden_count: report.overridden_count,
        override_issue_count: report.issue_count(),
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
        TerrainSpriteKind::TrenchMask00
        | TerrainSpriteKind::TrenchMask01
        | TerrainSpriteKind::TrenchMask02
        | TerrainSpriteKind::TrenchMask03
        | TerrainSpriteKind::TrenchMask04
        | TerrainSpriteKind::TrenchMask05
        | TerrainSpriteKind::TrenchMask06
        | TerrainSpriteKind::TrenchMask07
        | TerrainSpriteKind::TrenchMask08
        | TerrainSpriteKind::TrenchMask09
        | TerrainSpriteKind::TrenchMask10
        | TerrainSpriteKind::TrenchMask11
        | TerrainSpriteKind::TrenchMask12
        | TerrainSpriteKind::TrenchMask13
        | TerrainSpriteKind::TrenchMask14
        | TerrainSpriteKind::TrenchMask15 => (
            TerrainArtPieceKind::TrenchFloor,
            Some(GroundMaterial::TrenchFloor),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "trench", "trench-mask"],
        ),
        TerrainSpriteKind::TrenchFloorTop => (
            TerrainArtPieceKind::TrenchFloor,
            Some(GroundMaterial::TrenchFloor),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "trench", "floor"],
        ),
        TerrainSpriteKind::TrenchWallFront => (
            TerrainArtPieceKind::TrenchWallFront,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "trench", "front-face"],
        ),
        TerrainSpriteKind::TrenchLipFront | TerrainSpriteKind::TrenchLipBack => (
            TerrainArtPieceKind::TrenchLip,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "trench", "lip"],
        ),
        TerrainSpriteKind::TrenchEndCapLeft => (
            TerrainArtPieceKind::TrenchEndCapLeft,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "trench", "end-cap"],
        ),
        TerrainSpriteKind::TrenchEndCapRight => (
            TerrainArtPieceKind::TrenchEndCapRight,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "trench", "end-cap"],
        ),
        TerrainSpriteKind::TrenchCornerInner | TerrainSpriteKind::TrenchCornerOuter => (
            TerrainArtPieceKind::CornerCap,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "trench", "corner"],
        ),
        TerrainSpriteKind::TrenchContactShadow => (
            TerrainArtPieceKind::SoftShadow,
            None,
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "trench", "contact-shadow"],
        ),
        TerrainSpriteKind::TrenchSpoilPile => (
            TerrainArtPieceKind::TrenchSpoilPile,
            Some(GroundMaterial::TrenchWall),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "trench", "spoil"],
        ),
        TerrainSpriteKind::BermTop => (
            TerrainArtPieceKind::BermTop,
            Some(GroundMaterial::BermTop),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "berm", "top"],
        ),
        TerrainSpriteKind::BermFaceFront => (
            TerrainArtPieceKind::BermFaceFront,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "berm", "front-face"],
        ),
        TerrainSpriteKind::BermLipFront | TerrainSpriteKind::BermLipBack => (
            TerrainArtPieceKind::BrokenBermEdge,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "berm", "lip"],
        ),
        TerrainSpriteKind::BermEndCapLeft => (
            TerrainArtPieceKind::BermCornerLeft,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "berm", "end-cap"],
        ),
        TerrainSpriteKind::BermEndCapRight => (
            TerrainArtPieceKind::BermCornerRight,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "berm", "end-cap"],
        ),
        TerrainSpriteKind::BermCornerInner | TerrainSpriteKind::BermCornerOuter => (
            TerrainArtPieceKind::CornerCap,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "berm", "corner"],
        ),
        TerrainSpriteKind::BermContactShadow => (
            TerrainArtPieceKind::SoftShadow,
            None,
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "berm", "contact-shadow"],
        ),
        TerrainSpriteKind::BermSpoilPile => (
            TerrainArtPieceKind::BrokenBermEdge,
            Some(GroundMaterial::BermFace),
            TerrainArtRepeatMode::Stamp,
            vec!["spritegen", "cozy", "berm", "spoil"],
        ),
        TerrainSpriteKind::BermGrassFringe => (
            TerrainArtPieceKind::GrassFloorEdge,
            Some(GroundMaterial::Grass),
            TerrainArtRepeatMode::StretchMiddle,
            vec!["spritegen", "cozy", "berm", "grass-fringe"],
        ),
    };
    TerrainArtPiece {
        id: sprite.id.clone(),
        kind,
        material,
        width_px: sprite.image.width,
        height_px: sprite.image.height,
        anchor_px: sprite.metadata.anchor_px,
        footprint_cells: sprite.metadata.footprint_cells,
        repeat_mode,
        orientation: orientation_for_sprite(sprite.kind),
        z_bias: sprite.metadata.z_bias,
        opacity: 1.0,
        occlusion: if sprite.metadata.occludes {
            TerrainArtOcclusion::Soft
        } else {
            TerrainArtOcclusion::None
        },
        tags: tags
            .into_iter()
            .map(str::to_string)
            .chain(std::iter::once(format!(
                "role:{}",
                sprite.metadata.role.id()
            )))
            .collect(),
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
