use std::fs;
use std::path::PathBuf;

use eframe::egui;
use ground_core::{
    art_override_profile_path, derive_mutated_art_seed,
    export_art_contact_sheet as export_art_contact_sheet_file, export_art_lab_override_preview,
    export_art_lab_road_below_preview, export_art_variant_approved, generate_art_variants,
    load_art_lab_override_profile, promote_art_lab_art_pack, promoted_art_pack_profile_path,
    render_art_lab_override_preview, render_art_lab_road_below_preview,
    save_art_lab_override_profile, ArtLabOverrideAssignment, ArtLabOverrideRole, ArtSpriteFamily,
    ArtStyleControls, ArtVariant, ArtVariantBatch, ArtVariantMetadata, ArtVariantRequest,
    PixelImage,
};

use crate::{
    color_image_for_upload, put_texture, ApprovedArtSprite, ArtLabSessionSummary, ArtVariantReview,
    GroundLabApp, ART_REVIEW_TAGS,
};

const ART_PACK_0_1_ID: &str = "art_pack_0_1";
const ART_PACK_ASSETS_ROOT: &str = "assets/art_packs";
const ART_LAB_EXPORT_ROOT: &str = "exports/art_lab";

impl GroundLabApp {
    pub(crate) fn try_load_saved_art_override_profile_on_startup(&mut self, ctx: &egui::Context) {
        let path = art_override_profile_path(ART_LAB_EXPORT_ROOT);
        if !path.exists() {
            return;
        }
        self.load_saved_art_override_profile(ctx);
    }

    pub(crate) fn show_art_lab_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show_panel_tabs(ui);
        ui.heading("Art Lab");
        ui.label("Generate, compare, approve, assign, and preview sprites.");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            if ui.button("Generate").clicked() {
                self.generate_art_variants_for_lab(ctx);
            }
            if ui.button("Reroll").clicked() {
                self.reroll_art_seed();
                self.generate_art_variants_for_lab(ctx);
            }
            let can_mutate = self.selected_art_variant().is_some();
            if ui
                .add_enabled(can_mutate, egui::Button::new("Mutate selected"))
                .clicked()
            {
                self.mutate_selected_art_variant(ctx);
            }
            if ui
                .add_enabled(can_mutate, egui::Button::new("Approve + assign"))
                .clicked()
            {
                self.approve_assign_and_save_selected_art_variant_to_role();
                self.refresh_art_override_preview_texture(ctx);
            }
            if ui.button("Refresh preview").clicked() {
                self.refresh_art_override_preview(ctx);
            }
        });
        ui.separator();

        self.show_active_art_pack_panel(ui, ctx);
        ui.separator();

        ui.group(|ui| {
            ui.strong("Step 1 - Generate");
            ui.small(
                "Choose a family, reroll quickly, or lock style while exploring random batches.",
            );
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
            ui.checkbox(&mut self.art_lock_style, "Lock style");
            ui.horizontal_wrapped(|ui| {
                if ui.button("Reroll seed").clicked() {
                    self.reroll_art_seed();
                }
                if ui.button("Randomize style").clicked() {
                    self.randomize_art_style();
                }
                if ui.button("Generate random batch").clicked() {
                    self.generate_random_art_batch(ctx);
                }
                if ui.button("Reroll unpinned").clicked() {
                    self.reroll_unpinned_art_batch(ctx);
                }
                if ui.button("Reset style defaults").clicked() {
                    self.art_style = ArtStyleControls::default();
                    self.art_status = "Reset Art Lab style defaults.".to_string();
                }
            });
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
            egui::CollapsingHeader::new("Review / rubric")
                .default_open(false)
                .show(ui, |ui| {
                    self.show_art_review_panel(ui);
                });
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
            if ui.button("Load saved profile").clicked() {
                self.load_saved_art_override_profile(ctx);
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
                            if assignment.path.exists() {
                                ui.small(assignment.path.display().to_string());
                            } else {
                                ui.colored_label(
                                    egui::Color32::from_rgb(224, 128, 92),
                                    format!("missing file: {}", assignment.path.display()),
                                );
                            }
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

    fn show_active_art_pack_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.group(|ui| {
            ui.strong("Active Art Pack");
            let profile_path = art_override_profile_path(ART_LAB_EXPORT_ROOT);
            let required_count = ArtLabOverrideRole::REQUIRED.len();
            let assigned_required = self.required_art_role_assignment_count();
            let broken = self.art_broken_override_assignments();
            ui.small(format!("Profile: {}", profile_path.display()));
            ui.small(format!(
                "Core assignments: {assigned_required}/{required_count}"
            ));
            ui.small(format!(
                "Optional path kit assignments: {}/{}",
                self.art_path_kit_assignment_count(),
                ArtLabOverrideRole::PATH_KIT.len()
            ));
            let missing_roles = self.art_missing_override_roles();
            if missing_roles.is_empty() {
                ui.small("Missing core roles: none");
            } else {
                ui.small(format!("Missing core roles: {}", missing_roles.join(", ")));
            }
            if broken.is_empty() {
                ui.small("Broken assignment files: none");
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(224, 128, 92),
                    format!("Broken assignment files: {}", broken.len()),
                );
                for assignment in broken.iter().take(4) {
                    ui.small(format!(
                        "{} -> {}",
                        assignment.role.label(),
                        assignment.path.display()
                    ));
                }
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Load saved profile").clicked() {
                    self.load_saved_art_override_profile(ctx);
                }
                if ui.button("Load Art Pack 0.1").clicked() {
                    self.load_promoted_art_pack_0_1(ctx);
                }
                if ui.button("Save current profile").clicked() {
                    self.save_art_override_profile();
                }
                if ui.button("Refresh preview").clicked() {
                    self.refresh_art_override_preview(ctx);
                }
                if ui.button("Render Road Below preview").clicked() {
                    self.render_road_below_art_pack_preview(ctx);
                }
                if ui.button("Export pack summary").clicked() {
                    self.export_active_art_pack_summary();
                }
                if ui.button("Promote Art Pack 0.1").clicked() {
                    self.promote_active_art_pack_0_1(ctx);
                }
            });
            if PathBuf::from(ART_LAB_EXPORT_ROOT)
                .join(ART_PACK_0_1_ID)
                .join("art_pack_0_1_summary.json")
                .exists()
            {
                ui.small("Art Pack 0.1 summary detected.");
            }
            let promoted_path =
                promoted_art_pack_profile_path(ART_PACK_ASSETS_ROOT, ART_PACK_0_1_ID);
            if promoted_path.exists() {
                ui.small(format!("Promoted Art Pack: {}", promoted_path.display()));
            }
        });
    }

    pub(crate) fn show_art_lab_canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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

        self.show_pinned_art_variants(ui);

        let Some(batch) = &self.art_batch else {
            ui.label("No variant batch yet.");
            ui.small("Use Step 1 to generate deterministic sprite variants.");
            return;
        };
        if batch.variants.is_empty() {
            ui.label("No variants generated.");
            return;
        }
        let family_label = batch.request.family.label();
        let seed = batch.request.seed;
        let variants = batch.variants.clone();

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{} · seed {} · {} variant(s)",
                family_label,
                seed,
                variants.len()
            ));
        });
        ui.separator();

        let available_width = ui.available_width().max(180.0);
        let columns = ((available_width / 126.0).floor() as usize).clamp(2, 8);
        let thumb_size = (available_width / columns as f32 - 30.0).clamp(72.0, 112.0);
        let mut pin_toggle: Option<ArtVariant> = None;
        egui::Grid::new("art_lab_variant_grid")
            .num_columns(columns)
            .spacing([14.0, 14.0])
            .show(ui, |ui| {
                for (index, variant) in variants.iter().enumerate() {
                    ui.vertical(|ui| {
                        if let Some(texture) = self.art_variant_textures.get(index) {
                            let selected = self.art_selected_variant_index == Some(index);
                            let pinned = self.art_pinned_variant_ids.contains(&variant.id);
                            let frame = egui::Frame::new()
                                .fill(if selected {
                                    egui::Color32::from_rgb(55, 74, 50)
                                } else if pinned {
                                    egui::Color32::from_rgb(65, 54, 34)
                                } else {
                                    egui::Color32::from_rgb(28, 31, 28)
                                })
                                .stroke(egui::Stroke::new(
                                    if selected { 2.0 } else { 1.0 },
                                    if selected {
                                        egui::Color32::from_rgb(166, 214, 116)
                                    } else if pinned {
                                        egui::Color32::from_rgb(229, 181, 88)
                                    } else {
                                        egui::Color32::from_rgb(72, 78, 70)
                                    },
                                ))
                                .inner_margin(egui::Margin::same(6));
                            frame.show(ui, |ui| {
                                let size = egui::Vec2::splat(thumb_size);
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
                        let pinned = self.art_pinned_variant_ids.contains(&variant.id);
                        if ui
                            .small_button(if pinned { "Unpin" } else { "Pin" })
                            .clicked()
                        {
                            pin_toggle = Some(variant.clone());
                        }
                        ui.small(format!("#{:02}", variant.variant_index));
                    });
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        if let Some(variant) = pin_toggle {
            self.toggle_pinned_art_variant(ctx, &variant);
        }
    }

    fn show_pinned_art_variants(&self, ui: &mut egui::Ui) {
        if self.art_pinned_variants.is_empty() {
            return;
        }
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Kept variants");
                ui.small("Pinned candidates remain here while the main grid rerolls.");
            });
            egui::Grid::new("art_lab_pinned_grid")
                .num_columns(6)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    for (index, variant) in self.art_pinned_variants.iter().enumerate() {
                        ui.vertical(|ui| {
                            if let Some(texture) = self.art_pinned_textures.get(index) {
                                let sized = egui::load::SizedTexture::new(
                                    texture.id(),
                                    egui::Vec2::splat(64.0),
                                );
                                ui.add(
                                    egui::Image::from_texture(sized)
                                        .texture_options(egui::TextureOptions::NEAREST),
                                );
                            }
                            ui.small(short_art_id(&variant.id));
                        });
                        if (index + 1) % 6 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
        ui.separator();
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
                ui.label(
                    egui::RichText::new(format!("Selected: {}", short_art_id(&variant.id)))
                        .monospace(),
                )
                .on_hover_text(&variant.id);
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

    fn reroll_art_seed(&mut self) {
        self.art_seed = self.next_art_seed();
        self.art_status = format!("Rerolled seed to {}.", self.art_seed);
    }

    fn randomize_art_style(&mut self) {
        self.art_style = style_from_seed(self.next_art_seed());
        self.art_status = "Randomized style.".to_string();
    }

    fn generate_random_art_batch(&mut self, ctx: &egui::Context) {
        self.art_seed = self.next_art_seed();
        if !self.art_lock_style {
            self.art_style = style_from_seed(self.art_seed);
        }
        self.generate_art_variants_for_lab(ctx);
        self.art_status = if self.art_lock_style {
            format!(
                "Generated random batch with seed {} using locked style.",
                self.art_seed
            )
        } else {
            format!("Generated random batch with seed {}.", self.art_seed)
        };
    }

    fn reroll_unpinned_art_batch(&mut self, ctx: &egui::Context) {
        self.art_seed = self.next_art_seed();
        if !self.art_lock_style {
            self.art_style = style_from_seed(self.art_seed);
        }
        self.generate_art_variants_for_lab(ctx);
        self.art_status = format!(
            "Rerolled unpinned variants with seed {}; kept {} pinned candidate(s).",
            self.art_seed,
            self.art_pinned_variants.len()
        );
    }

    fn next_art_seed(&self) -> u64 {
        self.art_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407)
            ^ (self.art_family as u64).wrapping_mul(1_099_511_628_211)
            ^ (self.art_pinned_variants.len() as u64).wrapping_mul(97_531)
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

    fn toggle_pinned_art_variant(&mut self, ctx: &egui::Context, variant: &ArtVariant) {
        if self.art_pinned_variant_ids.remove(&variant.id) {
            if let Some(index) = self
                .art_pinned_variants
                .iter()
                .position(|pinned| pinned.id == variant.id)
            {
                self.art_pinned_variants.remove(index);
                let _ = self.art_pinned_textures.remove(index);
            }
            self.art_status = format!("Unpinned {}.", variant.id);
            return;
        }

        self.art_pinned_variant_ids.insert(variant.id.clone());
        self.art_pinned_textures.push(ctx.load_texture(
            format!("pinned_{}", variant.id),
            color_image_for_upload(&variant.image),
            egui::TextureOptions::NEAREST,
        ));
        self.art_pinned_variants.push(variant.clone());
        self.art_status = format!("Pinned {}.", variant.id);
    }

    fn export_selected_art_variant(&mut self) {
        self.approve_selected_art_variant();
    }

    fn approve_selected_art_variant(&mut self) -> Option<PathBuf> {
        let Some(variant) = self.selected_art_variant().cloned() else {
            self.art_status = "Select a variant before exporting.".to_string();
            return None;
        };
        match export_art_variant_approved(&variant, ART_LAB_EXPORT_ROOT) {
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
        match export_art_contact_sheet_file(batch, ART_LAB_EXPORT_ROOT) {
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
        match save_art_lab_override_profile(&self.art_override_profile, ART_LAB_EXPORT_ROOT) {
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
        match save_art_lab_override_profile(&self.art_override_profile, ART_LAB_EXPORT_ROOT) {
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

    fn load_saved_art_override_profile(&mut self, ctx: &egui::Context) {
        let path = art_override_profile_path(ART_LAB_EXPORT_ROOT);
        match load_art_lab_override_profile(&path) {
            Ok(profile) => {
                self.art_override_profile = profile;
                self.rebuild_approved_variants_from_profile();
                self.refresh_art_override_preview_texture(ctx);
                self.art_status = format!(
                    "Loaded saved Art Lab override profile with {} assignment(s).",
                    self.art_override_profile.assignments.len()
                );
            }
            Err(err) => {
                self.art_status = format!("Load saved profile failed: {err}");
            }
        }
    }

    fn load_promoted_art_pack_0_1(&mut self, ctx: &egui::Context) {
        let path = promoted_art_pack_profile_path(ART_PACK_ASSETS_ROOT, ART_PACK_0_1_ID);
        match load_art_lab_override_profile(&path) {
            Ok(profile) => {
                self.art_override_profile = profile;
                self.rebuild_approved_variants_from_profile();
                self.refresh_art_override_preview_texture(ctx);
                self.art_status = format!(
                    "Loaded Art Pack 0.1 with {} assignment(s).",
                    self.art_override_profile.assignments.len()
                );
            }
            Err(err) => {
                self.art_status = format!("Load Art Pack 0.1 failed: {err}");
            }
        }
    }

    fn promote_active_art_pack_0_1(&mut self, ctx: &egui::Context) {
        match promote_art_lab_art_pack(
            &self.art_override_profile,
            ART_PACK_0_1_ID,
            ART_PACK_ASSETS_ROOT,
            ART_LAB_EXPORT_ROOT,
        ) {
            Ok(summary) => match load_art_lab_override_profile(&summary.art_pack_path) {
                Ok(profile) => {
                    self.art_override_profile = profile;
                    self.rebuild_approved_variants_from_profile();
                    self.refresh_art_override_preview_texture(ctx);
                    self.art_preview_path = Some(summary.preview_path.clone());
                    self.art_status = format!(
                        "Promoted Art Pack 0.1 to {} with {}/{} core role(s) and {}/{} path kit role(s).",
                        summary.art_pack_path.display(),
                        summary.required_assignment_count,
                        summary.required_role_count,
                        summary.path_kit_assignment_count,
                        summary.path_kit_role_count
                    );
                }
                Err(err) => {
                    self.art_status = format!("Promoted Art Pack 0.1, but reload failed: {err}");
                }
            },
            Err(err) => {
                self.art_status = format!("Promote Art Pack 0.1 failed: {err}");
            }
        }
    }

    fn rebuild_approved_variants_from_profile(&mut self) {
        for assignment in &self.art_override_profile.assignments {
            if let Some(variant_id) = &assignment.variant_id {
                self.art_approved_variants
                    .insert(variant_id.clone(), assignment.path.clone());
            }
        }
    }

    fn save_art_override_profile(&mut self) {
        match save_art_lab_override_profile(&self.art_override_profile, ART_LAB_EXPORT_ROOT) {
            Ok(path) => {
                self.art_status = format!("Saved Art Lab override profile to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Save override profile failed: {err}");
            }
        }
    }

    fn required_art_role_assignment_count(&self) -> usize {
        ArtLabOverrideRole::REQUIRED
            .into_iter()
            .filter(|role| self.art_override_profile.assignment_path(*role).is_some())
            .count()
    }

    fn art_path_kit_assignment_count(&self) -> usize {
        ArtLabOverrideRole::PATH_KIT
            .into_iter()
            .filter(|role| self.art_override_profile.assignment_path(*role).is_some())
            .count()
    }

    fn art_broken_override_assignments(&self) -> Vec<&ArtLabOverrideAssignment> {
        self.art_override_profile
            .assignments
            .iter()
            .filter(|assignment| !assignment.path.exists())
            .collect()
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
        match export_art_lab_override_preview(&self.art_override_profile, ART_LAB_EXPORT_ROOT) {
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

    fn render_road_below_art_pack_preview(&mut self, ctx: &egui::Context) {
        let preview = render_art_lab_road_below_preview(&self.art_override_profile);
        put_texture(
            ctx,
            &mut self.art_preview_texture,
            "art_lab_road_below_preview",
            &preview,
        );
        match export_art_lab_road_below_preview(&self.art_override_profile, ART_LAB_EXPORT_ROOT) {
            Ok(path) => {
                self.art_preview_path = Some(path.clone());
                self.art_status =
                    format!("Rendered Road Below Art Pack preview to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Road Below Art Pack preview export failed: {err}");
            }
        }
    }

    fn art_missing_override_roles(&self) -> Vec<&'static str> {
        ArtLabOverrideRole::REQUIRED
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
        let root = PathBuf::from(ART_LAB_EXPORT_ROOT).join("approved");
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
        let dir = PathBuf::from(ART_LAB_EXPORT_ROOT).join("reviews");
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
        let path = PathBuf::from(ART_LAB_EXPORT_ROOT).join("session_summary.json");
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

    fn export_active_art_pack_summary(&mut self) {
        let path = PathBuf::from(ART_LAB_EXPORT_ROOT)
            .join(ART_PACK_0_1_ID)
            .join("art_pack_0_1_summary.json");
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                self.art_status = format!("Export pack summary failed: {err}");
                return;
            }
        }
        let missing_roles = self.art_missing_override_roles();
        let broken: Vec<_> = self
            .art_broken_override_assignments()
            .into_iter()
            .map(|assignment| {
                serde_json::json!({
                    "role": assignment.role,
                    "path": assignment.path,
                    "variant_id": assignment.variant_id.clone(),
                })
            })
            .collect();
        let summary = serde_json::json!({
            "id": "art_pack_0_1",
            "title": "Art Pack 0.1 - active Art Lab profile",
            "profile": art_override_profile_path(ART_LAB_EXPORT_ROOT),
            "assignment_count": self.art_override_profile.assignments.len(),
            "required_assignment_count": self.required_art_role_assignment_count(),
            "required_role_count": ArtLabOverrideRole::REQUIRED.len(),
            "path_kit_assignment_count": self.art_path_kit_assignment_count(),
            "path_kit_role_count": ArtLabOverrideRole::PATH_KIT.len(),
            "missing_required_roles": missing_roles,
            "broken_assignments": broken,
            "assignments": self.art_override_profile.assignments.clone(),
        });
        let result = (|| -> Result<(), String> {
            let json = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
            fs::write(&path, json).map_err(|err| err.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.art_status = format!("Exported active Art Pack summary to {}", path.display());
            }
            Err(err) => {
                self.art_status = format!("Export pack summary failed: {err}");
            }
        }
    }
}

fn short_art_id(id: &str) -> String {
    const MAX_LEN: usize = 30;
    if id.len() <= MAX_LEN {
        return id.to_string();
    }
    format!("{}...{}", &id[..18], &id[id.len() - 8..])
}

fn style_from_seed(seed: u64) -> ArtStyleControls {
    fn unit(seed: u64, shift: u32) -> f32 {
        (((seed.rotate_left(shift) ^ 0xa076_1d64_78bd_642f) >> 40) as f32 / ((1_u64 << 24) as f32))
            .clamp(0.0, 1.0)
    }

    ArtStyleControls {
        roughness: unit(seed, 5),
        contrast: unit(seed, 17),
        edge_emphasis: unit(seed, 29),
        noise: unit(seed, 41),
        warmth: unit(seed, 53),
    }
}
