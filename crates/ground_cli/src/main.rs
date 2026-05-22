use anyhow::{bail, Result};
use ground_core::{export_tileset_bundle, TerrainMap, Tileset, TilesetRecipe};

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
                .unwrap_or_else(|| "exports/milestone_01".to_string());
            let recipe = TilesetRecipe::default();
            let tileset = Tileset::generate(&recipe);
            let terrain = TerrainMap::demo(32, 24, recipe.seed);
            export_tileset_bundle(&tileset, &terrain, out_dir)?;
            println!("Exported GroundLab bundle.");
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
    eprintln!("  cargo run -p ground_cli -- export [out_dir]");
}
