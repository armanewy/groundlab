use eframe::egui;
use ground_core::{
    build_motif_heatmap, build_oblique_material_preview, build_palette_preview,
    build_path_autotile_sheet, build_path_mask_debug_preview, build_path_neighbor_seam_heatmap,
    build_path_preview_dense, build_path_preview_junctions, build_path_preview_loop,
    build_path_preview_random, build_path_preview_sparse, build_seam_heatmap,
    build_single_repeat_preview, build_sprite_contact_sheet, build_transition_edges_preview,
    build_transition_repeat_preview, build_trench_contact_sheet, build_trench_mask_debug_preview,
    build_trench_oblique_caps_preview, build_trench_oblique_corner_preview,
    build_trench_oblique_shadow_preview, build_trench_oblique_straight_preview,
    build_variant_repeat_preview, export_terrain_sprite_bundle, generate_terrain_sprites,
    scale_nearest, GeneratedTerrainSprite, PixelImage, TerrainSpriteKind, TerrainSpriteRecipe,
    BUILTIN_SPRITE_STYLE_PROFILES, DEFAULT_SPRITEGEN_EXPORT_DIR,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 820.0])
            .with_min_inner_size([860.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GroundLab Pixel Terrain Forge",
        options,
        Box::new(|cc| Ok(Box::new(SpriteForgeApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewPanel {
    Selected,
    ContactSheet,
    GrassSingleRepeat,
    GrassVariantRepeat,
    DirtSingleRepeat,
    DirtVariantRepeat,
    TransitionRepeat,
    TransitionEdges,
    PathAutotileSheet,
    PathRandomPreview,
    PathSparsePreview,
    PathDensePreview,
    PathLoopPreview,
    PathJunctionPreview,
    ObliqueMaterialPreview,
    TrenchContactSheet,
    TrenchStraightPreview,
    TrenchCapsPreview,
    TrenchCornerPreview,
    TrenchShadowPreview,
    TrenchMaskDebug,
    PathMaskDebug,
    PathNeighborSeam,
    SeamHeatmap,
    MotifHeatmap,
    Palette,
}

impl PreviewPanel {
    const ALL: [PreviewPanel; 26] = [
        PreviewPanel::Selected,
        PreviewPanel::ContactSheet,
        PreviewPanel::GrassSingleRepeat,
        PreviewPanel::GrassVariantRepeat,
        PreviewPanel::DirtSingleRepeat,
        PreviewPanel::DirtVariantRepeat,
        PreviewPanel::TransitionRepeat,
        PreviewPanel::TransitionEdges,
        PreviewPanel::PathAutotileSheet,
        PreviewPanel::PathRandomPreview,
        PreviewPanel::PathSparsePreview,
        PreviewPanel::PathDensePreview,
        PreviewPanel::PathLoopPreview,
        PreviewPanel::PathJunctionPreview,
        PreviewPanel::ObliqueMaterialPreview,
        PreviewPanel::TrenchContactSheet,
        PreviewPanel::TrenchStraightPreview,
        PreviewPanel::TrenchCapsPreview,
        PreviewPanel::TrenchCornerPreview,
        PreviewPanel::TrenchShadowPreview,
        PreviewPanel::TrenchMaskDebug,
        PreviewPanel::PathMaskDebug,
        PreviewPanel::PathNeighborSeam,
        PreviewPanel::SeamHeatmap,
        PreviewPanel::MotifHeatmap,
        PreviewPanel::Palette,
    ];

    fn label(self) -> &'static str {
        match self {
            PreviewPanel::Selected => "Selected tile",
            PreviewPanel::ContactSheet => "Contact sheet",
            PreviewPanel::GrassSingleRepeat => "Grass single repeat",
            PreviewPanel::GrassVariantRepeat => "Grass variant repeat",
            PreviewPanel::DirtSingleRepeat => "Dirt single repeat",
            PreviewPanel::DirtVariantRepeat => "Dirt variant repeat",
            PreviewPanel::TransitionRepeat => "Transition repeat",
            PreviewPanel::TransitionEdges => "Transition edges",
            PreviewPanel::PathAutotileSheet => "Path autotile sheet",
            PreviewPanel::PathRandomPreview => "Random path preview",
            PreviewPanel::PathSparsePreview => "Sparse path preview",
            PreviewPanel::PathDensePreview => "Dense path preview",
            PreviewPanel::PathLoopPreview => "Loop path preview",
            PreviewPanel::PathJunctionPreview => "Junction path preview",
            PreviewPanel::ObliqueMaterialPreview => "Oblique material preview",
            PreviewPanel::TrenchContactSheet => "Trench contact sheet",
            PreviewPanel::TrenchStraightPreview => "Trench straight preview",
            PreviewPanel::TrenchCapsPreview => "Trench caps preview",
            PreviewPanel::TrenchCornerPreview => "Trench corner preview",
            PreviewPanel::TrenchShadowPreview => "Trench shadow preview",
            PreviewPanel::TrenchMaskDebug => "Trench mask debug",
            PreviewPanel::PathMaskDebug => "Path mask debug",
            PreviewPanel::PathNeighborSeam => "Path neighbor seams",
            PreviewPanel::SeamHeatmap => "Seam heatmap",
            PreviewPanel::MotifHeatmap => "Motif heatmap",
            PreviewPanel::Palette => "Palette",
        }
    }
}

struct SpriteForgeApp {
    recipe: TerrainSpriteRecipe,
    selected_profile_index: usize,
    export_dir: String,
    sprites: Vec<GeneratedTerrainSprite>,
    selected_kind: TerrainSpriteKind,
    selected_index: usize,
    panel: PreviewPanel,
    zoom: f32,
    selected_texture: Option<egui::TextureHandle>,
    contact_texture: Option<egui::TextureHandle>,
    grass_single_texture: Option<egui::TextureHandle>,
    grass_variant_texture: Option<egui::TextureHandle>,
    dirt_single_texture: Option<egui::TextureHandle>,
    dirt_variant_texture: Option<egui::TextureHandle>,
    transition_repeat_texture: Option<egui::TextureHandle>,
    transition_edges_texture: Option<egui::TextureHandle>,
    path_autotile_texture: Option<egui::TextureHandle>,
    path_random_texture: Option<egui::TextureHandle>,
    path_sparse_texture: Option<egui::TextureHandle>,
    path_dense_texture: Option<egui::TextureHandle>,
    path_loop_texture: Option<egui::TextureHandle>,
    path_junction_texture: Option<egui::TextureHandle>,
    oblique_material_texture: Option<egui::TextureHandle>,
    trench_contact_texture: Option<egui::TextureHandle>,
    trench_straight_texture: Option<egui::TextureHandle>,
    trench_caps_texture: Option<egui::TextureHandle>,
    trench_corner_texture: Option<egui::TextureHandle>,
    trench_shadow_texture: Option<egui::TextureHandle>,
    trench_mask_debug_texture: Option<egui::TextureHandle>,
    path_mask_debug_texture: Option<egui::TextureHandle>,
    path_neighbor_seam_texture: Option<egui::TextureHandle>,
    seam_heatmap_texture: Option<egui::TextureHandle>,
    motif_heatmap_texture: Option<egui::TextureHandle>,
    palette_texture: Option<egui::TextureHandle>,
    dirty: bool,
    status: String,
}

impl SpriteForgeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            recipe: TerrainSpriteRecipe::from_default_style_profile(),
            selected_profile_index: 0,
            export_dir: DEFAULT_SPRITEGEN_EXPORT_DIR.to_string(),
            sprites: Vec::new(),
            selected_kind: TerrainSpriteKind::GrassTile,
            selected_index: 0,
            panel: PreviewPanel::Selected,
            zoom: 8.0,
            selected_texture: None,
            contact_texture: None,
            grass_single_texture: None,
            grass_variant_texture: None,
            dirt_single_texture: None,
            dirt_variant_texture: None,
            transition_repeat_texture: None,
            transition_edges_texture: None,
            path_autotile_texture: None,
            path_random_texture: None,
            path_sparse_texture: None,
            path_dense_texture: None,
            path_loop_texture: None,
            path_junction_texture: None,
            oblique_material_texture: None,
            trench_contact_texture: None,
            trench_straight_texture: None,
            trench_caps_texture: None,
            trench_corner_texture: None,
            trench_shadow_texture: None,
            trench_mask_debug_texture: None,
            path_mask_debug_texture: None,
            path_neighbor_seam_texture: None,
            seam_heatmap_texture: None,
            motif_heatmap_texture: None,
            palette_texture: None,
            dirty: true,
            status: "Ready.".to_string(),
        };
        app.refresh(&cc.egui_ctx);
        app
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        if !self.dirty {
            return;
        }
        self.recipe.sanitize();
        self.sprites = generate_terrain_sprites(&self.recipe);
        self.selected_index = self
            .sprites
            .iter()
            .position(|sprite| sprite.kind == self.selected_kind)
            .unwrap_or(0);
        self.refresh_textures(ctx);
        self.status = format!("Generated {} terrain sprites.", self.sprites.len());
        self.dirty = false;
    }

    fn refresh_textures(&mut self, ctx: &egui::Context) {
        if let Some(sprite) = self.sprites.get(self.selected_index) {
            put_texture(
                ctx,
                &mut self.selected_texture,
                "selected_sprite",
                &sprite.image,
            );
        }
        let contact = build_sprite_contact_sheet(&self.sprites, &self.recipe);
        put_texture(ctx, &mut self.contact_texture, "sprite_contact", &contact);
        let grass_single = build_single_repeat_preview(
            &self.sprites,
            TerrainSpriteKind::GrassTile,
            &self.recipe,
            5,
        );
        put_texture(
            ctx,
            &mut self.grass_single_texture,
            "grass_single_repeat",
            &grass_single,
        );
        let grass_variant = build_variant_repeat_preview(
            &self.sprites,
            TerrainSpriteKind::GrassTile,
            &self.recipe,
            5,
        );
        put_texture(
            ctx,
            &mut self.grass_variant_texture,
            "grass_variant_repeat",
            &grass_variant,
        );
        let dirt_single = build_single_repeat_preview(
            &self.sprites,
            TerrainSpriteKind::DirtTile,
            &self.recipe,
            5,
        );
        put_texture(
            ctx,
            &mut self.dirt_single_texture,
            "dirt_single_repeat",
            &dirt_single,
        );
        let dirt_variant = build_variant_repeat_preview(
            &self.sprites,
            TerrainSpriteKind::DirtTile,
            &self.recipe,
            5,
        );
        put_texture(
            ctx,
            &mut self.dirt_variant_texture,
            "dirt_variant_repeat",
            &dirt_variant,
        );
        let transition = build_transition_repeat_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.transition_repeat_texture,
            "transition_repeat",
            &transition,
        );
        let transition_edges = build_transition_edges_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.transition_edges_texture,
            "transition_edges",
            &transition_edges,
        );
        let path_autotile = build_path_autotile_sheet(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_autotile_texture,
            "path_autotile_sheet",
            &path_autotile,
        );
        let path_random = build_path_preview_random(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_random_texture,
            "path_random_preview",
            &path_random,
        );
        let path_sparse = build_path_preview_sparse(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_sparse_texture,
            "path_sparse_preview",
            &path_sparse,
        );
        let path_dense = build_path_preview_dense(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_dense_texture,
            "path_dense_preview",
            &path_dense,
        );
        let path_loop = build_path_preview_loop(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_loop_texture,
            "path_loop_preview",
            &path_loop,
        );
        let path_junction = build_path_preview_junctions(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_junction_texture,
            "path_junction_preview",
            &path_junction,
        );
        let oblique_material = build_oblique_material_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.oblique_material_texture,
            "oblique_material_preview",
            &oblique_material,
        );
        let trench_contact = build_trench_contact_sheet(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.trench_contact_texture,
            "trench_contact_sheet",
            &trench_contact,
        );
        let trench_straight = build_trench_oblique_straight_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.trench_straight_texture,
            "trench_straight_preview",
            &trench_straight,
        );
        let trench_caps = build_trench_oblique_caps_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.trench_caps_texture,
            "trench_caps_preview",
            &trench_caps,
        );
        let trench_corner = build_trench_oblique_corner_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.trench_corner_texture,
            "trench_corner_preview",
            &trench_corner,
        );
        let trench_shadow = build_trench_oblique_shadow_preview(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.trench_shadow_texture,
            "trench_shadow_preview",
            &trench_shadow,
        );
        let trench_mask_debug = build_trench_mask_debug_preview(&self.recipe);
        put_texture(
            ctx,
            &mut self.trench_mask_debug_texture,
            "trench_mask_debug",
            &trench_mask_debug,
        );
        let path_mask_debug = build_path_mask_debug_preview(&self.recipe);
        put_texture(
            ctx,
            &mut self.path_mask_debug_texture,
            "path_mask_debug",
            &path_mask_debug,
        );
        let path_neighbor_seam = build_path_neighbor_seam_heatmap(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.path_neighbor_seam_texture,
            "path_neighbor_seams",
            &path_neighbor_seam,
        );
        let seam_heatmap = build_seam_heatmap(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.seam_heatmap_texture,
            "seam_heatmap",
            &seam_heatmap,
        );
        let motif_heatmap = build_motif_heatmap(&self.sprites, &self.recipe);
        put_texture(
            ctx,
            &mut self.motif_heatmap_texture,
            "motif_heatmap",
            &motif_heatmap,
        );
        let palette = build_palette_preview(&self.recipe);
        put_texture(ctx, &mut self.palette_texture, "palette_preview", &palette);
    }

    fn show_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Pixel Terrain Forge");
        ui.label("ArtGen 2.0b: trench visual polish.");
        ui.separator();
        let selected_profile = BUILTIN_SPRITE_STYLE_PROFILES
            .get(self.selected_profile_index)
            .map(|(id, _)| *id)
            .unwrap_or("custom");
        egui::ComboBox::from_label("Style profile")
            .selected_text(selected_profile)
            .show_ui(ui, |ui| {
                for (index, (id, path)) in BUILTIN_SPRITE_STYLE_PROFILES.iter().enumerate() {
                    if ui
                        .selectable_value(&mut self.selected_profile_index, index, *id)
                        .clicked()
                    {
                        match TerrainSpriteRecipe::from_style_profile_path(path) {
                            Ok(recipe) => {
                                self.recipe = recipe;
                                self.dirty = true;
                                self.status = format!("Loaded style profile {id}.");
                                self.refresh(ctx);
                            }
                            Err(err) => {
                                self.status = format!("Failed to load style profile {id}: {err}");
                            }
                        }
                    }
                }
            });
        ui.label("Export directory");
        ui.text_edit_singleline(&mut self.export_dir);

        egui::ComboBox::from_label("Sprite")
            .selected_text(self.selected_kind.label())
            .show_ui(ui, |ui| {
                for kind in TerrainSpriteKind::ALL {
                    if ui
                        .selectable_value(&mut self.selected_kind, kind, kind.label())
                        .changed()
                    {
                        self.selected_index = self
                            .sprites
                            .iter()
                            .position(|sprite| sprite.kind == self.selected_kind)
                            .unwrap_or(0);
                        self.refresh_textures(ctx);
                    }
                }
            });
        egui::ComboBox::from_label("Preview")
            .selected_text(self.panel.label())
            .show_ui(ui, |ui| {
                for panel in PreviewPanel::ALL {
                    ui.selectable_value(&mut self.panel, panel, panel.label());
                }
            });

        ui.separator();
        let mut changed = false;
        changed |= ui
            .add(egui::DragValue::new(&mut self.recipe.seed).prefix("seed "))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.recipe.tile_size, 8..=32).text("tile size"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut self.recipe.variant_count, 1..=8).text("variants"))
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.display_scale, 2..=12)
                    .text("export scale"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.pixel.min_cluster_size, 1..=8)
                    .text("min cluster"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.pixel.max_cluster_size, 1..=10)
                    .text("max cluster"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.pixel.detail_density, 0.0..=1.0)
                    .text("detail"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut self.recipe.style.grass.blade_cluster_density,
                    0.0..=1.0,
                )
                .text("grass blades"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.grass.flower_density, 0.0..=0.08)
                    .text("flowers"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.dirt.pebble_density, 0.0..=0.25)
                    .text("pebbles"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.dirt.rut_density, 0.0..=0.30).text("ruts"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.transition.edge_jitter_px, 0..=8)
                    .text("edge jitter"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut self.recipe.style.transition.grass_intrusion_density,
                    0.0..=1.0,
                )
                .text("grass intrusion"),
            )
            .changed();
        ui.separator();
        ui.label("Oblique projection");
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.projection.cell_width_px, 32..=160)
                    .text("cell width"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.projection.cell_height_px, 24..=128)
                    .text("cell height"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.projection.face_height_px, 4..=64)
                    .text("face height"),
            )
            .changed();
        ui.separator();
        ui.label("Trench rules");
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.trench.floor_darkness, 0.0..=1.0)
                    .text("floor darkness"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.trench.wall_detail_density, 0.0..=1.0)
                    .text("wall detail"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.trench.wood_plank_density, 0.0..=1.0)
                    .text("wood planks"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut self.recipe.style.trench.spoil_density, 0.0..=1.0)
                    .text("spoil"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut self.recipe.style.trench.contact_shadow_strength,
                    0.0..=1.0,
                )
                .text("contact shadow"),
            )
            .changed();
        if changed {
            self.dirty = true;
        }
        ui.add(egui::Slider::new(&mut self.zoom, 2.0..=16.0).text("selected zoom"));

        ui.horizontal(|ui| {
            if ui.button("Regenerate").clicked() {
                self.dirty = true;
                self.refresh(ctx);
            }
            if ui.button("Export").clicked() {
                match export_terrain_sprite_bundle(&self.export_dir, &self.recipe) {
                    Ok(summary) => {
                        self.status = format!(
                            "Exported {} sprites to {} with {} validation issue(s).",
                            summary.sprite_count, summary.out_dir, summary.validation_issue_count
                        );
                    }
                    Err(err) => self.status = format!("Export failed: {err}"),
                }
            }
        });

        ui.separator();
        ui.label(&self.status);
    }

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        match self.panel {
            PreviewPanel::Selected => {
                if let Some(sprite) = self.sprites.get(self.selected_index) {
                    ui.label(format!(
                        "{} · {}x{}",
                        sprite.id, sprite.image.width, sprite.image.height
                    ));
                    let scaled = scale_nearest(&sprite.image, self.zoom.round() as u32);
                    put_temp_image(ui, &scaled);
                } else {
                    ui.label("No sprite");
                }
            }
            PreviewPanel::ContactSheet => {
                show_texture(ui, self.contact_texture.as_ref(), 1.0, "No contact sheet");
            }
            PreviewPanel::GrassSingleRepeat => {
                show_texture(
                    ui,
                    self.grass_single_texture.as_ref(),
                    1.0,
                    "No grass single repeat",
                );
            }
            PreviewPanel::GrassVariantRepeat => {
                show_texture(
                    ui,
                    self.grass_variant_texture.as_ref(),
                    1.0,
                    "No grass variant repeat",
                );
            }
            PreviewPanel::DirtSingleRepeat => {
                show_texture(
                    ui,
                    self.dirt_single_texture.as_ref(),
                    1.0,
                    "No dirt single repeat",
                );
            }
            PreviewPanel::DirtVariantRepeat => {
                show_texture(
                    ui,
                    self.dirt_variant_texture.as_ref(),
                    1.0,
                    "No dirt variant repeat",
                );
            }
            PreviewPanel::TransitionRepeat => {
                show_texture(
                    ui,
                    self.transition_repeat_texture.as_ref(),
                    1.0,
                    "No transition repeat",
                );
            }
            PreviewPanel::TransitionEdges => {
                show_texture(
                    ui,
                    self.transition_edges_texture.as_ref(),
                    1.0,
                    "No transition edge preview",
                );
            }
            PreviewPanel::PathAutotileSheet => {
                show_texture(
                    ui,
                    self.path_autotile_texture.as_ref(),
                    1.0,
                    "No path autotile sheet",
                );
            }
            PreviewPanel::PathRandomPreview => {
                show_texture(
                    ui,
                    self.path_random_texture.as_ref(),
                    1.0,
                    "No path preview",
                );
            }
            PreviewPanel::PathSparsePreview => {
                show_texture(
                    ui,
                    self.path_sparse_texture.as_ref(),
                    1.0,
                    "No sparse path preview",
                );
            }
            PreviewPanel::PathDensePreview => {
                show_texture(
                    ui,
                    self.path_dense_texture.as_ref(),
                    1.0,
                    "No dense path preview",
                );
            }
            PreviewPanel::PathLoopPreview => {
                show_texture(
                    ui,
                    self.path_loop_texture.as_ref(),
                    1.0,
                    "No loop path preview",
                );
            }
            PreviewPanel::PathJunctionPreview => {
                show_texture(
                    ui,
                    self.path_junction_texture.as_ref(),
                    1.0,
                    "No junction path preview",
                );
            }
            PreviewPanel::ObliqueMaterialPreview => {
                show_texture(
                    ui,
                    self.oblique_material_texture.as_ref(),
                    1.0,
                    "No oblique material preview",
                );
            }
            PreviewPanel::TrenchContactSheet => {
                show_texture(
                    ui,
                    self.trench_contact_texture.as_ref(),
                    1.0,
                    "No trench contact sheet",
                );
            }
            PreviewPanel::TrenchStraightPreview => {
                show_texture(
                    ui,
                    self.trench_straight_texture.as_ref(),
                    1.0,
                    "No trench straight preview",
                );
            }
            PreviewPanel::TrenchCapsPreview => {
                show_texture(
                    ui,
                    self.trench_caps_texture.as_ref(),
                    1.0,
                    "No trench caps preview",
                );
            }
            PreviewPanel::TrenchCornerPreview => {
                show_texture(
                    ui,
                    self.trench_corner_texture.as_ref(),
                    1.0,
                    "No trench corner preview",
                );
            }
            PreviewPanel::TrenchShadowPreview => {
                show_texture(
                    ui,
                    self.trench_shadow_texture.as_ref(),
                    1.0,
                    "No trench shadow preview",
                );
            }
            PreviewPanel::TrenchMaskDebug => {
                show_texture(
                    ui,
                    self.trench_mask_debug_texture.as_ref(),
                    1.0,
                    "No trench mask debug",
                );
            }
            PreviewPanel::PathMaskDebug => {
                show_texture(
                    ui,
                    self.path_mask_debug_texture.as_ref(),
                    1.0,
                    "No path mask debug preview",
                );
            }
            PreviewPanel::PathNeighborSeam => {
                show_texture(
                    ui,
                    self.path_neighbor_seam_texture.as_ref(),
                    1.0,
                    "No path neighbor seam heatmap",
                );
            }
            PreviewPanel::SeamHeatmap => {
                show_texture(
                    ui,
                    self.seam_heatmap_texture.as_ref(),
                    1.0,
                    "No seam heatmap",
                );
            }
            PreviewPanel::MotifHeatmap => {
                show_texture(
                    ui,
                    self.motif_heatmap_texture.as_ref(),
                    1.0,
                    "No motif heatmap",
                );
            }
            PreviewPanel::Palette => {
                show_texture(ui, self.palette_texture.as_ref(), 1.0, "No palette");
            }
        }
    }
}

impl eframe::App for SpriteForgeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.refresh(&ctx);

        egui::Panel::left("sprite_forge_controls")
            .resizable(true)
            .default_size(330.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_controls(ui, &ctx));
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| self.show_preview(ui));
        });
    }
}

fn put_temp_image(ui: &mut egui::Ui, image: &PixelImage) {
    let rgba = image.to_rgba_bytes();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(image.size(), &rgba);
    let texture = ui.ctx().load_texture(
        "selected_scaled_temp",
        color_image,
        egui::TextureOptions::NEAREST,
    );
    show_texture(ui, Some(&texture), 1.0, "No image");
}

fn put_texture(
    ctx: &egui::Context,
    handle: &mut Option<egui::TextureHandle>,
    name: &str,
    image: &PixelImage,
) {
    let rgba = image.to_rgba_bytes();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(image.size(), &rgba);
    if let Some(texture) = handle {
        texture.set(color_image, egui::TextureOptions::NEAREST);
    } else {
        *handle =
            Some(ctx.load_texture(name.to_string(), color_image, egui::TextureOptions::NEAREST));
    }
}

fn show_texture(
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
