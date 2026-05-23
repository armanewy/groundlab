# Architecture

GroundLab is intentionally not a general engine. It is a custom terrain/art/simulation workbench
for one future game family: terrain-first prepared-ground defense.

## What is custom-owned now

- Terrain data model
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
- Experimental angled terrain preview and legacy 2.5D erected preview
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

The important design rule is that generated art remains deterministic and metadata-rich. Tiles are
not just PNGs; they carry role, material, movement cost, cover hint, sight-blocking hint, height role,
transition metadata, and structure-face metadata.

## Simulation vs faked/custom

Custom gameplay systems:

- infantry movement cost is a grid query
- enemy routeing is A* over terrain costs
- line of sight is sampled over terrain height and sight blockers
- cover is a simple semantic category on cells

Visual/software-rendered systems:

- pixel tiles are deterministic recipes
- material transitions are generated from material pairs and edge masks
- structure faces and cut lips are generated as first-class tiles
- terrain preview is CPU-rasterized into an egui texture
- height and slope are shown as overlays
- 2.5D terrain is previewed by lifting cell tops and drawing generated exposed-face art

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
individual PNG pieces. Generated pieces remain a fallback and bootstrap path, but the perspective
scene renderer prefers the external kit so authored or AI-assisted sprites can replace placeholders
without rewriting terrain simulation, visual-form derivation, pathing, LOS, or export code.

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
