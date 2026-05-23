use std::fs;
use std::path::Path;

use anyhow::Result;
use ron::ser::PrettyConfig;
use serde::Serialize;

use crate::feature::TerrainFeatureMap;
use crate::mask::{
    build_height_mask_atlas, build_normal_map_atlas, build_occlusion_mask_atlas,
    build_shadow_mask_atlas,
};
use crate::palette::{palette_to_file, Palette};
use crate::preview::{render_terrain_preview, PreviewMode, PreviewOptions};
use crate::recipe::ViewOrientation;
use crate::target_style::TerrainStampResolver;
use crate::terrain::TerrainMap;
use crate::terrain_artkit::{TerrainArtKit, TerrainArtKitValidation};
use crate::tileset::{TileMetadata, Tileset};
use crate::validation::{build_seam_test_sheet, validate_tileset, ValidationReport};
use crate::visual_scene::VisualScene;

#[derive(Debug, Serialize)]
pub struct ExportedTile {
    pub metadata: TileMetadata,
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct TilesetExportMetadata {
    pub recipe_id: String,
    pub palette_id: String,
    pub tile_size: u32,
    pub projection_kind: String,
    pub faux_cell_width_px: u32,
    pub faux_cell_height_px: u32,
    pub faux_height_step_px: u32,
    pub faux_side_face_width_px: u32,
    pub angled_tile_screen_width_px: u32,
    pub angled_tile_screen_height_px: u32,
    pub angled_height_step_px: u32,
    pub default_orientation: ViewOrientation,
    pub atlas_path: String,
    pub height_mask_path: String,
    pub normal_map_path: String,
    pub shadow_mask_path: String,
    pub occlusion_mask_path: String,
    pub contact_sheet_path: String,
    pub seam_validation_path: String,
    pub terrain_preview_path: String,
    pub terrain_preview_2_5d_path: String,
    pub terrain_preview_cutaway_path: String,
    pub terrain_preview_faux_path: String,
    pub terrain_preview_faux_cutaway_path: String,
    pub terrain_preview_faux_debug_path: String,
    pub terrain_preview_faux_art_path: String,
    pub terrain_preview_faux_features_path: String,
    pub terrain_preview_faux_orientation_paths: Vec<String>,
    pub terrain_preview_visual_target_path: String,
    pub terrain_preview_visual_target_no_overlay_path: String,
    pub terrain_preview_visual_target_debug_path: String,
    pub terrain_forms_path: String,
    pub terrain_stamps_path: String,
    pub terrain_stamp_count: usize,
    pub terrain_artkit_atlas_path: String,
    pub terrain_artkit_manifest_path: String,
    pub terrain_artkit_validation_path: String,
    pub terrain_artkit_piece_count: usize,
    pub terrain_artkit_validation: TerrainArtKitValidation,
    pub visual_target_form_count: usize,
    pub visual_target_summary: String,
    pub art_preview_structural_edge_count: usize,
    pub art_preview_material_edge_count: usize,
    pub terrain_preview_angled_path: String,
    pub terrain_preview_angled_cutaway_path: String,
    pub terrain_preview_angled_orientation_paths: Vec<String>,
    pub columns: u32,
    pub padding: u32,
    pub tiles: Vec<ExportedTile>,
    pub validation: ValidationReport,
}

pub fn export_tileset_bundle(
    tileset: &Tileset,
    terrain: &TerrainMap,
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    export_tileset_bundle_with_palette(tileset, &tileset.palette, terrain, out_dir)
}

pub fn export_tileset_bundle_with_palette(
    tileset: &Tileset,
    palette: &Palette,
    terrain: &TerrainMap,
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let columns = tileset.recipe.variants_per_material.max(1);
    let padding = 2;
    let atlas = tileset.build_atlas(columns, padding);
    atlas.save_png(out_dir.join("terrain_atlas.png"))?;

    let height_mask = build_height_mask_atlas(tileset, columns, padding);
    height_mask.save_png(out_dir.join("terrain_height_mask.png"))?;

    let normal_map = build_normal_map_atlas(tileset, columns, padding);
    normal_map.save_png(out_dir.join("terrain_normal.png"))?;

    let shadow_mask = build_shadow_mask_atlas(tileset, columns, padding);
    shadow_mask.save_png(out_dir.join("terrain_shadow_mask.png"))?;

    let occlusion_mask = build_occlusion_mask_atlas(tileset, columns, padding);
    occlusion_mask.save_png(out_dir.join("terrain_occlusion_mask.png"))?;

    let contact_sheet = tileset.build_contact_sheet(columns, padding);
    contact_sheet.save_png(out_dir.join("contact_sheet.png"))?;

    let seam_sheet = build_seam_test_sheet(tileset);
    seam_sheet.save_png(out_dir.join("seam_validation.png"))?;

    let preview_options = PreviewOptions {
        show_grid: false,
        los_source: terrain.objective,
        los_range: 18,
        height_step_px: tileset.recipe.projection.faux_height_step_px,
        fade_raised_faces: false,
        enable_local_cutaway: true,
        inspect_cell: None,
        show_projected_route: true,
        show_scene_markers: true,
        show_structure_lips: true,
        show_feature_overlay: false,
        view_orientation: tileset.recipe.projection.default_orientation,
    };

    let preview = render_terrain_preview(terrain, tileset, PreviewMode::Material, &preview_options);
    preview.save_png(out_dir.join("terrain_preview.png"))?;

    let preview_2_5d = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::ErectedTerrain,
        &preview_options,
    );
    preview_2_5d.save_png(out_dir.join("terrain_preview_2_5d.png"))?;

    let visual_target_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::PerspectiveSpriteScene,
        &preview_options,
    );
    visual_target_preview.save_png(out_dir.join("terrain_preview_visual_target.png"))?;

    let mut visual_no_overlay_options = preview_options.clone();
    visual_no_overlay_options.show_projected_route = false;
    visual_no_overlay_options.show_scene_markers = false;
    visual_no_overlay_options.inspect_cell = None;
    visual_no_overlay_options.show_feature_overlay = false;
    visual_no_overlay_options.show_grid = false;
    let visual_target_no_overlay = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::PerspectiveSpriteScene,
        &visual_no_overlay_options,
    );
    visual_target_no_overlay
        .save_png(out_dir.join("terrain_preview_visual_target_no_overlay.png"))?;

    let mut visual_debug_options = preview_options.clone();
    visual_debug_options.show_feature_overlay = true;
    let visual_target_debug = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::PerspectiveSpriteScene,
        &visual_debug_options,
    );
    visual_target_debug.save_png(out_dir.join("terrain_preview_visual_target_debug.png"))?;

    let visual_scene = VisualScene::from_terrain(terrain);
    let visual_scene_json = serde_json::to_string_pretty(&visual_scene)?;
    fs::write(out_dir.join("terrain_forms.json"), visual_scene_json)?;
    let target_stamps = TerrainStampResolver::resolve(terrain);
    let target_stamps_json = serde_json::to_string_pretty(&target_stamps)?;
    fs::write(out_dir.join("terrain_stamps.json"), target_stamps_json)?;

    let artkit = TerrainArtKit::load_default_or_generate(tileset);
    let artkit_atlas = artkit.build_atlas(padding);
    artkit_atlas.save_png(out_dir.join("terrain_artkit_atlas.png"))?;
    let artkit_manifest = artkit.manifest("terrain_artkit_atlas.png", padding);
    let artkit_manifest_json = serde_json::to_string_pretty(&artkit_manifest)?;
    fs::write(
        out_dir.join("terrain_artkit_manifest.json"),
        artkit_manifest_json,
    )?;
    let artkit_validation = artkit.validate();
    let artkit_validation_json = serde_json::to_string_pretty(&artkit_validation)?;
    fs::write(
        out_dir.join("terrain_artkit_validation.json"),
        artkit_validation_json,
    )?;

    let faux_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::FauxPerspectiveTerrain,
        &preview_options,
    );
    faux_preview.save_png(out_dir.join("terrain_preview_faux.png"))?;

    for orientation in ViewOrientation::ALL {
        let mut orientation_options = preview_options.clone();
        orientation_options.view_orientation = orientation;
        let preview = render_terrain_preview(
            terrain,
            tileset,
            PreviewMode::FauxPerspectiveTerrain,
            &orientation_options,
        );
        preview.save_png(out_dir.join(format!("terrain_preview_faux_{}.png", orientation.id())))?;
    }

    let angled_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::AngledTerrain,
        &preview_options,
    );
    angled_preview.save_png(out_dir.join("terrain_preview_angled.png"))?;

    for orientation in ViewOrientation::ALL {
        let mut orientation_options = preview_options.clone();
        orientation_options.view_orientation = orientation;
        let preview = render_terrain_preview(
            terrain,
            tileset,
            PreviewMode::AngledTerrain,
            &orientation_options,
        );
        preview
            .save_png(out_dir.join(format!("terrain_preview_angled_{}.png", orientation.id())))?;
    }

    let mut cutaway_options = preview_options.clone();
    cutaway_options.inspect_cell = Some(terrain.objective);
    cutaway_options.fade_raised_faces = false;
    cutaway_options.enable_local_cutaway = true;
    let cutaway_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::ErectedTerrain,
        &cutaway_options,
    );
    cutaway_preview.save_png(out_dir.join("terrain_preview_cutaway.png"))?;

    let faux_cutaway_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::FauxPerspectiveTerrain,
        &cutaway_options,
    );
    faux_cutaway_preview.save_png(out_dir.join("terrain_preview_faux_cutaway.png"))?;

    let mut faux_debug_options = preview_options.clone();
    faux_debug_options.show_grid = true;
    faux_debug_options.show_feature_overlay = true;
    let faux_debug_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::FauxPerspectiveTerrain,
        &faux_debug_options,
    );
    faux_debug_preview.save_png(out_dir.join("terrain_preview_faux_debug.png"))?;

    let art_terrain = TerrainMap::art_preview(terrain.width, terrain.height, tileset.recipe.seed);
    let faux_art_preview = render_terrain_preview(
        &art_terrain,
        tileset,
        PreviewMode::FauxPerspectiveTerrain,
        &preview_options,
    );
    faux_art_preview.save_png(out_dir.join("terrain_preview_faux_art.png"))?;

    let mut faux_features_options = preview_options.clone();
    faux_features_options.show_feature_overlay = true;
    let faux_features_preview = render_terrain_preview(
        &art_terrain,
        tileset,
        PreviewMode::FauxPerspectiveTerrain,
        &faux_features_options,
    );
    faux_features_preview.save_png(out_dir.join("terrain_preview_faux_features.png"))?;

    let angled_cutaway_preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::AngledTerrain,
        &cutaway_options,
    );
    angled_cutaway_preview.save_png(out_dir.join("terrain_preview_angled_cutaway.png"))?;

    let validation = validate_tileset(tileset);
    let metadata = export_metadata(tileset, columns, padding, validation.clone());
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(out_dir.join("tileset_metadata.json"), json)?;

    let validation_json = serde_json::to_string_pretty(&validation)?;
    fs::write(out_dir.join("validation_report.json"), validation_json)?;

    let recipe = ron::ser::to_string_pretty(&tileset.recipe, PrettyConfig::new())?;
    fs::write(out_dir.join("recipe.ron"), recipe)?;

    let palette_file = palette_to_file(palette);
    let palette_text = ron::ser::to_string_pretty(&palette_file, PrettyConfig::new())?;
    fs::write(out_dir.join("palette.ron"), palette_text)?;

    let terrain_json = serde_json::to_string_pretty(terrain)?;
    fs::write(out_dir.join("terrain_demo.json"), terrain_json)?;

    Ok(())
}

pub fn export_metadata(
    tileset: &Tileset,
    columns: u32,
    padding: u32,
    validation: ValidationReport,
) -> TilesetExportMetadata {
    let tile_size = tileset.recipe.tile_size;
    let mut tiles = Vec::with_capacity(tileset.tiles.len());
    for (i, tile) in tileset.tiles.iter().enumerate() {
        let col = i as u32 % columns;
        let row = i as u32 / columns;
        tiles.push(ExportedTile {
            metadata: tile.meta.clone(),
            atlas_x: padding + col * (tile_size + padding),
            atlas_y: padding + row * (tile_size + padding),
            width: tile_size,
            height: tile_size,
        });
    }
    TilesetExportMetadata {
        recipe_id: tileset.recipe.id.clone(),
        palette_id: tileset.palette.id.clone(),
        tile_size,
        projection_kind: tileset.recipe.projection.kind.label().to_string(),
        faux_cell_width_px: tileset.recipe.projection.faux_cell_width_px,
        faux_cell_height_px: tileset.recipe.projection.faux_cell_height_px,
        faux_height_step_px: tileset.recipe.projection.faux_height_step_px,
        faux_side_face_width_px: tileset.recipe.projection.faux_side_face_width_px,
        angled_tile_screen_width_px: tileset.recipe.projection.tile_screen_width_px,
        angled_tile_screen_height_px: tileset.recipe.projection.tile_screen_height_px,
        angled_height_step_px: tileset.recipe.projection.height_step_px,
        default_orientation: tileset.recipe.projection.default_orientation,
        atlas_path: "terrain_atlas.png".to_string(),
        height_mask_path: "terrain_height_mask.png".to_string(),
        normal_map_path: "terrain_normal.png".to_string(),
        shadow_mask_path: "terrain_shadow_mask.png".to_string(),
        occlusion_mask_path: "terrain_occlusion_mask.png".to_string(),
        contact_sheet_path: "contact_sheet.png".to_string(),
        seam_validation_path: "seam_validation.png".to_string(),
        terrain_preview_path: "terrain_preview.png".to_string(),
        terrain_preview_2_5d_path: "terrain_preview_2_5d.png".to_string(),
        terrain_preview_cutaway_path: "terrain_preview_cutaway.png".to_string(),
        terrain_preview_faux_path: "terrain_preview_faux.png".to_string(),
        terrain_preview_faux_cutaway_path: "terrain_preview_faux_cutaway.png".to_string(),
        terrain_preview_faux_debug_path: "terrain_preview_faux_debug.png".to_string(),
        terrain_preview_faux_art_path: "terrain_preview_faux_art.png".to_string(),
        terrain_preview_faux_features_path: "terrain_preview_faux_features.png".to_string(),
        terrain_preview_faux_orientation_paths: ViewOrientation::ALL
            .iter()
            .map(|orientation| format!("terrain_preview_faux_{}.png", orientation.id()))
            .collect(),
        terrain_preview_visual_target_path: "terrain_preview_visual_target.png".to_string(),
        terrain_preview_visual_target_no_overlay_path:
            "terrain_preview_visual_target_no_overlay.png".to_string(),
        terrain_preview_visual_target_debug_path: "terrain_preview_visual_target_debug.png"
            .to_string(),
        terrain_forms_path: "terrain_forms.json".to_string(),
        terrain_stamps_path: "terrain_stamps.json".to_string(),
        terrain_stamp_count: {
            let visual = TerrainMap::visual_target(14, 9, tileset.recipe.seed);
            TerrainStampResolver::resolve(&visual).len()
        },
        terrain_artkit_atlas_path: "terrain_artkit_atlas.png".to_string(),
        terrain_artkit_manifest_path: "terrain_artkit_manifest.json".to_string(),
        terrain_artkit_validation_path: "terrain_artkit_validation.json".to_string(),
        terrain_artkit_piece_count: TerrainArtKit::load_default_or_generate(tileset)
            .pieces
            .len(),
        terrain_artkit_validation: TerrainArtKit::load_default_or_generate(tileset).validate(),
        visual_target_form_count: {
            let visual = TerrainMap::visual_target(14, 9, tileset.recipe.seed);
            VisualScene::from_terrain(&visual).forms.len()
        },
        visual_target_summary: {
            let visual = TerrainMap::visual_target(14, 9, tileset.recipe.seed);
            VisualScene::from_terrain(&visual).summary_line()
        },
        art_preview_structural_edge_count: {
            let art = TerrainMap::art_preview(32, 24, tileset.recipe.seed);
            TerrainFeatureMap::from_terrain(&art).structural_edge_count()
        },
        art_preview_material_edge_count: {
            let art = TerrainMap::art_preview(32, 24, tileset.recipe.seed);
            TerrainFeatureMap::from_terrain(&art).material_edge_count()
        },
        terrain_preview_angled_path: "terrain_preview_angled.png".to_string(),
        terrain_preview_angled_cutaway_path: "terrain_preview_angled_cutaway.png".to_string(),
        terrain_preview_angled_orientation_paths: ViewOrientation::ALL
            .iter()
            .map(|orientation| format!("terrain_preview_angled_{}.png", orientation.id()))
            .collect(),
        columns,
        padding,
        tiles,
        validation,
    }
}
