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
