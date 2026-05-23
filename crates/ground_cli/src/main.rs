use anyhow::{bail, Result};
use ground_core::{
    ensure_default_asset_files, export_tileset_bundle_with_palette, load_workbench_assets,
    TerrainMap, WorkbenchAssetPaths, DEFAULT_PALETTE_PATH, DEFAULT_RECIPE_PATH,
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
                .unwrap_or_else(|| "exports/milestone_04_1".to_string());
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
            let terrain = TerrainMap::demo(32, 24, loaded.recipe.seed);
            export_tileset_bundle_with_palette(
                &loaded.tileset,
                &loaded.palette,
                &terrain,
                out_dir,
            )?;
            println!("Exported GroundLab Milestone 4.1 bundle.");
            println!("{}", loaded.validation.summary_line());
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
    eprintln!("  cargo run -p ground_cli -- validate [recipe_path] [palette_path]");
}
