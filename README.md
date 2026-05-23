# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4.1 — faux-perspective 2D terrain renderer

Milestone 4.1 pivots the main visual target away from mathematical dimetric/isometric terrain and
toward a top-down, screen-aligned 2D scene whose sprites imply depth and physical height.

The map now stays rectangular and readable from above, while each terrain cell can emit a sprite
stack:

- top surface tile
- front terrain face
- left/right side-face hints
- lip / cut-edge strip
- contact shadow
- trench floor detail
- berm top detail
- route, selection, and inspection overlays

The prior angled/dimetric renderer is still included as an experimental preview mode. The flat
material view remains useful as the command/debug map.

New in this milestone:

- `ProjectionKind::FauxPerspective2D` as the default projection kind
- `PreviewMode::FauxPerspectiveTerrain` as the default workbench preview
- larger 64px default source tiles and 64x64 screen cells
- 18px default faux-perspective height step
- screen-aligned terrain projection with sprite-stacked faces/lips/shadows
- orientation-aware faux-perspective rendering and picking
- 90-degree view rotation retained for inspection
- hover cutaway lens retained for faux, angled, and legacy 2.5D views
- full-resolution CLI exports with UI texture downscaling for large previews
- faux-perspective export previews for the default view and all four orientations

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
cargo run -p ground_cli -- export exports/milestone_04_1
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_1 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.1 writes:

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
terrain_preview_faux.png
terrain_preview_faux_cutaway.png
terrain_preview_faux_ne.png
terrain_preview_faux_se.png
terrain_preview_faux_sw.png
terrain_preview_faux_nw.png
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
The next visual pass should make the sprite stacks more art-directed: slope/ramp sprites, face
corners, cliff caps, trench lip variants, smoother ground transitions, and placeable-object shadows.
