# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4.3 — perspective sprite scene prototype

Milestone 4.3 is a visual-target reset. The previous faux-perspective renderer is still useful as
a debug/diagnostic terrain view, but the main workbench view now targets a composed **2D sprite
scene**: the terrain grid remains the simulation layer, while the renderer derives larger visual
forms and draws those forms as floor regions, ledges, trench runs, berm runs, cliff faces, and scene
dressing.

New in this milestone:

- `PreviewMode::PerspectiveSpriteScene` is the default view
- `TerrainMap::visual_target(...)` creates a small hand-composed outpost/approach scene
- `VisualScene` and `VisualTerrainForm` export larger visual forms derived from the terrain grid
- renderer draws broad floor regions rather than one obvious square per cell
- continuous cliff-face, trench-run, berm-run, shadow, and dressing passes
- larger default sprite footprint: `96x80 px` cells with `32 px` height steps
- CLI/app export target defaults to `exports/milestone_04_3`
- export writes `terrain_preview_visual_target.png`, `terrain_preview_visual_target_debug.png`, and `terrain_forms.json`

The prior faux, angled, erected, and flat previews remain available as debug views. The design goal
remains terrain-first: trenches, berms, elevation, line of sight, route shaping, movement cost, and
rolling-hazard scaffolding are still the core systems.

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
cargo run -p ground_cli -- export exports/milestone_04_3
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_3 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.3 writes:

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
terrain_preview_visual_target.png
terrain_preview_visual_target_debug.png
terrain_forms.json
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
  ground_core/   # engine-owned data, terrain, visual forms, generation, masks, validation, preview, path/LOS, export
  ground_app/    # desktop workbench shell; not a game engine
  ground_cli/    # deterministic asset/export/validation command
```

## Important scope note

This is still not the final game runtime. Milestone 4.3 is meant to separate the hidden simulation
terrain from the visible scene composition. The next pass should improve the actual form art: slope
ramps, cliff caps, trench/berm inside and outside corners, prop silhouettes, and better authored
scene dressing.
