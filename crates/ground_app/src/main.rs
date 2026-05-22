use eframe::egui;
use ground_core::{
    export_tileset_bundle, find_path, preview_pixel_to_cell, render_terrain_preview, Brush,
    BrushKind, GroundMaterial, LightDirection, PixelImage, PreviewMode, PreviewOptions, TerrainMap,
    Tileset, TilesetRecipe,
};

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

struct GroundLabApp {
    recipe: TilesetRecipe,
    tileset: Tileset,
    terrain: TerrainMap,
    preview_mode: PreviewMode,
    preview_options: PreviewOptions,
    brush: Brush,
    zoom: f32,
    show_contact_sheet: bool,
    contact_texture: Option<egui::TextureHandle>,
    preview_texture: Option<egui::TextureHandle>,
    dirty_assets: bool,
    dirty_preview: bool,
    last_preview_size: [usize; 2],
    status: String,
}

impl GroundLabApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let recipe = TilesetRecipe::default();
        let tileset = Tileset::generate(&recipe);
        let terrain = TerrainMap::demo(32, 24, recipe.seed);
        let mut app = Self {
            preview_options: PreviewOptions {
                show_grid: true,
                los_source: terrain.objective,
                los_range: 18,
                height_step_px: (recipe.tile_size / 4).max(4),
                fade_raised_faces: true,
            },
            recipe,
            tileset,
            terrain,
            preview_mode: PreviewMode::Material,
            brush: Brush::new(BrushKind::DigTrench, 1, 1),
            zoom: 1.0,
            show_contact_sheet: false,
            contact_texture: None,
            preview_texture: None,
            dirty_assets: true,
            dirty_preview: true,
            last_preview_size: [1, 1],
            status: "Ready. Paint terrain or tune the tile recipe.".to_string(),
        };
        app.refresh_if_dirty(&cc.egui_ctx);
        app
    }

    fn refresh_if_dirty(&mut self, ctx: &egui::Context) {
        if self.dirty_assets {
            self.recipe.sanitize();
            self.tileset = Tileset::generate(&self.recipe);
            let columns = self.recipe.variants_per_material.max(1);
            let contact = self.tileset.build_contact_sheet(columns, 2);
            put_texture(ctx, &mut self.contact_texture, "contact_sheet", &contact);
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

    fn show_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("GroundLab");
        ui.label("Internal custom terrain + pixel asset workbench");
        ui.separator();

        ui.strong("Asset recipe");
        let mut recipe_changed = false;

        egui::ComboBox::from_label("Tile size")
            .selected_text(format!("{} px", self.recipe.tile_size))
            .show_ui(ui, |ui| {
                for size in [16_u32, 24, 32, 48] {
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

        if recipe_changed {
            self.dirty_assets = true;
            self.status = "Recipe changed; regenerated tiles next frame.".to_string();
            ctx.request_repaint();
        }

        ui.separator();
        ui.strong("Terrain preview");
        egui::ComboBox::from_label("Overlay")
            .selected_text(self.preview_mode.label())
            .show_ui(ui, |ui| {
                for mode in PreviewMode::ALL {
                    if ui
                        .selectable_value(&mut self.preview_mode, mode, mode.label())
                        .changed()
                    {
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
                egui::Slider::new(&mut self.preview_options.height_step_px, 2..=24)
                    .text("2.5D height px"),
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        if ui
            .checkbox(
                &mut self.preview_options.fade_raised_faces,
                "Fade raised faces for inspection",
            )
            .changed()
        {
            self.dirty_preview = true;
        }
        ui.add(egui::Slider::new(&mut self.zoom, 0.4..=3.0).text("zoom"));
        ui.checkbox(&mut self.show_contact_sheet, "Show contact sheet");

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
        if ui.button("Reset demo terrain").clicked() {
            self.terrain = TerrainMap::demo(32, 24, self.recipe.seed);
            self.preview_options.los_source = self.terrain.objective;
            self.dirty_preview = true;
            self.status = "Terrain reset.".to_string();
        }
        if ui.button("Set LOS source to objective").clicked() {
            self.preview_options.los_source = self.terrain.objective;
            self.preview_mode = PreviewMode::LineOfSight;
            self.dirty_preview = true;
        }
        if ui.button("Export bundle").clicked() {
            self.refresh_if_dirty(ctx);
            match export_tileset_bundle(&self.tileset, &self.terrain, "exports/milestone_01") {
                Ok(()) => self.status = "Exported to exports/milestone_01".to_string(),
                Err(err) => self.status = format!("Export failed: {err}"),
            }
        }

        ui.separator();
        let path = find_path(&self.terrain, self.terrain.spawn, self.terrain.objective);
        ui.label(format!("Route cost: {:.1}", path.total_cost));
        ui.label(format!("Route reached objective: {}", path.reached_goal));
        ui.separator();
        ui.label(&self.status);
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.show_contact_sheet {
            if let Some(texture) = &self.contact_texture {
                let size = texture.size_vec2() * self.zoom;
                let sized = egui::load::SizedTexture::new(texture.id(), size);
                ui.add(
                    egui::Image::from_texture(sized).texture_options(egui::TextureOptions::NEAREST),
                );
            }
            return;
        }

        let Some(texture) = &self.preview_texture else {
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

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }

        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let local = pos - response.rect.min;
                let px = (local.x / self.zoom).floor().max(0.0) as u32;
                let py = (local.y / self.zoom).floor().max(0.0) as u32;
                if let Some((x, y)) = preview_pixel_to_cell(
                    &self.terrain,
                    &self.tileset,
                    self.preview_mode,
                    &self.preview_options,
                    px,
                    py,
                ) {
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
        }

        ui.label("Left-drag paints. Right-click sets LOS source. Blue = spawn, yellow = objective. The 2.5D view supports approximate hit-testing for elevated surfaces.");
    }
}

impl eframe::App for GroundLabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.refresh_if_dirty(&ctx);

        egui::Panel::left("groundlab_controls")
            .resizable(true)
            .default_size(340.0)
            .min_size(280.0)
            .max_size(460.0)
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
