# Visual Target 0.1

## Target Description

Road Below should read as a high-oblique illustrated tactical terrain scene:
continuous ground, integrated earthworks, readable props, and tactical clarity.
The player should see terrain first and grid logic second.

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

## First Prototype Status

`assets/art_packs/art_pack_0_1/road_below_beauty.png` is the first beauty
composition prototype. It intentionally avoids the visible tactical grid and
draws continuous path, trench, and berm shapes before placing Art Pack props.

It is closer to the target than the stamped grid preview because it is composed
as a scene, but it is still not visually accepted. The comparison still shows
major gaps:

- Road and earthwork shapes are too soft and airbrushed.
- Trench depth lacks the crisp wood/soil wall detail in the reference.
- Props still come from simple pixel sprites and do not match the richer
  illustrated object style.
- Marker language remains too symbolic.
- Grass and lighting need more authored variation and scene-level direction.
