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

Implemented in this drop.

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

Milestone 2 is deliberately still CPU/software-preview based. It gives the renderer a stronger asset contract before `wgpu` runtime work begins.

## Milestone 2.1 — Terrain extrusion asset upgrade

Recommended next sub-milestone before the GPU renderer if visual readability still feels weak.

- dedicated trench-wall and berm-face tile generation
- exposed-face atlas separate from top-surface atlas
- slope/ramp tiles
- cliff/ledge lip tiles
- cutaway and hidden-object silhouette preview
- better 2.5D hit testing for walls vs top surfaces

## Milestone 3 — Custom renderer/runtime

Next major milestone.

- introduce `ground_render`
- `wgpu` native renderer
- sprite batching
- tilemap layer renderer
- nearest-neighbor sampling / pixel-perfect camera
- y + elevation sorting
- debug overlay render passes
- keep `eframe` either as editor shell or replace it with raw `winit + wgpu + egui`

## Milestone 4 — Terrain gameplay sandbox

- fixed-step simulation
- enemy agents
- route preview vs actual movement
- cover queries
- defender positions
- selected-defense LOS overlay
- objective and spawn definitions

## Milestone 5 — Rolling hazard sandbox

- custom rolling-log model based on height gradient
- controllable release triggers
- collision against enemies/obstacles at grid/sprite level
- path trace overlay
- impact/damage summary

## Milestone 6 — Prepared-ground vertical slice

- prep phase
- assault phase
- budget/labor model
- post-run explanation
- one polished dry upland outpost map
