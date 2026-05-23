# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Milestone 4.8R — target-look editable scene renderer

Milestone 4.8R keeps editable terrain as the source of truth and changes the default visual renderer
from row-merged rectangles to target-style stamps. The renderer resolves connected terrain features
into art-directed stamp definitions, then draws organic grass, dirt-road, trench, berm, stone, mud,
shadow, and dressing passes from the existing terrain grid, art kit, and hero scene.

New in this milestone:

- `TerrainStampResolver` derives target-style stamps from `TerrainMap` + `TerrainFeatureMap`
- `TerrainStampDefinition` / `StampPiece` bridge editable terrain to art-directed sprite groups
- `terrain_stamps.json` exports the resolved stamp list for debugging/decomposition
- `assets/artkits/dry_upland_outpost/manifest.ron` now describes 50 source art pieces
- `assets/artkits/dry_upland_outpost/pieces/*.png` contains authored-looking variants, props, caps, and shadows
- `assets/heroscenes/dry_upland_outpost_hero_01.ron` places hero-scene dressing over the terrain forms
- `TerrainArtKit::piece_variant(kind, seed)` picks stable variants for duplicate piece kinds
- `HeroScene` / `HeroPlacement` keep hand-placed visual direction separate from simulation
- `TerrainArtPiece` now carries footprint, z-bias, opacity, and occlusion metadata
- art-kit validation reports missing pieces, duplicate ids, bad footprints, and size mismatches
- `terrain_artkit_atlas.png`, `terrain_artkit_manifest.json`, and `terrain_artkit_validation.json` are exported with the bundle
- `terrain_preview_visual_target_no_overlay.png` is exported for judging art without route/marker overlays
- perspective scene rendering now uses the target-style stamp pass instead of big visual rectangles
- roads, trenches, berms, grass, stone, and mud draw with organic software masks and stamp groups
- `PreviewMode::PerspectiveSpriteScene` remains the default view, now labeled target-style editable scene
- `VisualScene` and `VisualTerrainForm` still export larger forms derived from the terrain grid
- CLI/app export target defaults to `exports/milestone_04_8r`

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
assets/artkits/dry_upland_outpost/manifest.ron
assets/heroscenes/dry_upland_outpost_hero_01.ron
```

The UI can reload, save, and auto-reload the recipe/palette files while the app is running. The art
kit and hero-scene manifest are loaded by the perspective sprite renderer and can be edited on disk
between runs.

## CLI export

```bash
cargo run -p ground_cli -- export exports/milestone_04_8r
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_8r recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.8R writes:

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
terrain_preview_visual_target_no_overlay.png
terrain_preview_visual_target_debug.png
terrain_forms.json
terrain_stamps.json
terrain_artkit_atlas.png
terrain_artkit_manifest.json
terrain_artkit_validation.json
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

This is still not the final game runtime. Milestone 4.8R is a renderer/art-kit correction: the target
image remains a reference, while the editable GroundLab terrain remains the source of truth.
