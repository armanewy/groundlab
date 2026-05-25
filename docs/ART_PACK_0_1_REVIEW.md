# Art Pack 0.1 Review

Art Pack 0.1 is the first versioned Art Lab output. It lives under
`assets/art_packs/art_pack_0_1/` and is no longer dependent on ignored
`exports/` scratch state.

This pack is accepted as a functional placeholder and integration test pack.
It proves that generated/approved art can move through Art Lab, become
versioned project assets, and render inside Mission Lab. It is not accepted as
the target visual direction. The assembled result still reads as stamped
32x32 sprites on a tactical grid, not as the intended high-oblique illustrated
terrain scene.

The desired target is a cohesive high-oblique illustrated scene. The tactical
grid and Art Pack sprites are tools, not the final visual language.

## Pack State

- Profile: `assets/art_packs/art_pack_0_1/art_pack.json`
- Summary: `assets/art_packs/art_pack_0_1/art_pack_summary.json`
- Preview: `assets/art_packs/art_pack_0_1/preview.png`
- Road Below preview: `assets/art_packs/art_pack_0_1/road_below_preview.png`
- Road Below beauty prototype: `assets/art_packs/art_pack_0_1/road_below_beauty.png`
- Selected sheet: `assets/art_packs/art_pack_0_1/selected_sheet.png`
- Required role coverage: 11/11
- Path kit coverage: 7/7
- Broken assignments: 0

## What Works

- Tree and log are the strongest object reads. They have clear silhouettes and already feel like usable gameplay objects.
- Trench and berm carry the terrain-defense identity well enough for internal previews. They are readable as dug and raised earthwork roles.
- Wire has crossed the minimum readability bar. It now reads more like an obstacle than scattered pixels.
- Objective and spawn markers are legible at preview scale and do not block the terrain read.
- The path kit is now represented by separate role sprites instead of one repeated path tile.
- In the Road Below context preview, path, trench, and berm read as distinct terrain roles at gameplay-preview scale.
- Stakes and wire read as interactable obstacles in context instead of loose texture noise.
- The pack is good enough to test interaction, role assignment, prep feedback,
  and Mission Lab integration.

## What Needs Work

- The whole assembled scene still looks like a tile/stamp board. The grid and
  per-cell composition dominate the visual read.
- Terrain features are still isolated tiles, not continuous earth shapes. Path,
  trench, and berm need to become blended multi-cell features in a beauty view.
- Perspective is inconsistent across terrain, props, objects, and markers.
- The scene lacks shared lighting and shadow composition; each sprite mostly
  reads as lit in isolation.
- Objective and spawn markers are too board-game-like for the final target.
- Object scale and ground anchoring are inconsistent enough that props can read
  like icons rather than terrain objects.
- Earthworks lack authored structure: trench walls/supports/floor and berm
  crest/slope/clods are not yet strong enough.
- The visual scene lacks the authored composition of the target reference.
- Wall is improved but still the most placeholder-like object in the pack. It is readable, but it needs a stronger old-stone or ruin identity.
- Path pieces are readable in mission context, but the route still has a stepped/tile-kit feel. This is no longer blocking, but path composition remains an Art Pack 0.2 target.
- Objective and spawn markers are acceptable for internal use, but they still read somewhat board-game-like.
- Rock is readable but not yet very expressive. It is good enough for Art Pack 0.1, but it is a likely Art Pack 0.2 polish target.

## Acceptance Status

- Usable for internal Art Lab previews: yes.
- Usable for first gameplay-context visual tests: yes.
- Usable as final production art: no.
- Needs Art Pack 0.2 before broader gameplay work: no.
- Accepted as visual target: no.
- Visual acceptance: accepted for internal visual direction via Visual Target
  0.1 paintover.

Art Pack 0.1 should be treated as a working placeholder pack, not as proof that
the final art direction is close. More individual 32x32 sprite generation will
not close the main gap by itself; the next visual problem is scene composition.

## Screenshots Reviewed

- `assets/art_packs/art_pack_0_1/preview.png`
- `assets/art_packs/art_pack_0_1/selected_sheet.png`
- `assets/art_packs/art_pack_0_1/road_below_preview.png`

## Road Below Context Review

Reviewed questions:

- Path, trench, and berm are distinguishable at preview scale.
- The path kit avoids the old single-sprite repeated-diagonal stripe problem. It still reads as a tile kit, but not as a blocking visual failure.
- Tree, log, stakes, and wire read as interactable objects.
- Wall remains the weakest object and still needs a stronger ruin/stone identity.
- Objective and spawn markers are acceptable for internal tests, but they remain more board-game-like than final art should be.

## Next Decision

Road Below is readable enough to use Art Pack 0.1 as the default visual pack for
Mission Lab interaction tests. It is not visually close enough to the desired
high-oblique target. The next visual milestone is:

> Visual Target 0.1 -- cohesive high-oblique Road Below scene.

Use `docs/VISUAL_TARGET_0_1.md` as the target spec before doing more art or
renderer work.

The current beauty-composition prototype is versioned at
`assets/art_packs/art_pack_0_1/road_below_beauty.png`. It is closer to the
target because it removes the tactical grid, draws continuous scene shapes, and
uses scene-authored props/markers instead of relying on pasted 32x32 sprites.
It is still a prototype, not visual acceptance.

The procedural beauty compositor should be treated as a layout/blockout and
review tool. It is useful for composition, layer separation, target comparison,
and paintover setup, but it is not expected to close the reference-quality gap
alone. The next visual direction is the Layered Road Below paintover pipeline:
export scene layers, improve them with authored or AI-assisted art, import the
paintover, compare against the target, and score honestly.

Visual Target 0.1 is gated by `docs/VISUAL_TARGET_0_1_SCORECARD.md`. The
closure milestone is “Visual Target 0.1 Closure -- Road Below beauty
composition matches the reference direction.”

Update: Visual Target 0.1 is now accepted for internal visual direction through
the promoted paintover candidate at
`assets/art_packs/art_pack_0_1/road_below_beauty_paintover.png`. Art Pack 0.1
itself remains a functional placeholder/integration pack; the paintover is the
beauty/reference image that should guide future visual work.

## Mission Lab Integration

Art Pack 0.1 is now loaded by Mission Lab by default, rendered directly into the
Visual-mode tactical grid, and used alongside queued/completed prep feedback,
selected-cell action guidance, prep consequence summaries, and debrief "What
mattered" notes.

The next decision checkpoint is `docs/PLAYABLE_SLICE_0_1_REVIEW.md`, which
should decide whether Road Below is accepted for internal iteration or needs one
more narrow UI, art, or content pass before new systems resume.
