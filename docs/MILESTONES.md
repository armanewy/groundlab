# GroundLab milestones

## Milestone 0 — Project seed

- Rust workspace
- Pure core crate with no UI dependencies
- Workbench shell crate
- CLI export crate
- Default dry upland art recipe

## Milestone 1 — Live pixel terrain asset lab

Implemented in this starter:

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

## Milestone 2 — Better asset pipeline

- RON/YAML recipe loading in UI
- multiple palettes
- palette linter / forbidden color detector
- seam-test generator with transition tiles
- generated normal, height, shadow, and occlusion masks
- import bundle structure for the eventual runtime
- hot reload from watched recipe files

## Milestone 3 — Custom renderer/runtime

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


## Milestone 1.1 — Erected terrain preview

Added a first software-rendered 2.5D pass. Terrain height is now visualized by raising top tiles and drawing exposed terrain faces between neighboring cells with different effective heights. This is not the final GPU renderer, but it prevents the workbench from relying only on height-color overlays.

Important constraints:

- The projection is still high-top-down/rectangular, not full diamond isometric.
- Raised faces can be faded in the workbench so hidden cells remain inspectable.
- Final runtime should use conditional occluder fade, silhouettes, cutaway, and/or camera rotation so units and important objects behind high terrain are never lost.
- The asset generator still needs dedicated wall/face/edge tiles for trench walls, berm faces, cliffs, slopes, and ramps.
