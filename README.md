# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 3

Milestone 3 upgrades the 2.5D terrain workbench so elevation is no longer just a displaced
flat tile. It adds generated structural terrain art and local occlusion/cutaway tooling:

- generated structure-face tiles for exposed terrain walls
- generated lip/cut-edge strips for cliffs, berms, and trench cuts
- structure-face metadata in the atlas/export bundle
- height/normal/shadow/occlusion masks that understand structure faces
- validation and seam-test coverage for structure faces
- 2.5D preview now draws generated face art instead of flat debug rectangles
- projected route overlay in 2.5D mode
- hover-driven local cutaway lens for inspecting cells behind raised faces
- cleaner default 2.5D view with grid off by default, still available as a debug toggle
- extra export preview: `terrain_preview_cutaway.png`

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
cargo run -p ground_cli -- export exports/milestone_03
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_03 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 3 writes:

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
terrain_preview_cutaway.png
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

This is still not the final game runtime. It is the internal workbench and asset pipeline foundation.
The next major step can be either a `ground_render` custom renderer/runtime crate or a short
Milestone 3.1 pass for slope/ramp/corner structure tiles if the 2.5D terrain still needs more body.
