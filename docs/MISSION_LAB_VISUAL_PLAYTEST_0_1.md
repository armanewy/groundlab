# Mission Lab Visual Playtest 0.1

Mission Lab Visual Playtest 0.1 evaluates the first usable Road Below loop with
Art Pack 0.1 active in the tactical grid.

## Scope

- Mission: The Road Below
- Visual pack: `assets/art_packs/art_pack_0_1/art_pack.json`
- Mode: Mission Lab Visual
- Loop: briefing -> prep -> queued work -> completed work -> assault -> debrief

This is not a final art review. It is a readability and usability checkpoint for
the tactical engineering loop.

## What Works

- Art Pack sprites now live in the tactical grid instead of only in a reference
  preview.
- Path, trench, berm, trees, logs, rocks, wall, stakes, wire, objective, and
  spawn markers are visible through assigned role sprites.
- Queued work intent is visible through blue affected-cell outlines.
- Recently completed work is visible through gold affected-cell outlines.
- Work-order preview cards now explain the expected visual result before the
  player runs the order.
- Route, selection, assault, and heat overlays still sit on top of the visual
  grid instead of being replaced by the art.

## What Is Still Confusing

- The selected-cell context panel still reads like a debug description first and
  an action guide second.
- The app lists possible buttons, but it does not yet make the best next action
  obvious.
- Availability reasons are mostly implicit. A rejected preview says what failed,
  but the panel does not summarize why an action is good, risky, or blocked.
- Local material usefulness is not surfaced strongly enough next to the selected
  cell.
- Route/debrief effects are still separated from the work-order choice. The
  player can see a trench appear, but the UI does not yet clearly say how it is
  expected to affect route cost, delay, or assault outcome.

## Playtest Verdict

Mission Lab Visual mode is now coherent enough for internal Road Below testing.
The next bottleneck is not art generation or another visual pack pass. The next
bottleneck is selected-cell guidance: when a player clicks a cell or object, the
UI should answer "what can I do here, and why would I do it?"

## Next Fixes

1. Make selected-cell context action-oriented.
2. Show recommended actions with visual result text.
3. Surface required tools and material inputs/outputs before queuing.
4. Explain unavailable actions with concise rejection reasons.
5. Keep route/debrief connection as the next pass after action guidance.

## Deferred

- Art Pack 0.2 polish: path composition, wall identity, marker style.
- Deeper generated mission/campaign work.
- Renderer rewrite.
- New work-order mechanics.
