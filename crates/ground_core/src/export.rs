use std::fs;
use std::path::Path;

use anyhow::Result;
use ron::ser::PrettyConfig;
use serde::Serialize;

use crate::mask::{
    build_height_mask_atlas, build_normal_map_atlas, build_occlusion_mask_atlas,
    build_shadow_mask_atlas,
};
use crate::palette::{palette_to_file, Palette};
use crate::preview::{render_terrain_preview, PreviewMode, PreviewOptions};
use crate::terrain::TerrainMap;
use crate::tileset::{TileMetadata, Tileset};
use crate::validation::{build_seam_test_sheet, validate_tileset, ValidationReport};

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
        height_step_px: (tileset.recipe.tile_size / 4).max(4),
        fade_raised_faces: false,
        enable_local_cutaway: true,
        inspect_cell: None,
        show_projected_route: true,
        show_structure_lips: true,
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
        columns,
        padding,
        tiles,
        validation,
    }
}
