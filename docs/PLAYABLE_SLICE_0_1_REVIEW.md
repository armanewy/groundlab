# Playable Slice 0.1 Review

## Slice Goal

Road Below can be played from briefing to debrief in Mission Lab Visual mode
using Art Pack 0.1.

This review is a decision checkpoint for the first playable slice. It records
the current acceptance state after a Codex-assisted pass: the app launches,
Art Pack 0.1 loads, the Road Below visual context render is readable, and the
Mission Lab code path exposes the intended phase flow. A human tactile pass
through every click is still useful before adding larger systems.

Important visual caveat: this accepts the slice for internal interaction
iteration, not for final visual direction. Art Pack 0.1 is a functional
placeholder/integration pack. The current visual result still reads as stamped
32x32 sprites on a grid, while the target is a cohesive high-oblique illustrated
terrain scene.

## Test Script

1. Open the app.
2. Stay in Mission Lab Visual mode.
3. Start Prep.
4. Select terrain or an object using the visual grid.
5. Queue at least three work orders.
6. Run all queued work.
7. Inspect prep impact.
8. Commit / Start Assault.
9. Run Assault.
10. Read What mattered.
11. Retry.

## What Works

- Art Pack 0.1 is durable project state and loads with complete required-role
  coverage: 11/11 core roles, 7/7 path-kit roles, and no broken assignments.
- The Road Below visual context preview reads as a coherent tactical terrain
  scene: path, trench, berm, tree, log, stakes, wire, rock, objective, and
  spawn markers are distinguishable enough for internal iteration.
- Mission Lab now has a clear player-facing phase path in code: Current goal,
  Start Prep, Commit Plan / Start Assault, Run Assault, Debrief, What mattered,
  Retry with same plan, and Retry from briefing.
- Prep feedback is aimed at the right loop: queued intent, completed changes,
  prep impact, route consequences, and debrief influence are all surfaced as
  player-facing concepts rather than hidden debug data.
- The Art Lab-to-Mission Lab chain is connected end to end: generated sprites
  can become approved pack assets, and those assets now feed the playable
  tactical surface.
- The current pack is useful for testing the play loop, not for judging final
  art quality.

## What Is Confusing

- The app still opens into Art Lab, which is correct for the product direction
  but means the playable Road Below path is one tab away rather than the first
  visible experience.
- The Road Below preview confirms the pack is readable, but it is still hard to
  judge from static output whether route consequences are visually obvious
  during the actual assault.
- The debrief now says what mattered, but the relationship between those text
  notes and the exact changed cells on the map may still require mental
  matching.
- The wall and marker sprites are serviceable, but they still read more like
  early tactical symbols than final in-world terrain objects.
- The current UI has accumulated a lot of useful panels; the main player path
  may still compete with workbench controls until a human pass confirms the
  ordering feels natural.
- The visual result can be mistaken for accepted art direction because the
  pipeline works. It should not be; the grid/stamp composition remains a visual
  failure against the desired target.

## Visual Gaps

- Visible grid/stamp composition still dominates the beauty read.
- Path, trench, and berm are tile pieces, not continuous terrain shapes.
- Terrain, props, markers, and objects do not share one coherent high-oblique
  perspective.
- Scene-level lighting, contact shadows, and ground integration are weak.
- Objective and spawn markers remain too board-game-like.
- Some object scales and styles still read as icons rather than anchored world
  props.

## What Feels Satisfying

- The Art Pack preview finally makes GroundLab look like a tactical terrain
  game instead of a schematic board.
- The trench, berm, path-kit, tree, log, stakes, and wire set gives the prep
  loop a believable material language.
- The retry framing is the right shape: review what mattered, then either run
  the same plan again or reset to briefing.

## What Feels Tedious

- A full acceptance pass still requires stepping through a native UI with many
  panels, which makes it easy for the play loop to feel like a workbench unless
  the primary actions remain visually dominant.
- Route/debrief understanding may require reading several small text panels
  rather than seeing the most important cells called out directly on the map.
- There is no beginner ghost plan yet, so the first run still asks the player to
  infer a plausible trench/berm/harvest/stakes plan from the available actions.

## Acceptance Decision

- Accepted for internal iteration: yes.
- Needs UI pass before more systems: yes.
- Needs art pass before more systems: no for interaction iteration, yes before
  visual acceptance.
- Needs content/balance pass before more systems: yes.
- Visual acceptance: pending.

The next visual milestone is `Visual Target 0.1 -- cohesive high-oblique Road
Below scene`. See `docs/VISUAL_TARGET_0_1.md`.

## Next Narrow Fixes

1. Make the Mission Lab player path more dominant than the workbench controls:
   keep Current goal, primary action, selected-cell guidance, prep impact, and
   debrief visible before secondary panels.
2. Connect debrief consequence text back to the map by highlighting the most
   delayed, most damaging, best obstacle, breach, and unused-defense cells when
   What mattered is shown.
3. Add one beginner hint plan for Road Below so the first playthrough teaches a
   plausible trench, berm, harvest, and obstacle sequence without adding new
   mechanics.
