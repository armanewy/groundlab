# Visual Target 0.1 Scorecard

## Inputs

- Procedural beauty baseline: `assets/art_packs/art_pack_0_1/road_below_beauty.png`
- Paintover candidate: `assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png`
- Target reference: `assets/visual_targets/dry_upland_outpost_01/visual_target.png`
- Baseline comparison: `exports/visual_target_0_1/beauty_vs_target.png`
- Triple comparison: `exports/visual_target_0_1/original_paintover_target.png`

If the paintover candidate exists, it is the main image being scored. If it does
not exist, the procedural beauty baseline remains the scored candidate.

## Passing Rule

- No category below 4.
- Average score must be at least 4.25.
- If any category is below 4, Visual Target 0.1 is not accepted.

## Status

Current status: not accepted.

Current scored candidate: procedural beauty baseline. No promoted paintover
candidate is present yet.

The current procedural beauty image is now a target-first Road Below scene
rather than a tile-grid preview. It removes the square-board read and uses
continuous roads, earthworks, scene-authored props, muted markers, and a unified
lighting pass. It is closer to the reference direction than Art Pack 0.1's
stamped tactical output, but it still does not pass the visual gate. The
procedural compositor is now treated as a layer/blockout generator, not as the
expected final art generator.

## Scores

| Category | Score | Notes |
| --- | ---: | --- |
| Scene composition | 3 | The image is scene-first instead of grid-first and roughly follows the reference layout, but the composition still feels procedural and sparse compared with the authored reference. |
| Ground continuity | 4 | No visible grid or square tile layout remains in the beauty image. Grass field and roads are continuous. |
| Path quality | 3 | Roads are continuous, warmer, and have ruts/flecks, but they are still too smooth and ribbon-like compared with the reference's compacted dirt and grass-edge complexity. |
| Trench quality | 3 | Trench reads as recessed and has lip, wall, dark floor, and revetment marks, but it remains too stroke-like and too dark compared with the reference's constructed trench volume. |
| Berm quality | 3 | Berm reads as raised terrain with crest and shadow, but it still lacks enough clods, broken silhouette, and slope detail to match the reference. |
| Prop integration | 3 | Props are now scene-authored with shadows instead of pasted 32x32 icons, but trees, wall, rocks, and stakes remain simplified and less painterly than the reference. |
| Marker integration | 3 | Markers are more muted and diegetic than the board-game flags, but they still read as schematic cues rather than fully integrated scene objects. |
| Lighting and palette | 3 | Palette is more coherent and debug colors are reduced, but the lighting is still simple and lacks the reference's dense local shadow and value variation. |
| Target resemblance | 3 | The output is meaningfully closer to the target than the stamped grid, but still falls short in authored density, foliage/stone detail, and painterly earthwork structure. |

Average score: 3.11.

## Decision

Visual Target 0.1 is not accepted.

The next candidate should come from the Layered Road Below paintover pipeline,
not from more isolated sprite generation or more procedural primitive polish.

## Current Top Blockers

1. The scene still lacks the reference's authored foliage, stone, grass, and
   dirt-detail density.
2. Trench and berm need stronger constructed geometry and local shadowing, not
   only soft strokes with detail marks.
3. Props and markers need richer perspective and material treatment so they stop
   reading as simplified vector objects.
