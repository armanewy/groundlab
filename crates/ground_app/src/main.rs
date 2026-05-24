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
    export_road_below_seed, load_generated_mission_browser_index, load_mission_spec,
    load_work_order_script, mission_rating_for_state, road_below_balance_scripts,
    road_below_seed_orders, run_work_order_script, save_work_order_script, AssaultEventKind,
    CellCoord, CoverClass, EarthState, EnemyAgentStatus, EnvironmentObject, EnvironmentObjectKind,
    GeneratedMissionBrowserEntry, GeneratedMissionBrowserIndex, GroundKind, LogState, MissionPhase,
    MissionState, MissionTheme, ScriptedWorkOrder, TreeState, WorkOrderKind, WorkOrderScript,
    WorkOrderStatus, WorkTarget, DEFAULT_MISSION_EXPORT_DIR,
};

const MAX_UI_TEXTURE_SIDE: usize = 2048;
const PLAYER_PLAN_PATH: &str = "exports/gamepivot_08/player_plan.ron";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteOverlayMode {
    None,
    Initial,
    Current,
    Delta,
}

impl RouteOverlayMode {
    const ALL: [RouteOverlayMode; 4] = [
        RouteOverlayMode::Current,
        RouteOverlayMode::Delta,
        RouteOverlayMode::Initial,
        RouteOverlayMode::None,
    ];

    fn label(self) -> &'static str {
        match self {
            RouteOverlayMode::None => "none",
            RouteOverlayMode::Initial => "initial",
            RouteOverlayMode::Current => "current",
            RouteOverlayMode::Delta => "delta",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MissionActionMode {
    Inspect,
    Dig,
    Build,
    Harvest,
    Deploy,
    Cancel,
}

impl MissionActionMode {
    const ALL: [MissionActionMode; 6] = [
        MissionActionMode::Inspect,
        MissionActionMode::Dig,
        MissionActionMode::Build,
        MissionActionMode::Harvest,
        MissionActionMode::Deploy,
        MissionActionMode::Cancel,
    ];

    fn label(self) -> &'static str {
        match self {
            MissionActionMode::Inspect => "Inspect",
            MissionActionMode::Dig => "Dig",
            MissionActionMode::Build => "Build",
            MissionActionMode::Harvest => "Harvest",
            MissionActionMode::Deploy => "Deploy",
            MissionActionMode::Cancel => "Cancel",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MissionMapMode {
    Terrain,
    Height,
    Cover,
    Resources,
    Delay,
    Pressure,
    Actual,
    Hazards,
}

impl MissionMapMode {
    const ALL: [MissionMapMode; 8] = [
        MissionMapMode::Terrain,
        MissionMapMode::Height,
        MissionMapMode::Cover,
        MissionMapMode::Resources,
        MissionMapMode::Delay,
        MissionMapMode::Pressure,
        MissionMapMode::Actual,
        MissionMapMode::Hazards,
    ];

    fn label(self) -> &'static str {
        match self {
            MissionMapMode::Terrain => "terrain",
            MissionMapMode::Height => "height",
            MissionMapMode::Cover => "cover",
            MissionMapMode::Resources => "resources",
            MissionMapMode::Delay => "delay",
            MissionMapMode::Pressure => "pressure",
            MissionMapMode::Actual => "actual",
            MissionMapMode::Hazards => "hazards",
        }
    }
}

#[derive(Clone, Debug)]
struct MissionBalanceDashboardRow {
    label: String,
    stars: u8,
    score: i32,
    outcome: String,
    stopped: u32,
    reached: u32,
    prep_time_used_seconds: u32,
}

struct GroundLabApp {
    active_panel: WorkbenchPanel,
    mission_state: MissionState,
    selected_mission_cell: CellCoord,
    mission_action_mode: MissionActionMode,
    mission_map_mode: MissionMapMode,
    route_overlay_mode: RouteOverlayMode,
    route_group_filter: usize,
    balance_dashboard: Vec<MissionBalanceDashboardRow>,
    notifications: Vec<String>,
    mission_load_path_text: String,
    mission_browser_path_text: String,
    mission_browser: Option<GeneratedMissionBrowserIndex>,
    mission_browser_theme_filter: Option<MissionTheme>,
    mission_browser_accepted_only: bool,
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
            mission_state: playable_road_below_state(),
            selected_mission_cell: CellCoord::new(5, 4),
            mission_action_mode: MissionActionMode::Inspect,
            mission_map_mode: MissionMapMode::Terrain,
            route_overlay_mode: RouteOverlayMode::Current,
            route_group_filter: 0,
            balance_dashboard: build_balance_dashboard(),
            notifications: vec![
                "Road Below briefing ready. Start prep when you are ready to issue orders."
                    .to_string(),
            ],
            mission_load_path_text: "exports/procgen_01/candidates/seed_0001/mission.ron"
                .to_string(),
            mission_browser_path_text: "exports/procgen_04/browser_index.json".to_string(),
            mission_browser: None,
            mission_browser_theme_filter: None,
            mission_browser_accepted_only: true,
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
            status: "Ready. GamePivot 5 assault sandbox is active.".to_string(),
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
        ui.label("ProcGen 4: generated mission browser and playable prep loop");
        ui.separator();

        self.show_mission_status_panel(ui);
        ui.separator();
        self.show_mission_lifecycle_panel(ui);
        ui.separator();
        self.show_generated_mission_browser_panel(ui);
        ui.separator();
        self.show_tutorial_panel(ui);
        ui.separator();
        self.show_mission_action_toolbar(ui);
        ui.separator();
        self.show_mission_route_panel(ui);
        ui.separator();
        self.show_assault_panel(ui);
        ui.separator();
        self.show_selected_mission_context(ui);
        ui.separator();
        self.show_work_order_queue_panel(ui);
        ui.separator();
        self.show_enemy_intel_panel(ui);
        ui.separator();
        self.show_objective_panel(ui);
        ui.separator();
        self.show_mission_scenario_controls(ui);
        ui.separator();
        self.show_balance_dashboard_panel(ui);
        ui.separator();
        self.show_feedback_panel(ui);
    }

    fn show_mission_status_panel(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.strong(format!("Mission: {}", self.mission_state.spec.title));
            ui.label(&self.mission_state.spec.objective.label);
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "Prep {}",
                    format_duration(self.mission_state.remaining_prep_seconds)
                ));
                ui.separator();
                ui.label(format!(
                    "Labor {}",
                    format_duration(self.mission_state.remaining_labor_seconds)
                ));
                ui.separator();
                ui.label(format!("Crew {}", self.mission_state.spec.crew.crews));
                ui.separator();
                ui.label(format!("Queued {}", self.mission_state.work_queue.len()));
            });
            let material_summary = self.mission_state.material_totals().positive_summary();
            ui.small(format!(
                "Materials: {}",
                if material_summary.is_empty() {
                    "none stockpiled".to_string()
                } else {
                    material_summary.join(" · ")
                }
            ));
            ui.small(format!(
                "Tools: {}",
                self.mission_state
                    .spec
                    .starting_tools
                    .tools
                    .iter()
                    .map(|tool| tool.label())
                    .collect::<Vec<_>>()
                    .join(" · ")
            ));
        });
    }

    fn show_mission_lifecycle_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("Mission flow");
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    matches!(self.mission_state.phase, MissionPhase::Briefing),
                    egui::Button::new("Start prep"),
                )
                .clicked()
            {
                self.mission_state.phase = MissionPhase::Prep;
                self.notify("Prep started. Issue work orders, preview routes, then start assault.");
            }
            if ui
                .add_enabled(
                    !matches!(self.mission_state.phase, MissionPhase::Briefing),
                    egui::Button::new("Start assault"),
                )
                .clicked()
            {
                if !self.mission_state.work_queue.is_empty() {
                    self.notify("Run or clear queued work orders before starting assault.");
                } else {
                    self.mission_state.start_assault();
                    self.notify("Assault started from current prepared terrain.");
                }
            }
            if ui
                .add_enabled(
                    self.mission_state.assault.is_some(),
                    egui::Button::new("Retry assault"),
                )
                .clicked()
            {
                self.mission_state.reset_assault();
                let summary = self.mission_state.run_assault_to_completion(160);
                self.notify(format!("Retried assault: {}", summary.outcome_label));
            }
            if ui.button("Reset to briefing").clicked() {
                self.mission_state = playable_road_below_state();
                self.selected_mission_cell = CellCoord::new(5, 4);
                self.notify("Reset Road Below to briefing.");
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Save prep plan").clicked() {
                let script = self.current_player_plan_script();
                match save_work_order_script(PLAYER_PLAN_PATH, &script) {
                    Ok(()) => self.notify(format!("Saved prep plan to {PLAYER_PLAN_PATH}.")),
                    Err(err) => self.notify(format!("Save prep plan failed: {err}")),
                }
            }
            if ui.button("Load plan to queue").clicked() {
                match load_work_order_script(PLAYER_PLAN_PATH) {
                    Ok(script) => {
                        self.reset_to_prep_with_script(&script, false);
                        self.notify(format!(
                            "Loaded {} order(s) from {PLAYER_PLAN_PATH} into the queue.",
                            script.orders.len()
                        ));
                    }
                    Err(err) => self.notify(format!("Load prep plan failed: {err}")),
                }
            }
            if ui.button("Apply saved plan").clicked() {
                match load_work_order_script(PLAYER_PLAN_PATH) {
                    Ok(script) => {
                        self.reset_to_prep_with_script(&script, true);
                        self.notify(format!(
                            "Applied saved prep plan with {} order(s).",
                            script.orders.len()
                        ));
                    }
                    Err(err) => self.notify(format!("Apply saved plan failed: {err}")),
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Mission file");
            ui.add(
                egui::TextEdit::singleline(&mut self.mission_load_path_text).desired_width(310.0),
            );
            if ui.button("Load mission").clicked() {
                let path = self.mission_load_path_text.trim().to_string();
                self.load_mission_file(&path);
            }
        });
        ui.small("Playable flow: briefing -> prep -> assault -> debrief -> retry.");
    }

    fn load_mission_file(&mut self, path: &str) {
        match load_mission_spec(path) {
            Ok(spec) => {
                let objective = spec.objective.defend_cell;
                let title = spec.title.clone();
                self.mission_state = MissionState::from_spec(spec);
                self.mission_state.phase = MissionPhase::Briefing;
                self.selected_mission_cell = objective;
                self.route_group_filter = 0;
                self.notify(format!("Loaded generated mission: {title}."));
            }
            Err(err) => self.notify(format!("Load mission failed: {err}")),
        }
    }

    fn show_generated_mission_browser_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Generated Missions", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Index");
                ui.add(
                    egui::TextEdit::singleline(&mut self.mission_browser_path_text)
                        .desired_width(310.0),
                );
                if ui.button("Load index").clicked() {
                    let path = self.mission_browser_path_text.trim();
                    match load_generated_mission_browser_index(path) {
                        Ok(index) => {
                            let accepted = index.accepted_count;
                            let generated = index.generated_count;
                            self.mission_browser = Some(index);
                            self.notify(format!(
                                "Loaded generated mission index: {accepted}/{generated} accepted."
                            ));
                        }
                        Err(err) => self.notify(format!("Load mission index failed: {err}")),
                    }
                }
            });

            ui.horizontal_wrapped(|ui| {
                egui::ComboBox::from_label("Theme")
                    .selected_text(
                        self.mission_browser_theme_filter
                            .map(|theme| theme.label())
                            .unwrap_or("All themes"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.mission_browser_theme_filter,
                            None,
                            "All themes",
                        );
                        for theme in MissionTheme::GENERATABLE {
                            ui.selectable_value(
                                &mut self.mission_browser_theme_filter,
                                Some(theme),
                                theme.label(),
                            );
                        }
                    });
                ui.checkbox(&mut self.mission_browser_accepted_only, "Accepted only");
            });

            let Some(index) = &self.mission_browser else {
                ui.small("Load a ProcGen browser_index.json to browse generated candidates.");
                return;
            };
            ui.small(format!(
                "{} generated · {} accepted · {} rejected · source {}",
                index.generated_count, index.accepted_count, index.rejected_count, index.source_dir
            ));
            let entries = self
                .filtered_mission_browser_entries()
                .into_iter()
                .take(18)
                .cloned()
                .collect::<Vec<_>>();
            if entries.is_empty() {
                ui.small("No generated missions match the current filters.");
                return;
            }
            for entry in entries {
                self.show_generated_mission_card(ui, &entry);
            }
        });
    }

    fn filtered_mission_browser_entries(&self) -> Vec<&GeneratedMissionBrowserEntry> {
        let Some(index) = &self.mission_browser else {
            return Vec::new();
        };
        index
            .candidates
            .iter()
            .filter(|entry| !self.mission_browser_accepted_only || entry.accepted)
            .filter(|entry| {
                self.mission_browser_theme_filter
                    .map(|theme| entry.theme == theme)
                    .unwrap_or(true)
            })
            .collect()
    }

    fn show_generated_mission_card(
        &mut self,
        ui: &mut egui::Ui,
        entry: &GeneratedMissionBrowserEntry,
    ) {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&entry.title);
                ui.label(format!("[{}]", entry.theme_slug));
                ui.label(format!("seed {}", entry.seed));
                ui.label(format!("score {}", entry.tactical_interest_score));
                ui.label(if entry.accepted {
                    "accepted"
                } else {
                    "rejected"
                });
                let can_load = entry.mission_path.is_some();
                if ui
                    .add_enabled(can_load, egui::Button::new("Load"))
                    .clicked()
                {
                    if let Some(path) = &entry.mission_path {
                        let path = path.clone();
                        self.mission_load_path_text = path.clone();
                        self.load_mission_file(&path);
                    }
                }
            });
            ui.small(format!(
                "best {} · baseline {} -> best {} · spread {} · affordance {}",
                entry.best_plan_label,
                entry.baseline_score,
                entry.best_score,
                entry.best_minus_worst,
                entry.primary_affordance
            ));
            ui.small(format!(
                "route {:.2} · material {:.2} · hazard {:.2}",
                entry.route_diversity_score,
                entry.local_material_score,
                entry.hazard_viability_score
            ));
            if let Some(reason) = &entry.top_rejection_reason {
                ui.small(format!(
                    "reject: {}{}",
                    entry
                        .top_rejection_kind
                        .map(|kind| format!("{kind:?}: "))
                        .unwrap_or_default(),
                    reason
                ));
            }
        });
    }

    fn show_tutorial_panel(&self, ui: &mut egui::Ui) {
        ui.collapsing("Road Below guide", |ui| {
            for (done, text) in self.tutorial_steps() {
                ui.small(format!("{} {text}", if done { "Done:" } else { "Next:" }));
            }
        });
    }

    fn show_mission_action_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.strong("Orders");
        ui.horizontal_wrapped(|ui| {
            for mode in MissionActionMode::ALL {
                ui.selectable_value(&mut self.mission_action_mode, mode, mode.label());
            }
        });
        ui.small(match self.mission_action_mode {
            MissionActionMode::Inspect => {
                "Inspect selected terrain, object state, and route consequences."
            }
            MissionActionMode::Dig => "Dig or flatten earthwork cells.",
            MissionActionMode::Build => "Spend local material to raise cover or positions.",
            MissionActionMode::Harvest => "Transform trees and objects into local material.",
            MissionActionMode::Deploy => "Place prepared obstacles and field defenses.",
            MissionActionMode::Cancel => "Review and clear queued work before execution.",
        });
    }

    fn show_mission_route_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("Overlays");
        ui.horizontal_wrapped(|ui| {
            for mode in MissionMapMode::ALL {
                ui.selectable_value(&mut self.mission_map_mode, mode, mode.label());
            }
        });
        egui::ComboBox::from_label("Route")
            .selected_text(self.route_overlay_mode.label())
            .show_ui(ui, |ui| {
                for mode in RouteOverlayMode::ALL {
                    ui.selectable_value(&mut self.route_overlay_mode, mode, mode.label());
                }
            });
        let current_routes = self.mission_state.route_preview();
        if self.route_group_filter > current_routes.routes.len() {
            self.route_group_filter = 0;
        }
        egui::ComboBox::from_label("Enemy group")
            .selected_text(route_filter_label(&current_routes, self.route_group_filter))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.route_group_filter, 0, "all groups");
                for (index, route) in current_routes.routes.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.route_group_filter,
                        index + 1,
                        route.group_label.as_str(),
                    );
                }
            });
        for route in &current_routes.routes {
            ui.small(format!(
                "{} · {} · {} cell(s) · cost {:.1}",
                route.group_label,
                route.doctrine.label(),
                route.points.len(),
                route.total_cost
            ));
        }
    }

    fn show_assault_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("Assault");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Start / restart assault").clicked() {
                if matches!(self.mission_state.phase, MissionPhase::Briefing) {
                    self.notify("Start prep before launching the assault.");
                } else if !self.mission_state.work_queue.is_empty() {
                    self.notify("Run or clear queued work orders before starting assault.");
                } else {
                    self.mission_state.start_assault();
                    self.notify("Assault started from current prepared terrain.");
                }
            }
            if ui.button("Step").clicked() {
                if matches!(self.mission_state.phase, MissionPhase::Briefing) {
                    self.notify("Start prep before stepping the assault.");
                } else if !self.mission_state.work_queue.is_empty() {
                    self.notify("Run or clear queued work orders before stepping the assault.");
                } else {
                    let events = self.mission_state.step_assault();
                    if let Some(event) = events.last() {
                        self.notify(format!("Assault step: {}", event.note));
                    } else {
                        self.notify("Assault step produced no new event.");
                    }
                }
            }
            if ui.button("Run").clicked() {
                if matches!(self.mission_state.phase, MissionPhase::Briefing) {
                    self.notify("Start prep before running the assault.");
                } else if !self.mission_state.work_queue.is_empty() {
                    self.notify("Run or clear queued work orders before running the assault.");
                } else {
                    let summary = self.mission_state.run_assault_to_completion(160);
                    self.notify(summary.outcome_label);
                }
            }
            if ui.button("Release logs").clicked() {
                let count = self.mission_state.release_prepared_rolling_hazards();
                self.notify(format!("Scheduled {count} prepared rolling log(s)."));
            }
            if ui.button("Reset to prep").clicked() {
                self.mission_state.reset_assault();
                self.notify("Assault reset. Prep state remains intact.");
            }
        });

        if let Some(assault) = &self.mission_state.assault {
            let active = assault
                .agents
                .iter()
                .filter(|agent| {
                    matches!(
                        agent.status,
                        EnemyAgentStatus::Advancing | EnemyAgentStatus::Delayed
                    )
                })
                .count();
            let eliminated = assault
                .agents
                .iter()
                .filter(|agent| matches!(agent.status, EnemyAgentStatus::Eliminated))
                .count();
            let reached = assault
                .agents
                .iter()
                .filter(|agent| matches!(agent.status, EnemyAgentStatus::ReachedObjective))
                .count();
            ui.small(format!(
                "{} · tick {} · objective {} · active {} · stopped {} · reached {}",
                assault.status.label(),
                assault.tick,
                assault.objective_health.max(0),
                active,
                eliminated,
                reached
            ));
            if let Some(summary) = &assault.summary {
                ui.small(format!(
                    "{} · damage {}",
                    summary.outcome_label, summary.objective_damage_taken
                ));
            }
            if let Some(debrief) = self.mission_state.assault_debrief() {
                ui.separator();
                self.show_debrief_breakdown(ui, &debrief);
                if let Some(group) = &debrief.influence.most_delayed_group {
                    ui.small(format!(
                        "Most delayed: {} · {} tick(s)",
                        group.group_label, group.magnitude
                    ));
                }
                if let Some(cell) = &debrief.influence.most_effective_obstacle {
                    ui.small(format!(
                        "Best obstacle: ({}, {}) · {}",
                        cell.cell.x, cell.cell.y, cell.label
                    ));
                } else {
                    ui.small("Best obstacle: none affected the assault route.");
                }
                if let Some(cell) = debrief.influence.breach_cells.first() {
                    ui.small(format!(
                        "Breach point: ({}, {}) · {} hit(s)",
                        cell.cell.x, cell.cell.y, cell.count
                    ));
                } else {
                    ui.small("Breach point: none.");
                }
                ui.small(format!(
                    "Prediction accuracy: {:.0}% · divergence cells {}",
                    debrief.route_prediction_accuracy.average_accuracy * 100.0,
                    debrief.route_prediction_accuracy.total_divergence_cells
                ));
                ui.small(format!(
                    "Rolling hazards: {} released · {} enemy hit(s) · {} obstacle(s) destroyed",
                    debrief.rolling_hazards.released_count,
                    debrief.rolling_hazards.enemies_hit,
                    debrief.rolling_hazards.obstacles_destroyed
                ));
                if let Some(unused) = debrief.influence.unused_defenses.first() {
                    ui.small(format!("Unused defense: {unused}"));
                }
            }
        } else {
            ui.small("No assault is running. Start uses the current prepared mission state.");
        }
    }

    fn show_debrief_breakdown(&self, ui: &mut egui::Ui, debrief: &ground_game::AssaultDebrief) {
        ui.strong("Debrief");
        ui.small(format!(
            "Rating: {} star(s) · {} · score {}",
            debrief.rating.stars, debrief.rating.label, debrief.rating.score
        ));
        ui.small(format!(
            "Objective survived: {} · health {:.0}% · damage {}",
            if debrief.rating.objective_survived {
                "yes"
            } else {
                "no"
            },
            debrief.rating.objective_health_ratio * 100.0,
            debrief.summary.objective_damage_taken
        ));
        ui.small(format!(
            "Attackers stopped: {} / {} · reached {}",
            debrief.summary.enemies_eliminated,
            debrief.summary.enemies_spawned,
            debrief.summary.enemies_reached_objective
        ));
        ui.small(format!(
            "Prep used: {} · friendly-risk cells {} · unused defenses {}",
            format_duration(debrief.rating.prep_time_used_seconds),
            debrief.rating.friendly_risk_count,
            debrief.rating.unused_defense_count
        ));
        ui.small(format!(
            "Hazard impact: {} enemy hit(s) · route accuracy {:.0}%",
            debrief.rating.hazard_enemies_hit,
            debrief.route_prediction_accuracy.average_accuracy * 100.0
        ));
    }

    fn show_work_order_queue_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("Work order queue");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Queue seed plan").clicked() {
                for (kind, target) in road_below_seed_orders() {
                    self.queue_mission_order(kind, target);
                }
                self.notify("Queued scripted Road Below engineering plan.");
            }
            if ui.button("Run next").clicked() {
                match self.mission_state.run_next_queued_order() {
                    Some(order) => self.notify(format!(
                        "Ran order #{:02}: {} · {}",
                        order.id,
                        order.kind.label(),
                        order.status.label()
                    )),
                    None => self.notify("No queued work order to run."),
                }
            }
            if ui.button("Run all").clicked() {
                self.mission_state.run_all_queued_orders();
                self.notify("Ran all queued work orders.");
            }
            if ui.button("Clear queue").clicked() {
                self.mission_state.work_queue.clear();
                self.notify("Cleared queued work orders.");
            }
        });
        if self.mission_state.work_queue.is_empty() {
            ui.label("No queued work orders.");
        } else {
            for order in &self.mission_state.work_queue {
                let color = if matches!(order.status, ground_game::WorkOrderStatus::Rejected { .. })
                {
                    egui::Color32::from_rgb(230, 160, 120)
                } else {
                    egui::Color32::from_rgb(224, 214, 176)
                };
                ui.colored_label(
                    color,
                    format!(
                        "#{:02} {} · {} · crew {} · {}",
                        order.id,
                        order.kind.label(),
                        format_duration(order.duration_seconds),
                        order.assigned_crews,
                        order.status.label()
                    ),
                );
            }
        }
    }

    fn show_enemy_intel_panel(&self, ui: &mut egui::Ui) {
        ui.strong("Enemy intel");
        let current_routes = self.mission_state.route_preview();
        for (index, group) in self.mission_state.spec.enemy_groups.iter().enumerate() {
            let cost = current_routes
                .routes
                .get(index)
                .map(|route| format!("{:.1}", route.total_cost))
                .unwrap_or_else(|| "unknown".to_string());
            ui.label(format!(
                "{} · {} units · {} · route cost {}",
                group.label,
                group.count,
                group.doctrine.label(),
                cost
            ));
        }
    }

    fn show_objective_panel(&self, ui: &mut egui::Ui) {
        ui.strong(&self.mission_state.spec.title);
        if !self.mission_state.spec.briefing.summary.is_empty() {
            ui.label(&self.mission_state.spec.briefing.summary);
        }
        ui.separator();
        ui.strong("Objective");
        let primary = if self.mission_state.spec.briefing.primary.is_empty() {
            self.mission_state.spec.objective.label.as_str()
        } else {
            self.mission_state.spec.briefing.primary.as_str()
        };
        ui.label(format!("Primary: {primary}"));
        ui.small(format!(
            "Hold cell ({}, {}) · health {}",
            self.mission_state.spec.objective.defend_cell.x,
            self.mission_state.spec.objective.defend_cell.y,
            self.mission_state.spec.objective.objective_health
        ));
        for optional in &self.mission_state.spec.briefing.optional_objectives {
            ui.small(format!("Optional: {optional}"));
        }
        if !self.mission_state.spec.briefing.intel.is_empty() {
            ui.separator();
            ui.strong("Briefing intel");
            for intel in &self.mission_state.spec.briefing.intel {
                ui.small(intel);
            }
        }
        if let Some(debrief) = self.mission_state.assault_debrief() {
            ui.separator();
            ui.strong("Mission rating");
            ui.label(format!(
                "{} star(s) · {} · score {}",
                debrief.rating.stars, debrief.rating.label, debrief.rating.score
            ));
            for note in debrief.rating.notes.iter().take(4) {
                ui.small(note);
            }
        }
    }

    fn show_mission_scenario_controls(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Scenario controls", |ui| {
            if ui.button("Apply seed plan immediately").clicked() {
                if matches!(self.mission_state.phase, MissionPhase::Briefing) {
                    self.notify("Start prep before applying a scripted plan.");
                } else {
                    self.mission_state.apply_seed_orders();
                    self.notify("Applied scripted Road Below engineering plan immediately.");
                }
            }
            if ui.button("Reset mission").clicked() {
                self.mission_state = playable_road_below_state();
                self.selected_mission_cell = CellCoord::new(5, 4);
                self.notify("Reset Road Below mission state.");
            }
            if ui.button("Export mission seed").clicked() {
                match export_road_below_seed(DEFAULT_MISSION_EXPORT_DIR) {
                    Ok(()) => self.notify(format!(
                        "Exported mission seed to {DEFAULT_MISSION_EXPORT_DIR}"
                    )),
                    Err(err) => self.notify(format!("Mission seed export failed: {err}")),
                }
            }
        });
        ui.collapsing("Scripted quick queue", |ui| {
            if ui.button("Trench across road").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::DigTrench,
                    WorkTarget::Rect(ground_game::CellRect {
                        origin: CellCoord::new(5, 4),
                        width: 2,
                        height: 1,
                    }),
                );
            }
            if ui.button("Berm behind trench").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::RaiseBerm,
                    WorkTarget::Rect(ground_game::CellRect {
                        origin: CellCoord::new(5, 3),
                        width: 2,
                        height: 1,
                    }),
                );
            }
            if ui.button("Fell roadside pine").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::FellTree,
                    WorkTarget::Object("tree_west_01".to_string()),
                );
            }
            if ui.button("Cut pine into logs").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::CutIntoLogs,
                    WorkTarget::Object("tree_west_01".to_string()),
                );
            }
            if ui.button("Stakes in road").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::PlaceStakes,
                    WorkTarget::Cell(CellCoord::new(3, 4)),
                );
            }
            if ui.button("Prepare ridge log").clicked() {
                self.queue_mission_order(
                    WorkOrderKind::PrepareRollingLog,
                    WorkTarget::Object("ridge_log_01".to_string()),
                );
            }
        });
    }

    fn show_balance_dashboard_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Mission balance dashboard", |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Refresh dashboard").clicked() {
                    self.balance_dashboard = build_balance_dashboard();
                    self.notify("Refreshed Road Below balance dashboard.");
                }
                ui.small("Scripted benchmark plans for tuning Road Below.");
            });
            for row in &self.balance_dashboard {
                ui.small(format!(
                    "{} · {} star(s) · score {} · stopped {} · reached {} · prep {} · {}",
                    row.label,
                    row.stars,
                    row.score,
                    row.stopped,
                    row.reached,
                    format_duration(row.prep_time_used_seconds),
                    row.outcome
                ));
            }
        });
    }

    fn show_feedback_panel(&self, ui: &mut egui::Ui) {
        ui.strong("Notifications");
        ui.label(&self.status);
        for note in self.notifications.iter().rev().take(5) {
            ui.small(note);
        }
        ui.separator();
        ui.strong("Order validation");
        if self.mission_state.order_validation.is_empty() {
            ui.label("No order validation issues.");
        } else {
            for issue in self.mission_state.order_validation.iter().rev().take(6) {
                ui.colored_label(
                    egui::Color32::from_rgb(230, 160, 120),
                    format!("{:?} · {}", issue.severity, issue.message),
                );
            }
        }
        ui.separator();
        ui.strong("Material ledger");
        if self.mission_state.material_ledger.is_empty() {
            ui.label("No material changes yet.");
        } else {
            for entry in self.mission_state.material_ledger.iter().rev().take(6) {
                let summary = entry.net.signed_summary().join(", ");
                ui.small(format!(
                    "#{:02} {} · {}",
                    entry.order_id,
                    entry.order_kind.label(),
                    if summary.is_empty() {
                        "no material delta".to_string()
                    } else {
                        summary
                    }
                ));
            }
        }
        ui.separator();
        ui.strong("Work log");
        if self.mission_state.work_orders.is_empty() {
            ui.label("No work orders applied yet.");
        } else {
            for order in self.mission_state.work_orders.iter().rev().take(6) {
                ui.small(format!(
                    "#{:02} {} · {} · {}",
                    order.id,
                    order.kind.label(),
                    order.status.label(),
                    format_duration(order.labor_seconds)
                ));
            }
        }
    }

    fn show_selected_mission_context(&mut self, ui: &mut egui::Ui) {
        let cell_coord = self.selected_mission_cell;
        ui.strong(format!("{} context", self.mission_action_mode.label()));
        if matches!(self.mission_state.phase, MissionPhase::Briefing) {
            ui.label("Start prep to issue engineering work orders.");
            return;
        }
        let Some(cell) = self.mission_state.map.cell(cell_coord) else {
            ui.label("Selection is outside the mission map.");
            return;
        };
        ui.label(format!(
            "Cell ({}, {}) · {} · {:?} · height {} · cover {:?} · move {:.1}",
            cell_coord.x,
            cell_coord.y,
            cell.ground.label(),
            cell.earth_state,
            cell.height,
            cell.cover,
            cell.movement_cost
        ));
        if let Some(object) = self
            .mission_state
            .map
            .objects
            .iter()
            .find(|object| object.cell == cell_coord)
            .cloned()
        {
            ui.label(format!(
                "Object: {} · {}",
                object.label,
                mission_object_state_label(&object)
            ));
            self.show_object_order_buttons(ui, &object);
        } else {
            self.show_cell_order_buttons(ui, cell_coord);
        }
    }

    fn show_cell_order_buttons(&mut self, ui: &mut egui::Ui, cell: CellCoord) {
        ui.horizontal_wrapped(|ui| {
            if matches!(
                self.mission_action_mode,
                MissionActionMode::Inspect | MissionActionMode::Dig
            ) {
                if ui.button("Dig trench").clicked() {
                    self.queue_mission_order(WorkOrderKind::DigTrench, WorkTarget::Cell(cell));
                }
                if ui.button("Flatten").clicked() {
                    self.queue_mission_order(WorkOrderKind::Flatten, WorkTarget::Cell(cell));
                }
            }
            if matches!(
                self.mission_action_mode,
                MissionActionMode::Inspect | MissionActionMode::Build
            ) && ui.button("Raise berm").clicked()
            {
                self.queue_mission_order(WorkOrderKind::RaiseBerm, WorkTarget::Cell(cell));
            }
            if matches!(
                self.mission_action_mode,
                MissionActionMode::Inspect | MissionActionMode::Deploy
            ) && ui.button("Place stakes").clicked()
            {
                self.queue_mission_order(WorkOrderKind::PlaceStakes, WorkTarget::Cell(cell));
            }
        });
        match self.mission_action_mode {
            MissionActionMode::Build => {
                self.show_order_preview_card(ui, WorkOrderKind::RaiseBerm, WorkTarget::Cell(cell));
            }
            MissionActionMode::Deploy => {
                self.show_order_preview_card(
                    ui,
                    WorkOrderKind::PlaceStakes,
                    WorkTarget::Cell(cell),
                );
            }
            MissionActionMode::Cancel => {
                ui.label("Use the queue controls to clear or run queued work.");
            }
            _ => {
                self.show_order_preview_card(ui, WorkOrderKind::DigTrench, WorkTarget::Cell(cell));
            }
        }
    }

    fn show_object_order_buttons(&mut self, ui: &mut egui::Ui, object: &EnvironmentObject) {
        match &object.kind {
            EnvironmentObjectKind::Tree(TreeState::Standing)
            | EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => {
                if matches!(
                    self.mission_action_mode,
                    MissionActionMode::Inspect | MissionActionMode::Harvest
                ) {
                    if ui.button("Fell tree").clicked() {
                        self.queue_mission_order(
                            WorkOrderKind::FellTree,
                            WorkTarget::Object(object.id.clone()),
                        );
                    }
                    self.show_order_preview_card(
                        ui,
                        WorkOrderKind::FellTree,
                        WorkTarget::Object(object.id.clone()),
                    );
                } else {
                    ui.label("Switch to Harvest to work this tree.");
                }
            }
            EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. }) => {
                if matches!(
                    self.mission_action_mode,
                    MissionActionMode::Inspect | MissionActionMode::Harvest
                ) {
                    if ui.button("Cut into logs").clicked() {
                        self.queue_mission_order(
                            WorkOrderKind::CutIntoLogs,
                            WorkTarget::Object(object.id.clone()),
                        );
                    }
                    self.show_order_preview_card(
                        ui,
                        WorkOrderKind::CutIntoLogs,
                        WorkTarget::Object(object.id.clone()),
                    );
                    if ui.button("Prepare roll").clicked() {
                        self.queue_mission_order(
                            WorkOrderKind::PrepareRollingLog,
                            WorkTarget::Object(object.id.clone()),
                        );
                    }
                    self.show_order_preview_card(
                        ui,
                        WorkOrderKind::PrepareRollingLog,
                        WorkTarget::Object(object.id.clone()),
                    );
                } else {
                    ui.label("Switch to Harvest to process this trunk.");
                }
            }
            EnvironmentObjectKind::Log(
                LogState::Loose { .. }
                | LogState::DragPrepared { .. }
                | LogState::Positioned { .. }
                | LogState::Braced { .. },
            ) => {
                if matches!(
                    self.mission_action_mode,
                    MissionActionMode::Inspect | MissionActionMode::Deploy
                ) {
                    if ui.button("Prepare roll").clicked() {
                        self.queue_mission_order(
                            WorkOrderKind::PrepareRollingLog,
                            WorkTarget::Object(object.id.clone()),
                        );
                    }
                    self.show_order_preview_card(
                        ui,
                        WorkOrderKind::PrepareRollingLog,
                        WorkTarget::Object(object.id.clone()),
                    );
                } else {
                    ui.label("Switch to Deploy to prepare this log as a rolling hazard.");
                }
            }
            EnvironmentObjectKind::Log(LogState::PreparedRoll { predicted_path, .. }) => {
                ui.label(format!(
                    "Prepared rolling hazard · predicted path {} cell(s). Use Hazards map mode or Release logs during assault.",
                    predicted_path.len()
                ));
            }
            _ => {
                ui.label("No context work orders for this object yet.");
            }
        }
    }

    fn show_order_preview_card(&self, ui: &mut egui::Ui, kind: WorkOrderKind, target: WorkTarget) {
        let preview = self.mission_state.preview_work_order(kind, target);
        ui.group(|ui| {
            ui.strong(format!("Preview: {}", preview.kind.label()));
            ui.label(format!(
                "Cost: {}s labor / {}s duration · crew {} · tools {}",
                preview.labor_seconds,
                preview.duration_seconds,
                preview.assigned_crews,
                preview
                    .required_tools
                    .iter()
                    .map(|tool| tool.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            let inputs = preview.material_inputs.signed_summary();
            let outputs = preview.material_outputs.signed_summary();
            if !inputs.is_empty() {
                ui.label(format!("Consumes: {}", inputs.join(", ")));
            }
            if !outputs.is_empty() {
                ui.label(format!("Creates: {}", outputs.join(", ")));
            }
            for note in &preview.preview.notes {
                ui.small(format!("Effect: {note}"));
            }
            if matches!(
                preview.status,
                ground_game::WorkOrderStatus::Rejected { .. }
            ) {
                ui.colored_label(
                    egui::Color32::from_rgb(230, 160, 120),
                    preview.status.label(),
                );
            }
        });
    }

    fn queue_mission_order(&mut self, kind: WorkOrderKind, target: WorkTarget) {
        if matches!(self.mission_state.phase, MissionPhase::Briefing) {
            self.notify("Start prep before queuing work orders.");
            return;
        }
        let order = self.mission_state.queue_work_order(kind, target);
        self.notify(format!(
            "Queued order #{:02}: {} · {}",
            order.id,
            order.kind.label(),
            order.status.label()
        ));
    }

    fn current_player_plan_script(&self) -> WorkOrderScript {
        let orders = self
            .mission_state
            .work_orders
            .iter()
            .chain(self.mission_state.work_queue.iter())
            .filter(|order| !matches!(order.status, WorkOrderStatus::Rejected { .. }))
            .map(|order| ScriptedWorkOrder {
                kind: order.kind,
                target: order.target.clone(),
            })
            .collect();
        WorkOrderScript {
            id: "road_below_player_plan".to_string(),
            label: "Road Below player prep plan".to_string(),
            orders,
        }
    }

    fn reset_to_prep_with_script(&mut self, script: &WorkOrderScript, run_all: bool) {
        let spec = self.mission_state.spec.clone();
        self.mission_state = MissionState::from_spec(spec);
        self.mission_state.phase = MissionPhase::Prep;
        self.selected_mission_cell = self.mission_state.spec.objective.defend_cell;
        for order in &script.orders {
            self.mission_state
                .queue_work_order(order.kind, order.target.clone());
        }
        if run_all {
            self.mission_state.run_all_queued_orders();
        }
    }

    fn tutorial_steps(&self) -> Vec<(bool, &'static str)> {
        let has_routes = self.route_overlay_mode != RouteOverlayMode::None;
        let has_earthwork = self.has_order_kind(WorkOrderKind::DigTrench)
            || self.has_order_kind(WorkOrderKind::RaiseBerm);
        let has_material_order = self.has_order_kind(WorkOrderKind::FellTree)
            || self.has_order_kind(WorkOrderKind::CutIntoLogs)
            || self.has_order_kind(WorkOrderKind::PlaceStakes);
        let has_log_plan = self.has_order_kind(WorkOrderKind::PrepareRollingLog)
            || !self.mission_state.rolling_hazard_plans().is_empty();
        let assault_started = self.mission_state.assault.is_some();
        let has_debrief = self.mission_state.assault_debrief().is_some();
        vec![
            (
                !matches!(self.mission_state.phase, MissionPhase::Briefing),
                "Start prep from the briefing.",
            ),
            (
                has_routes,
                "Preview enemy routes and choose one group to inspect.",
            ),
            (
                has_earthwork,
                "Change the ground with a trench or berm work order.",
            ),
            (
                has_material_order,
                "Use local material: fell, cut, or place stakes.",
            ),
            (
                has_log_plan,
                "Prepare the ridge log or decide to ignore it.",
            ),
            (assault_started, "Start, step, or run the assault."),
            (has_debrief, "Read the debrief and rating, then retry."),
        ]
    }

    fn has_order_kind(&self, kind: WorkOrderKind) -> bool {
        self.mission_state
            .work_orders
            .iter()
            .chain(self.mission_state.work_queue.iter())
            .any(|order| {
                order.kind == kind && !matches!(order.status, WorkOrderStatus::Rejected { .. })
            })
    }

    fn notify(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.status = message.clone();
        self.notifications.push(message);
        if self.notifications.len() > 12 {
            let excess = self.notifications.len() - 12;
            self.notifications.drain(0..excess);
        }
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
        ui.heading("Mission Prep");
        self.show_mission_command_bar(ui);
        ui.horizontal(|ui| {
            self.show_mission_minimap(ui);
            ui.label("Legend: S spawn, O objective, T tree, L logs/trunk, ^ stakes, = trench, # berm, : road.");
        });
        ui.add_space(8.0);
        let cell_size = 44.0;
        let map_w = self.mission_state.map.width as f32 * cell_size;
        let map_h = self.mission_state.map.height as f32 * cell_size;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(map_w, map_h), egui::Sense::click());
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - rect.min;
                let x = (local.x / cell_size).floor().max(0.0) as u32;
                let y = (local.y / cell_size).floor().max(0.0) as u32;
                if x < self.mission_state.map.width && y < self.mission_state.map.height {
                    self.selected_mission_cell = CellCoord::new(x, y);
                    self.notify(format!("Selected mission cell ({x}, {y})."));
                }
            }
        }
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
                painter.rect_filled(
                    tile_rect,
                    0.0,
                    mission_cell_color(cell, self.mission_map_mode),
                );
                if let Some(color) = self.assault_heat_color(coord) {
                    painter.rect_filled(tile_rect.shrink(3.0), 0.0, color);
                }
                painter.rect_stroke(
                    tile_rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_gray(45)),
                    egui::StrokeKind::Inside,
                );
                if self.selected_mission_cell == coord {
                    painter.rect_stroke(
                        tile_rect.shrink(2.0),
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(240, 220, 110)),
                        egui::StrokeKind::Inside,
                    );
                }
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
        self.draw_mission_route_overlay(&painter, rect, cell_size);
        self.draw_mission_assault_overlay(&painter, rect, cell_size);

        ui.add_space(8.0);
        ui.label(
            "Blue route = initial plan. Gold/red route = current terrain. Hazards show predicted rolling-log paths; delay/pressure/actual modes use assault timeline data.",
        );
    }

    fn show_mission_command_bar(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(self.mission_state.spec.title.as_str());
                ui.separator();
                ui.label(format!(
                    "phase {} · time {}",
                    self.mission_state.phase.label(),
                    format_duration(self.mission_state.remaining_prep_seconds)
                ));
                ui.separator();
                ui.label(format!(
                    "crew {} · labor {}",
                    self.mission_state.spec.crew.crews,
                    format_duration(self.mission_state.remaining_labor_seconds)
                ));
                ui.separator();
                ui.label(format!("mode {}", self.mission_action_mode.label()));
                ui.separator();
                ui.label(format!("map {}", self.mission_map_mode.label()));
            });
        });
    }

    fn show_mission_minimap(&self, ui: &mut egui::Ui) {
        let scale = 8.0;
        let size = egui::vec2(
            self.mission_state.map.width as f32 * scale,
            self.mission_state.map.height as f32 * scale,
        );
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        for y in 0..self.mission_state.map.height {
            for x in 0..self.mission_state.map.width {
                let coord = CellCoord::new(x, y);
                let cell = self
                    .mission_state
                    .map
                    .cell(coord)
                    .expect("mission minimap only reads in-bounds cells");
                let tile = egui::Rect::from_min_size(
                    egui::pos2(
                        rect.left() + x as f32 * scale,
                        rect.top() + y as f32 * scale,
                    ),
                    egui::vec2(scale - 1.0, scale - 1.0),
                );
                painter.rect_filled(tile, 0.0, mission_cell_color(cell, self.mission_map_mode));
                if self.mission_state.map.spawn_cells.contains(&coord) {
                    painter.rect_filled(
                        tile.shrink(2.0),
                        0.0,
                        egui::Color32::from_rgb(88, 140, 230),
                    );
                }
                if self.mission_state.spec.objective.defend_cell == coord {
                    painter.rect_filled(
                        tile.shrink(2.0),
                        0.0,
                        egui::Color32::from_rgb(232, 205, 86),
                    );
                }
                if self.selected_mission_cell == coord {
                    painter.rect_stroke(
                        tile,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(250, 235, 120)),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }

    fn assault_heat_color(&self, coord: CellCoord) -> Option<egui::Color32> {
        let assault = self.mission_state.assault.as_ref()?;
        match self.mission_map_mode {
            MissionMapMode::Delay => {
                let value: i32 = assault
                    .timeline
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.kind,
                            AssaultEventKind::DelayedByTerrain
                                | AssaultEventKind::DelayedByObstacle
                        ) && event.cell == Some(coord)
                    })
                    .map(|event| event.magnitude.max(1))
                    .sum();
                if value == 0 {
                    return None;
                }
                let max_value = assault
                    .timeline
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.kind,
                            AssaultEventKind::DelayedByTerrain
                                | AssaultEventKind::DelayedByObstacle
                        )
                    })
                    .map(|event| event.magnitude.max(1))
                    .max()
                    .unwrap_or(1);
                let alpha = heat_alpha(value, max_value);
                Some(egui::Color32::from_rgba_unmultiplied(236, 174, 62, alpha))
            }
            MissionMapMode::Pressure => {
                let value: i32 = assault
                    .timeline
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.kind,
                            AssaultEventKind::SuppressedByDefender
                                | AssaultEventKind::DamagedByDefender
                                | AssaultEventKind::DamagedByObstacle
                        ) && event.cell == Some(coord)
                    })
                    .map(|event| event.magnitude.max(1))
                    .sum();
                if value == 0 {
                    return None;
                }
                let max_value = assault
                    .timeline
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.kind,
                            AssaultEventKind::SuppressedByDefender
                                | AssaultEventKind::DamagedByDefender
                                | AssaultEventKind::DamagedByObstacle
                        )
                    })
                    .map(|event| event.magnitude.max(1))
                    .max()
                    .unwrap_or(1);
                let alpha = heat_alpha(value, max_value);
                Some(egui::Color32::from_rgba_unmultiplied(218, 72, 82, alpha))
            }
            MissionMapMode::Actual => {
                let crossings = assault
                    .agents
                    .iter()
                    .flat_map(|agent| {
                        let end = agent.route_index.min(agent.route.len().saturating_sub(1));
                        agent.route.iter().take(end + 1)
                    })
                    .filter(|cell| **cell == coord)
                    .count();
                if crossings == 0 {
                    None
                } else {
                    Some(egui::Color32::from_rgba_unmultiplied(232, 94, 70, 130))
                }
            }
            MissionMapMode::Terrain
            | MissionMapMode::Height
            | MissionMapMode::Cover
            | MissionMapMode::Resources
            | MissionMapMode::Hazards => None,
        }
    }

    fn draw_mission_route_overlay(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        cell_size: f32,
    ) {
        match self.route_overlay_mode {
            RouteOverlayMode::None => {}
            RouteOverlayMode::Initial => {
                let initial = MissionState::road_below_seed().route_preview();
                draw_route_set_on_mission(
                    painter,
                    rect,
                    cell_size,
                    &initial,
                    RouteOverlayMode::Initial,
                    self.route_group_filter,
                );
            }
            RouteOverlayMode::Current => {
                let current = self.mission_state.route_preview();
                draw_route_set_on_mission(
                    painter,
                    rect,
                    cell_size,
                    &current,
                    RouteOverlayMode::Current,
                    self.route_group_filter,
                );
            }
            RouteOverlayMode::Delta => {
                let initial = MissionState::road_below_seed().route_preview();
                let current = self.mission_state.route_preview();
                draw_route_set_on_mission(
                    painter,
                    rect,
                    cell_size,
                    &initial,
                    RouteOverlayMode::Initial,
                    self.route_group_filter,
                );
                draw_route_set_on_mission(
                    painter,
                    rect,
                    cell_size,
                    &current,
                    RouteOverlayMode::Current,
                    self.route_group_filter,
                );
            }
        }
    }

    fn draw_mission_assault_overlay(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        cell_size: f32,
    ) {
        for defender in &self.mission_state.spec.defender_positions {
            let pos = mission_route_point(rect, cell_size, defender.cell);
            painter.rect_filled(
                egui::Rect::from_center_size(pos, egui::vec2(12.0, 12.0)),
                0.0,
                egui::Color32::from_rgb(90, 172, 226),
            );
        }
        if self.mission_map_mode == MissionMapMode::Hazards {
            let hazards = self.mission_state.rolling_hazard_plans();
            for hazard in hazards {
                for window in hazard.path.windows(2) {
                    let a = mission_route_point(rect, cell_size, window[0].cell);
                    let b = mission_route_point(rect, cell_size, window[1].cell);
                    painter.line_segment(
                        [a, b],
                        egui::Stroke::new(
                            5.0,
                            egui::Color32::from_rgba_unmultiplied(236, 170, 64, 180),
                        ),
                    );
                }
                if let Some(first) = hazard.path.first() {
                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            mission_route_point(rect, cell_size, first.cell),
                            egui::vec2(12.0, 12.0),
                        ),
                        0.0,
                        egui::Color32::from_rgb(118, 72, 38),
                    );
                }
                if let Some(last) = hazard.path.last() {
                    painter.circle_filled(
                        mission_route_point(rect, cell_size, last.cell),
                        7.0,
                        egui::Color32::from_rgba_unmultiplied(238, 82, 64, 210),
                    );
                }
            }
        }
        let Some(assault) = &self.mission_state.assault else {
            return;
        };
        if self.mission_map_mode == MissionMapMode::Actual {
            for agent in &assault.agents {
                let end = agent.route_index.min(agent.route.len().saturating_sub(1));
                let points: Vec<_> = agent.route.iter().take(end + 1).copied().collect();
                for window in points.windows(2) {
                    let a = mission_route_point(rect, cell_size, window[0]);
                    let b = mission_route_point(rect, cell_size, window[1]);
                    painter.line_segment(
                        [a, b],
                        egui::Stroke::new(
                            3.0,
                            egui::Color32::from_rgba_unmultiplied(232, 94, 70, 115),
                        ),
                    );
                }
            }
        }
        for agent in &assault.agents {
            let pos = mission_route_point(rect, cell_size, agent.cell);
            let color = match agent.status {
                EnemyAgentStatus::Advancing => egui::Color32::from_rgb(220, 76, 60),
                EnemyAgentStatus::Delayed => egui::Color32::from_rgb(230, 146, 58),
                EnemyAgentStatus::Eliminated => egui::Color32::from_rgb(68, 70, 68),
                EnemyAgentStatus::ReachedObjective => egui::Color32::from_rgb(178, 72, 178),
            };
            painter.circle_filled(pos, 6.5, color);
            painter.circle_stroke(pos, 6.5, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
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

fn format_duration(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn route_filter_label(routes: &ground_game::DoctrineRouteSet, route_filter: usize) -> String {
    if route_filter == 0 {
        return "all groups".to_string();
    }
    routes
        .routes
        .get(route_filter.saturating_sub(1))
        .map(|route| route.group_label.clone())
        .unwrap_or_else(|| "all groups".to_string())
}

fn playable_road_below_state() -> MissionState {
    let mut state = MissionState::road_below_seed();
    state.phase = MissionPhase::Briefing;
    state
}

fn build_balance_dashboard() -> Vec<MissionBalanceDashboardRow> {
    let spec = ground_game::road_below_spec();
    road_below_balance_scripts()
        .into_iter()
        .filter_map(|script| {
            let mut state = run_work_order_script(spec.clone(), &script);
            let prep_time_used_seconds = state
                .spec
                .prep_time_seconds
                .saturating_sub(state.remaining_prep_seconds);
            let summary = state.run_assault_to_completion(160);
            let rating = mission_rating_for_state(&state)?;
            Some(MissionBalanceDashboardRow {
                label: script.label,
                stars: rating.stars,
                score: rating.score,
                outcome: rating.label,
                stopped: summary.enemies_eliminated,
                reached: summary.enemies_reached_objective,
                prep_time_used_seconds,
            })
        })
        .collect()
}

fn mission_cell_color(cell: &ground_game::MissionCell, mode: MissionMapMode) -> egui::Color32 {
    match mode {
        MissionMapMode::Terrain
        | MissionMapMode::Delay
        | MissionMapMode::Pressure
        | MissionMapMode::Actual
        | MissionMapMode::Hazards => match cell.earth_state {
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
        },
        MissionMapMode::Height => {
            let value = (72 + cell.height.max(0) as u8 * 34).min(220);
            egui::Color32::from_rgb(value / 2, value, 72)
        }
        MissionMapMode::Cover => match cell.cover {
            CoverClass::None => egui::Color32::from_rgb(62, 72, 58),
            CoverClass::Light => egui::Color32::from_rgb(91, 105, 66),
            CoverClass::Partial => egui::Color32::from_rgb(117, 126, 68),
            CoverClass::Strong => egui::Color32::from_rgb(148, 139, 78),
        },
        MissionMapMode::Resources => {
            if !cell.local_material.is_zero() {
                egui::Color32::from_rgb(156, 128, 66)
            } else if matches!(
                cell.earth_state,
                EarthState::Trench | EarthState::Berm | EarthState::SpoilPile
            ) {
                egui::Color32::from_rgb(118, 86, 50)
            } else {
                egui::Color32::from_rgb(63, 88, 54)
            }
        }
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

fn mission_object_state_label(object: &EnvironmentObject) -> &'static str {
    match object.kind {
        EnvironmentObjectKind::Tree(TreeState::Standing) => "standing tree",
        EnvironmentObjectKind::Tree(TreeState::PartiallyCut { .. }) => "partially cut tree",
        EnvironmentObjectKind::Tree(TreeState::Falling { .. }) => "falling tree",
        EnvironmentObjectKind::Tree(TreeState::FallenTrunk { .. }) => "fallen trunk",
        EnvironmentObjectKind::Tree(TreeState::CutLogs) => "cut logs",
        EnvironmentObjectKind::Tree(TreeState::StakesBundle) => "stakes bundle",
        EnvironmentObjectKind::Tree(TreeState::Stump) => "stump",
        EnvironmentObjectKind::Log(LogState::PreparedRoll { .. }) => "prepared rolling log",
        EnvironmentObjectKind::Log(LogState::Released { .. } | LogState::Rolling { .. }) => {
            "released rolling log"
        }
        EnvironmentObjectKind::Log(LogState::Spent { .. }) => "spent rolling log",
        EnvironmentObjectKind::Log(_) => "log",
        EnvironmentObjectKind::Rock(_) => "rock",
        EnvironmentObjectKind::Wall(_) => "wall",
        EnvironmentObjectKind::Wire(_) => "wire",
        EnvironmentObjectKind::Stakes(_) => "stakes",
        EnvironmentObjectKind::FightingPosition(_) => "fighting position",
    }
}

fn draw_route_set_on_mission(
    painter: &egui::Painter,
    rect: egui::Rect,
    cell_size: f32,
    routes: &ground_game::DoctrineRouteSet,
    mode: RouteOverlayMode,
    route_filter: usize,
) {
    for (index, route) in routes.routes.iter().enumerate() {
        if route_filter > 0 && route_filter != index + 1 {
            continue;
        }
        if route.points.is_empty() {
            continue;
        }
        let color = mission_route_color(index, mode);
        for window in route.points.windows(2) {
            let a = mission_route_point(rect, cell_size, window[0]);
            let b = mission_route_point(rect, cell_size, window[1]);
            painter.line_segment([a, b], egui::Stroke::new(5.0, color));
        }
        if let Some(first) = route.points.first().copied() {
            painter.circle_filled(
                mission_route_point(rect, cell_size, first),
                4.5,
                egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 230),
            );
        }
        if let Some(last) = route.points.last().copied() {
            painter.circle_filled(
                mission_route_point(rect, cell_size, last),
                5.5,
                egui::Color32::from_rgba_unmultiplied(245, 229, 124, 230),
            );
        }
    }
}

fn mission_route_point(rect: egui::Rect, cell_size: f32, cell: CellCoord) -> egui::Pos2 {
    egui::pos2(
        rect.left() + cell.x as f32 * cell_size + cell_size * 0.5,
        rect.top() + cell.y as f32 * cell_size + cell_size * 0.5,
    )
}

fn mission_route_color(index: usize, mode: RouteOverlayMode) -> egui::Color32 {
    let initial = [
        egui::Color32::from_rgba_unmultiplied(78, 142, 230, 140),
        egui::Color32::from_rgba_unmultiplied(96, 170, 238, 135),
        egui::Color32::from_rgba_unmultiplied(116, 190, 246, 135),
    ];
    let current = [
        egui::Color32::from_rgba_unmultiplied(238, 196, 78, 190),
        egui::Color32::from_rgba_unmultiplied(102, 210, 142, 185),
        egui::Color32::from_rgba_unmultiplied(226, 108, 86, 185),
    ];
    match mode {
        RouteOverlayMode::Initial => initial[index % initial.len()],
        RouteOverlayMode::Current | RouteOverlayMode::Delta => current[index % current.len()],
        RouteOverlayMode::None => egui::Color32::TRANSPARENT,
    }
}

fn heat_alpha(value: i32, max_value: i32) -> u8 {
    let scale = value.max(1) as f32 / max_value.max(1) as f32;
    (70.0 + 150.0 * scale.clamp(0.0, 1.0)).round() as u8
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
