# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4 — angled projection pivot

Milestone 4 pivots the visual target from mostly top-down square terrain to a larger-tile
angled/dimetric 2.5D view. The flat renderer still exists as the command/debug map, but the main
visual preview now uses diamond terrain footprints, projected height extrusion, orientation-aware
picking, and 90-degree view rotation.

New in this milestone:

- `ProjectionSpec` in the recipe for dimetric projection settings
- larger default generated source tiles: `64 px`
- larger default screen footprint: `96x48 px`
- stronger default angled height step: `24 px`
- new `PreviewMode::AngledTerrain`
- four `ViewOrientation` states with rotate-left / rotate-right workbench controls
- angled inverse picking for editing projected top surfaces and faces
- generated structure-face art reused in the angled renderer
- local cutaway lens and x-ray-style selected-cell outline in angled view
- flat material view retained as command/debug map
- export previews for the default angled view and all four orientations

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
cargo run -p ground_cli -- export exports/milestone_04
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4 writes:

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
terrain_preview_angled.png
terrain_preview_angled_cutaway.png
terrain_preview_angled_ne.png
terrain_preview_angled_se.png
terrain_preview_angled_sw.png
terrain_preview_angled_nw.png
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
The next major step should be either a more art-directed angled terrain pass — ramps, corners,
cliff caps, trench wall variants — or a custom `ground_render` crate once the projection feels right.
