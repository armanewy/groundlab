# Road Below Paintover Brief

## Goal

Turn the current Road Below layered export into a high-oblique illustrated
tactical terrain scene resembling:

`assets/visual_targets/dry_upland_outpost_01/visual_target.png`

The current procedural beauty render is a layout/blockout. Preserve its tactical
structure, but replace its flat procedural look with authored or AI-assisted
material depth, detail density, and unified lighting.

## Keep

- Overall terrain-defense layout.
- Road/path network and scene orientation.
- Trench and berm positions.
- Trees, logs, rocks, wall/ruin, stakes, wire, objective, and spawn roles.
- Readable tactical intent: path, trench, berm, obstacles, objective, and spawn
  must remain distinguishable.

## Improve

- Depth: top planes, side planes, occlusion, contact shadows, and height.
- Grass and foliage density.
- Dirt-road material: compacted center, ruts, stones, worn edges, grass
  encroachment.
- Trench geometry: floor, side walls, lips, wood supports, shadow, disturbed
  earth.
- Berm geometry: raised crest, slope, clods, grass/dirt blend, shadowed face.
- Stone and wood material detail.
- Prop integration, scale, anchoring, and shadows.
- Unified lighting and palette.
- Diegetic objective/spawn markers.

## Avoid

- Visible grid.
- Board-game tokens.
- Flat vector shapes.
- Overly clean procedural bands.
- Changing the gameplay layout too much.
- Adding unrelated fantasy, sci-fi, buildings, characters, UI, or weapons.

## Desired Style

- High-oblique or isometric-ish illustrated terrain.
- Warm dirt roads.
- Olive grass with varied tufts and small highlights.
- Stone, wood, and earthwork materials.
- Painterly but tactically readable.
- Grounded shadows from one shared light direction.

## Layer Guidance

### Base Grass

Increase grass density and local color variation. Add clusters, darker patches,
small flowers or bright flecks sparingly, and directional texture. Avoid a flat
noise field.

### Paths

Make roads feel walked and compacted. Add ruts, embedded stones, broken edges,
grass encroachment, and subtle directional dirt marks. Avoid smooth ribbons.

### Trench / Berm

Make trench and berm physically distinct. Trench should be recessed with floor,
walls, lips, support posts, and dark occlusion. Berm should be raised with crest,
slope, clods, and a shadowed lower face.

### Props

Paint trees, logs, rocks, wall/ruin, stakes, and wire as scene objects with
consistent perspective and contact shadows. The old wall should become irregular
stonework or a ruin, not a rectangle.

### Markers

Objective and spawn cues should be diegetic: rally marker, small banner, supply
crate, signpost, stone platform, or defended landmark. Flags can be tiny accents
but should not dominate.

### Detail Decals

Use details to integrate layers: grass tufts at road edges, loose dirt around
earthworks, stones at trench lips, small rubble at wall base, bark chips near
logs.

### Lighting / Shadows

Unify the scene with one light direction. Add contact shadows, occlusion under
trees and props, trench interior darkness, and subtle vignette only if it helps
focus.

## Acceptance Checklist

- Looks like one cohesive high-oblique scene.
- No tile/stamp feel.
- Road, trench, and berm are instantly distinct.
- Props are anchored to the ground.
- Objective/spawn markers are diegetic.
- It is visibly closer to the target reference than the current procedural
  composite.
