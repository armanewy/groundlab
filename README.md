# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 2

Milestone 2 turns the starter workbench into a more serious internal asset pipeline:

- external RON recipe loading and saving
- external RON palette loading and saving
- auto-reload polling for recipe/palette edits
- deterministic surface tile generation
- generated material transition/autotile pieces
- generated height masks
- generated normal maps from height masks
- generated shadow masks
- generated occlusion masks
- contact sheet preview
- seam-test sheet preview
- validation report for palette ramps, same-material seams, tile counts, and palette drift
- metadata-rich export bundle
- editable terrain preview with flat and 2.5D erected views

The terrain/gameplay core remains focused on the prepared-ground defense fantasy: trenches, berms,
elevation, line of sight, route shaping, movement cost, and rolling-hazard scaffolding.

## Run

```bash
cargo run -p ground_app
```

The workbench loads these by default:

```txt
recipes/dry_upland_outpost.ron
palettes/muted_field_32.ron
```

The UI can reload, save, and auto-reload those files while the app is running.

## CLI export

```bash
cargo run -p ground_cli -- export exports/milestone_02
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_02 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 2 writes:

```txt
terrain_atlas.png
terrain_height_mask.png
terrain_normal.png
terrain_shadow_mask.png
terrain_occlusion_mask.png
contact_sheet.png
seam_validation.png
terrain_preview.png
terrain_preview_2_5d.png
tileset_metadata.json
validation_report.json
recipe.ron
palette.ron
terrain_demo.json
```

## Project shape

```txt
crates/
  ground_core/   # engine-owned data, terrain, generation, masks, validation, preview, path/LOS, export
  ground_app/    # desktop workbench shell; not a game engine
  ground_cli/    # deterministic asset/export/validation command
```

## Important scope note

This is still not the game runtime. It is the internal workbench and asset pipeline foundation.
The next major step is a custom renderer/runtime crate, likely `ground_render`, while preserving
`ground_core` as the stable terrain/art/simulation model.
