use std::path::PathBuf;

use eframe::egui;
use ground_core::{
    build_seam_test_sheet, ensure_default_asset_files, export_edit_scenario_suite,
    export_tileset_bundle_with_palette, find_path, load_workbench_assets, muted_field_32,
    preview_pixel_to_cell, render_terrain_preview, save_palette_file, save_recipe_file,
    validate_tileset, Brush, BrushKind, FileSnapshot, GroundMaterial, LightDirection, Palette,
    PixelImage, PreviewMode, PreviewOptions, ProjectionKind, TerrainMap, Tileset, TilesetRecipe,
    ValidationReport, ViewOrientation, WorkbenchAssetPaths,
};
use ground_game::{
    export_road_below_seed, CellCoord, CoverClass, EarthState, EnvironmentObjectKind, GroundKind,
    MissionState, TreeState, WorkOrderKind, WorkTarget, DEFAULT_MISSION_EXPORT_DIR,
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
        "GroundLab — tactical engineering workbench",
        options,
        Box::new(|cc| Ok(Box::new(GroundLabApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchPanel {
    MissionLab,
    TerrainForge,
}

impl WorkbenchPanel {
    const ALL: [WorkbenchPanel; 2] = [WorkbenchPanel::MissionLab, WorkbenchPanel::TerrainForge];

    fn label(self) -> &'static str {
        match self {
            WorkbenchPanel::MissionLab => "Mission Lab",
            WorkbenchPanel::TerrainForge => "Terrain Forge",
        }
    }
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
    active_panel: WorkbenchPanel,
    mission_state: MissionState,
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
            active_panel: WorkbenchPanel::MissionLab,
            mission_state: MissionState::road_below_seed(),
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
                show_patch_dirty_cells: true,
                show_patch_bounds: true,
                show_patch_signatures: true,
                show_cover_patches_only: false,
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
            status: "Ready. GamePivot 1 mission workbench seed is active.".to_string(),
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

    fn show_panel_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for panel in WorkbenchPanel::ALL {
                ui.selectable_value(&mut self.active_panel, panel, panel.label());
            }
        });
        ui.separator();
    }

    fn show_mission_controls(&mut self, ui: &mut egui::Ui) {
        self.show_panel_tabs(ui);
        ui.heading("Mission Lab");
        ui.label("GamePivot 1: commander/engineer prep-phase workbench");
        ui.separator();

        ui.strong(&self.mission_state.spec.title);
        ui.label(&self.mission_state.spec.objective.label);
        ui.small(format!(
            "Prep: {}s · labor: {}s · crews: {}",
            self.mission_state.remaining_prep_seconds,
            self.mission_state.remaining_labor_seconds,
            self.mission_state.spec.crew.crews
        ));
        ui.small(format!(
            "Objective cell: ({}, {}) · health {}",
            self.mission_state.spec.objective.defend_cell.x,
            self.mission_state.spec.objective.defend_cell.y,
            self.mission_state.spec.objective.objective_health
        ));

        ui.separator();
        ui.strong("Tools");
        ui.label(
            self.mission_state
                .spec
                .starting_tools
                .tools
                .iter()
                .map(|tool| tool.label())
                .collect::<Vec<_>>()
                .join(" · "),
        );

        ui.separator();
        ui.strong("Local material");
        let material_summary = self.mission_state.material_totals().positive_summary();
        if material_summary.is_empty() {
            ui.label("none stockpiled");
        } else {
            for material in material_summary {
                ui.label(material);
            }
        }

        ui.separator();
        ui.strong("Seed work orders");
        if ui.button("Apply Road Below seed plan").clicked() {
            self.mission_state.apply_seed_orders();
            self.status = "Applied scripted Road Below engineering plan.".to_string();
        }
        if ui.button("Reset mission").clicked() {
            self.mission_state = MissionState::road_below_seed();
            self.status = "Reset Road Below mission state.".to_string();
        }
        if ui.button("Export mission seed").clicked() {
            match export_road_below_seed(DEFAULT_MISSION_EXPORT_DIR) {
                Ok(()) => {
                    self.status = format!("Exported mission seed to {DEFAULT_MISSION_EXPORT_DIR}");
                }
                Err(err) => {
                    self.status = format!("Mission seed export failed: {err}");
                }
            }
        }

        ui.collapsing("Apply one order", |ui| {
            if ui.button("Dig trench across road").clicked() {
                self.apply_mission_order(
                    WorkOrderKind::DigTrench,
                    WorkTarget::Rect(ground_game::CellRect {
                        origin: CellCoord::new(5, 4),
                        width: 2,
                        height: 1,
                    }),
                );
            }
            if ui.button("Raise berm behind trench").clicked() {
                self.apply_mission_order(
                    WorkOrderKind::RaiseBerm,
                    WorkTarget::Rect(ground_game::CellRect {
                        origin: CellCoord::new(5, 3),
                        width: 2,
                        height: 1,
                    }),
                );
            }
            if ui.button("Fell roadside pine").clicked() {
                self.apply_mission_order(
                    WorkOrderKind::FellTree,
                    WorkTarget::Object("tree_west_01".to_string()),
                );
            }
            if ui.button("Cut pine into logs").clicked() {
                self.apply_mission_order(
                    WorkOrderKind::CutIntoLogs,
                    WorkTarget::Object("tree_west_01".to_string()),
                );
            }
            if ui.button("Place stakes in road").clicked() {
                self.apply_mission_order(
                    WorkOrderKind::PlaceStakes,
                    WorkTarget::Cell(CellCoord::new(3, 4)),
                );
            }
        });

        ui.separator();
        ui.strong("Enemy intel");
        for group in &self.mission_state.spec.enemy_groups {
            ui.label(format!(
                "{} · {} units · {:?}",
                group.label, group.count, group.doctrine
            ));
        }

        ui.separator();
        ui.strong("Work log");
        if self.mission_state.work_orders.is_empty() {
            ui.label("No work orders applied yet.");
        } else {
            for order in self.mission_state.work_orders.iter().rev().take(10) {
                ui.label(format!(
                    "#{:02} {} · {} · {}s",
                    order.id,
                    order.kind.label(),
                    order.status.label(),
                    order.labor_seconds
                ));
            }
        }

        ui.separator();
        ui.label(&self.status);
    }

    fn apply_mission_order(&mut self, kind: WorkOrderKind, target: WorkTarget) {
        let order = self.mission_state.apply_work_order(kind, target);
        self.status = format!(
            "Mission order #{:02}: {} · {}",
            order.id,
            order.kind.label(),
            order.status.label()
        );
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show_panel_tabs(ui);
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
                "Target-grid / patch debug",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        ui.indent("target_patch_debug_options", |ui| {
            if ui
                .checkbox(
                    &mut self.preview_options.show_patch_dirty_cells,
                    "Show dirty cells",
                )
                .changed()
            {
                self.preview_options.show_feature_overlay = true;
                self.dirty_preview = true;
            }
            if ui
                .checkbox(
                    &mut self.preview_options.show_patch_bounds,
                    "Show patch bounds",
                )
                .changed()
            {
                self.preview_options.show_feature_overlay = true;
                self.dirty_preview = true;
            }
            if ui
                .checkbox(
                    &mut self.preview_options.show_patch_signatures,
                    "Show terrain signatures",
                )
                .changed()
            {
                self.preview_options.show_feature_overlay = true;
                self.dirty_preview = true;
            }
            if ui
                .checkbox(
                    &mut self.preview_options.show_cover_patches_only,
                    "Show cover patches only",
                )
                .changed()
            {
                self.preview_mode = PreviewMode::PerspectiveSpriteScene;
                self.dirty_preview = true;
            }
        });
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
                "exports/milestone_04_12",
            ) {
                Ok(()) => self.status = "Exported to exports/milestone_04_12".to_string(),
                Err(err) => self.status = format!("Export failed: {err}"),
            }
        }
        if ui.button("Export edit stress tests").clicked() {
            self.refresh_if_dirty(ctx);
            match export_edit_scenario_suite(
                &self.tileset,
                &self.terrain,
                "exports/milestone_04_12/edit_scenarios",
            ) {
                Ok(()) => {
                    self.status =
                        "Exported edit stress tests to exports/milestone_04_12/edit_scenarios"
                            .to_string()
                }
                Err(err) => self.status = format!("Edit stress export failed: {err}"),
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

    fn show_mission_canvas(&mut self, ui: &mut egui::Ui) {
        ui.heading("Road Below mission seed");
        ui.label("Legend: S spawn, O objective, T tree, L logs/trunk, ^ stakes, = trench, # berm, : road, . grass.");
        let cell_size = 44.0;
        let map_w = self.mission_state.map.width as f32 * cell_size;
        let map_h = self.mission_state.map.height as f32 * cell_size;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(map_w, map_h), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        for y in 0..self.mission_state.map.height {
            for x in 0..self.mission_state.map.width {
                let coord = CellCoord::new(x, y);
                let cell = self
                    .mission_state
                    .map
                    .cell(coord)
                    .expect("mission canvas only reads in-bounds cells");
                let x0 = rect.left() + x as f32 * cell_size;
                let y0 = rect.top() + y as f32 * cell_size;
                let tile_rect = egui::Rect::from_min_size(
                    egui::pos2(x0, y0),
                    egui::vec2(cell_size - 1.0, cell_size - 1.0),
                );
                painter.rect_filled(tile_rect, 0.0, mission_cell_color(cell));
                painter.rect_stroke(
                    tile_rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_gray(45)),
                    egui::StrokeKind::Inside,
                );
                let glyph = mission_cell_glyph(&self.mission_state, coord, cell);
                if !glyph.is_empty() {
                    painter.text(
                        tile_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        glyph,
                        egui::FontId::proportional(18.0),
                        egui::Color32::from_rgb(235, 232, 202),
                    );
                }
            }
        }

        ui.add_space(8.0);
        ui.label(
            "This is a data-first mission view. It is intentionally not the final 2.5D renderer.",
        );
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
                egui::ScrollArea::vertical().show(ui, |ui| match self.active_panel {
                    WorkbenchPanel::MissionLab => self.show_mission_controls(ui),
                    WorkbenchPanel::TerrainForge => self.show_controls(ui, &ctx),
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.active_panel {
                    WorkbenchPanel::MissionLab => self.show_mission_canvas(ui),
                    WorkbenchPanel::TerrainForge => self.show_canvas(ui, &ctx),
                });
        });
    }
}

fn mission_cell_color(cell: &ground_game::MissionCell) -> egui::Color32 {
    match cell.earth_state {
        EarthState::Trench | EarthState::DeepTrench => egui::Color32::from_rgb(42, 31, 24),
        EarthState::Berm | EarthState::SpoilPile => egui::Color32::from_rgb(122, 88, 50),
        EarthState::Scraped | EarthState::Ditch => egui::Color32::from_rgb(110, 84, 55),
        _ => match cell.ground {
            GroundKind::Grass => egui::Color32::from_rgb(73, 105, 53),
            GroundKind::Dirt => egui::Color32::from_rgb(135, 91, 52),
            GroundKind::Mud => egui::Color32::from_rgb(71, 61, 48),
            GroundKind::Rock => egui::Color32::from_rgb(101, 105, 96),
            GroundKind::Road => egui::Color32::from_rgb(154, 109, 66),
        },
    }
}

fn mission_cell_glyph(
    state: &MissionState,
    coord: CellCoord,
    cell: &ground_game::MissionCell,
) -> String {
    if state.spec.objective.defend_cell == coord {
        return "O".to_string();
    }
    if state.map.spawn_cells.contains(&coord) {
        return "S".to_string();
    }
    if let Some(object) = state.map.objects.iter().find(|object| object.cell == coord) {
        return match object.kind {
            EnvironmentObjectKind::Tree(TreeState::Standing)
            | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => "T",
            EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. })
            | EnvironmentObjectKind::Tree(TreeState::CutLogs) => "L",
            EnvironmentObjectKind::Stakes(_) => "^",
            EnvironmentObjectKind::Rock(_) => "r",
            EnvironmentObjectKind::Wall(_) => "W",
            EnvironmentObjectKind::Wire(_) => "w",
            EnvironmentObjectKind::FightingPosition(_) => "F",
            EnvironmentObjectKind::Log(_) => "L",
            _ => "t",
        }
        .to_string();
    }
    match cell.earth_state {
        EarthState::Trench | EarthState::DeepTrench => "=".to_string(),
        EarthState::Berm | EarthState::SpoilPile => "#".to_string(),
        EarthState::Scraped | EarthState::Ditch => "_".to_string(),
        _ => {
            if matches!(cell.cover, CoverClass::Strong) {
                "+".to_string()
            } else {
                String::new()
            }
        }
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
