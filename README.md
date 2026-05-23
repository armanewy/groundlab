# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4.2 — feature-aware faux-perspective terrain

Milestone 4.2 keeps the top-down, screen-aligned 2D camera from Milestone 4.1, but changes the
terrain read from “independent square cells with ledge strips” toward coherent terrain features.
The renderer now derives a `TerrainFeatureMap` from the height/material grid and uses that feature
map to draw more intentional terrain structures.

New in this milestone:

- art-directed preview terrain with broad regions, a readable road, coherent shelves, trenches,
  berms, mud basin, and rock outcrop
- stress-test terrain preserved separately for noisy edge-case coverage
- `TerrainFeatureMap` with material, ledge, trench, and berm edge masks
- transition-aware faux terrain rendering using generated transition tiles in the map view
- cropped top-tile sampling to reduce visible grid seams in art previews
- stronger front faces, contact shadows, and lips for height changes
- dedicated trench and berm surface-detail passes
- optional feature-mask overlay for debugging derived terrain features
- export comparison views: `terrain_preview_faux_debug.png`, `terrain_preview_faux_art.png`,
  and `terrain_preview_faux_features.png`
- default faux height step increased to 24px and side-face width to 16px for stronger terrain body

The prior angled/dimetric renderer is still included as an experimental preview mode. The flat
material view remains useful as the command/debug map.

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
cargo run -p ground_cli -- export exports/milestone_04_2
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_2 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.2 writes:

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
terrain_preview_faux_debug.png
terrain_preview_faux_art.png
terrain_preview_faux_features.png
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
The next visual pass should focus on actual authored/generated slope and corner feature sprites:
ramp tops, trench inside/outside corners, berm corner caps, continuous feature-run rendering, and
placeable-object shadows.
