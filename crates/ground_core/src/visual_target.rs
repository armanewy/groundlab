use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::pixel_image::PixelImage;

pub const DEFAULT_VISUAL_TARGET_DIR: &str = "assets/visual_targets/dry_upland_outpost_01";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualTargetSpec {
    pub id: String,
    pub image: String,
    pub image_size_px: (u32, u32),
    pub map_size_cells: (u32, u32),
    pub grid_origin_px: (i32, i32),
    pub cell_size_px: (u32, u32),
    pub spawn_cell: (u32, u32),
    pub objective_cell: (u32, u32),
    pub light_direction: String,
    pub notes: String,
}

#[derive(Clone, Debug)]
pub struct VisualTarget {
    pub spec: VisualTargetSpec,
    pub image: PixelImage,
}

impl VisualTarget {
    pub fn load_default() -> Result<Self> {
        Self::load(DEFAULT_VISUAL_TARGET_DIR)
    }

    pub fn load(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let manifest_path = dir.join("manifest.ron");
        let text = fs::read_to_string(&manifest_path).with_context(|| {
            format!("reading visual target manifest {}", manifest_path.display())
        })?;
        let spec: VisualTargetSpec = ron::de::from_str(&text).with_context(|| {
            format!("parsing visual target manifest {}", manifest_path.display())
        })?;
        let image_path = dir.join(&spec.image);
        let image = PixelImage::load_png(&image_path)
            .with_context(|| format!("loading visual target image {}", image_path.display()))?;

        Ok(Self { spec, image })
    }

    pub fn image_path(&self) -> PathBuf {
        Path::new(DEFAULT_VISUAL_TARGET_DIR).join(&self.spec.image)
    }

    pub fn cell_rect(&self, cell: (u32, u32)) -> Option<ImageCellRect> {
        if cell.0 >= self.spec.map_size_cells.0 || cell.1 >= self.spec.map_size_cells.1 {
            return None;
        }
        let x = self.spec.grid_origin_px.0 + cell.0 as i32 * self.spec.cell_size_px.0 as i32;
        let y = self.spec.grid_origin_px.1 + cell.1 as i32 * self.spec.cell_size_px.1 as i32;
        Some(ImageCellRect {
            x,
            y,
            width: self.spec.cell_size_px.0,
            height: self.spec.cell_size_px.1,
        })
    }

    pub fn cell_center(&self, cell: (u32, u32)) -> Option<(i32, i32)> {
        let rect = self.cell_rect(cell)?;
        Some((
            rect.x + rect.width as i32 / 2,
            rect.y + rect.height as i32 / 2,
        ))
    }

    pub fn pixel_to_cell(&self, px: u32, py: u32) -> Option<(u32, u32)> {
        let px = px as i32;
        let py = py as i32;
        for y in 0..self.spec.map_size_cells.1 {
            for x in 0..self.spec.map_size_cells.0 {
                let rect = self.cell_rect((x, y))?;
                if px >= rect.x
                    && py >= rect.y
                    && px < rect.x + rect.width as i32
                    && py < rect.y + rect.height as i32
                {
                    return Some((x, y));
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ImageCellRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
