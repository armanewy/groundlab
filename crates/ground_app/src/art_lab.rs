use std::fs;
use std::path::PathBuf;

use eframe::egui;
use ground_core::{
    derive_mutated_art_seed, export_art_contact_sheet as export_art_contact_sheet_file,
    export_art_lab_override_preview, export_art_variant_approved, generate_art_variants,
    render_art_lab_override_preview, save_art_lab_override_profile, ArtLabOverrideRole,
    ArtSpriteFamily, ArtVariantBatch, ArtVariantMetadata, ArtVariantRequest, PixelImage,
};

use crate::{
    color_image_for_upload, put_texture, ApprovedArtSprite, ArtLabSessionSummary, ArtVariantReview,
    GroundLabApp, ART_REVIEW_TAGS,
};

impl GroundLabApp {
    pub(crate) fn show_art_lab_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show_panel_tabs(ui);
        ui.heading("Art Lab");
        ui.label("Generate, compare, approve, assign, and preview sprites.");
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 1 - Generate");
            ui.small("Choose a family, tune the style, then generate a batch.");
            egui::ComboBox::from_id_salt("art_family_selector")
                .selected_text(self.art_family.label())
                .show_ui(ui, |ui| {
                    for family in ArtSpriteFamily::ALL {
                        ui.selectable_value(&mut self.art_family, family, family.label());
                    }
                });
            ui.add(egui::DragValue::new(&mut self.art_seed).prefix("seed "));
            ui.add(
                egui::DragValue::new(&mut self.art_variant_count)
                    .range(1..=64)
                    .prefix("variants "),
            );
            ui.label("Sprite size: 32x32");
            ui.separator();
            ui.strong("Style controls");
            ui.add(egui::Slider::new(&mut self.art_style.roughness, 0.0..=1.0).text("roughness"));
            ui.add(egui::Slider::new(&mut self.art_style.contrast, 0.0..=1.0).text("contrast"));
            ui.add(
                egui::Slider::new(&mut self.art_style.edge_emphasis, 0.0..=1.0)
                    .text("edge emphasis"),
            );
            ui.add(egui::Slider::new(&mut self.art_style.noise, 0.0..=1.0).text("noise"));
            ui.add(egui::Slider::new(&mut self.art_style.warmth, 0.0..=1.0).text("warmth"));
            ui.horizontal_wrapped(|ui| {
                if ui.button("Generate variants").clicked() {
                    self.generate_art_variants_for_lab(ctx);
                }
                if ui.button("Clear variants").clicked() {
                    self.art_batch = None;
                    self.art_variant_textures.clear();
                    self.art_selected_variant_index = None;
                    self.art_status = "Cleared Art Lab variants.".to_string();
                }
            });
        });
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 2 - Select / Inspect");
            ui.small("Click a thumbnail in the variant grid to choose the candidate you want to inspect.");
            self.show_selected_art_variant_inspector(ui);
            self.show_art_review_panel(ui);
        });
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 3 - Refine");
            ui.small("Generate a related batch around the selected variant.");
            if let Some(parent_id) = &self.art_batch_parent_id {
                ui.small(format!("Current batch is a mutation of {parent_id}."));
            }
            let can_mutate = self.selected_art_variant().is_some();
            if ui
                .add_enabled(can_mutate, egui::Button::new("Mutate selected"))
                .clicked()
            {
                self.mutate_selected_art_variant(ctx);
            }
            if !can_mutate {
                ui.small("Select a variant before mutating.");
            } else {
                ui.small("Mutation preserves family, size, and style controls while deriving a related seed.");
            }
        });
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 4 - Approve / Assign Role");
            ui.small("Export the selected sprite, then assign it to a preview role.");
            egui::ComboBox::from_id_salt("art_override_role_selector")
                .selected_text(self.art_override_role.label())
                .show_ui(ui, |ui| {
                    for role in ArtLabOverrideRole::ALL {
                        ui.selectable_value(&mut self.art_override_role, role, role.label());
                    }
                });
            ui.small(format!("Current role: {}", self.art_override_role.label()));
            if let Some(path) = self
                .art_override_profile
                .assignment_path(self.art_override_role)
            {
                ui.small(format!("Assigned: {}", path.display()));
            } else {
                ui.small("Assigned: none yet");
            }
            ui.horizontal_wrapped(|ui| {
                let can_assign = self.selected_art_variant().is_some();
                if ui
                    .add_enabled(can_assign, egui::Button::new("Approve selected"))
                    .clicked()
                {
                    self.export_selected_art_variant();
                }
                if ui
                    .add_enabled(
                        can_assign,
                        egui::Button::new("Assign selected variant to role"),
                    )
                    .clicked()
                {
                    self.assign_selected_art_variant_to_role();
                    self.refresh_art_override_preview_texture(ctx);
                }
                if ui
                    .add_enabled(can_assign, egui::Button::new("Approve + assign to role"))
                    .clicked()
                {
                    self.approve_assign_and_save_selected_art_variant_to_role();
                    self.refresh_art_override_preview_texture(ctx);
                }
            });
            if !self.art_override_profile.assignments.is_empty()
                && ui.button("Save override profile").clicked()
            {
                self.save_art_override_profile();
            }
            ui.small(format!(
                "{} assigned role(s)",
                self.art_override_profile.assignments.len()
            ));
            ui.separator();
            ui.strong("Current role assignments");
            egui::Grid::new("art_lab_role_assignments")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    ui.small("Role");
                    ui.small("Variant");
                    ui.small("Path");
                    ui.end_row();
                    for role in ArtLabOverrideRole::ALL {
                        let assignment = self
                            .art_override_profile
                            .assignments
                            .iter()
                            .find(|assignment| assignment.role == role);
                        ui.small(role.label());
                        if let Some(assignment) = assignment {
                            ui.small(assignment.variant_id.as_deref().unwrap_or("approved file"));
                            ui.small(assignment.path.display().to_string());
                        } else {
                            ui.small("missing");
                            ui.small("-");
                        }
                        ui.end_row();
                    }
                });
            ui.separator();
            self.show_art_compare_panel(ui, ctx);
            ui.separator();
            self.show_approved_art_gallery(ui, ctx);
        });
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 5 - Preview");
            ui.small("Render a small fixed scene with the current Art Lab role assignments.");
            ui.horizontal_wrapped(|ui| {
                if ui.button("Refresh preview").clicked() {
                    self.refresh_art_override_preview(ctx);
                }
                if ui.button("Export preview PNG").clicked() {
                    self.export_art_override_preview();
                }
                let can_sheet = self
                    .art_batch
                    .as_ref()
                    .is_some_and(|batch| !batch.variants.is_empty());
                if ui
                    .add_enabled(can_sheet, egui::Button::new("Export contact sheet"))
                    .clicked()
                {
                    self.export_art_contact_sheet();
                }
                if ui.button("Export session summary").clicked() {
                    self.export_art_session_summary();
                }
            });
            if let Some(path) = &self.art_preview_path {
                ui.small(format!("Preview export: {}", path.display()));
            }
            if let Some(path) = &self.art_contact_sheet_path {
                ui.small(format!("Contact sheet: {}", path.display()));
            }
            let missing_roles = self.art_missing_override_roles();
            if missing_roles.is_empty() {
                ui.small("All preview roles are assigned.");
            } else if self.art_override_profile.assignments.is_empty() {
                ui.small(
                    "No roles assigned yet. Assign a selected variant before judging the preview.",
                );
            } else {
                ui.small(format!("Missing roles: {}", missing_roles.join(", ")));
            }
        });
        ui.separator();

        ui.group(|ui| {
            ui.strong("Status");
            ui.label(&self.art_status);
        });
    }

    pub(crate) fn show_art_lab_canvas(&mut self, ui: &mut egui::Ui) {
        ui.heading("Art Lab Variants");
        if let Some(texture) = &self.art_preview_texture {
            ui.horizontal_wrapped(|ui| {
                ui.label("Art Lab override preview");
                ui.small("fixed scene using assigned roles");
            });
            let sized = egui::load::SizedTexture::new(texture.id(), texture.size_vec2() * 2.0);
            ui.add(egui::Image::from_texture(sized).texture_options(egui::TextureOptions::NEAREST));
            ui.separator();
        }

        let Some(batch) = &self.art_batch else {
            ui.label("No variant batch yet.");
            ui.small("Use Step 1 to generate deterministic sprite variants.");
            return;
        };
        if batch.variants.is_empty() {
            ui.label("No variants generated.");
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{} · seed {} · {} variant(s)",
                batch.request.family.label(),
                batch.request.seed,
                batch.variants.len()
            ));
        });
        ui.separator();

        egui::Grid::new("art_lab_variant_grid")
            .num_columns(4)
            .spacing([14.0, 14.0])
            .show(ui, |ui| {
                for (index, variant) in batch.variants.iter().enumerate() {
                    ui.vertical(|ui| {
                        if let Some(texture) = self.art_variant_textures.get(index) {
                            let selected = self.art_selected_variant_index == Some(index);
                            let frame = egui::Frame::new()
                                .fill(if selected {
                                    egui::Color32::from_rgb(55, 74, 50)
                                } else {
                                    egui::Color32::from_rgb(28, 31, 28)
                                })
                                .stroke(egui::Stroke::new(
                                    if selected { 2.0 } else { 1.0 },
                                    if selected {
                                        egui::Color32::from_rgb(166, 214, 116)
                                    } else {
                                        egui::Color32::from_rgb(72, 78, 70)
                                    },
                                ))
                                .inner_margin(egui::Margin::same(6));
                            frame.show(ui, |ui| {
                                let size = egui::Vec2::splat(96.0);
                                let sized = egui::load::SizedTexture::new(texture.id(), size);
                                let response = ui.add(
                                    egui::Image::from_texture(sized)
                                        .texture_options(egui::TextureOptions::NEAREST)
                                        .sense(egui::Sense::click()),
                                );
                                if response.clicked() {
                                    self.art_selected_variant_index = Some(index);
                                    self.art_status = format!("Selected {}", variant.id);
                                }
                            });
                        }
                        ui.small(format!("#{:02}", variant.variant_index));
                    });
                    if (index + 1) % 4 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn show_selected_art_variant_inspector(&self, ui: &mut egui::Ui) {
        let Some(index) = self.art_selected_variant_index else {
            ui.label("Generate variants and select one to inspect it.");
            return;
        };
        let Some(batch) = &self.art_batch else {
            ui.label("Generate variants and select one to inspect it.");
            return;
        };
        let Some(variant) = batch.variants.get(index) else {
            ui.label("Selected variant is no longer available.");
            return;
        };

        ui.horizontal(|ui| {
            if let Some(texture) = self.art_variant_textures.get(index) {
                let sized = egui::load::SizedTexture::new(texture.id(), egui::Vec2::splat(128.0));
                ui.add(
                    egui::Image::from_texture(sized).texture_options(egui::TextureOptions::NEAREST),
                );
            }
            ui.vertical(|ui| {
                ui.label(format!("Selected: {}", variant.id));
                ui.small(format!("Family: {}", variant.family.label()));
                ui.small(format!("Seed: {}", variant.seed));
                ui.small(format!("Variant index: {}", variant.variant_index));
                ui.small(format!(
                    "Size: {}x{}",
                    variant.image.width, variant.image.height
                ));
                if let Some(parent_id) = &variant.parent_id {
                    ui.small(format!("Parent: {parent_id}"));
                }
                ui.small(format!(
                    "Style: roughness {:.2} · contrast {:.2} · edge {:.2} · noise {:.2} · warmth {:.2}",
                    variant.style.roughness,
                    variant.style.contrast,
                    variant.style.edge_emphasis,
                    variant.style.noise,
                    variant.style.warmth
                ));
            });
        });

        if !variant.notes.is_empty() {
            ui.separator();
            ui.strong("Notes");
            for note in &variant.notes {
                ui.small(note);
            }
        }
    }

    fn show_art_review_panel(&mut self, ui: &mut egui::Ui) {
        let Some(variant) = self.selected_art_variant().cloned() else {
            return;
        };
        ui.separator();
        ui.strong("Review");
        let review = self
            .art_reviews
            .entry(variant.id.clone())
            .or_insert_with(|| ArtVariantReview {
                variant_id: variant.id.clone(),
                family: variant.family,
                seed: variant.seed,
                silhouette_readability: 3,
                color_material_read: 3,
                style_fit: 3,
                in_context_usefulness: 3,
                tags: Vec::new(),
                notes: String::new(),
            });
        for tag in ART_REVIEW_TAGS {
            let mut enabled = review.tags.iter().any(|existing| existing == tag);
            if ui.checkbox(&mut enabled, tag).changed() {
                if enabled {
                    review.tags.push(tag.to_string());
                    review.tags.sort();
                    review.tags.dedup();
                } else {
                    review.tags.retain(|existing| existing != tag);
                }
            }
        }
        ui.add(
            egui::Slider::new(&mut review.silhouette_readability, 1..=5)
                .text("silhouette/readability"),
        );
        ui.add(
            egui::Slider::new(&mut review.color_material_read, 1..=5).text("color/material read"),
        );
        ui.add(egui::Slider::new(&mut review.style_fit, 1..=5).text("style fit"));
        ui.add(
            egui::Slider::new(&mut review.in_context_usefulness, 1..=5)
                .text("in-context usefulness"),
        );
        ui.text_edit_singleline(&mut review.notes);
        if ui.button("Save review").clicked() {
            self.save_art_review_for_variant(&variant.id);
        }
    }

    fn show_art_compare_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.strong("Compare selected vs assigned role");
        let Some(selected_index) = self.art_selected_variant_index else {
            ui.small("Select a variant to compare it with the current role assignment.");
            return;
        };
        let Some(selected_texture_id) = self
            .art_variant_textures
            .get(selected_index)
            .map(|texture| texture.id())
        else {
            ui.small("Selected variant texture is not ready.");
            return;
        };
        let assignment = self
            .art_override_profile
            .assignments
            .iter()
            .find(|assignment| assignment.role == self.art_override_role)
            .cloned();
        let Some(assignment) = assignment else {
            ui.small("No assigned sprite for this role.");
            return;
        };
        self.refresh_art_compare_texture(ctx, &assignment.path);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.small("Selected candidate");
                let sized =
                    egui::load::SizedTexture::new(selected_texture_id, egui::Vec2::splat(96.0));
                ui.add(
                    egui::Image::from_texture(sized).texture_options(egui::TextureOptions::NEAREST),
                );
            });
            ui.vertical(|ui| {
                ui.small("Assigned role sprite");
                if let Some(texture) = &self.art_compare_texture {
                    let sized =
                        egui::load::SizedTexture::new(texture.id(), egui::Vec2::splat(96.0));
                    ui.add(
                        egui::Image::from_texture(sized)
                            .texture_options(egui::TextureOptions::NEAREST),
                    );
                } else {
                    ui.small("Could not load assigned PNG.");
                }
            });
        });
        if ui.button("Replace role assignment with selected").clicked() {
            self.approve_assign_and_save_selected_art_variant_to_role();
            self.refresh_art_override_preview_texture(ctx);
        }
    }

    fn show_approved_art_gallery(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.strong("Approved sprite gallery");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Refresh approved gallery").clicked() {
                self.refresh_approved_art_gallery(ctx);
            }
            if ui
                .add_enabled(
                    self.art_selected_gallery_index.is_some(),
                    egui::Button::new("Assign approved sprite to selected role"),
                )
                .clicked()
            {
                self.assign_selected_gallery_sprite_to_role();
                self.refresh_art_override_preview_texture(ctx);
            }
        });
        if self.art_gallery.is_empty() {
            ui.small("No approved sprites found under exports/art_lab/approved.");
            return;
        }
        egui::ScrollArea::vertical()
            .max_height(170.0)
            .show(ui, |ui| {
                egui::Grid::new("art_lab_gallery_grid")
                    .num_columns(4)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for index in 0..self.art_gallery.len() {
                            let selected = self.art_selected_gallery_index == Some(index);
                            let sprite = &self.art_gallery[index];
                            ui.vertical(|ui| {
                                if let Some(texture) = self.art_gallery_textures.get(index) {
                                    let sized = egui::load::SizedTexture::new(
                                        texture.id(),
                                        egui::Vec2::splat(48.0),
                                    );
                                    let response = ui.add(
                                        egui::Image::from_texture(sized)
                                            .texture_options(egui::TextureOptions::NEAREST)
                                            .sense(egui::Sense::click()),
                                    );
                                    if response.clicked() {
                                        self.art_selected_gallery_index = Some(index);
                                    }
                                }
                                ui.small(if selected { "selected" } else { "" });
                                ui.small(sprite.metadata.family.label());
                            });
                            if (index + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        if let Some(index) = self.art_selected_gallery_index {
            if let Some(sprite) = self.art_gallery.get(index) {
                ui.separator();
                ui.small(format!("Approved: {}", sprite.metadata.id));
                ui.small(format!("Family: {}", sprite.metadata.family.label()));
                ui.small(format!("PNG: {}", sprite.png_path.display()));
                ui.small(format!("Metadata: {}", sprite.json_path.display()));
            }
        }
    }

    fn generate_art_variants_for_lab(&mut self, ctx: &egui::Context) {
        let request = ArtVariantRequest {
            family: self.art_family,
            seed: self.art_seed,
            count: self.art_variant_count,
            width: 32,
            height: 32,
            style: self.art_style,
            parent_id: None,
        };
        let batch = generate_art_variants(&request);
        self.load_art_variant_batch(ctx, batch, "Generated");
    }

    fn mutate_selected_art_variant(&mut self, ctx: &egui::Context) {
        let Some(parent) = self.selected_art_variant().cloned() else {
            self.art_status = "Select a variant before mutating.".to_string();
            return;
        };
        let seed = derive_mutated_art_seed(&parent);
        self.art_family = parent.family;
        self.art_seed = seed;
        self.art_style = parent.style;
        let request = ArtVariantRequest {
            family: parent.family,
            seed,
            count: self.art_variant_count,
            width: parent.image.width,
            height: parent.image.height,
            style: parent.style,
            parent_id: Some(parent.id.clone()),
        };
        let batch = generate_art_variants(&request);
        self.load_art_variant_batch(ctx, batch, &format!("Mutated from {}", parent.id));
    }

    fn load_art_variant_batch(
        &mut self,
        ctx: &egui::Context,
        batch: ArtVariantBatch,
        action_label: &str,
    ) {
        self.art_variant_textures.clear();
        for variant in &batch.variants {
            let color_image = color_image_for_upload(&variant.image);
            self.art_variant_textures.push(ctx.load_texture(
                variant.id.clone(),
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }
        self.art_selected_variant_index = batch.variants.first().map(|_| 0);
        self.art_batch_parent_id = batch.request.parent_id.clone();
        self.art_status = format!(
            "{} {} {} variant(s).",
            action_label,
            batch.variants.len(),
            batch.request.family.label()
        );
        self.art_batch = Some(batch);
    }

    fn selected_art_variant(&self) -> Option<&ground_core::ArtVariant> {
        let index = self.art_selected_variant_index?;
        self.art_batch.as_ref()?.variants.get(index)
    }

    fn export_selected_art_variant(&mut self) {
        self.approve_selected_art_variant();
    }

    fn approve_selected_art_variant(&mut self) -> Option<PathBuf> {
        let Some(variant) = self.selected_art_variant().cloned() else {
            self.art_status = "Select a variant before exporting.".to_string();
            return None;
        };
        match export_art_variant_approved(&variant, "exports/art_lab") {
            Ok((png_path, _json_path)) => {
                self.art_approved_variants
                    .insert(variant.id.clone(), png_path.clone());
                self.art_status = format!("Approved {} to {}", variant.id, png_path.display());
                Some(png_path)
            }
            Err(err) => {
                self.art_status = format!("Approve selected failed: {err}");
                None
            }
        }
    }

    fn export_art_contact_sheet(&mut self) {
        let Some(batch) = &self.art_batch else {
            self.art_status = "Generate variants before exporting a contact sheet.".to_string();
            return;
        };
        match export_art_contact_sheet_file(batch, "exports/art_lab") {
            Ok(path) => {
                self.art_contact_sheet_path = Some(path.clone());
                self.art_status = format!("Exported contact sheet to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Export contact sheet failed: {err}");
            }
        }
    }

    fn assign_selected_art_variant_to_role(&mut self) {
        let role = self.art_override_role;
        let Some(variant) = self.selected_art_variant().cloned() else {
            self.art_status = "Select a variant before assigning it to a role.".to_string();
            return;
        };
        let png_path = self
            .art_approved_variants
            .get(&variant.id)
            .cloned()
            .or_else(|| self.approve_selected_art_variant());
        if let Some(png_path) = png_path {
            self.art_override_profile.set_assignment(
                role,
                png_path.clone(),
                Some(variant.id.clone()),
            );
            self.art_status = format!(
                "Assigned {} to {} using {}",
                variant.id,
                role.label(),
                png_path.display()
            );
        }
    }

    fn assign_selected_gallery_sprite_to_role(&mut self) {
        let role = self.art_override_role;
        let Some(index) = self.art_selected_gallery_index else {
            self.art_status = "Select an approved sprite before assigning it.".to_string();
            return;
        };
        let Some(sprite) = self.art_gallery.get(index).cloned() else {
            self.art_status = "Selected approved sprite is no longer available.".to_string();
            return;
        };
        self.art_override_profile.set_assignment(
            role,
            sprite.png_path.clone(),
            Some(sprite.metadata.id.clone()),
        );
        self.art_approved_variants
            .insert(sprite.metadata.id.clone(), sprite.png_path.clone());
        match save_art_lab_override_profile(&self.art_override_profile, "exports/art_lab") {
            Ok(profile_path) => {
                self.art_status = format!(
                    "Assigned approved {} to {}; saved profile to {}",
                    sprite.metadata.id,
                    role.label(),
                    profile_path.display()
                );
            }
            Err(err) => {
                self.art_status =
                    format!("Assigned approved sprite, but profile save failed: {err}");
            }
        }
    }

    fn approve_assign_and_save_selected_art_variant_to_role(&mut self) {
        let role = self.art_override_role;
        let Some(variant) = self.selected_art_variant().cloned() else {
            self.art_status = "Select a variant before approving and assigning.".to_string();
            return;
        };
        let Some(png_path) = self
            .art_approved_variants
            .get(&variant.id)
            .cloned()
            .or_else(|| self.approve_selected_art_variant())
        else {
            return;
        };
        self.art_override_profile
            .set_assignment(role, png_path, Some(variant.id.clone()));
        match save_art_lab_override_profile(&self.art_override_profile, "exports/art_lab") {
            Ok(profile_path) => {
                self.art_status = format!(
                    "Approved + assigned {} to {}; saved profile to {}",
                    variant.id,
                    role.label(),
                    profile_path.display()
                );
            }
            Err(err) => {
                self.art_status = format!("Approved + assigned, but profile save failed: {err}");
            }
        }
    }

    fn save_art_override_profile(&mut self) {
        match save_art_lab_override_profile(&self.art_override_profile, "exports/art_lab") {
            Ok(path) => {
                self.art_status = format!("Saved Art Lab override profile to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Save override profile failed: {err}");
            }
        }
    }

    fn refresh_art_override_preview_texture(&mut self, ctx: &egui::Context) {
        let preview = render_art_lab_override_preview(&self.art_override_profile);
        put_texture(
            ctx,
            &mut self.art_preview_texture,
            "art_lab_override_preview",
            &preview,
        );
    }

    fn refresh_art_override_preview(&mut self, ctx: &egui::Context) {
        self.refresh_art_override_preview_texture(ctx);
        self.art_status = "Refreshed Art Lab override preview.".to_string();
    }

    fn export_art_override_preview(&mut self) {
        match export_art_lab_override_preview(&self.art_override_profile, "exports/art_lab") {
            Ok(path) => {
                self.art_preview_path = Some(path.clone());
                self.art_status =
                    format!("Exported Art Lab override preview to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Export override preview failed: {err}");
            }
        }
    }

    fn art_missing_override_roles(&self) -> Vec<&'static str> {
        ArtLabOverrideRole::ALL
            .into_iter()
            .filter(|role| self.art_override_profile.assignment_path(*role).is_none())
            .map(ArtLabOverrideRole::label)
            .collect()
    }

    fn refresh_art_compare_texture(&mut self, ctx: &egui::Context, path: &PathBuf) {
        if self.art_compare_texture_path.as_ref() == Some(path)
            && self.art_compare_texture.is_some()
        {
            return;
        }
        match PixelImage::load_png(path) {
            Ok(image) => {
                put_texture(
                    ctx,
                    &mut self.art_compare_texture,
                    "art_lab_compare_role",
                    &image,
                );
                self.art_compare_texture_path = Some(path.clone());
            }
            Err(_) => {
                self.art_compare_texture = None;
                self.art_compare_texture_path = Some(path.clone());
            }
        }
    }

    fn refresh_approved_art_gallery(&mut self, ctx: &egui::Context) {
        let root = PathBuf::from("exports/art_lab/approved");
        let mut sprites = Vec::new();
        let mut textures = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                if path.file_name().and_then(|name| name.to_str()) == Some("art_lab_overrides.json")
                {
                    continue;
                }
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(metadata) = serde_json::from_str::<ArtVariantMetadata>(&text) else {
                    continue;
                };
                let png_path = path.with_extension("png");
                let Ok(image) = PixelImage::load_png(&png_path) else {
                    continue;
                };
                textures.push(ctx.load_texture(
                    format!("approved_{}", metadata.id),
                    color_image_for_upload(&image),
                    egui::TextureOptions::NEAREST,
                ));
                sprites.push(ApprovedArtSprite {
                    metadata,
                    png_path,
                    json_path: path,
                });
            }
        }
        sprites.sort_by(|a, b| {
            a.metadata
                .family
                .slug()
                .cmp(b.metadata.family.slug())
                .then_with(|| a.metadata.id.cmp(&b.metadata.id))
        });
        self.art_gallery = sprites;
        self.art_gallery_textures = textures;
        self.art_selected_gallery_index = self
            .art_selected_gallery_index
            .filter(|index| *index < self.art_gallery.len());
        self.art_status = format!(
            "Loaded {} approved Art Lab sprite(s).",
            self.art_gallery.len()
        );
    }

    fn save_art_review_for_variant(&mut self, variant_id: &str) {
        let Some(review) = self.art_reviews.get(variant_id) else {
            self.art_status = "No review is available for the selected variant.".to_string();
            return;
        };
        let dir = PathBuf::from("exports/art_lab/reviews");
        if let Err(err) = fs::create_dir_all(&dir) {
            self.art_status = format!("Save review failed: {err}");
            return;
        }
        let path = dir.join(format!("{variant_id}.json"));
        let result = (|| -> Result<(), String> {
            let json = serde_json::to_string_pretty(review).map_err(|err| err.to_string())?;
            fs::write(&path, json).map_err(|err| err.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.art_status = format!("Saved review to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Save review failed: {err}");
            }
        }
    }

    fn export_art_session_summary(&mut self) {
        let selected_variant_id = self
            .selected_art_variant()
            .map(|variant| variant.id.clone());
        let summary = ArtLabSessionSummary {
            current_family: self.art_family,
            current_seed: self.art_seed,
            current_variant_count: self.art_variant_count,
            current_style: self.art_style,
            selected_variant_id,
            current_override_role: self.art_override_role,
            override_profile: self.art_override_profile.clone(),
            approved_variants: self.art_approved_variants.clone(),
            latest_preview_path: self.art_preview_path.clone(),
            latest_contact_sheet_path: self.art_contact_sheet_path.clone(),
        };
        let path = PathBuf::from("exports/art_lab/session_summary.json");
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                self.art_status = format!("Export session summary failed: {err}");
                return;
            }
        }
        let result = (|| -> Result<(), String> {
            let json = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
            fs::write(&path, json).map_err(|err| err.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.art_status = format!("Exported session summary to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Export session summary failed: {err}");
            }
        }
    }
}
