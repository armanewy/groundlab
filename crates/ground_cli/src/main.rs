use anyhow::{bail, Result};
use ground_core::{
    ensure_default_asset_files, export_edit_scenario_suite, export_tileset_bundle_with_palette,
    load_workbench_assets, TerrainArtKit, TerrainMap, WorkbenchAssetPaths, DEFAULT_PALETTE_PATH,
    DEFAULT_RECIPE_PATH,
};
use ground_game::{
    export_assault_run, export_hazard_sandbox_run, export_mission_balance_run,
    export_order_script_run, export_road_below_seed, load_mission_spec, load_work_order_script,
    road_below_basic_prep_script, road_below_hazard_prep_script, road_below_spec,
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
}
