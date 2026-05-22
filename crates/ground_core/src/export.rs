use std::fs;
use std::path::Path;

use anyhow::Result;
use ron::ser::PrettyConfig;
use serde::Serialize;

use crate::pixel_image::PixelImage;
use crate::preview::{render_terrain_preview, PreviewMode, PreviewOptions};
use crate::terrain::TerrainMap;
use crate::tileset::{TileMetadata, Tileset};

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
    pub tile_size: u32,
    pub atlas_path: String,
    pub contact_sheet_path: String,
    pub terrain_preview_path: String,
    pub columns: u32,
    pub padding: u32,
    pub tiles: Vec<ExportedTile>,
}

pub fn export_tileset_bundle(
    tileset: &Tileset,
    terrain: &TerrainMap,
    out_dir: impl AsRef<Path>,
) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let columns = tileset.recipe.variants_per_material.max(1);
    let padding = 2;
    let atlas = tileset.build_atlas(columns, padding);
    atlas.save_png(out_dir.join("terrain_atlas.png"))?;

    let contact_sheet = tileset.build_contact_sheet(columns, padding);
    contact_sheet.save_png(out_dir.join("contact_sheet.png"))?;

    let preview = render_terrain_preview(
        terrain,
        tileset,
        PreviewMode::Material,
        &PreviewOptions {
            show_grid: true,
            los_source: terrain.objective,
            los_range: 18,
        },
    );
    preview.save_png(out_dir.join("terrain_preview.png"))?;

    let normal_atlas = build_placeholder_normal_atlas(&atlas);
    normal_atlas.save_png(out_dir.join("terrain_normal_placeholder.png"))?;

    let metadata = export_metadata(tileset, columns, padding);
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(out_dir.join("tileset_metadata.json"), json)?;

    let recipe = ron::ser::to_string_pretty(&tileset.recipe, PrettyConfig::new())?;
    fs::write(out_dir.join("recipe.ron"), recipe)?;

    let terrain_json = serde_json::to_string_pretty(terrain)?;
    fs::write(out_dir.join("terrain_demo.json"), terrain_json)?;

    Ok(())
}

pub fn export_metadata(tileset: &Tileset, columns: u32, padding: u32) -> TilesetExportMetadata {
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
        tile_size,
        atlas_path: "terrain_atlas.png".to_string(),
        contact_sheet_path: "contact_sheet.png".to_string(),
        terrain_preview_path: "terrain_preview.png".to_string(),
        columns,
        padding,
        tiles,
    }
}

fn build_placeholder_normal_atlas(atlas: &PixelImage) -> PixelImage {
    // A neutral normal map. Milestone 2 replaces this with generated normals from height masks.
    PixelImage::new(
        atlas.width,
        atlas.height,
        crate::color::Rgba8::opaque(128, 128, 255),
    )
}
