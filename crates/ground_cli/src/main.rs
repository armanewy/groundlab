use anyhow::{bail, Result};
use ground_core::{
    build_art_variant_contact_sheet, ensure_default_asset_files, export_art_variant_batch,
    export_edit_scenario_suite, export_tileset_bundle_with_palette, generate_art_variants,
    load_workbench_assets, parse_art_variant_cli, TerrainArtKit, TerrainMap, WorkbenchAssetPaths,
    DEFAULT_PALETTE_PATH, DEFAULT_RECIPE_PATH,
};
use ground_game::{
    export_assault_run, export_generated_campaign_set,
    export_generated_campaign_set_playtest_from_file, export_generated_campaign_set_quality_gate,
    export_generated_mission_batch, export_generated_mission_pack_playtest_from_file,
    export_generated_mission_pack_quality_gate, export_generated_mission_pack_with_curve,
    export_generated_mission_theme_batch, export_hazard_sandbox_run, export_mission_balance_run,
    export_mission_visuals, export_order_script_run, export_road_below_seed,
    export_theme_calibration_report, export_visual_lock_art_acceptance_gate,
    export_visual_lock_benchmark, export_visual_lock_theme_consistency, load_mission_spec,
    load_work_order_script, road_below_basic_prep_script, road_below_hazard_prep_script,
    road_below_spec, MissionGeneratorSpec, MissionPackCurve, MissionTheme,
    DEFAULT_MISSION_EXPORT_DIR,
};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "export" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/milestone_04_12".to_string());
            let recipe_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_RECIPE_PATH.to_string());
            let palette_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_PALETTE_PATH.to_string());
            let paths = WorkbenchAssetPaths {
                recipe_path: recipe_path.into(),
                palette_path: palette_path.into(),
            };
            ensure_default_asset_files(&paths)?;
            let loaded = load_workbench_assets(&paths)?;
            let terrain = TerrainMap::target_derived(16, 12, loaded.recipe.seed);
            export_tileset_bundle_with_palette(
                &loaded.tileset,
                &loaded.palette,
                &terrain,
                out_dir,
            )?;
            println!("Exported GroundLab Milestone 4.12 bundle.");
            println!("{}", loaded.validation.summary_line());
            let artkit_validation =
                TerrainArtKit::load_default_or_generate(&loaded.tileset).validate();
            println!(
                "art kit: {} pieces, {} issue(s)",
                artkit_validation.present_piece_count,
                artkit_validation.issues.len()
            );
        }
        "edit-scenarios" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/milestone_04_12/edit_scenarios".to_string());
            let recipe_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_RECIPE_PATH.to_string());
            let palette_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_PALETTE_PATH.to_string());
            let paths = WorkbenchAssetPaths {
                recipe_path: recipe_path.into(),
                palette_path: palette_path.into(),
            };
            ensure_default_asset_files(&paths)?;
            let loaded = load_workbench_assets(&paths)?;
            let terrain = TerrainMap::target_derived(16, 12, loaded.recipe.seed);
            export_edit_scenario_suite(&loaded.tileset, &terrain, out_dir)?;
            println!("Exported GroundLab Milestone 4.12 edit scenario suite.");
        }
        "validate" => {
            let recipe_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_RECIPE_PATH.to_string());
            let palette_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_PALETTE_PATH.to_string());
            let paths = WorkbenchAssetPaths {
                recipe_path: recipe_path.into(),
                palette_path: palette_path.into(),
            };
            ensure_default_asset_files(&paths)?;
            let loaded = load_workbench_assets(&paths)?;
            println!("{}", loaded.validation.summary_line());
            let artkit_validation =
                TerrainArtKit::load_default_or_generate(&loaded.tileset).validate();
            println!(
                "art kit: {} required, {} present, {} issue(s)",
                artkit_validation.required_piece_count,
                artkit_validation.present_piece_count,
                artkit_validation.issues.len()
            );
            for issue in loaded.validation.issues.iter().take(32) {
                println!(
                    "{} · {} · {}{}",
                    issue.severity.label(),
                    issue.category,
                    issue.message,
                    issue
                        .metric
                        .map(|m| format!(" ({m:.1})"))
                        .unwrap_or_default()
                );
            }
            if loaded.validation.issues.len() > 32 {
                println!("… plus {} more", loaded.validation.issues.len() - 32);
            }
        }
        "mission-seed" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| DEFAULT_MISSION_EXPORT_DIR.to_string());
            export_road_below_seed(&out_dir)?;
            println!("Exported GamePivot 5.1 mission seed to {out_dir}.");
            println!("Mission: The Road Below");
            println!(
                "Files: mission_spec.ron/json, order_script.ron/json, mission_before.json, mission_after.json, scripted_work_orders.json, material_ledger.json, order_validation.json, enemy_routes_initial.json, enemy_routes_after_orders.json, enemy_route_delta.json, mission_preview.png, mission_route_preview.png, mission_route_debug.png, mission_summary.txt"
            );
        }
        "mission-orders" | "mission-routes" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| DEFAULT_MISSION_EXPORT_DIR.to_string());
            let spec = match args.next() {
                Some(path) => load_mission_spec(path)?,
                None => road_below_spec(),
            };
            let script = match args.next() {
                Some(path) => load_work_order_script(path)?,
                None => road_below_basic_prep_script(),
            };
            let after = export_order_script_run(&out_dir, spec, script)?;
            println!("Exported GamePivot 5.1 mission order and route-preview run to {out_dir}.");
            println!(
                "Completed {} order(s), queued {} order(s), validation issue(s): {}.",
                after.work_orders.len(),
                after.work_queue.len(),
                after.order_validation.len()
            );
            println!(
                "Prep remaining: {}s · labor remaining: {}s",
                after.remaining_prep_seconds, after.remaining_labor_seconds
            );
            println!(
                "Route files: enemy_routes_initial.json, enemy_routes_after_orders.json, enemy_route_delta.json, mission_route_preview.png, mission_route_debug.png"
            );
        }
        "mission-assault" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| DEFAULT_MISSION_EXPORT_DIR.to_string());
            let spec = match args.next() {
                Some(path) => load_mission_spec(path)?,
                None => road_below_spec(),
            };
            let script = match args.next() {
                Some(path) => load_work_order_script(path)?,
                None => road_below_basic_prep_script(),
            };
            let summary = export_assault_run(&out_dir, spec, script)?;
            println!("Exported GamePivot 5.1 assault readability run to {out_dir}.");
            println!(
                "{} · stopped {} · reached {} · objective health {}",
                summary.outcome_label,
                summary.enemies_eliminated,
                summary.enemies_reached_objective,
                summary.objective_health_remaining
            );
            println!(
                "Assault files: mission_prep_final.json, assault_initial_routes.json, assault_timeline.json, assault_summary.json, assault_debrief.json, route_prediction_accuracy.json, assault_delay_heatmap.png, assault_pressure_heatmap.png, assault_prediction_vs_actual.png, assault_preview_start.png, assault_preview_end.png, assault_path_trace.png"
            );
        }
        "mission-hazards" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/gamepivot_06".to_string());
            let spec = match args.next() {
                Some(path) => load_mission_spec(path)?,
                None => road_below_spec(),
            };
            let script = match args.next() {
                Some(path) => load_work_order_script(path)?,
                None => road_below_hazard_prep_script(),
            };
            let summary = export_hazard_sandbox_run(&out_dir, spec, script)?;
            println!("Exported GamePivot 6 rolling hazard sandbox run to {out_dir}.");
            println!(
                "{} · stopped {} · reached {} · objective health {}",
                summary.outcome_label,
                summary.enemies_eliminated,
                summary.enemies_reached_objective,
                summary.objective_health_remaining
            );
            println!(
                "Hazard files: rolling_hazards.json, rolling_hazard_preview.png, rolling_hazard_path_debug.png, assault_hazard_summary.json, assault_timeline.json, assault_debrief.json, assault_pressure_heatmap.png"
            );
        }
        "mission-balance" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/gamepivot_07".to_string());
            let spec = match args.next() {
                Some(path) => load_mission_spec(path)?,
                None => road_below_spec(),
            };
            let report = export_mission_balance_run(&out_dir, spec)?;
            println!("Exported GamePivot 7 mission balance run to {out_dir}.");
            println!(
                "{} scenario(s): {}",
                report.scenarios.len(),
                report.rating_breakdown.join(" | ")
            );
            println!(
                "Balance files: mission_balance_summary.json, scenario_comparison.json, rating_breakdown.json, route_shift_summary.json, hazard_effectiveness.json, scenarios/*/assault_debrief.json"
            );
        }
        "generate-missions" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_06".to_string());
            let mut count = 10;
            let mut seed = 0x5eed_0001;
            let mut theme = Some(MissionTheme::RidgeTrap);
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--render-visuals" => {}
                    "--count" => {
                        let Some(value) = args.next() else {
                            bail!("--count requires a value");
                        };
                        count = value.parse()?;
                    }
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--theme" => {
                        let Some(value) = args.next() else {
                            bail!("--theme requires a value");
                        };
                        theme = if value == "all" {
                            None
                        } else {
                            Some(value.parse().map_err(|err: String| anyhow::anyhow!(err))?)
                        };
                    }
                    other => bail!("unknown generate-missions option: {other}"),
                }
            }
            let mut generator = MissionGeneratorSpec::road_below(seed);
            if let Some(theme) = theme {
                generator.theme = theme;
                let report = export_generated_mission_batch(&out_dir, generator, count)?;
                println!("Exported ProcGen 7 mission batch to {out_dir}.");
                println!(
                    "Generated {} candidate(s): {} accepted, {} rejected.",
                    report.generated_count, report.accepted_count, report.rejected_count
                );
                if let Some(best) = report.ranked_candidates.first() {
                    println!(
                        "Best: {} · score {} · plan {} · seed {}.",
                        best.title, best.tactical_interest_score, best.best_plan_label, best.seed
                    );
                }
            } else {
                let report = export_generated_mission_theme_batch(
                    &out_dir,
                    generator,
                    count,
                    &MissionTheme::GENERATABLE,
                )?;
                println!("Exported ProcGen 7 all-theme mission batch to {out_dir}.");
                println!(
                    "Generated {} candidate(s): {} accepted, {} rejected across {} theme(s).",
                    report.total_generated_count,
                    report.total_accepted_count,
                    report.total_rejected_count,
                    report.theme_summaries.len()
                );
                if let Some(best) = report.all_ranked_candidates.first() {
                    println!(
                        "Best: {} [{}] · score {} · plan {} · seed {}.",
                        best.title,
                        best.theme_slug,
                        best.tactical_interest_score,
                        best.best_plan_label,
                        best.seed
                    );
                }
            }
            println!(
                "ProcGen files: generator_summary.json or theme_summary.json, ranked candidate JSON, accepted/rejected/top contact sheets, visual contact sheets, candidates/*/mission.ron, candidates/*/mission_visual_*.png, candidates/*/generated_feature_map.json"
            );
        }
        "generate-mission-pack" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_06_pack".to_string());
            let mut seed = 0x5eed_0001;
            let mut missions = 6;
            let mut candidates_per_theme = 20;
            let mut curve = MissionPackCurve::Balanced;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--render-visuals" => {}
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--missions" => {
                        let Some(value) = args.next() else {
                            bail!("--missions requires a value");
                        };
                        missions = value.parse()?;
                    }
                    "--candidates-per-theme" => {
                        let Some(value) = args.next() else {
                            bail!("--candidates-per-theme requires a value");
                        };
                        candidates_per_theme = value.parse()?;
                    }
                    "--curve" => {
                        let Some(value) = args.next() else {
                            bail!("--curve requires a value");
                        };
                        curve = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown generate-mission-pack option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let summary = export_generated_mission_pack_with_curve(
                &out_dir,
                generator,
                missions,
                candidates_per_theme,
                curve,
            )?;
            println!("Exported ProcGen 7 mission pack to {out_dir}.");
            println!(
                "Selected {} {}-curve mission(s) from {} generated candidate(s), {} accepted.",
                summary.pack.missions.len(),
                summary.curve.label(),
                summary.total_generated_count,
                summary.total_accepted_count
            );
            for mission in &summary.pack.missions {
                println!(
                    "{}. {} [{}] · score {} · difficulty {} · complexity {} · {}",
                    mission.order,
                    mission.title,
                    mission.theme_slug,
                    mission.tactical_interest_score,
                    mission.difficulty_score,
                    mission.complexity_score,
                    mission.best_plan_label
                );
            }
            println!(
                "Pack files: mission_pack.ron, mission_pack_summary.json, mission_pack_contact_sheet.png, mission_pack_visual_sheet.png, difficulty_curve.json, complexity_curve.json, pack_diversity_report.json, pack_playtest_summary.json, per_mission_playtest/*, source_candidates/browser_index.json"
            );
        }
        "playtest-mission-pack" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_07_pack_playtest".to_string());
            let pack_path = args
                .next()
                .unwrap_or_else(|| "exports/procgen_07_pack/mission_pack.ron".to_string());
            let report = export_generated_mission_pack_playtest_from_file(&out_dir, &pack_path)?;
            println!("Exported ProcGen 7 mission pack playtest to {out_dir}.");
            println!(
                "{} mission(s) · avg no-prep {:.1} · avg best {:.1} · avg spread {:.1}.",
                report.mission_count,
                report.average_no_prep_score,
                report.average_best_score,
                report.average_plan_spread
            );
            for mission in &report.missions {
                println!(
                    "{}. {} [{}] · no prep {} · best {} ({}) · spread {} · visual warnings {}",
                    mission.order,
                    mission.title,
                    mission.theme_slug,
                    mission.no_prep_score,
                    mission.best_score,
                    mission.best_plan_label,
                    mission.best_minus_worst,
                    mission.visual_qa.warning_count
                );
            }
            println!(
                "Playtest files: pack_playtest_summary.json, per_mission_playtest/*/mission_balance_summary.json, per_mission_playtest/*/visual/mission_visual_beauty.png, per_mission_playtest/*/visual_qa.json"
            );
        }
        "quality-gate-mission-packs" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_07_1".to_string());
            let mut seed = 99_418_113;
            let mut seed_count = 3;
            let mut missions = 6;
            let mut candidates_per_theme = 20;
            let mut curve = MissionPackCurve::Tutorial;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--render-visuals" => {}
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--seed-count" => {
                        let Some(value) = args.next() else {
                            bail!("--seed-count requires a value");
                        };
                        seed_count = value.parse()?;
                    }
                    "--missions" => {
                        let Some(value) = args.next() else {
                            bail!("--missions requires a value");
                        };
                        missions = value.parse()?;
                    }
                    "--candidates-per-theme" => {
                        let Some(value) = args.next() else {
                            bail!("--candidates-per-theme requires a value");
                        };
                        candidates_per_theme = value.parse()?;
                    }
                    "--curve" => {
                        let Some(value) = args.next() else {
                            bail!("--curve requires a value");
                        };
                        curve = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown quality-gate-mission-packs option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let report = export_generated_mission_pack_quality_gate(
                &out_dir,
                generator,
                seed_count,
                missions,
                candidates_per_theme,
                curve,
            )?;
            println!("Exported ProcGen 7.1 mission pack quality gate to {out_dir}.");
            println!(
                "{} pack(s), {} passed · avg acceptance {:.0}% · avg spread {:.1} · weak missions {}.",
                report.pack_count,
                report.passed_pack_count,
                report.average_acceptance_rate * 100.0,
                report.average_plan_spread,
                report.weak_missions.len()
            );
            println!(
                "Quality files: seed_matrix_summary.json, pack_quality_report.json, theme_stability_report.json, difficulty_curve_report.json, complexity_curve_report.json, visual_qa_summary.json, weak_mission_reports/*, generated_pack_contact_sheets/*"
            );
        }
        "generate-campaign-set" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_08_campaign".to_string());
            let mut seed = 99_418_113;
            let mut missions = 6;
            let mut candidates_per_theme = 20;
            let mut curve = MissionPackCurve::Tutorial;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--render-visuals" => {}
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--missions" => {
                        let Some(value) = args.next() else {
                            bail!("--missions requires a value");
                        };
                        missions = value.parse()?;
                    }
                    "--candidates-per-theme" => {
                        let Some(value) = args.next() else {
                            bail!("--candidates-per-theme requires a value");
                        };
                        candidates_per_theme = value.parse()?;
                    }
                    "--curve" => {
                        let Some(value) = args.next() else {
                            bail!("--curve requires a value");
                        };
                        curve = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown generate-campaign-set option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let summary = export_generated_campaign_set(
                &out_dir,
                generator,
                missions,
                candidates_per_theme,
                curve,
            )?;
            println!("Exported ProcGen 8 generated campaign set to {out_dir}.");
            println!(
                "{} · {} mission(s) · curve {}.",
                summary.mission_set.title,
                summary.mission_set.missions.len(),
                summary.mission_set.curve.label()
            );
            for slot in &summary.mission_set.missions {
                println!(
                    "{}. {} [{}] · lesson {} · unlocks {}",
                    slot.order,
                    slot.title,
                    slot.theme_slug,
                    slot.lesson.label(),
                    if slot.unlocks_after.is_empty() {
                        "none".to_string()
                    } else {
                        slot.unlocks_after
                            .iter()
                            .map(|unlock| unlock.label())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
            }
            println!(
                "Campaign files: mission_set.ron, mission_set_summary.json, mission_set_contact_sheet.png, unlock_curve.json, difficulty_curve.json, complexity_curve.json, missions/*/mission.ron, source_pack/*"
            );
        }
        "playtest-campaign-set" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_08_playtest".to_string());
            let mission_set_path = args
                .next()
                .unwrap_or_else(|| "exports/procgen_08_campaign/mission_set.ron".to_string());
            let report =
                export_generated_campaign_set_playtest_from_file(&out_dir, &mission_set_path)?;
            println!("Exported ProcGen 8 campaign set playtest to {out_dir}.");
            println!(
                "{} mission(s) · avg no-prep {:.1} · avg best {:.1} · avg spread {:.1}.",
                report.mission_count,
                report.average_no_prep_score,
                report.average_best_score,
                report.average_plan_spread
            );
            println!(
                "Campaign playtest files: mission_set_playtest_summary.json, pack_playtest_summary.json, per_mission_playtest/*"
            );
        }
        "quality-gate-campaign-sets" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_08_1".to_string());
            let mut seed = 99_418_113;
            let mut seed_count = 3;
            let mut missions = 6;
            let mut candidates_per_theme = 20;
            let mut curve = MissionPackCurve::Tutorial;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--render-visuals" => {}
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--seed-count" => {
                        let Some(value) = args.next() else {
                            bail!("--seed-count requires a value");
                        };
                        seed_count = value.parse()?;
                    }
                    "--missions" => {
                        let Some(value) = args.next() else {
                            bail!("--missions requires a value");
                        };
                        missions = value.parse()?;
                    }
                    "--candidates-per-theme" => {
                        let Some(value) = args.next() else {
                            bail!("--candidates-per-theme requires a value");
                        };
                        candidates_per_theme = value.parse()?;
                    }
                    "--curve" => {
                        let Some(value) = args.next() else {
                            bail!("--curve requires a value");
                        };
                        curve = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown quality-gate-campaign-sets option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let report = export_generated_campaign_set_quality_gate(
                &out_dir,
                generator,
                seed_count,
                missions,
                candidates_per_theme,
                curve,
            )?;
            println!("Exported ProcGen 8.1 campaign set quality gate to {out_dir}.");
            println!(
                "{} campaign set(s), {} passed · avg spread {:.1} · weak campaigns {}.",
                report.campaign_count,
                report.passed_campaign_count,
                report.average_plan_spread,
                report.weak_campaigns.len()
            );
            println!(
                "Campaign quality files: campaign_set_matrix_summary.json, campaign_quality_report.json, lesson_role_report.json, unlock_curve_report.json, campaign_difficulty_curve_report.json, campaign_complexity_curve_report.json, weak_campaign_reports/*, campaign_contact_sheets/*"
            );
        }
        "visual-lock-benchmark" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/visual_lock_06".to_string());
            let mut seed = 99_418_113;
            let mut count = 8;
            let mut theme = MissionTheme::RidgeTrap;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--count" => {
                        let Some(value) = args.next() else {
                            bail!("--count requires a value");
                        };
                        count = value.parse()?;
                    }
                    "--theme" => {
                        let Some(value) = args.next() else {
                            bail!("--theme requires a value");
                        };
                        theme = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown visual-lock-benchmark option: {other}"),
                }
            }
            let mut generator = MissionGeneratorSpec::road_below(seed);
            generator.theme = theme;
            let report = export_visual_lock_benchmark(&out_dir, generator, count)?;
            println!("Exported Visual Lock benchmark to {out_dir}.");
            println!(
                "{} [{}] | seed {} | score {} | best plan {}.",
                report.title,
                report.theme_slug,
                report.seed,
                report.accepted_score,
                report.best_plan_label
            );
            println!(
                "Visual lock files: benchmark_visual_full_board.png, benchmark_visual_playable_crop.png, benchmark_visual_close_detail.png, benchmark_visual_audit.json, benchmark_prepared_visual_beauty.png, benchmark_prepared_diff.png, benchmark_prepared_feature_overlay.png, benchmark_override_report.json, benchmark_override_before_after.png, benchmark_override_diff.png"
            );
        }
        "visual-lock-theme-consistency" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/visual_lock_07".to_string());
            let mut seed = 99_418_113;
            let mut count = 20;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--count" => {
                        let Some(value) = args.next() else {
                            bail!("--count requires a value");
                        };
                        count = value.parse()?;
                    }
                    other => bail!("unknown visual-lock-theme-consistency option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let report = export_visual_lock_theme_consistency(&out_dir, generator, count)?;
            println!("Exported Visual Lock theme consistency check to {out_dir}.");
            println!(
                "{} theme render(s) · {} shared high-impact piece(s) · weakest placeholder load: {}.",
                report.theme_entries.len(),
                report.shared_high_impact_pieces.len(),
                report
                    .weakest_visual_identity_theme
                    .as_deref()
                    .unwrap_or("none")
            );
            println!(
                "Visual lock theme files: per_theme/*/generated_beauty.png, per_theme/*/override_beauty.png, per_theme/*/before_after.png, theme_visual_contact_sheet_generated.png, theme_visual_contact_sheet_overrides.png, theme_visual_contact_sheet_before_after.png, theme_visual_consistency_report.json"
            );
        }
        "visual-lock-art-acceptance" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/visual_lock_09".to_string());
            let mut seed = 99_418_113;
            let mut benchmark_count = 8;
            let mut theme_count = 20;
            let mut theme = MissionTheme::RidgeTrap;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    "--benchmark-count" => {
                        let Some(value) = args.next() else {
                            bail!("--benchmark-count requires a value");
                        };
                        benchmark_count = value.parse()?;
                    }
                    "--count" | "--theme-count" => {
                        let Some(value) = args.next() else {
                            bail!("{arg} requires a value");
                        };
                        theme_count = value.parse()?;
                    }
                    "--theme" => {
                        let Some(value) = args.next() else {
                            bail!("--theme requires a value");
                        };
                        theme = value.parse().map_err(|err: String| anyhow::anyhow!(err))?;
                    }
                    other => bail!("unknown visual-lock-art-acceptance option: {other}"),
                }
            }
            let mut generator = MissionGeneratorSpec::road_below(seed);
            generator.theme = theme;
            let report = export_visual_lock_art_acceptance_gate(
                &out_dir,
                generator,
                benchmark_count,
                theme_count,
            )?;
            println!("Exported Visual Lock 9 art acceptance gate to {out_dir}.");
            println!(
                "accepted: {} · decision: {} · {} check(s).",
                report.accepted,
                report.decision,
                report.checks.len()
            );
            println!(
                "Visual lock acceptance files: visual_lock_06_07_08_comparison.png, per_theme_close_detail_sheet.png, per_theme_playable_crop_sheet.png, visual_acceptance_report.json, remaining_art_risk_report.json"
            );
        }
        "art-variants" => {
            let Some(family) = args.next() else {
                bail!("art-variants requires a sprite family");
            };
            let Some(seed) = args.next() else {
                bail!("art-variants requires a seed");
            };
            let Some(count) = args.next() else {
                bail!("art-variants requires a count");
            };
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/art_lab/cli".to_string());
            let request = parse_art_variant_cli(&family, &seed, &count)?;
            let batch = generate_art_variants(&request);
            export_art_variant_batch(&batch, &out_dir)?;
            let contact_sheet_path = std::path::Path::new(&out_dir).join("contact_sheet.png");
            build_art_variant_contact_sheet(&batch).save_png(&contact_sheet_path)?;
            println!(
                "Exported {} {} art variant(s) to {}.",
                batch.variants.len(),
                batch.request.family.label(),
                out_dir
            );
            println!("Contact sheet: {}", contact_sheet_path.display());
        }
        "render-mission" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_06".to_string());
            let spec = match args.next() {
                Some(path) => load_mission_spec(path)?,
                None => road_below_spec(),
            };
            let report = export_mission_visuals(&out_dir, spec)?;
            println!("Exported ProcGen 6 mission visual preview to {out_dir}.");
            println!(
                "Visual profile: {} · {} effective sprite(s) · {} override(s) · {} issue(s).",
                report.sprite_style_profile,
                report.effective_sprite_count,
                report.overridden_sprite_count,
                report.override_issue_count
            );
            if !report.warnings.is_empty() {
                println!("Visual warnings: {}", report.warnings.join(" | "));
            }
            println!(
                "Visual files: mission_visual_beauty.png, mission_visual_preview.png, mission_visual_routes.png, mission_visual_debug.png, generated_feature_map.json, visual_asset_report.json"
            );
        }
        "calibrate-themes" => {
            let out_dir = args
                .next()
                .unwrap_or_else(|| "exports/procgen_05".to_string());
            let mut count = 200;
            let mut seed = 0x5eed_0001;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--count" => {
                        let Some(value) = args.next() else {
                            bail!("--count requires a value");
                        };
                        count = value.parse()?;
                    }
                    "--seed" => {
                        let Some(value) = args.next() else {
                            bail!("--seed requires a value");
                        };
                        seed = value.parse()?;
                    }
                    other => bail!("unknown calibrate-themes option: {other}"),
                }
            }
            let generator = MissionGeneratorSpec::road_below(seed);
            let report = export_theme_calibration_report(&out_dir, generator, count)?;
            println!("Exported ProcGen 5 theme calibration to {out_dir}.");
            println!(
                "Generated {} candidate(s): {} accepted, {} rejected.",
                report.total_generated_count,
                report.total_accepted_count,
                report.total_rejected_count
            );
            for theme in &report.theme_summaries {
                println!(
                    "{}: {:.0}% accepted · avg score {:.1} · avg difficulty {:.1} · avg complexity {:.1}",
                    theme.theme_slug,
                    theme.acceptance_rate * 100.0,
                    theme.average_score,
                    theme.average_difficulty_score,
                    theme.average_complexity_score
                );
            }
            println!(
                "Calibration files: theme_calibration_report.json, theme_calibration_summary.png, rejection_reason_histogram.png, difficulty_complexity_scatter.png, browser_index.json"
            );
        }
        "help" | "--help" | "-h" => print_help(),
        other => bail!("unknown command: {other}"),
    }

    Ok(())
}

fn print_help() {
    eprintln!("GroundLab CLI");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  cargo run -p ground_cli -- export [out_dir] [recipe_path] [palette_path]");
    eprintln!("  cargo run -p ground_cli -- edit-scenarios [out_dir] [recipe_path] [palette_path]");
    eprintln!("  cargo run -p ground_cli -- validate [recipe_path] [palette_path]");
    eprintln!("  cargo run -p ground_cli -- mission-seed [out_dir]");
    eprintln!("  cargo run -p ground_cli -- mission-orders [out_dir] [mission_spec.ron|json] [order_script.ron|json]");
    eprintln!("  cargo run -p ground_cli -- mission-routes [out_dir] [mission_spec.ron|json] [order_script.ron|json]");
    eprintln!("  cargo run -p ground_cli -- mission-assault [out_dir] [mission_spec.ron|json] [order_script.ron|json]");
    eprintln!("  cargo run -p ground_cli -- mission-hazards [out_dir] [mission_spec.ron|json] [order_script.ron|json]");
    eprintln!("  cargo run -p ground_cli -- mission-balance [out_dir] [mission_spec.ron|json]");
    eprintln!("  cargo run -p ground_cli -- generate-missions [out_dir] [--theme dry_road_below|ridge_trap|orchard_approach|dry_wash|old_wall|split_approach|all] [--count 10] [--seed 99418113] [--render-visuals]");
    eprintln!("  cargo run -p ground_cli -- generate-mission-pack [out_dir] [--seed 99418113] [--missions 6] [--candidates-per-theme 20] [--curve balanced|tutorial] [--render-visuals]");
    eprintln!(
        "  cargo run -p ground_cli -- playtest-mission-pack [out_dir] [mission_pack.ron|json]"
    );
    eprintln!("  cargo run -p ground_cli -- quality-gate-mission-packs [out_dir] [--seed 99418113] [--seed-count 3] [--missions 6] [--candidates-per-theme 20] [--curve balanced|tutorial] [--render-visuals]");
    eprintln!("  cargo run -p ground_cli -- generate-campaign-set [out_dir] [--seed 99418113] [--missions 6] [--candidates-per-theme 20] [--curve balanced|tutorial] [--render-visuals]");
    eprintln!(
        "  cargo run -p ground_cli -- playtest-campaign-set [out_dir] [mission_set.ron|json]"
    );
    eprintln!("  cargo run -p ground_cli -- quality-gate-campaign-sets [out_dir] [--seed 99418113] [--seed-count 3] [--missions 6] [--candidates-per-theme 20] [--curve balanced|tutorial] [--render-visuals]");
    eprintln!("  cargo run -p ground_cli -- visual-lock-benchmark [out_dir] [--theme ridge_trap] [--seed 99418113] [--count 8]");
    eprintln!("  cargo run -p ground_cli -- visual-lock-theme-consistency [out_dir] [--seed 99418113] [--count 20]");
    eprintln!("  cargo run -p ground_cli -- visual-lock-art-acceptance [out_dir] [--theme ridge_trap] [--seed 99418113] [--benchmark-count 8] [--count 20]");
    eprintln!("  cargo run -p ground_cli -- art-variants [family] [seed] [count] [out_dir]");
    eprintln!("  cargo run -p ground_cli -- render-mission [out_dir] [mission_spec.ron|json]");
    eprintln!(
        "  cargo run -p ground_cli -- calibrate-themes [out_dir] [--count 200] [--seed 99418113]"
    );
}
