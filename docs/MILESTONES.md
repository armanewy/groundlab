# GroundLab milestones

## Milestone 0 — Project seed

Implemented.

- Rust workspace
- Pure core crate with no UI dependencies
- Workbench shell crate
- CLI export crate
- Default dry upland art recipe

## Milestone 1 — Live pixel terrain asset lab

Implemented.

- deterministic tile generation from `TilesetRecipe`
- fixed palette ramps
- terrain materials: grass, dirt, mud, rock, trench floor, trench wall, berm top, berm face
- tile variants
- contact sheet and atlas export
- editable terrain grid preview
- brushes: paint, trench, berm, ditch, flatten
- overlays: material, height, slope, movement, route, line of sight
- custom A* route preview
- custom grid-based LOS query

## Milestone 1.1 — Erected terrain preview

Implemented.

- first software-rendered 2.5D pass
- top surfaces displaced upward by height
- exposed faces between height deltas
- workbench face fading for inspectability
- approximate hit testing in erected view

## Milestone 2 — Better asset pipeline

Implemented.

- external RON recipe loading/saving
- external RON palette loading/saving
- auto-reload polling for recipe/palette edits
- material transition tiles
- generated height masks
- generated normal maps
- generated shadow masks
- generated occlusion masks
- seam-test sheet
- validation report
- metadata-rich export bundle
- CLI `export` and `validate` commands

## Milestone 3 — Terrain extrusion and occlusion workbench

Implemented.

- generated structure-face tiles: front, left, right, and lip
- structure-face metadata in `TileMetadata`
- structure face masks for height, normal, shadow, and occlusion outputs
- validation count coverage for structure faces
- seam-test/contact-sheet display of structure faces
- 2.5D terrain preview uses generated face art instead of flat debug rectangles
- terrain lips/cut-edges are rendered using generated lip art
- projected route overlay draws in erected terrain mode
- hover-driven local cutaway lens fades occluding faces near the inspected cell
- global face fade remains as a workbench-only debug option

## Milestone 4 — Angled projection pivot

Implemented.

- `ProjectionSpec` added to `TilesetRecipe`
- default tile source size changed to `64 px`
- new `PreviewMode::AngledTerrain`
- new `ViewOrientation` enum with NE/SE/SW/NW views
- workbench rotate-left / rotate-right controls
- orientation-aware inverse picking in the angled preview
- angled diamond top-surface renderer using existing generated material tiles
- angled exposed-face renderer using generated structure-face/lip art
- local cutaway/selection support in angled view
- flat material view preserved as command/debug map
- export bundle writes default angled preview, cutaway preview, and all four orientation previews

## Milestone 4.1 — Faux-perspective 2D terrain renderer

Implemented in this drop.

- `ProjectionKind::FauxPerspective2D` added and made the default
- `PreviewMode::FauxPerspectiveTerrain` added and made the default workbench view
- default screen cell footprint set to `64x64 px`
- default faux height step set to `18 px` initially
- rectangular top-down renderer using sprite-stacked terrain faces/lips/shadows
- orientation-aware picking and 90-degree rotation retained in the faux view
- hover cutaway/selection support in faux view
- UI texture uploads downscale large previews while CLI exports stay full resolution
- CLI/app export target defaults to `exports/milestone_04_1`
- export bundle writes default faux preview, cutaway preview, and all four orientation previews

This milestone is a visual-direction pivot, not a gameplay expansion. It preserves the terrain data,
asset pipeline, pathing, LOS, validation, and experimental angled renderer while changing the main
visual projection to “actually 2D, but drawn to look 3D.”

## Milestone 4.2 — Terrain feature sprite system

Implemented in this drop.

- default demo terrain replaced with an art-directed preview map
- noisy renderer stress-test terrain preserved as `TerrainMap::stress_test`
- `TerrainFeatureMap` derives material, ledge, trench, and berm edge masks
- faux renderer uses generated transition tiles in the actual map preview
- top-tile sampling crops generated tile edges to reduce debug-grid appearance
- stronger faux-perspective front faces and contact shadows
- dedicated trench top, trench lip, berm top, berm lip, and feature-detail passes
- optional feature-mask overlay in the workbench
- CLI/app export target defaults to `exports/milestone_04_2`
- export bundle writes comparison views: `terrain_preview_faux_debug.png`,
  `terrain_preview_faux_art.png`, and `terrain_preview_faux_features.png`

This milestone is still software-preview rendering, but it moves the visual model from cell-by-cell
height strips toward coherent terrain features.

## Milestone 4.3 — Perspective sprite scene prototype

Implemented in this drop.

- `PreviewMode::PerspectiveSpriteScene` added and made the default workbench view
- `TerrainMap::visual_target` adds a small hand-composed outpost/approach scene
- `VisualScene` and `VisualTerrainForm` derive larger visual scene forms from the terrain grid
- renderer draws broad floor regions rather than one obvious square per cell
- continuous cliff-face, trench-run, berm-run, shadow, and dressing passes
- larger visual footprint defaults: `96x80 px` cells and `32 px` faux height steps
- debug overlay can outline exported visual forms
- CLI/app export target defaults to `exports/milestone_04_3`
- export bundle writes `terrain_preview_visual_target.png`, `terrain_preview_visual_target_debug.png`, and `terrain_forms.json`

This milestone intentionally demotes the previous cell-feature renderer to a diagnostic view. The
new target is an illustrated 2D scene whose sprites imply physical terrain while the simulation grid
remains hidden underneath.

## Milestone 4.4 — Terrain art-kit renderer

Implemented in this drop.

- `TerrainArtKit` and `TerrainArtPiece` added as the sprite-piece composition layer
- generated local art kit exports `terrain_artkit_atlas.png` and `terrain_artkit_manifest.json`
- perspective sprite scene consumes named pieces for floor regions, roads, trench runs, berms, cliff faces, shadows, and dressing
- art-kit pieces include irregular alpha edges, textured faces, lips, soft shadows, corner caps, and debris
- CLI/app export target defaults to `exports/milestone_04_4`

## Milestone 4.5 — External art kit + hero scene

Implemented in this drop.

- `assets/artkits/dry_upland_outpost/manifest.ron` added as the external source art-kit contract
- `assets/artkits/dry_upland_outpost/pieces/*.png` added as replaceable source sprite pieces
- `TerrainArtPiece` now includes footprint, z-bias, opacity, and occlusion metadata
- perspective sprite scene prefers the external art kit and falls back to generated placeholders
- art-kit validation reports missing required pieces, duplicate ids, bad footprints, bad opacity, and manifest/image size mismatches
- `TerrainMap::visual_target` now defaults to a smaller 16x12 hero scene for visual judgment
- CLI/app export target defaults to `exports/milestone_04_5`
- export bundle writes `terrain_artkit_validation.json` in addition to the packed atlas and manifest

The next pass should replace the generated placeholder PNGs with authored or AI-assisted art pieces:
stronger slope/ramp silhouettes, trench and berm corners/caps, prop silhouettes, cast shadows, and
scene dressing.

## Milestone 5 — Custom renderer/runtime

- introduce `ground_render`
- `wgpu` native renderer
- sprite batching
- tilemap layer renderer
- nearest-neighbor sampling / pixel-perfect camera
- y + elevation sorting
- debug overlay render passes
- keep `eframe` either as editor shell or replace it with raw `winit + wgpu + egui`

## Milestone 6 — Terrain gameplay sandbox

- fixed-step simulation
- enemy agents
- route preview vs actual movement
- cover queries
- defender positions
- selected-defense LOS overlay
- objective and spawn definitions

## Milestone 7 — Rolling hazard sandbox

- custom rolling-log model based on height gradient
- controllable release triggers
- collision against enemies/obstacles at grid/sprite level
- path trace overlay
- impact/damage summary

## Milestone 8 — Prepared-ground vertical slice

- prep phase
- assault phase
- budget/labor model
- post-run explanation
- one polished dry upland outpost map
