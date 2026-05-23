use anyhow::{bail, Result};
use ground_core::{
    export_terrain_sprite_bundle, TerrainSpriteRecipe, DEFAULT_SPRITEGEN_EXPORT_DIR,
    DEFAULT_SPRITE_STYLE_PATH,
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
                .unwrap_or_else(|| DEFAULT_SPRITEGEN_EXPORT_DIR.to_string());
            let profile_path = args
                .next()
                .unwrap_or_else(|| DEFAULT_SPRITE_STYLE_PATH.to_string());
            let recipe = TerrainSpriteRecipe::from_style_profile_path(&profile_path)?;
            let summary = export_terrain_sprite_bundle(&out_dir, &recipe)?;
            println!(
                "Exported cozy terrain sprite bundle to {}.",
                summary.out_dir
            );
            println!(
                "{} sprites, {} validation issue(s).",
                summary.sprite_count, summary.validation_issue_count
            );
        }
        "help" | "--help" | "-h" => print_help(),
        other => bail!("unknown command: {other}"),
    }

    Ok(())
}

fn print_help() {
    eprintln!("GroundLab SpriteGen CLI");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  cargo run -p ground_sprite_cli -- export [out_dir] [style_profile]");
}
