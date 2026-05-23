# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4.4 — terrain art-kit renderer

Milestone 4.4 keeps the composed **2D sprite scene** from Milestone 4.3, but changes the visual
implementation from primitive rectangle drawing toward an internal terrain art kit. The terrain grid
remains the simulation layer; the renderer derives larger visual forms, then composes those forms
from named sprite pieces such as grass floors, road edges, trench floors, trench lips, berm faces,
stone walls, soft shadows, corner caps, and debris.

New in this milestone:

- `TerrainArtKit` generates a first deterministic local sprite-piece kit
- `terrain_artkit_atlas.png` and `terrain_artkit_manifest.json` are exported with the bundle
- perspective scene rendering consumes named art pieces instead of only raw rectangles
- floor regions, roads, trench runs, berm runs, cliff faces, shadows, and dressing use the art kit
- `PreviewMode::PerspectiveSpriteScene` remains the default view
- `VisualScene` and `VisualTerrainForm` still export larger forms derived from the terrain grid
- CLI/app export target defaults to `exports/milestone_04_4`

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
cargo run -p ground_cli -- export exports/milestone_04_4
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_4 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.4 writes:

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
terrain_artkit_atlas.png
terrain_artkit_manifest.json
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
  ground_core/   # engine-owned data, terrain, art kits, visual forms, generation, masks, validation, preview, path/LOS, export
  ground_app/    # desktop workbench shell; not a game engine
  ground_cli/    # deterministic asset/export/validation command
```

## Important scope note

This is still not the final game runtime. Milestone 4.4 is the first pass at separating visual
composition from the source of art pieces. The next pass should make the art-kit pieces externally
loadable and improve authored slope ramps, cliff caps, trench/berm corners, prop silhouettes, and
scene dressing.
