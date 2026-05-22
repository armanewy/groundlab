# Architecture

GroundLab is intentionally not a general engine. It is a custom terrain/art/simulation workbench
for one future game family: terrain-first prepared-ground defense.

## What is custom-owned now

- Terrain data model
- Terrain editing brushes
- Pixel tile recipes
- Palette ramps
- Tile metadata
- Software preview renderer
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

## Simulation vs faked/custom

Custom gameplay systems:

- infantry movement cost is a grid query
- enemy routeing is A* over terrain costs
- line of sight is sampled over terrain height and sight blockers
- cover is a simple semantic category on cells

Visual/software-rendered systems in Milestone 1:

- pixel tiles are deterministic recipes
- terrain preview is CPU-rasterized into an egui texture
- height and slope are shown as overlays

Future physics/hazard systems:

- rolling logs should start as a custom height-gradient simulation
- raw rigid-body physics should only be added where spectacle improves the game


## 2.5D terrain rendering policy

Elevation is not just an overlay. The terrain grid stores simulation height, but rendering must extrude it into readable top surfaces and exposed faces. The first implementation lives in `ground_core::preview::render_erected_terrain_preview`: each cell top is displaced upward by `height_step_px`, then vertical faces are drawn where a cell is higher than its south/east/west neighbors.

Visibility policy for hidden objects should be explicit in the runtime renderer:

1. draw the terrain normally using depth/order,
2. detect when an important unit, selected cell, route, or objective is behind an occluding face or tall prop,
3. fade or cut away that occluder locally,
4. draw an outline/silhouette for the hidden object through the occluder, and
5. keep overlays screen-space readable even when terrain overlaps.

The workbench uses global face fading as a temporary inspection aid. The game should make fading conditional so terrain still feels solid.
