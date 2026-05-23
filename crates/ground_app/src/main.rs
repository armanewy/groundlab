use std::path::PathBuf;

use eframe::egui;
use ground_core::{
    build_seam_test_sheet, ensure_default_asset_files, export_tileset_bundle_with_palette,
    find_path, load_workbench_assets, muted_field_32, preview_pixel_to_cell,
    render_terrain_preview, save_palette_file, save_recipe_file, validate_tileset, Brush,
    BrushKind, FileSnapshot, GroundMaterial, LightDirection, Palette, PixelImage, PreviewMode,
    PreviewOptions, ProjectionKind, TerrainMap, Tileset, TilesetRecipe, ValidationReport,
    ViewOrientation, WorkbenchAssetPaths,
};

const MAX_UI_TEXTURE_SIDE: usize = 2048;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1050.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GroundLab — custom terrain asset workbench",
        options,
        Box::new(|cc| Ok(Box::new(GroundLabApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasView {
    TerrainPreview,
    ContactSheet,
    SeamTest,
}

impl CanvasView {
    const ALL: [CanvasView; 3] = [
        CanvasView::TerrainPreview,
        CanvasView::ContactSheet,
        CanvasView::SeamTest,
    ];

    fn label(self) -> &'static str {
        match self {
            CanvasView::TerrainPreview => "Terrain preview",
            CanvasView::ContactSheet => "Contact sheet",
            CanvasView::SeamTest => "Seam test",
        }
    }
}

struct GroundLabApp {
    paths: WorkbenchAssetPaths,
    recipe_path_text: String,
    palette_path_text: String,
    file_snapshot: FileSnapshot,
    auto_reload: bool,
    recipe: TilesetRecipe,
    palette: Palette,
    tileset: Tileset,
    validation: ValidationReport,
    terrain: TerrainMap,
    preview_mode: PreviewMode,
    preview_options: PreviewOptions,
    brush: Brush,
    zoom: f32,
    canvas_view: CanvasView,
    contact_texture: Option<egui::TextureHandle>,
    seam_texture: Option<egui::TextureHandle>,
    preview_texture: Option<egui::TextureHandle>,
    dirty_assets: bool,
    dirty_preview: bool,
    last_preview_size: [usize; 2],
    status: String,
}

impl GroundLabApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let paths = WorkbenchAssetPaths::default();
        let _ = ensure_default_asset_files(&paths);
        let loaded = load_workbench_assets(&paths).unwrap_or_else(|_| {
            let recipe = TilesetRecipe::default();
            let palette = muted_field_32();
            let tileset = Tileset::generate_with_palette(&recipe, &palette);
            let validation = validate_tileset(&tileset);
            ground_core::LoadedWorkbenchAssets {
                recipe,
                palette,
                tileset,
                validation,
            }
        });
        let terrain = TerrainMap::target_derived(16, 12, loaded.recipe.seed);
        let mut app = Self {
            recipe_path_text: paths.recipe_path.to_string_lossy().to_string(),
            palette_path_text: paths.palette_path.to_string_lossy().to_string(),
            file_snapshot: FileSnapshot::capture(&paths),
            paths,
            auto_reload: true,
            preview_options: PreviewOptions {
                show_grid: false,
                los_source: terrain.objective,
                los_range: 18,
                height_step_px: loaded.recipe.projection.faux_height_step_px,
                fade_raised_faces: false,
                enable_local_cutaway: true,
                inspect_cell: None,
                show_projected_route: true,
                show_scene_markers: true,
                show_structure_lips: true,
                show_feature_overlay: false,
                view_orientation: loaded.recipe.projection.default_orientation,
            },
            recipe: loaded.recipe,
            palette: loaded.palette,
            tileset: loaded.tileset,
            validation: loaded.validation,
            terrain,
            preview_mode: PreviewMode::PerspectiveSpriteScene,
            brush: Brush::new(BrushKind::DigTrench, 1, 1),
            zoom: 0.80,
            canvas_view: CanvasView::TerrainPreview,
            contact_texture: None,
            seam_texture: None,
            preview_texture: None,
            dirty_assets: true,
            dirty_preview: true,
            last_preview_size: [1, 1],
            status: "Ready. Milestone 4.10 target-derived editable scene is active: the visual target image is the base art source and edits render as local terrain patches.".to_string(),
        };
        app.refresh_if_dirty(&cc.egui_ctx);
        app
    }

    fn active_paths_from_text(&self) -> WorkbenchAssetPaths {
        WorkbenchAssetPaths {
            recipe_path: PathBuf::from(self.recipe_path_text.trim()),
            palette_path: PathBuf::from(self.palette_path_text.trim()),
        }
    }

    fn poll_hot_reload(&mut self, ctx: &egui::Context) {
        if !self.auto_reload {
            return;
        }
        let paths = self.active_paths_from_text();
        let snapshot = FileSnapshot::capture(&paths);
        if snapshot.changed_since(&self.file_snapshot) {
            match load_workbench_assets(&paths) {
                Ok(loaded) => {
                    self.paths = paths;
                    self.file_snapshot = snapshot;
                    self.recipe = loaded.recipe;
                    self.palette = loaded.palette;
                    self.tileset = loaded.tileset;
                    self.validation = loaded.validation;
                    self.preview_options.height_step_px =
                        self.recipe.projection.faux_height_step_px;
                    self.preview_options.view_orientation =
                        self.recipe.projection.default_orientation;
                    self.dirty_assets = true;
                    self.dirty_preview = true;
                    self.status = "Hot reloaded recipe/palette files.".to_string();
                    ctx.request_repaint();
                }
                Err(err) => {
                    self.file_snapshot = snapshot;
                    self.status = format!("Hot reload failed: {err}");
                }
            }
        }
    }

    fn refresh_if_dirty(&mut self, ctx: &egui::Context) {
        self.poll_hot_reload(ctx);

        if self.dirty_assets {
            self.recipe.sanitize();
            self.tileset = Tileset::generate_with_palette(&self.recipe, &self.palette);
            self.validation = validate_tileset(&self.tileset);
            let columns = self.recipe.variants_per_material.max(1);
            let contact = self.tileset.build_contact_sheet(columns, 2);
            put_texture(ctx, &mut self.contact_texture, "contact_sheet", &contact);
            let seam = build_seam_test_sheet(&self.tileset);
            put_texture(ctx, &mut self.seam_texture, "seam_test_sheet", &seam);
            self.dirty_assets = false;
            self.dirty_preview = true;
        }

        if self.dirty_preview {
            let preview = render_terrain_preview(
                &self.terrain,
                &self.tileset,
                self.preview_mode,
                &self.preview_options,
            );
            self.last_preview_size = preview.size();
            put_texture(ctx, &mut self.preview_texture, "terrain_preview", &preview);
            self.dirty_preview = false;
        }
    }

    fn load_from_paths(&mut self, ctx: &egui::Context) {
        let paths = self.active_paths_from_text();
        match load_workbench_assets(&paths) {
            Ok(loaded) => {
                self.paths = paths;
                self.file_snapshot = FileSnapshot::capture(&self.paths);
                self.recipe = loaded.recipe;
                self.palette = loaded.palette;
                self.tileset = loaded.tileset;
                self.validation = loaded.validation;
                self.preview_options.height_step_px = self.recipe.projection.faux_height_step_px;
                self.preview_options.view_orientation = self.recipe.projection.default_orientation;
                self.dirty_assets = true;
                self.dirty_preview = true;
                self.status = "Loaded recipe and palette from disk.".to_string();
                ctx.request_repaint();
            }
            Err(err) => {
                self.status = format!("Load failed: {err}");
            }
        }
    }

    fn save_recipe(&mut self) {
        let paths = self.active_paths_from_text();
        match save_recipe_file(&paths.recipe_path, &self.recipe) {
            Ok(()) => {
                self.paths = paths;
                self.file_snapshot = FileSnapshot::capture(&self.paths);
                self.status = format!("Saved recipe to {}", self.paths.recipe_path.display());
            }
            Err(err) => self.status = format!("Save recipe failed: {err}"),
        }
    }

    fn save_palette(&mut self) {
        let paths = self.active_paths_from_text();
        match save_palette_file(&paths.palette_path, &self.palette) {
            Ok(()) => {
                self.paths = paths;
                self.file_snapshot = FileSnapshot::capture(&self.paths);
                self.status = format!("Saved palette to {}", self.paths.palette_path.display());
            }
            Err(err) => self.status = format!("Save palette failed: {err}"),
        }
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("GroundLab");
        ui.label("Internal custom terrain + pixel asset workbench");
        ui.separator();

        ui.strong("Pipeline files");
        ui.label("Recipe");
        ui.text_edit_singleline(&mut self.recipe_path_text);
        ui.label("Palette");
        ui.text_edit_singleline(&mut self.palette_path_text);
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.load_from_paths(ctx);
            }
            if ui.button("Save recipe").clicked() {
                self.save_recipe();
            }
            if ui.button("Save palette").clicked() {
                self.save_palette();
            }
        });
        ui.checkbox(&mut self.auto_reload, "Auto reload recipe/palette files");
        ui.small(format!(
            "Palette: {} · ramps: {}",
            self.palette.id,
            self.palette.ramps.len()
        ));
        ui.small(self.validation.summary_line());
        ui.separator();

        ui.strong("Asset recipe");
        let mut recipe_changed = false;

        egui::ComboBox::from_label("Tile size")
            .selected_text(format!("{} px", self.recipe.tile_size))
            .show_ui(ui, |ui| {
                for size in [16_u32, 24, 32, 48, 64, 96] {
                    recipe_changed |= ui
                        .selectable_value(&mut self.recipe.tile_size, size, format!("{size} px"))
                        .changed();
                }
            });

        recipe_changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.variants_per_material, 1..=12)
                    .text("variants/material"),
            )
            .changed();
        recipe_changed |= ui
            .add(egui::Slider::new(&mut self.recipe.detail_density, 0.0..=1.0).text("detail"))
            .changed();
        recipe_changed |= ui
            .add(egui::Slider::new(&mut self.recipe.shadow_strength, 0.0..=1.0).text("shadow"))
            .changed();
        recipe_changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.highlight_strength, 0.0..=1.0).text("highlight"),
            )
            .changed();
        recipe_changed |= ui
            .add(egui::Slider::new(&mut self.recipe.outline_strength, 0.0..=1.0).text("outline"))
            .changed();

        egui::ComboBox::from_label("Light")
            .selected_text(self.recipe.light_direction.label())
            .show_ui(ui, |ui| {
                for direction in LightDirection::ALL {
                    recipe_changed |= ui
                        .selectable_value(
                            &mut self.recipe.light_direction,
                            direction,
                            direction.label(),
                        )
                        .changed();
                }
            });

        ui.horizontal(|ui| {
            ui.label("Seed");
            recipe_changed |= ui
                .add(egui::DragValue::new(&mut self.recipe.seed).speed(1.0))
                .changed();
        });

        ui.collapsing("Milestone 2 asset pipeline", |ui| {
            recipe_changed |= ui
                .checkbox(
                    &mut self.recipe.generate_transitions,
                    "Generate material transition tiles",
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.transition_feather, 0.05..=0.45)
                        .text("transition feather"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.mask_strength, 0.0..=2.0)
                        .text("normal/mask strength"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.seam_warning_threshold, 8.0..=160.0)
                        .text("seam warn threshold"),
                )
                .changed();
            ui.label(format!(
                "Tiles: {} surface, {} transition, {} face",
                self.tileset.surface_tile_count(),
                self.tileset.transition_tile_count(),
                self.tileset.structure_face_tile_count()
            ));
        });

        ui.collapsing("Milestone 3 terrain extrusion", |ui| {
            recipe_changed |= ui
                .checkbox(
                    &mut self.recipe.generate_structure_faces,
                    "Generate structure-face tiles",
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.face_shadow_strength, 0.0..=1.0)
                        .text("face shadow"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.face_lip_strength, 0.0..=1.0)
                        .text("lip highlight"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.face_detail_density, 0.0..=1.0)
                        .text("face detail"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.cutaway_alpha, 0.15..=1.0)
                        .text("cutaway alpha"),
                )
                .changed();
            recipe_changed |= ui
                .add(
                    egui::Slider::new(&mut self.recipe.cutaway_radius_px, 16..=384)
                        .text("cutaway radius px"),
                )
                .changed();
            ui.small("Structure faces are generated art assets, not flat debug rectangles.");
        });

        ui.collapsing("Milestone 4 angled projection pivot", |ui| {
            egui::ComboBox::from_label("Projection kind")
                .selected_text(self.recipe.projection.kind.label())
                .show_ui(ui, |ui| {
                    recipe_changed |= ui
                        .selectable_value(
                            &mut self.recipe.projection.kind,
                            ProjectionKind::FauxPerspective2D,
                            ProjectionKind::FauxPerspective2D.label(),
                        )
                        .changed();
                    recipe_changed |= ui
                        .selectable_value(&mut self.recipe.projection.kind, ProjectionKind::Dimetric, ProjectionKind::Dimetric.label())
                        .changed();
                    recipe_changed |= ui
                        .selectable_value(&mut self.recipe.projection.kind, ProjectionKind::SquareTopDown, ProjectionKind::SquareTopDown.label())
                        .changed();
                });
            ui.label("Faux-perspective main view");
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.faux_cell_width_px, 32..=160).text("faux cell width px"))
                .changed();
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.faux_cell_height_px, 32..=160).text("faux cell height px"))
                .changed();
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.faux_height_step_px, 4..=64).text("faux height step px"))
                .changed();
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.faux_side_face_width_px, 2..=48).text("side face strip px"))
                .changed();

            ui.separator();
            ui.label("Angled/dimetric alternate view");
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.tile_screen_width_px, 32..=192).text("angled tile width px"))
                .changed();
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.tile_screen_height_px, 24..=128).text("angled tile height px"))
                .changed();
            recipe_changed |= ui
                .add(egui::Slider::new(&mut self.recipe.projection.height_step_px, 4..=96).text("angled height step px"))
                .changed();
            if ui.button("Use visual-target sprite defaults").clicked() {
                self.recipe.projection.kind = ProjectionKind::FauxPerspective2D;
                self.recipe.projection.faux_cell_width_px = 96;
                self.recipe.projection.faux_cell_height_px = 80;
                self.recipe.projection.faux_height_step_px = 32;
                self.recipe.projection.faux_side_face_width_px = 20;
                self.preview_options.height_step_px = 32;
                self.preview_mode = PreviewMode::PerspectiveSpriteScene;
                recipe_changed = true;
                self.dirty_preview = true;
            }
            egui::ComboBox::from_label("Default orientation")
                .selected_text(self.recipe.projection.default_orientation.label())
                .show_ui(ui, |ui| {
                    for orientation in ViewOrientation::ALL {
                        recipe_changed |= ui
                            .selectable_value(
                                &mut self.recipe.projection.default_orientation,
                                orientation,
                                orientation.label(),
                            )
                            .changed();
                    }
                });
            recipe_changed |= ui
                .checkbox(&mut self.recipe.projection.supports_four_way_rotation, "Support 90° rotation")
                .changed();
            ui.small("Faux-perspective keeps the map top-down and rectangular while sprite faces/lips/shadows imply physical elevation. Angled dimetric remains available as an experiment.");
        });

        if recipe_changed {
            self.dirty_assets = true;
            self.preview_options.height_step_px = match self.preview_mode {
                PreviewMode::AngledTerrain => self.recipe.projection.height_step_px,
                _ => self.recipe.projection.faux_height_step_px,
            };
            self.status = "Recipe changed; regenerated tiles next frame.".to_string();
            ctx.request_repaint();
        }

        ui.separator();
        ui.strong("Canvas");
        egui::ComboBox::from_label("View")
            .selected_text(self.canvas_view.label())
            .show_ui(ui, |ui| {
                for view in CanvasView::ALL {
                    ui.selectable_value(&mut self.canvas_view, view, view.label());
                }
            });
        egui::ComboBox::from_label("Overlay")
            .selected_text(self.preview_mode.label())
            .show_ui(ui, |ui| {
                for mode in PreviewMode::ALL {
                    if ui
                        .selectable_value(&mut self.preview_mode, mode, mode.label())
                        .changed()
                    {
                        self.preview_options.height_step_px = match self.preview_mode {
                            PreviewMode::AngledTerrain => self.recipe.projection.height_step_px,
                            _ => self.recipe.projection.faux_height_step_px,
                        };
                        self.dirty_preview = true;
                    }
                }
            });
        ui.horizontal(|ui| {
            if ui.button("⟲ Rotate view").clicked() {
                self.preview_options.view_orientation =
                    self.preview_options.view_orientation.rotate_ccw();
                self.preview_mode = PreviewMode::PerspectiveSpriteScene;
                self.dirty_preview = true;
            }
            if ui.button("Rotate view ⟳").clicked() {
                self.preview_options.view_orientation =
                    self.preview_options.view_orientation.rotate_cw();
                self.preview_mode = PreviewMode::PerspectiveSpriteScene;
                self.dirty_preview = true;
            }
        });
        egui::ComboBox::from_label("View orientation")
            .selected_text(self.preview_options.view_orientation.label())
            .show_ui(ui, |ui| {
                for orientation in ViewOrientation::ALL {
                    if ui
                        .selectable_value(
                            &mut self.preview_options.view_orientation,
                            orientation,
                            orientation.label(),
                        )
                        .changed()
                    {
                        self.preview_mode = PreviewMode::PerspectiveSpriteScene;
                        self.dirty_preview = true;
                    }
                }
            });
        if ui
            .checkbox(&mut self.preview_options.show_grid, "Grid")
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .add(egui::Slider::new(&mut self.preview_options.los_range, 4..=36).text("LOS range"))
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .add(
                egui::Slider::new(&mut self.preview_options.height_step_px, 4..=96)
                    .text("view height step px"),
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.fade_raised_faces,
                "Global face fade",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.enable_local_cutaway,
                "Hover cutaway lens",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.show_projected_route,
                "Projected route on terrain",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.show_structure_lips,
                "Generated cut lips",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.show_feature_overlay,
                "Feature-mask overlay",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        ui.add(egui::Slider::new(&mut self.zoom, 0.4..=3.0).text("zoom"));

        ui.separator();
        ui.strong("Brush");
        let mut brush_changed = false;
        brush_changed |= ui
            .radio_value(&mut self.brush.kind, BrushKind::DigTrench, "Dig trench")
            .changed();
        brush_changed |= ui
            .radio_value(&mut self.brush.kind, BrushKind::RaiseBerm, "Raise berm")
            .changed();
        brush_changed |= ui
            .radio_value(&mut self.brush.kind, BrushKind::Ditch, "Ditch / depression")
            .changed();
        brush_changed |= ui
            .radio_value(&mut self.brush.kind, BrushKind::Flatten, "Flatten")
            .changed();
        ui.collapsing("Paint ground", |ui| {
            for material in [
                GroundMaterial::Grass,
                GroundMaterial::Dirt,
                GroundMaterial::Mud,
                GroundMaterial::Rock,
            ] {
                brush_changed |= ui
                    .radio_value(
                        &mut self.brush.kind,
                        BrushKind::Paint(material),
                        material.display_name(),
                    )
                    .changed();
            }
        });
        brush_changed |= ui
            .add(egui::Slider::new(&mut self.brush.radius, 1..=6).text("radius"))
            .changed();
        brush_changed |= ui
            .add(egui::Slider::new(&mut self.brush.intensity, 1..=4).text("intensity"))
            .changed();
        if brush_changed {
            self.status = format!("Selected brush: {}", self.brush.kind.label());
        }

        ui.separator();
        ui.strong("Actions");
        if ui.button("Reset visual-target scene").clicked() {
            self.terrain = TerrainMap::target_derived(16, 12, self.recipe.seed);
            self.preview_options.los_source = self.terrain.objective;
            self.preview_mode = PreviewMode::PerspectiveSpriteScene;
            self.dirty_preview = true;
            self.status = "Visual-target scene reset.".to_string();
        }
        if ui.button("Reset art-preview terrain").clicked() {
            self.terrain = TerrainMap::art_preview(32, 24, self.recipe.seed);
            self.preview_options.los_source = self.terrain.objective;
            self.dirty_preview = true;
            self.status = "Art-preview terrain reset.".to_string();
        }
        if ui.button("Reset stress-test terrain").clicked() {
            self.terrain = TerrainMap::stress_test(32, 24, self.recipe.seed);
            self.preview_options.los_source = self.terrain.objective;
            self.dirty_preview = true;
            self.status = "Stress-test terrain reset.".to_string();
        }
        if ui.button("Set LOS source to objective").clicked() {
            self.preview_options.los_source = self.terrain.objective;
            self.preview_mode = PreviewMode::LineOfSight;
            self.dirty_preview = true;
        }
        if ui.button("Export bundle").clicked() {
            self.refresh_if_dirty(ctx);
            match export_tileset_bundle_with_palette(
                &self.tileset,
                &self.palette,
                &self.terrain,
                "exports/milestone_04_10",
            ) {
                Ok(()) => self.status = "Exported to exports/milestone_04_10".to_string(),
                Err(err) => self.status = format!("Export failed: {err}"),
            }
        }

        ui.separator();
        let path = find_path(&self.terrain, self.terrain.spawn, self.terrain.objective);
        ui.label(format!("Route cost: {:.1}", path.total_cost));
        ui.label(format!("Route reached objective: {}", path.reached_goal));
        ui.collapsing("Validation issues", |ui| {
            if self.validation.issues.is_empty() {
                ui.label("No validation issues reported.");
            } else {
                for issue in self.validation.issues.iter().take(16) {
                    ui.label(format!(
                        "{} · {} · {}{}",
                        issue.severity.label(),
                        issue.category,
                        issue.message,
                        issue
                            .metric
                            .map(|m| format!(" ({m:.1})"))
                            .unwrap_or_default()
                    ));
                }
                if self.validation.issues.len() > 16 {
                    ui.label(format!("… plus {} more", self.validation.issues.len() - 16));
                }
            }
        });
        ui.separator();
        ui.label(&self.status);
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.canvas_view {
            CanvasView::TerrainPreview => self.show_terrain_canvas(ui, ctx),
            CanvasView::ContactSheet => {
                let texture = self.contact_texture.clone();
                show_texture_only(
                    ui,
                    texture.as_ref(),
                    self.zoom,
                    "Contact sheet is not ready yet.",
                );
            }
            CanvasView::SeamTest => {
                let texture = self.seam_texture.clone();
                show_texture_only(
                    ui,
                    texture.as_ref(),
                    self.zoom,
                    "Seam test is not ready yet.",
                );
            }
        }
    }

    fn show_terrain_canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let texture = self.preview_texture.clone();
        let Some(texture) = texture else {
            ui.label("Preview texture is not ready yet.");
            return;
        };

        let display_size = texture.size_vec2() * self.zoom;
        let sized = egui::load::SizedTexture::new(texture.id(), display_size);
        let response = ui.add(
            egui::Image::from_texture(sized)
                .texture_options(egui::TextureOptions::NEAREST)
                .sense(egui::Sense::click_and_drag()),
        );

        let pointer_cell = if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            ctx.input(|i| i.pointer.hover_pos()).and_then(|pos| {
                let local = pos - response.rect.min;
                let texture_size = texture.size();
                let source_w = self.last_preview_size[0].max(1) as f32;
                let source_h = self.last_preview_size[1].max(1) as f32;
                let upload_w = texture_size[0].max(1) as f32;
                let upload_h = texture_size[1].max(1) as f32;
                let upload_px = (local.x / self.zoom).floor().max(0.0);
                let upload_py = (local.y / self.zoom).floor().max(0.0);
                let px = (upload_px * source_w / upload_w).floor().max(0.0) as u32;
                let py = (upload_py * source_h / upload_h).floor().max(0.0) as u32;
                preview_pixel_to_cell(
                    &self.terrain,
                    &self.tileset,
                    self.preview_mode,
                    &self.preview_options,
                    px,
                    py,
                )
            })
        } else {
            None
        };

        if self.preview_options.inspect_cell != pointer_cell {
            self.preview_options.inspect_cell = pointer_cell;
            if matches!(
                self.preview_mode,
                PreviewMode::PerspectiveSpriteScene
                    | PreviewMode::FauxPerspectiveTerrain
                    | PreviewMode::AngledTerrain
                    | PreviewMode::ErectedTerrain
            ) && self.preview_options.enable_local_cutaway
            {
                self.dirty_preview = true;
                ctx.request_repaint();
            }
        }

        if response.clicked() || response.dragged() {
            if let Some((x, y)) = pointer_cell {
                let primary = ctx.input(|i| i.pointer.primary_down());
                let secondary = ctx.input(|i| i.pointer.secondary_down());
                if secondary {
                    self.preview_options.los_source = (x, y);
                    self.preview_mode = PreviewMode::LineOfSight;
                    self.status = format!("LOS source set to ({x}, {y}).");
                } else if primary {
                    self.terrain.apply_brush(x, y, self.brush);
                    self.status = format!("Applied {} at ({x}, {y}).", self.brush.kind.label());
                }
                self.dirty_preview = true;
                ctx.request_repaint();
            }
        }

        ui.label("Left-drag paints. Right-click sets LOS source. Hovering in sprite/faux/angled views drives a local cutaway lens. Blue = spawn, yellow = objective.");
    }
}

impl eframe::App for GroundLabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.refresh_if_dirty(&ctx);

        egui::Panel::left("groundlab_controls")
            .resizable(true)
            .default_size(360.0)
            .min_size(300.0)
            .max_size(520.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_controls(ui, &ctx);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.show_canvas(ui, &ctx);
                });
        });
    }
}

fn show_texture_only(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    zoom: f32,
    missing: &str,
) {
    if let Some(texture) = texture {
        let size = texture.size_vec2() * zoom;
        let sized = egui::load::SizedTexture::new(texture.id(), size);
        ui.add(egui::Image::from_texture(sized).texture_options(egui::TextureOptions::NEAREST));
    } else {
        ui.label(missing);
    }
}

fn put_texture(
    ctx: &egui::Context,
    handle: &mut Option<egui::TextureHandle>,
    name: &str,
    image: &PixelImage,
) {
    let color_image = color_image_for_upload(image);
    if let Some(texture) = handle {
        texture.set(color_image, egui::TextureOptions::NEAREST);
    } else {
        *handle =
            Some(ctx.load_texture(name.to_string(), color_image, egui::TextureOptions::NEAREST));
    }
}

fn color_image_for_upload(image: &PixelImage) -> egui::ColorImage {
    let [width, height] = image.size();
    let max_side = width.max(height);
    if max_side <= MAX_UI_TEXTURE_SIDE {
        let rgba = image.to_rgba_bytes();
        return egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
    }

    let factor = max_side.div_ceil(MAX_UI_TEXTURE_SIDE);
    let upload_width = width.div_ceil(factor).max(1);
    let upload_height = height.div_ceil(factor).max(1);
    let mut rgba = Vec::with_capacity(upload_width * upload_height * 4);

    for y in 0..upload_height {
        let source_y = (y * factor).min(height - 1) as u32;
        for x in 0..upload_width {
            let source_x = (x * factor).min(width - 1) as u32;
            rgba.extend_from_slice(&image.get(source_x, source_y).to_array());
        }
    }

    egui::ColorImage::from_rgba_unmultiplied([upload_width, upload_height], &rgba)
}
