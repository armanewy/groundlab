# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

This starter milestone focuses on:

- recipe-driven pixel terrain tile generation
- palette-disciplined material ramps
- a contact sheet / atlas export path
- an editable terrain grid
- height, slope, movement-cost, path, and line-of-sight overlays
- trench / berm / ditch / flatten / ground-paint brushes
- a tiny A* route preview and custom LOS query

## Run

```bash
cargo run -p ground_app
```

## CLI export

```bash
cargo run -p ground_cli -- export exports/milestone_01
```

This writes:

- `terrain_atlas.png`
- `contact_sheet.png`
- `terrain_preview.png`
- `tileset_metadata.json`
- `recipe.ron`
- `terrain_demo.json`

## Project shape

```txt
crates/
  ground_core/   # engine-owned data, terrain, generation, preview, path/LOS, export
  ground_app/    # desktop workbench shell; not a game engine
  ground_cli/    # deterministic asset/export command
```

## Current milestone status

This is a Milestone 1 starter. It is deliberately narrow. The next target is replacing the
software preview with a custom `wgpu` sprite/tile renderer while keeping all of the recipe,
terrain, and simulation code intact.
