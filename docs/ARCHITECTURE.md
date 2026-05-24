# Architecture

GroundLab is intentionally not a general engine. It is a custom terrain/art/simulation workbench
for one future game family: terrain-first prepared-ground defense.

The active roadmap has pivoted to generated mission candidates. SpriteGen remains the terrain art
forge, while `ground_game` owns mission specs, prep-phase work orders, local materials,
environment-object states, doctrine route preview, deterministic assault sandbox, assault
readability/debrief artifacts, the first deterministic rolling hazard sandbox, mission balance
scenario comparison, seeded mission generation/ranking with structured rejection diagnostics across
multiple mission theme grammars, generated mission browsing, theme calibration, and automatic
mission pack selection with difficulty/complexity curves. Generated missions now also carry visual
theme bindings and can be rendered through effective SpriteGen assets as high-oblique beauty,
route-overlay, and debug mission previews. Generated mission packs can also be replayed through the
balance/debrief harness with per-mission visual QA so pack quality is evaluated as a playable set.
ProcGen 7.1 runs that same pack pipeline across seed matrices and aggregates stability, theme drift,
curve quality, visual QA, and weak-mission diagnostics. ProcGen 8 packages those selected packs into
mission sets with ordered lessons, capability unlock metadata, save templates, per-slot mission
folders, campaign playtest exports, and Mission Lab mission-set navigation.
`ground_app` now treats Mission Lab as the primary tactical prep surface rather than a generic
terrain editor panel, and Road Below starts as a playable briefing-to-debrief slice. The same
mission loop now evaluates generated Road-Below-like candidates so GroundLab can batch-generate,
score, reject, and export compact terrain-defense problems.

## What is custom-owned now

- Terrain data model
- Mission data model
- State-based terrain and environment objects
- Prep-phase work order model
- Local material stock and side-effect tracking
- Doctrine-specific enemy route preview
- Deterministic assault agents, typed timeline, debrief summary, and influence heatmaps
- Deterministic rolling-log hazard path prediction, release, impact, and debrief reporting
- Mission briefing, rating, and balance-scenario comparison
- Playable mission lifecycle, prep-plan replay, tutorial prompts, and player-facing debrief panels
- Seeded mission generator specs, generated affordance reports, tactical-interest scoring, and batch
  ranking/rejection exports
- Generated mission fingerprints, plan-sensitivity scores, rejection taxonomies, duplicate filtering,
  and candidate contact sheets
- Mission theme grammars for road, orchard, dry wash, ridge, old wall, and split-approach tactical
  problems
- Generated mission browser indexes, candidate-card metadata, and Mission Lab candidate loading
- Theme calibration reports, rejection histograms, difficulty/complexity scatter charts, and
  tuning recommendations
- Automatic mission pack selection, difficulty/complexity curves, pack diversity reports, and pack
  contact sheets
- Pack playtest reports, per-mission balance replays, visual readability QA, and Mission Lab pack
  loading
- Multi-seed pack quality gates with stability reports, theme acceptance drift, curve diagnostics,
  visual QA summaries, and weak-mission recommendation exports
- Generated campaign / mission-set manifests with ordered mission slots, lesson roles, capability
  unlock curves, save-data templates, and Mission Lab slot loading
- Mission visual theme bindings, high-oblique generated mission beauty previews, visual route/debug
  overlays, generated feature maps, and visual asset reports
- Terrain editing brushes
- Pixel tile recipes
- Palette ramps and palette file format
- Tile metadata
- Material transition tile generation
- Structure-face and lip tile generation
- Height/normal/shadow/occlusion mask generation
- Seam/palette/structure validation
- Software preview renderer
- Faux-perspective 2D terrain preview with sprite-stacked faces/lips/shadows
- Experimental angled terrain preview and diagnostic 2.5D erected preview
- Target-style stamp resolver for editable terrain features
- Target-look composition renderer for the active perspective scene
- Target-derived visual source image and grid alignment manifest
- Hero-scene dressing overlay for visual art-direction passes
- A* route query
- Line-of-sight query
- Export bundle format
- Rolling-hazard skeleton

## What is borrowed now

- `eframe/egui`: native desktop window and workbench UI shell
- `image`: PNG encoding
- `serde`/`ron`/`serde_json`: data formats

These are libraries, not a game engine. The core can be moved into a raw `winit + wgpu` runtime
without changing the terrain, generation, pathing, or LOS systems.

## Data flow

GamePivot mission flow:

```txt
MissionSpec
  -> MissionState
  -> tactical prep UI
  -> prep-phase work-order queue
  -> order validation
  -> terrain/object/material state changes
  -> material ledger + work log
  -> doctrine route preview + route delta reports
  -> LOS / cover consequences
  -> assault sandbox timeline
  -> typed event causes + magnitudes
  -> deterministic rolling hazard events
  -> debrief summary + influence heatmaps
  -> prediction-vs-actual route comparison
  -> rating + scenario comparison reports
  -> player retry / saved prep-plan iteration
```

ProcGen mission flow:

```txt
MissionGeneratorSpec + MissionTheme
  -> deterministic terrain/object/enemy/objective grammar
  -> GeneratedMissionCandidate
  -> affordance report
  -> doctrine route preview
  -> scripted balance scenarios
  -> deterministic assault/debrief/rating
  -> score breakdown + plan sensitivity
  -> map/route/material/hazard fingerprint
  -> tactical-interest score
  -> duplicate/similarity filtering
  -> accepted/rejected candidate reports with structured rejection kinds
  -> per-theme and all-theme ranked batches
  -> accepted/rejected/top-candidate contact sheets
  -> browser_index.json candidate cards
  -> generated mission browser filters
  -> theme calibration report + rejection/actionability diagnostics
  -> mission pack selection + difficulty/complexity curves
  -> theme-to-sprite-profile visual binding
  -> high-oblique mission visual previews + visual asset reports
  -> pack-level playtest report + per-mission visual QA
  -> multi-seed pack quality gate + weak-mission diagnostics
  -> generated campaign / mission set packaging
  -> mission slot lessons + capability unlock curve
  -> campaign playtest replay + mission-set save template
  -> Mission Lab inspection / playable retry
```

Art asset flow:

```txt
recipe.ron + palette.ron
  -> TilesetRecipe + Palette
  -> Tileset::generate_with_palette
  -> surface tiles + transition tiles + structure-face tiles
  -> height/normal/shadow/occlusion masks
  -> validation report + seam test sheet
  -> export bundle
  -> future runtime renderer/importer
```

Generated mission visual flow:

```txt
MissionSpec + MissionVisualTheme
  -> MissionState
  -> TerrainSpriteRecipe::from_style_profile_path
  -> generated/effective SpriteGen terrain sprites
  -> high-oblique software mission renderer
  -> mission_visual_beauty.png / mission_visual_routes.png / mission_visual_debug.png
  -> generated_feature_map.json + visual_asset_report.json
  -> generated mission browser and mission-pack visual sheets
  -> pack visual QA reports
```

The important design rule is that generated art remains deterministic and metadata-rich. Tiles are
not just PNGs; they carry role, material, movement cost, cover hint, sight-blocking hint, height role,
transition metadata, and structure-face metadata.

The matching gameplay rule is that important world changes are state-based and deterministic. A tree
does not become abstract wood; it moves through standing, fallen, cut-log, stake/timber, and cleared
states. Digging changes earth state, height, cover, movement, and local spoil. Building a berm spends
nearby spoil and changes cover, sight, movement, and route cost.

## Simulation vs faked/custom

Custom gameplay systems:

- infantry movement cost is a grid query
- enemy routeing is A* over doctrine-specific terrain costs
- assault movement is deterministic route-following over prepared terrain
- defender pressure is a deterministic range/line-of-sight query
- assault readability is derived from typed timeline events instead of hidden simulation state
- rolling logs are custom rule-driven hazards that follow predicted height/direction paths
- mission balance compares scripted prep plans through the same deterministic assault/debrief path
- line of sight is sampled over terrain height and sight blockers
- cover is a simple semantic category on cells

Visual/software-rendered systems:

- pixel tiles are deterministic recipes
- material transitions are generated from material pairs and edge masks
- structure faces and cut lips are generated as first-class tiles
- terrain preview is CPU-rasterized into an egui texture
- height and slope are shown as overlays
- 2.5D terrain is previewed by lifting cell tops and drawing generated exposed-face art
- generated missions can be rendered as high-oblique previews using effective SpriteGen art,
  optional tactical markers, route overlays, generated feature maps, and placeholder/fallback asset
  diagnostics

Hazard systems:

- rolling logs start as custom deterministic height/direction path simulation
- raw rigid-body physics should only be added where spectacle improves the game

## 2.5D terrain rendering policy

Elevation is not just an overlay. The terrain grid stores simulation height, but rendering must extrude it into readable top surfaces and exposed faces. The first implementation lives in `ground_core::preview::render_erected_terrain_preview`: each cell top is displaced upward by `height_step_px`, then vertical faces are drawn where a cell is higher than its south/east/west neighbors.

Visibility policy for hidden objects should be explicit in the runtime renderer:

1. draw the terrain normally using depth/order,
2. detect when an important unit, selected cell, route, or objective is behind an occluding face or tall prop,
3. fade or cut away that occluder locally,
4. draw an outline/silhouette for the hidden object through the occluder, and
5. keep overlays screen-space readable even when terrain overlaps.

The workbench now supports both global face fading and a local hover-driven cutaway lens. The game should prefer conditional/local fading so terrain still feels solid.

## Milestone 4.2 feature-aware faux-perspective renderer

The main visual preview is now `PreviewMode::FauxPerspectiveTerrain`. It keeps the world top-down and
rectangular, then uses sprite stacks to imply physical height:

```txt
screen_x = left_padding + oriented_x * cell_width_px
screen_y = top_padding + oriented_y * cell_height_px - effective_height * height_step_px
```

Each visible height delta can emit a front face, side-face hint, lip strip, and contact shadow. This
keeps planning/editing readable while letting trenches, berms, cliffs, and ridges feel like objects
with real terrain body.

`PreviewMode::AngledTerrain` is still available as an experimental dimetric view:

```txt
screen_x = origin_x + (u - v) * tile_screen_width_px / 2
screen_y = origin_y + (u + v) * tile_screen_height_px / 2 - effective_height * height_step_px
```

The flat `Material` preview is intentionally preserved as a command/debug map for pathfinding, LOS,
movement-cost overlays, and schematic inspection.

Current faux renderer policy:

- derive a `TerrainFeatureMap` from the terrain grid before drawing
- draw generated square source tiles as cropped screen-aligned top surfaces to reduce grid seams
- draw generated transition tiles where neighboring material regions meet
- draw trench and berm top/lip/detail passes from feature masks
- draw exposed terrain body using generated structure-face tiles
- draw generated lip strips along terrain cuts
- draw route, markers, feature-mask debug overlays, and selection above terrain
- use orientation-aware inverse picking for edit tools
- expose 90-degree view rotation in the workbench
- preserve full-resolution exports while downscaling only UI texture uploads

This remains a software preview. Once the projection and asset contract feel right, the GPU runtime
should implement the same command model with sprite batching and y/elevation sort keys.


## Milestone 4.3 visual-target scene note

Milestone 4.3 separates the hidden simulation grid from the intended visual composition. The new
`PerspectiveSpriteScene` preview derives `VisualScene` / `VisualTerrainForm` records from the terrain
map and draws larger forms such as floor regions, cliff faces, trench runs, berm runs, shadows, and
field-engineering dressing. The older faux/angled/flat previews remain as diagnostic tools.

## Milestone 4.4 terrain art-kit note

Milestone 4.4 adds the first `TerrainArtKit` layer. The perspective sprite scene still derives large visual forms from the simulation grid, but the renderer now composes those forms from named sprite pieces instead of relying only on procedural rectangles. The generated kit exports an atlas and manifest so the same contract can later be backed by hand-authored or external generated art.

## Milestone 4.5 external art-kit note

Milestone 4.5 keeps the layer split intact:

```txt
simulation grid -> visual forms -> external art-kit pieces -> composed preview/export
```

The default art kit now lives under `assets/artkits/dry_upland_outpost/` as `manifest.ron` plus
individual PNG pieces. Generated pieces are a bootstrap path for new kits; authored or AI-assisted
sprites can replace placeholders without rewriting terrain simulation, visual-form derivation,
pathing, LOS, or export code.

The visual benchmark scene is intentionally smaller at 16x12 cells. It is meant to judge composition,
terrain body, trenches, berms, caps, shadows, and dressing before adding more gameplay systems.

## Milestone 4.6 hero art-pass note

Milestone 4.6 exercises the same layer split with a larger art-kit set:

```txt
simulation grid -> visual forms -> variant art-kit pieces -> composed preview/export
```

The renderer now uses deterministic art-piece variant selection when the manifest contains multiple
pieces of the same kind. This keeps the art-pass work isolated in the art-kit manifest and PNGs
instead of leaking visual variation rules into terrain simulation or gameplay systems.

## Milestone 4.7 hero-scene overlay note

Milestone 4.7 adds one more visual-only layer:

```txt
simulation grid -> visual forms -> art-kit pieces -> hero-scene placements -> debug overlays
```

`HeroScene` is intentionally allowed to be hand-authored. It gives the workbench a way to prove a
small art-directed battlefield with props, caps, broken edges, vertical silhouettes, and cast shadows
while preserving the underlying terrain simulation, route preview, LOS, and form export contracts.

## Milestone 4.8R target-look editable scene note

Milestone 4.8R keeps editable terrain as the source of truth and changes the default perspective
scene from row-merged visual rectangles to target-style terrain stamps:

```txt
simulation grid -> feature map -> target-style stamps -> art-kit pieces -> hero-scene placements -> debug overlays
```

`TerrainStampResolver` derives connected grass, road, mud, stone, trench, and berm components from
`TerrainMap` plus `TerrainFeatureMap`. Each component becomes a `TerrainStampDefinition` with
`StampPiece` entries describing the art-kit pieces, offsets, opacity, and z-bias needed to draw that
feature as an art-directed group.

This is a renderer/art-kit correction, not a new backdrop system. The target reference image is used
as an art-direction guide only. When a brush edits terrain, the terrain data changes first; the stamp
resolver can then rebuild the affected visual feature groups while pathing, LOS, cover, and debug
overlays continue to query the same terrain map.

The export bundle now writes both `terrain_forms.json` and `terrain_stamps.json`. Forms remain useful
for high-level inspection, while stamps are the bridge toward target-look assets such as organic road
patches, trench bodies, berm mounds, stone platforms, prop clusters, and cast shadows.

## Milestone 4.9 target-look terrain composition note

Milestone 4.9 keeps the same source-of-truth rule but makes the default perspective scene call
`target_look::render_target_look_scene`:

```txt
editable terrain -> target-style stamps -> feature-specific composition -> hero dressing -> lighting -> overlays
```

The new renderer is still driven by `TerrainMap`; it does not draw a static generated backdrop.
Roads, trenches, berms, mud, grass, and stone platforms each get dedicated composition code for worn
edges, planks, lips, steps, soil/stone detail, shadows, and final scene lighting. Picking routes
through `target_look_pixel_to_cell`, so brush editing remains tied to the terrain grid while the
rendered output chases the target image's art grammar.

## Milestone 4.10 target-derived editable scene note

Milestone 4.10 stops asking the renderer to recreate the target image from procedural pieces. The
target image is now committed as source art:

```txt
assets/visual_targets/dry_upland_outpost_01/
  manifest.ron
  visual_target.png
```

The active perspective view now follows this flow:

```txt
visual target image -> aligned semantic terrain grid -> local edit patches -> route/LOS/debug overlays
```

`TerrainMap::target_derived(16, 12, seed)` creates the semantic terrain map aligned to the source
image. `VisualTarget` loads the manifest and maps pixels back to grid cells, so terrain brushes still
edit the simulation map. If the user edits a cell, the renderer draws only a local replacement patch
over the target image; unchanged cells continue to use the source art directly.

## Milestone 4.11 target-derived edit patch note

Milestone 4.11 makes those local edits explicit data:

```txt
visual target image -> aligned semantic terrain grid -> edit patch records -> local patch renderer -> overlays
```

`edit_patch::build_edit_patches` compares the current `TerrainMap` with the aligned
`TerrainMap::target_derived` baseline. Changed cells are grouped into connected dirty regions by
terrain patch kind: grass, road, mud, stone, trench, berm, or mixed. Each `TerrainEditPatch` records
the dirty cells, neighboring context cells, image-space patch bounds, representative old/new
signatures, and per-cell `TerrainCellChange` entries.

`target_look::render_target_look_scene` now renders from those patch records. The unchanged scene is
still the source image. Changed regions get target-derived color sampling, local material/trench/berm
patches, and optional debug overlays showing semantic cell material, height marks, dirty cells,
neighbor context, and patch bounds.

## Milestone 4.12 edit patch stress-test note

Milestone 4.12 extends edit patches with operation metadata and scripted stress scenarios:

```txt
visual target image -> aligned terrain edit -> patch operation classification -> cover pass -> replacement detail -> debug/export metrics
```

`TerrainEditPatch` now records `old_kind`, `new_kind`, `operation`, `cover_required`, and
`touches_target_baked_feature`. This distinguishes additive edits, such as grass to trench, from
subtractive edits, such as trench to grass. Subtractive edits draw a cover pass first so baked target
features can be visually overwritten before replacement detail is drawn.

`edit_scenario.rs` defines repeatable edits for new trenches, berms, roads, feature removal,
flattening, and stone painting. `export_edit_scenario_suite` renders those scenarios to
`edit_scenarios/` with base, edited, cover-only, debug, patch JSON, and summary reports. This makes
visual edit quality testable without manually brushing the map every time.
