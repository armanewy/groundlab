use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use ron::ser::PrettyConfig;

use crate::palette::{load_palette_file, muted_field_32, save_palette_file, Palette};
use crate::recipe::TilesetRecipe;
use crate::terrain_artkit::{TerrainArtKit, DEFAULT_ARTKIT_DIR};
use crate::tileset::Tileset;
use crate::validation::{validate_tileset, ValidationReport};

pub const DEFAULT_RECIPE_PATH: &str = "recipes/dry_upland_outpost.ron";
pub const DEFAULT_PALETTE_PATH: &str = "palettes/muted_field_32.ron";

#[derive(Clone, Debug)]
pub struct WorkbenchAssetPaths {
    pub recipe_path: PathBuf,
    pub palette_path: PathBuf,
}

impl Default for WorkbenchAssetPaths {
    fn default() -> Self {
        Self {
            recipe_path: PathBuf::from(DEFAULT_RECIPE_PATH),
            palette_path: PathBuf::from(DEFAULT_PALETTE_PATH),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoadedWorkbenchAssets {
    pub recipe: TilesetRecipe,
    pub palette: Palette,
    pub tileset: Tileset,
    pub validation: ValidationReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileSnapshot {
    pub recipe_modified: Option<SystemTime>,
    pub palette_modified: Option<SystemTime>,
}

impl FileSnapshot {
    pub fn capture(paths: &WorkbenchAssetPaths) -> Self {
        Self {
            recipe_modified: file_modified(&paths.recipe_path),
            palette_modified: file_modified(&paths.palette_path),
        }
    }

    pub fn changed_since(&self, previous: &FileSnapshot) -> bool {
        self.recipe_modified != previous.recipe_modified
            || self.palette_modified != previous.palette_modified
    }
}

pub fn load_workbench_assets(paths: &WorkbenchAssetPaths) -> Result<LoadedWorkbenchAssets> {
    let mut recipe = if paths.recipe_path.exists() {
        load_recipe_file(&paths.recipe_path)?
    } else {
        TilesetRecipe::default()
    };
    recipe.sanitize();

    let palette = if paths.palette_path.exists() {
        load_palette_file(&paths.palette_path)?
    } else {
        muted_field_32()
    };

    let tileset = Tileset::generate_with_palette(&recipe, &palette);
    let validation = validate_tileset(&tileset);
    Ok(LoadedWorkbenchAssets {
        recipe,
        palette,
        tileset,
        validation,
    })
}

pub fn load_recipe_file(path: impl AsRef<Path>) -> Result<TilesetRecipe> {
    let text = fs::read_to_string(path.as_ref())?;
    let mut recipe: TilesetRecipe = ron::de::from_str(&text)?;
    recipe.sanitize();
    Ok(recipe)
}

pub fn save_recipe_file(path: impl AsRef<Path>, recipe: &TilesetRecipe) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut recipe = recipe.clone();
    recipe.sanitize();
    let text = ron::ser::to_string_pretty(&recipe, PrettyConfig::new())?;
    fs::write(path, text)?;
    Ok(())
}

pub fn ensure_default_asset_files(paths: &WorkbenchAssetPaths) -> Result<()> {
    if !paths.recipe_path.exists() {
        save_recipe_file(&paths.recipe_path, &TilesetRecipe::default())?;
    }
    if !paths.palette_path.exists() {
        save_palette_file(&paths.palette_path, &muted_field_32())?;
    }
    let loaded = load_workbench_assets(paths)?;
    TerrainArtKit::ensure_external_files(&loaded.tileset, DEFAULT_ARTKIT_DIR)?;
    Ok(())
}

pub fn file_modified(path: impl AsRef<Path>) -> Option<SystemTime> {
    fs::metadata(path.as_ref())
        .ok()
        .and_then(|meta| meta.modified().ok())
}
