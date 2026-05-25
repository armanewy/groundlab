# Visual Target 0.1

## Target Description

Road Below should read as a high-oblique illustrated tactical terrain scene:
continuous ground, integrated earthworks, readable props, and tactical clarity.
The player should see terrain first and grid logic second.

The desired target is a cohesive high-oblique illustrated scene. The tactical
grid and Art Pack sprites are tools, not the final visual language.

This is not a request for final production art. It is the first visual target
that should prove GroundLab can compose generated or approved assets into a
scene that resembles a small battlefield rather than a debug board.

## What Art Pack 0.1 Gets Wrong

- The assembled result still has a stamped tile/grid look.
- Terrain features are isolated per cell instead of shaped across the scene.
- Perspective is inconsistent between terrain, props, objects, and markers.
- Objects are not integrated strongly enough with the ground.
- Objective and spawn markers read as board-game tokens.
- There is no shared lighting, shadow, or composition pass tying the scene
  together.
- Earthworks lack authored structure: trench walls, floor, lip, supports, berm
  crest, slope, and clods are not yet convincing enough.
- The visual scene lacks authored composition compared with the reference.

Art Pack 0.1 remains useful as a functional placeholder pack. It is not the
visual target.

## Desired Scene Properties

- No visible square tile boundaries in beauty view.
- Dirt paths should be continuous blended strokes through grass.
- Trenches should have depth: recessed floor, side walls, lip, shadow, and
  disturbed soil.
- Berms should read as continuous raised mounds with crest highlights and lower
  shadow.
- Trees, logs, stakes, wire, rocks, and walls should sit on the ground with
  anchors and contact shadows.
- Objective and spawn markers should become more diegetic: a rally point,
  supply crate, banner, defended cart, barricade, shrine, sign, or terrain
  landmark rather than pure board markers.
- Tactical overlays can exist, but they should be optional and drawn above the
  art.

## Composition Rules

1. Draw base terrain first as a continuous field.
2. Draw paths as blended strokes, not repeated path tiles.
3. Draw trenches and berms as multi-cell terrain features.
4. Draw props with anchor points, depth ordering, and shadows.
5. Add detail/noise/decals: grass tufts, dirt flecks, loose stones, edge
   breakup, and disturbed soil.
6. Draw route, selection, queued work, assault heat, and debug overlays last.

## Acceptance Criteria

- At normal zoom, the image reads as terrain rather than a board.
- Path, trench, berm, obstacles, objective, and spawn are distinguishable.
- Repeated tile artifacts are not obvious.
- Trenches and berms feel built into the ground, not stamped on top.
- Props are anchored with believable scale and shadows.
- A screenshot is meaningfully closer to the high-oblique terrain reference than
  to the current Art Pack 0.1 grid/stamp preview.
- `docs/VISUAL_TARGET_0_1_SCORECARD.md` passes: no category below 4 and average
  score at least 4.25.

Visual Target 0.1 is accepted when the scorecard passes.

## Explicit Non-Goals

- Not final production art.
- Not a full renderer rewrite.
- Not new gameplay.
- Not procgen or campaign work.
- Not another broad Art Lab generator pass.

## Comparison Workflow

When a desired reference image is available, compare GroundLab output against it
directly instead of judging pipeline success alone.

Export the current Road Below beauty prototype:

```powershell
cargo run -p ground_cli -- art-pack-road-below-beauty assets/art_packs/art_pack_0_1/art_pack.json assets/art_packs/art_pack_0_1/road_below_beauty.png
```

Compare it against the current target reference:

```powershell
cargo run -p ground_cli -- visual-target-compare assets/art_packs/art_pack_0_1/road_below_beauty.png assets/visual_targets/dry_upland_outpost_01/visual_target.png exports/visual_target_0_1/beauty_vs_target.png
```

The comparison should make the current gap obvious: Art Pack 0.1 is a connected
placeholder pipeline, while Visual Target 0.1 requires a cohesive high-oblique
composition.

## Closure Milestone

Visual Target 0.1 Closure means Road Below beauty composition matches the
reference direction well enough for internal visual direction. It does not mean
final production art, but it does require target-first scene composition rather
than grid-first or sprite-first rendering.

Gate: [VISUAL_TARGET_0_1_SCORECARD.md](VISUAL_TARGET_0_1_SCORECARD.md).

## Accepted Paintover Candidate

`assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png` is accepted as
the Visual Target 0.1 internal visual direction image.

This candidate closes the major gap that the procedural compositor could not:
it reads as a cohesive high-oblique illustrated terrain scene with continuous
ground, compacted roads, constructed earthworks, grounded props, dense foliage
and stone detail, and unified lighting.

The tactical-grid view remains the interaction view. The paintover is the
beauty/reference view used to steer future art, layer replacement, and Mission
Lab presentation work. It is not final production art and does not need to
preserve every gameplay cell exactly.

## Current Prototype Status

`assets/art_packs/art_pack_0_1/road_below_beauty.png` is the current beauty
composition prototype. It intentionally avoids the visible tactical grid and
draws continuous path, trench, and berm shapes before placing scene-authored
props and muted diegetic markers.

It is closer to the target than the stamped grid preview because it is composed
as a scene, but it is still not visually accepted. The comparison still shows
major gaps:

- Road and ground detail are still too smooth and sparse compared with the
  reference.
- Trench and berm structure is readable but still too stroke-like and lacks
  enough constructed volume.
- Props are authored directly in the compositor now, but they remain simplified
  vector-like objects rather than richly illustrated terrain props.
- Marker language is more muted and diegetic, but still schematic.
- Grass, stones, foliage, and lighting need more authored density and local
  variation.

The current procedural beauty compositor should be treated as a layout/blockout
and review tool. It establishes composition, role placement, and layer structure,
but it is not sufficient by itself to reach the target's painterly high-oblique
quality. Closing the gap requires authored, generated, or paint-over-quality
visual assets.

## Layered Road Below Paintover Pipeline

The accepted visual target came from a layered paintover workflow:

1. Export Road Below as editable scene layers.
2. Paint or generate improved layers/assets against the target reference.
3. Import and recompose the improved layers.
4. Compare the candidate against the target.
5. Score honestly and repeat until the scorecard passes.

This keeps GroundLab useful for structure, deterministic layout, layer
management, validation, and comparison while letting authored or AI-assisted art
do the material, depth, density, and painterly work that procedural primitives
are not closing.

Layer export:

```powershell
cargo run -p ground_cli -- art-pack-road-below-layers assets/art_packs/art_pack_0_1/art_pack.json exports/visual_target_0_1/layers
```

Recompose edited layers:

```powershell
cargo run -p ground_cli -- compose-road-below-layers exports/visual_target_0_1/layers exports/visual_target_0_1/paintover_composite.png
```

Promote a paintover candidate:

```powershell
cargo run -p ground_cli -- promote-road-below-paintover exports/visual_target_0_1/paintover_composite.png assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png
```

Compare original, paintover, and target:

```powershell
cargo run -p ground_cli -- visual-target-triple-compare assets/art_packs/art_pack_0_1/road_below_beauty.png assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png assets/visual_targets/dry_upland_outpost_01/visual_target.png exports/visual_target_0_1/original_paintover_target.png
```

## Final Closure Decision

Visual Target 0.1 is accepted for internal visual direction. Future visual work
should be target-driven and should translate the accepted paintover back into
usable layers/assets rather than expanding random sprite generation or
procedural primitive drawing.
