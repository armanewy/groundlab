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

Current status: accepted for internal visual direction.

Current scored candidate:
`assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png`

The procedural beauty image remains useful as a deterministic blockout and layer
source. It is not the accepted visual target. The accepted candidate is the
promoted paintover image, which resolves the main target gap: it reads as a
cohesive high-oblique illustrated terrain scene rather than as a grid, sprite
stamp, or flat procedural primitive composition.

## Scores

| Category | Score | Notes |
| --- | ---: | --- |
| Scene composition | 5 | The paintover reads as a cohesive battlefield scene with roads, trench, berm, props, and objective structure arranged in one authored composition. |
| Ground continuity | 5 | No visible tile/grid structure remains. Grass, road, trench, and props share continuous ground treatment. |
| Path quality | 4 | Roads are compacted, worn, and integrated with grass and stones. Some exact gameplay-layout fidelity is approximate, but the target road treatment is achieved for visual direction. |
| Trench quality | 5 | Trench reads as a constructed recessed earthwork with floor, side walls, wood supports, lips, shadow, and material detail. |
| Berm quality | 4 | Berm/raised cover reads as integrated earthwork with clods, slope, shadow, and grass blend. It is less dominant than the trench but passes the target read. |
| Prop integration | 5 | Trees, log, rocks, wall, stakes, wire, and small objects are grounded with shadows, scale, material texture, and consistent perspective. |
| Marker integration | 4 | Objective/spawn cues are more diegetic and scene-integrated. Small banner/sign cues remain, but they no longer dominate as board-game tokens. |
| Lighting and palette | 5 | Warm daylight, olive grass, muted stone, warm dirt, and consistent local shadows match the reference direction. |
| Target resemblance | 5 | The candidate is visibly closer to the target reference than to the procedural baseline and matches the intended high-oblique illustrated terrain direction. |

Average score: 4.67.

## Decision

Visual Target 0.1 is accepted for internal visual direction.

This does not mean final production art. It means Road Below now has a visual
target image that is good enough to steer future art, Mission Lab presentation,
and asset generation work. The tactical-grid view remains the interaction view;
the paintover image is the beauty/reference view.

## Residual Caveats

- The paintover does not preserve every gameplay cell exactly. It should guide
  visual direction, not replace tactical-grid logic.
- Future production work should reuse this image as a target for layer-specific
  art, not as proof that procedural primitives can close the gap alone.
- Further art work should stay narrow and target-driven: path fidelity,
  earthwork layer assets, and prop/material translation back into usable layers.
