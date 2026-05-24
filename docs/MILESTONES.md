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

## Milestone 4.6 — Hero art pass

Implemented in this drop.

- external dry-upland art kit expanded from 15 pieces to 34 source pieces
- added art-directed variants for grass, road, mud, stone, trench, berm, shadow, corner, and debris kinds
- `TerrainArtKit::piece_variant(kind, seed)` added for deterministic variant selection
- perspective sprite scene now chooses stable variants for repeated art-piece kinds
- CLI/app export target defaults to `exports/milestone_04_6`

This milestone intentionally leaves the renderer architecture alone. It tests whether the external
art-kit contract can absorb a stronger source-art pass without touching simulation, visual-form
derivation, pathing, LOS, or export structure.

## Milestone 4.7 — Hero scene art direction lock

Implemented in this drop.

- `HeroScene` and `HeroPlacement` added as a hand-placed visual overlay layer
- default hero-scene manifest added at `assets/heroscenes/dry_upland_outpost_hero_01.ron`
- dry-upland art kit expanded from 34 pieces to 50 source pieces
- added prop/decal/cap/shadow kinds: grass tufts, loose rocks, dirt scrapes, trench spoil, broken berm edge, fallen log, stakes, sandbags, tool marks, large cast shadow, trench end caps, berm corners, broken ledge corner, and worn road edge patch
- perspective sprite scene draws hero placements after terrain forms and before debug/route overlays
- visual target reduced to a 14x9 scene for art-direction judgment
- export bundle writes `terrain_preview_visual_target_no_overlay.png`
- CLI/app export target defaults to `exports/milestone_04_7`

This milestone is allowed to cheat visually. The simulation grid and visual forms remain intact, but
the art-direction pass can place non-rectangular props, caps, silhouettes, and shadows to prove the
look before the renderer becomes more procedural.

## Milestone 4.8R — Target-look editable scene renderer

Implemented in this drop.

- `TerrainStampResolver` added to derive target-style stamps from `TerrainMap` and `TerrainFeatureMap`
- `TerrainStampDefinition` and `StampPiece` added as the feature-to-art bridge
- default perspective preview now renders connected terrain features as organic stamp groups instead of row-merged rectangles
- grass, road, mud, stone, trench, and berm components draw from editable terrain with software masks, lip/edge passes, shadows, pebbles, planks, and deterministic dressing
- hero-scene placements still draw on top, while route/LOS/debug overlays remain functional
- export bundle writes `terrain_stamps.json`
- CLI/app export target defaults to `exports/milestone_04_8r`

The target image is now treated as a style reference, not a backdrop. Editable GroundLab terrain
remains the source of truth.

## Milestone 4.9 — Target-look terrain composition

Implemented in this drop.

- `target_look.rs` added as the active perspective-scene renderer
- `PreviewMode::PerspectiveSpriteScene` now calls the target-look composition pass directly
- superseded 4.8R perspective-scene helper code removed rather than kept as dormant logic
- target-look picking added through `target_look_pixel_to_cell`
- roads, trenches, berms, stone platforms, mud, grass, hero dressing, and final lighting are composed from editable terrain
- CLI/app export target defaults to `exports/milestone_04_9`

The generated target image remains a style reference. GroundLab terrain data remains the editable
source of truth for rendering, pathing, LOS, and debug overlays.

## Milestone 4.10 — Target-derived editable scene

Implemented in this drop.

- `assets/visual_targets/dry_upland_outpost_01/visual_target.png` added as the committed source art
- `assets/visual_targets/dry_upland_outpost_01/manifest.ron` added for image/grid alignment
- `VisualTarget` added to load the source image and map preview pixels back to semantic cells
- `TerrainMap::target_derived(16, 12, seed)` added as the semantic map matching the image composition
- default perspective preview now draws the source image first and local replacement patches only after edits
- route, marker, grid, selection, and stamp debug overlays still draw over the target-derived scene
- export bundle writes `visual_target_source.png`
- CLI/app export target defaults to `exports/milestone_04_10`

This is the source-art correction: the target image is no longer only a reference, and the renderer
does not try to recreate the entire scene from placeholder art-kit pieces.

## Milestone 4.11 — Target-derived local edit patches

Implemented in this drop.

- `edit_patch.rs` added to compare editable terrain against the target-derived semantic baseline
- changed cells are grouped into connected dirty regions by terrain patch kind
- each patch records dirty cells, neighboring cells, image-space bounds, and old/new terrain signatures
- default perspective preview now renders local edits from patch records instead of implicit cell scanning
- patch drawing samples the target image under each edited cell so replacement art inherits local color
- target-grid / patch debug overlay shows material swatches, height marks, dirty cells, neighbor cells, and patch bounds
- export bundle writes `terrain_preview_target_base.png`
- export bundle writes `terrain_preview_target_with_edits.png`
- export bundle writes `terrain_preview_target_patch_debug.png`
- export bundle writes `terrain_edit_patches.json`
- CLI/app export target defaults to `exports/milestone_04_11`

This is the editability checkpoint for the target-derived source-art path. The base image keeps the
scene cohesive, while brush edits now produce explicit visual deltas that can be inspected, exported,
and replaced with stronger target-style patch pieces.

## Milestone 4.12 — Edit patch stress test and cover patches

Implemented in this drop.

- edit patches now record old/new patch kind, operation, cover requirement, and baked-feature touch
- patch metrics summarize dirty cells, patch count, cover patch count, patch kinds, operations, and patch bounds area
- target renderer adds a cover pass for subtractive edits before drawing replacement detail
- workbench exposes dirty-cell, patch-bounds, terrain-signature, and cover-only inspection toggles
- `edit_scenario.rs` adds scripted new-trench, new-berm, new-road, remove-trench, remove-road, flatten-trench, and paint-stone scenarios
- CLI adds `edit-scenarios` export command
- app adds `Export edit stress tests`
- bundle export writes `edit_scenarios/base.png`, edited previews, patch-debug previews, cover-only previews, per-scenario patch JSON, and `summary.json`
- CLI/app export target defaults to `exports/milestone_04_12`

This is the first visual stress-test milestone for target-derived editing. It proves the unedited
scene is no longer the only export and gives additive/removal edits concrete artifacts for judging
patch blending quality.

## ArtGen 2.1b — Trench topology polish

Implemented in this drop.

- active visual work remains in Pixel Terrain Forge instead of the full-scene renderer
- trench mask generation still exports `trench_mask_00` through `trench_mask_15`
- connected trench openings suppress shared internal lips and walls
- endpoint caps draw only on true dead ends and are blended into the trench body
- T-junctions and crosses receive a central floor resolver before edge/lip detail
- corner, dead-end, dense-clean, and single-mask preview exports added
- neighbor-seam, lip-continuity, and floor-continuity edge heatmaps added
- `trench_neighbor_pairs.json` lists the worst connected mask pairs for follow-up polish
- sprite CLI/app default export target is `exports/artgen_02_1b`

This is a narrow topology pass. It does not add berms, stone, props, scene rendering, or terrain
editor integration.

## ArtGen 3.0 — Oblique berm / mound sprite kit

Implemented in this drop.

- active visual work remains in Pixel Terrain Forge instead of the full-scene renderer
- berm rules are data-driven through the swappable sprite style profiles
- generated berm pieces include top, front face, front/back lips, end caps, inner/outer corners, contact shadow, spoil pile, and grass fringe
- berm pieces export with sprite role, anchor, footprint, z-bias, occlusion intent, and projection metadata
- `berm_contact_sheet.png` collects the generated raised-earth kit
- `berm_preview_oblique_straight.png` stages top, face, lip, shadow, spoil, and fringe as a raised mound
- `berm_preview_oblique_caps.png`, `berm_preview_oblique_corner.png`, and `berm_preview_oblique_shadow.png` provide focused visual checks
- `berm_mask_debug.png` documents the basic straight/cap/corner/shadow preview grammar
- berm validation reports piece coverage, role coverage, face/top contrast, shadow continuity, cap presence, and anchor validity
- sprite CLI/app default export target is `exports/artgen_03_0`

This is the raised-terrain counterpart to the trench work. It does not add full berm topology,
stone, props, scene rendering, or terrain editor integration.

## ArtGen 3.0b — Berm visual polish

Implemented in this drop.

- keeps the 3.0 berm piece categories and metadata contract intact
- does not add berm masks, berm topology, stone, props, scene rendering, or terrain editor integration
- front-face generation now uses an irregular mound silhouette instead of a full rectangular strip
- face shading adds a darker lower band, crevice under the top lip, and stronger base grounding
- end caps taper more naturally into the surrounding grass instead of reading as pasted wedges
- lips use chunkier broken segments with small vertical slumps
- contact shadow is stronger and broader under the mound
- berm validation now reports rectangularity, silhouette variance, base shadow strength, cap taper, and corner continuity
- sprite CLI/app default export target is `exports/artgen_03_0b`

This is the raised-terrain equivalent of the trench 2.0b polish pass. Berm topology should wait
until the straight/cap/corner previews read as mounded earth rather than a flat retaining wall.

## ArtGen 3.0c — Art override / replacement workflow

Implemented in this drop.

- each sprite style profile can now declare an `overrides/` folder
- generated sprites remain the deterministic source layer
- compatible `overrides/{sprite_id}.png` files replace generated pixels while preserving sprite id, kind, role, anchor, footprint, z-bias, occlusion, and projection metadata
- missing overrides fall back to generated sprites and are reported as `Missing override`, not validation failures
- invalid override sizes and load errors fall back to generated sprites and are reported in `override_report.json`
- override warnings report suspicious alpha changes and unused override PNGs
- `sprite_manifest.ron` and `sprite_manifest.json` now record whether each effective sprite source is generated or overridden
- exports now include `generated_pieces/`, effective `pieces/`, `generated_contact_sheet.png`, `effective_contact_sheet.png`, `override_contact_sheet.png`, `override_diff_sheet.png`, and `override_report.json`
- Pixel Terrain Forge adds generated/effective/override/diff preview panels and selected-sprite override status
- sprite CLI adds `promote-overrides [style_profile]` to copy the current generated sprites into a profile override folder as editable starting art
- sprite CLI/app default export target is `exports/artgen_03_0c`

This is a workflow milestone, not a new terrain feature. It makes generated art replaceable without
breaking the metadata contract that later berm topology, stone, props, and editable-scene integration
will consume.

## ArtGen 3.1 — Berm autotile / mound topology

Implemented in this drop.

- generated `berm_mask_00` through `berm_mask_15` using the same 4-bit cardinal mask contract as paths and trenches
- connected berm openings suppress shared internal lips, faces, and caps so adjacent mound cells read as one feature
- dead-end caps draw only on true endpoints
- corners, T-junctions, and crosses receive mound-aware center/top composition instead of overlapping straight pieces
- `berm_autotile_sheet.png` exports the full mask set
- sparse, dense, loop, junction, dead-end, and corner berm topology previews added
- neighbor-seam, lip-continuity, face-continuity, and shadow-continuity heatmaps added
- `berm_neighbor_pairs.json` lists the worst connected mask pairs for follow-up polish
- berm validation now reports mask coverage, missing masks, neighbor seam score, face/lip/shadow continuity, cap coverage, corner coverage, and junction coverage
- generated/effective/override/diff contact sheets and override reports remain part of the export bundle
- sprite CLI/app default export target is `exports/artgen_03_1`

This is the raised-terrain topology counterpart to the trench 2.1b pass. It keeps rough generated art
replaceable through the 3.0c override workflow while proving that berms can connect into straights,
corners, dead ends, loops, T-junctions, and crosses without baking internal faces into shared edges.

## ArtGen 3.2 — Primitive style tuning studio

Implemented in this drop.

- Pixel Terrain Forge now presents native primitive panels for grass, dirt, path/transition, trench, berm, and projection/global tuning
- grass and dirt panels expose their color ramps plus density/detail controls
- path tuning exposes path width, core width, corner rounding, edge noise, edge jitter, softness, grass intrusion, and dirt speckle controls
- trench tuning exposes floor darkness, inner/contact/wall shadows, floor/wall detail, wood plank/knot density, lip irregularity, spoil, and grass intrusion
- berm tuning exposes mound height, face/contact shadow, top grass blend, lip highlight, edge irregularity, spoil, and grass intrusion
- projection/global tuning exposes seed, tile size, variant count, export scale, cluster discipline, and high-oblique cell/face/shadow sizing
- Forge can save the current style profile, reload/revert it, or clone it by changing the profile save path
- Forge can ignore override PNGs while tuning generated style and can promote generated sprites into overrides from the same UI
- sprite CLI/app default export target is `exports/artgen_03_2`

This is a tool milestone, not a new sprite-family milestone. It makes the current primitive families
tunable without Rust edits or manual RON patching while preserving deterministic generation,
metadata, validation, overrides, and art-kit-compatible export.

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
