# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: GamePivot 1 — mission workbench seed

GroundLab has pivoted from art-generation milestones back toward the game workbench. The primary
product direction is now a 2.5D tactical engineering defense game: the player is a commander /
engineer who reads a compact level, issues prep-phase work orders, transforms terrain and local
objects, then later tests those choices against predictable enemy doctrine.

SpriteGen remains in the repository as the terrain art forge. It still provides swappable style
profiles, override PNGs, sprite manifests, validation, and exportable grass/dirt/path/trench/berm/
stone pieces. It is now supporting infrastructure rather than the main roadmap driver.

GamePivot 1 adds a new `ground_game` crate with:

- `MissionSpec`, `MissionMap`, and `MissionCell`
- earth states such as normal, scraped, trench, deep trench, spoil pile, berm, unstable, and muddy
- environment object states for trees, logs, rocks, walls, wire, stakes, and fighting positions
- `ToolLoadout`, `CrewPool`, local material stock, and enemy group doctrine specs
- deterministic work orders for dig trench, raise berm, flatten, fell tree, cut into logs, and place stakes
- a seed mission, `The Road Below`, with a small road/ridge/tree terrain problem
- CLI export of mission spec, before/after mission state, scripted work orders, ASCII maps, and a summary
- a `Mission Lab` tab in `ground_app` beside the older terrain forge controls

Run the sprite workbench:

```bash
cargo run -p ground_sprite_app
```

Export the sprite bundle:

```bash
cargo run -p ground_sprite_cli -- export exports/artgen_04_0 assets/sprite_styles/cozy_upland/style.ron
```

Promote the current generated sprites into a profile's override folder as editable starting art:

```bash
cargo run -p ground_sprite_cli -- promote-overrides assets/sprite_styles/cozy_upland/style.ron
```

Export the GamePivot 1 mission seed:

```bash
cargo run -p ground_cli -- mission-seed exports/gamepivot_01
```

Run the mission workbench:

```bash
cargo run -p ground_app
```

The full-scene terrain renderer and ArtGen outputs remain downstream infrastructure for terrain
data, pathing, LOS, and future art-kit composition, but the active gameplay roadmap now starts with
mission prep, work orders, local materials, and predictable terrain consequences.

## Previous status: ArtGen 4.0 — stone platform / raised terrain kit

ArtGen 4.0 added the first hard raised-terrain sprite family: stone platform sprites. The generator
now produces stone top surfaces, front/side faces, bevels, steps, caps, corners, contact shadow,
crack decals, and moss/grass edge pieces with role, anchor, footprint, z-bias, occlusion, override,
and validation metadata.

## Previous status: Milestone 4.12 — edit patch stress tests and cover patches

Milestone 4.12 tests whether the target-derived scene survives editing. It adds scripted edit
scenarios, cover/erase patch handling for subtractive edits, patch quality metrics, and workbench
toggles for inspecting dirty cells, patch bounds, terrain signatures, and cover passes.

Milestone 4.11 built on the target-derived source image and made terrain edits explicit local
patches. The generated target image is still the base scene, but every divergence from the aligned
semantic terrain baseline becomes a tracked dirty region with cell changes, neighbor context, pixel
bounds, old/new terrain signatures, and now operation metadata.

Milestone 4.10 made the generated target image a real source asset instead of a comparison image.
The default renderer draws `assets/visual_targets/dry_upland_outpost_01/visual_target.png` as the
base scene, aligns a 16x12 semantic terrain grid to it, and renders only local replacement patches
when editable terrain diverges from that target-derived baseline.

New in this milestone:

- `assets/visual_targets/dry_upland_outpost_01/visual_target.png` is the committed visual source
- `assets/visual_targets/dry_upland_outpost_01/manifest.ron` defines the grid/image alignment
- `VisualTarget` loads the image and maps pixels to terrain cells for editing
- `TerrainMap::target_derived(16, 12, seed)` creates the matching semantic terrain map
- `edit_patch.rs` records dirty cells, neighbor cells, patch bounds, and old/new terrain signatures
- edit patches now classify additive, cover, replacement, height-only, and mixed operations
- cover patches repaint baked target-scene features before drawing replacement details
- `edit_scenario.rs` defines scripted new-trench, new-berm, new-road, remove-trench, remove-road, flatten-trench, and paint-stone scenarios
- `target_look.rs` draws the target image first, then patch-record-driven local edits and overlays
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
- `terrain_preview_target_base.png` exports the untouched source-image base
- `terrain_preview_target_with_edits.png` exports the current editable scene without route/marker overlays
- `terrain_preview_target_patch_debug.png` exports semantic grid, dirty cells, patch bounds, material swatches, and height marks
- `terrain_edit_patches.json` exports the active dirty-region records
- `edit_scenarios/` exports stress-test comparison images, cover-only images, debug images, per-scenario patch JSON, and summary metrics
- perspective scene rendering now uses the target-derived source image instead of trying to repaint the whole scene procedurally
- roads, trenches, berms, grass, stone, and mud edits draw as local replacement patches
- `PreviewMode::PerspectiveSpriteScene` remains the default view, now labeled target-derived editable scene
- `VisualScene` and `VisualTerrainForm` still export larger forms derived from the terrain grid
- CLI/app export target defaults to `exports/milestone_04_12`

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
assets/visual_targets/dry_upland_outpost_01/manifest.ron
```

The UI can reload, save, and auto-reload the recipe/palette files while the app is running. The art
kit and hero-scene manifest are loaded by the perspective sprite renderer and can be edited on disk
between runs.

## CLI export

```bash
cargo run -p ground_cli -- export exports/milestone_04_12
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_12 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Edit scenario suite only:

```bash
cargo run -p ground_cli -- edit-scenarios exports/milestone_04_12/edit_scenarios
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.12 writes:

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
visual_target_source.png
terrain_preview_target_base.png
terrain_preview_target_with_edits.png
terrain_preview_target_patch_debug.png
terrain_edit_patches.json
edit_scenarios/base.png
edit_scenarios/edit_new_trench.png
edit_scenarios/patch_debug_new_trench.png
edit_scenarios/cover_only_new_trench.png
edit_scenarios/terrain_edit_patches_new_trench.json
edit_scenarios/summary.json
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
  ground_sprite_app/ # fast terrain sprite generator workbench
  ground_sprite_cli/ # deterministic sprite export command
```

## Important scope note

This is still not the final game runtime. Milestone 4.12 makes the edit layer testable: the target
image is the visual base, while editable GroundLab terrain remains the source of truth for gameplay
data, local modifications, patch records, scripted edit stress tests, pathing, LOS, and debug
overlays.
