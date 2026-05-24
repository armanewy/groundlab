# GroundLab milestones

## Current pivot — Visual lock before more systems

GroundLab's active roadmap is paused on new game/procgen/campaign expansion while the generated
mission art direction is locked. SpriteGen remains the terrain art forge, and Mission Lab remains
the 2.5D tactical engineering defense workbench, but the current priority is making one generated
mission beauty render read as a coherent high-oblique pixel-art terrain scene rather than a
schematic board. Visual Lock 2 keeps the benchmark fixed and focuses on the prepared-work terrain
layer: trenches, berms, path edges, prepared diff artifacts, and visual-impact audit data.

## GamePivot 1 — Mission workbench seed

Implemented in this drop.

- added `ground_game` as a new workspace crate for mission/prep-phase gameplay data
- added `MissionSpec`, `MissionMap`, `MissionCell`, `MissionObjective`, `ToolLoadout`, `CrewPool`, `MissionConstraints`, and enemy group doctrine specs
- added stateful terrain/object enums for earth states, trees, logs, rocks, walls, obstacles, and fighting positions
- added local material stock for spoil, timber, logs, stakes, loose stone, scrap, and rope uses
- added deterministic work orders for dig trench, raise berm, flatten, fell tree, cut into logs, and place stakes
- work orders check tool availability, prep/labor budget, local material availability, and apply state changes immediately for the seed milestone
- added seed mission `The Road Below`
- added scripted seed work orders that dig a trench, raise a berm from the spoil, fell/cut a tree, and place stakes
- added `cargo run -p ground_cli -- mission-seed [out_dir]`
- mission seed export writes `mission_spec.ron`, `mission_spec.json`, `mission_before.json`, `mission_after.json`, `scripted_work_orders.json`, ASCII before/after maps, and `mission_summary.txt`
- `ground_app` now opens to a `Mission Lab` tab with prep budget, tools, local materials, enemy intel, work-order buttons, work log, export, and a simple tactical grid
- the older terrain asset workbench remains available as the `Terrain Forge` tab

This milestone is intentionally data-first. It does not add assault simulation, enemy movement, a
new renderer, or route doctrine preview yet; it establishes the commander/engineer prep loop as the
new center of the project.

## GamePivot 2 — Work orders and local materials

Implemented in this drop.

- work orders now enter a queue before execution
- queued orders preserve status, progress, crew requirement, assigned crews, labor seconds, elapsed prep duration, tool requirements, material inputs, material outputs, affected cells/objects, and preview notes
- execution validates tools, prep time, crew labor, target compatibility, and local material availability
- dependent scripts can be queued before execution; object/material preconditions are checked when each order runs
- added material ledger entries for spoil, timber, logs, stakes, loose stone, scrap, and rope deltas
- added order validation entries for rejected or illegal work orders
- `mission-seed` export now writes order scripts, material ledger, order validation, and a PNG mission preview
- added `cargo run -p ground_cli -- mission-orders [out_dir] [mission_spec.ron|json] [order_script.ron|json]`
- `mission-orders` runs either the built-in Road Below script or supplied mission/script files and exports initial/after state, work log, material ledger, validation, ASCII maps, PNG preview, and summary
- Mission Lab now shows selected-cell context, relevant order buttons, preview cards, queue controls, material ledger, validation feedback, and click-to-select tactical grid cells

This milestone keeps combat paused. The goal is to make prep-phase engineering visible and testable:
select terrain/object, preview a work order, queue it, run it, and inspect the terrain/object/material
side effects.

## GamePivot 3 — Doctrine route preview

Implemented in this drop.

- added deterministic doctrine route previews on `MissionState`
- added per-doctrine path cost profiles for shortest-path rushers, cover-seekers, concealment flankers, obstacle-avoiders, push-through troops, and obstacle clearers
- route costs now react to movement cost, height changes, trenches/ditches, berms/spoil, local objects, cover, concealment, and roads
- Road Below now includes multiple enemy groups so the same terrain can produce different likely routes
- `mission-seed`, `mission-orders`, and `mission-routes` export initial routes, post-order routes, route deltas, and PNG route previews/debug images
- route reports include per-group explanations for route cost, terrain features crossed, object cells touched, cover use, and road use
- Mission Lab now has route overlay modes for initial, current, delta, and none
- added route-preview regression coverage for the Road Below scripted prep sequence

This milestone still avoids moving enemies, combat, waves, morale, damage, or assault simulation.
Its job is only to answer whether prep-phase terrain work changes likely enemy approach routes.

## GamePivot 4 — Tactical prep UI

Implemented in this drop.

- reorganized Mission Lab around a prep-screen layout instead of an editor/debug panel
- added a persistent mission status panel for prep time, labor, crew, tools, materials, and queued orders
- added order mode buttons for inspect, dig, build, harvest, deploy, and cancel workflows
- added route/map controls for terrain, height, cover, resources, initial routes, current routes, route deltas, and per-enemy route filtering
- kept context-sensitive order buttons tied to the selected terrain cell or environment object
- improved the work-order queue panel with explicit run/clear controls and status coloring
- added enemy intel and objective panels to keep doctrine and mission goals visible during prep
- added a compact minimap/schematic beside the main mission grid
- added a notification stack for queue, run, export, reset, and selection feedback

This milestone is UI-only. It does not add enemy movement, combat resolution, assault simulation,
new art systems, or new terrain rules.

## GamePivot 5 — Assault sandbox

Implemented in this drop.

- added mission phase state for prep, assault, and debrief
- added deterministic assault state, enemy agents, agent statuses, timeline events, and assault summary data
- enemy agents spawn from the current doctrine route preview and advance cell by cell
- defender positions apply simple deterministic range/line-of-sight pressure
- trenches, ditches, berms, mud, unstable ground, stakes, wire, and logs now affect assault movement, delay, or damage
- added `cargo run -p ground_cli -- mission-assault [out_dir] [mission_spec.ron|json] [order_script.ron|json]`
- assault export writes final prep state, initial assault routes, timeline, summary, start/end previews, and path trace
- Mission Lab now includes Start Assault, Step, Run, and Reset to Prep controls
- mission canvas now displays defender markers and enemy agent markers during assault
- added regression coverage for running the Road Below assault to a deterministic debrief summary

This milestone intentionally avoids full combat, projectiles, morale, animations, rolling hazards,
advanced AI, and defender micromanagement. It only proves that prepared terrain can be exercised by
a deterministic assault loop.

## GamePivot 5.1 — Assault readability and debrief

Implemented in this drop.

- expanded assault timeline events with explicit event kind, cause, magnitude, cell, agent, group, and explanation
- split generic delay/damage events into terrain delay, obstacle delay, defender suppression, defender damage, obstacle damage, reached-objective, elimination, and lifecycle events
- added `AssaultDebrief` with influence summaries for crossed cells, delayed cells, damaging cells, defender pressure cells, breach cells, most effective obstacle, most delayed group, and unused defenses
- added prediction-vs-actual route accuracy reports comparing doctrine route previews against actual assault traces
- `mission-assault` now exports `assault_debrief.json`, `route_prediction_accuracy.json`, `assault_delay_heatmap.png`, `assault_pressure_heatmap.png`, and `assault_prediction_vs_actual.png`
- Mission Lab now exposes delay, pressure, and actual-path map modes
- the assault panel now shows a compact debrief after or during a run: most delayed group, best obstacle, breach point, prediction accuracy, and unused defense hint

This milestone still avoids rolling hazards, combat animation, morale, projectile simulation, and
advanced enemy AI. It makes the first assault loop explainable before adding more spectacle.

## GamePivot 6 — Rolling hazard sandbox

Implemented in this drop.

- added explicit rolling-log states for loose, positioned/braced, prepared, released, rolling, spent, and piled logs
- added `PrepareRollingLog` as a work order with rope/tool validation and predicted-path preview notes
- added a preplaced Road Below ridge log and a `road_below_hazard_prep_script`
- added deterministic rolling-log path prediction over discrete height cells using prepared direction, energy, terrain stops, and object blockers
- assault state now carries rolling hazards that release on a deterministic tick or can be manually scheduled from Mission Lab
- added typed hazard events: released, moved, hit enemy, destroyed obstacle, blocked, and spent
- rolling hazards can damage/delay enemies on their path and clear light wire/stake obstacles
- assault debrief now includes rolling hazard impact summary, best hazard cell, enemies hit, obstacles destroyed, and friendly-risk cells
- added `cargo run -p ground_cli -- mission-hazards [out_dir] [mission_spec.ron|json] [order_script.ron|json]`
- `mission-hazards` exports `rolling_hazards.json`, `rolling_hazard_preview.png`, `rolling_hazard_path_debug.png`, `rolling_hazards_final.json`, and `assault_hazard_summary.json`
- Mission Lab now includes a Hazards map mode, context actions to prepare logs, and a Release logs assault control

This milestone still avoids freeform physics, animation/VFX, demolition, many hazard types, and
complex collision geometry. The purpose is to prove one deterministic, previewable chain-reaction
terrain mechanic.

## GamePivot 7 — First balanced mission pass

Implemented in this drop.

- added mission briefing metadata to `MissionSpec` for summary text, primary objective, optional objectives, and enemy intel
- added `MissionRating` to score objective survival, stopped attackers, remaining objective health, prep-time efficiency, friendly-risk hazard paths, unused defenses, and rolling-hazard impact
- assault debriefs now include the mission rating beside influence, route-accuracy, and rolling-hazard summaries
- tuned Road Below into a first balance target where no-prep holds weakly, several plausible bad plans fail or score poorly, and a deliberate ridge chokepoint earns the top rating
- added built-in balance scripts: no prep, basic trench line, berm and stakes, basic prep, rolling hazard prep, ridge chokepoint, and overbuilt bad plan
- added `cargo run -p ground_cli -- mission-balance [out_dir] [mission_spec.ron|json]`
- `mission-balance` exports top-level `mission_balance_summary.json`, `scenario_comparison.json`, `rating_breakdown.json`, `route_shift_summary.json`, and `hazard_effectiveness.json`
- each balance scenario exports its prep state, work log, material ledger, order validation, route deltas, assault timeline, assault summary, assault debrief, rating, hazard summary, route accuracy, mission/route previews, and heatmaps
- Mission Lab objective/debrief panels now show the mission briefing, optional objectives, enemy intel, and rating notes

This milestone does not add new mechanics. It uses the existing work-order, route-preview, assault,
debrief, and rolling-hazard systems to answer whether one small mission has meaningful strategic
space.

## GamePivot 8 — First playable Road Below slice

Implemented in this drop.

- Mission Lab now starts Road Below in the briefing phase instead of dropping directly into prep
- added explicit lifecycle controls for Start Prep, Start Assault, Retry Assault, and Reset to Briefing
- blocked accidental order queuing or assault stepping before prep starts
- added a Road Below guide checklist covering route preview, earthwork, local material use, ridge log decisions, assault, and debrief
- added player prep-plan save/load/apply controls backed by `exports/gamepivot_08/player_plan.ron`
- added a richer player-facing debrief breakdown with objective survival, health remaining, enemies stopped/reached, prep time, friendly-risk hazard cells, unused defenses, hazard hits, and route accuracy
- added a compact in-app balance dashboard that computes the scripted Road Below benchmark plans and displays star/score/outcome summaries
- kept the existing debug/scripted scenario tools available, but moved the primary loop toward human playthrough and retry

This milestone does not add new mechanics or new art. It turns the existing Road Below systems into
an end-to-end playable slice that can be attempted, rated, retried, and compared against benchmark
plans.

## ProcGen 1 — Terrain / mission generator seed

Implemented in this drop.

- added `MissionGeneratorSpec` with deterministic seed, theme, terrain archetype, difficulty band,
  objective kind, doctrine mix, material budget style, and required affordances
- added Road-Below-like `ridge_trap` mission generation with compact road/ridge terrain, objective
  placement, enemy spawns, doctrine groups, tree clusters, loose-log hazard opportunities, defenders,
  tool loadouts, crew/prep budgets, and local material affordances
- added generated affordance reports for road cells, ridge cells, trenchable soil, tree count, loose
  logs, spawn count, route count, rolling hazard path cells, and rolling-hazard route intersections
- added candidate evaluation using the existing route-preview, balance-scenario, assault, debrief,
  and rating harnesses
- added tactical-interest scoring across baseline/best rating spread, route diversity, height
  interest, local materials, work-order opportunities, rolling hazards, doctrine spread, and objective
  vulnerability
- added accepted/rejected candidate reports with rejection reasons instead of silently discarding weak
  seeds
- added `cargo run -p ground_cli -- generate-missions [out_dir] [--theme ridge_trap] [--count N] [--seed N]`
- each candidate exports `mission.ron`, `mission.json`, `mission_preview.png`, `route_preview.png`,
  `affordance_report.json`, route previews, candidate evaluation, and a nested balance export
- batch exports include `generator_spec`, `generator_summary.json`, `ranked_candidates.json`,
  `rejected_candidates.json`, and `top_10_contact_sheet.png`

This milestone changes the priority from hand-tuning one mission to batch-generating compact
terrain-defense problems. SpriteGen and Mission Lab remain essential, but their new role is to
support candidate generation, evaluation, inspection, and export.

## ProcGen 2 — Candidate evaluation and batch ranking

Implemented in this drop.

- expanded candidate evaluation with `GeneratedMissionScoreBreakdown` for baseline pressure, prep
  delta, route diversity, terrain interest, material affordances, work-order opportunities, hazard
  viability, doctrine spread, objective vulnerability, and duplicate penalty
- added `GeneratedMissionPlanSensitivity` so each candidate records best-vs-baseline,
  best-vs-worst, rolling-log ratio, and overbuilt-plan score behavior
- added structured `GeneratedMissionRejectionKind` categories including too easy, too hard, no route
  diversity, no useful materials, no hazard opportunity, hazard too dominant, objective unreachable,
  terrain too flat, spawn placement issues, invalid map, and duplicate candidate
- added generated mission fingerprints over objective, spawns, ridge cells, tree cells, route cells,
  route lengths, and rolling-hazard route intersections
- added near-duplicate filtering so samey high-scoring candidates are moved into the rejected set
  with `duplicate_of_seed` and `similarity_to_duplicate`
- batch exports now include `accepted_contact_sheet.png`, `rejected_contact_sheet.png`, and
  `top_ranked_contact_sheet.png` in addition to `top_10_contact_sheet.png`
- contact sheets now draw route overlays, accepted/rejected/duplicate borders, tactical score bars,
  and plan-sensitivity bars for faster visual scanning
- Mission Lab can load a generated candidate from `mission.ron` using a simple file path field
- saved prep plans now apply against the currently loaded mission instead of always resetting to
  the original Road Below seed

The generator was smoke-tested with a deterministic 100-candidate ridge-trap batch. That run
accepted 11 candidates and rejected 89, including near-duplicate candidates with explicit duplicate
diagnostics.

## ProcGen 3 — Theme classes

Implemented in this drop.

- expanded `MissionTheme` with `orchard_approach`, `dry_wash`, `old_wall`, and `split_approach`
  beside the existing dry road / ridge-trap generation path
- added theme-specific terrain grammars:
  - Orchard Approach: tree clusters, timber/LOS tradeoffs, cover-seeking pressure, and restrained
    ridge-log affordance
  - Dry Wash: lowered wash/ditch ground, mud/soft crossing cells, overlook positions, and optional
    hazard interaction
  - Old Wall: damaged/breached/collapsed wall objects, rock ground, hard-cover route pressure, and
    wall-side earthwork opportunities
  - Split Approach: two approach lanes, split enemy spawns, tighter prep budget, and resource
    prioritization pressure
- `generate-missions --theme all` now runs every generatable theme into `per_theme/{theme_slug}/`
  and writes combined all-theme reports
- all-theme exports include `theme_summary.json`, `all_ranked_candidates.json`,
  `all_rejected_candidates.json`, and `contact_sheets/top_ranked_all_themes.png`
- all-theme contact sheets also include `accepted_by_theme.png` and `rejected_by_reason.png`
- single-theme generation still uses the same ProcGen 2 evaluator, duplicate filtering, contact
  sheets, and Mission Lab-loadable `mission.ron` files

The all-theme smoke run generated 120 candidates across six themes, accepted 26, rejected 94, and
produced accepted candidates in every theme class. This keeps the generator focused on tactical
problem classes instead of adding new gameplay systems.

## ProcGen 4 — Generated mission browser and mission pack builder

Implemented in this drop.

- all generated mission batches now export `browser_index.json` beside the ranked/rejected reports
- the browser index stores a compact card for each candidate: theme, seed, accepted/rejected state,
  score, best plan, baseline/best score, plan sensitivity, route diversity, hazard viability, local
  material score, primary affordance, rejection reason, candidate directory, and `mission.ron` path
- Mission Lab now includes a `Generated Missions` browser panel that loads a `browser_index.json`,
  filters by theme, toggles accepted-only browsing, shows candidate cards, and loads any candidate
  directly into the playable briefing/prep/assault loop
- added `cargo run -p ground_cli -- generate-mission-pack [out_dir] [--seed N] [--missions N]
  [--candidates-per-theme N]`
- mission pack generation runs the existing all-theme generator/evaluator into
  `source_candidates/`, selects diverse accepted candidates across themes, fills from the strongest
  remaining candidates, and orders the pack by a deterministic difficulty score
- pack exports include `mission_pack.ron`, `mission_pack_summary.json`,
  `mission_pack_contact_sheet.png`, and `difficulty_curve.json`
- source candidates for a pack keep the full ProcGen inspection bundle, including
  `source_candidates/browser_index.json`, per-theme reports, candidate missions, previews, balance
  reports, and contact sheets

This milestone turns procedural generation into a usable content pipeline: generate candidates,
browse/rank/filter them, open them in Mission Lab, and build a small diverse mission set without
manual map curation.

## ProcGen 5 — Theme calibration and difficulty curves

Implemented in this drop.

- added `ThemeCalibrationReport` with per-theme generated count, accepted count, rejected count,
  acceptance rate, target acceptance band, target difficulty label, average score, best score,
  average difficulty, average complexity, average plan sensitivity, route diversity, hazard
  usefulness, material affordance, top rejection reason, and recommendations
- added `cargo run -p ground_cli -- calibrate-themes [out_dir] [--count N] [--seed N]`
- calibration export writes `theme_calibration_report.json`, `theme_calibration_summary.png`,
  `rejection_reason_histogram.png`, `difficulty_complexity_scatter.png`, and the normal all-theme
  ProcGen batch outputs including `browser_index.json`
- added per-theme target bands for dry road below, orchard approach, dry wash, ridge trap, old wall,
  and split approach
- added structured generator-tuning recommendations for common failures such as missing hazard
  opportunity, low route diversity, weak local materials, too-flat dry wash maps, and bad spawn
  pressure
- added mission complexity scoring separate from difficulty; complexity tracks route count, doctrine
  spread, material types, hazard presence, height interest, and meaningful affordance count
- mission browser entries now include difficulty and complexity scores
- mission pack entries now include complexity scores beside tactical-interest and difficulty scores
- mission pack summaries now export `complexity_curve.json` and `pack_diversity_report.json`
- added `--curve balanced|tutorial` to `generate-mission-pack`
- the tutorial curve prefers a teaching sequence across dry road below, orchard approach, dry wash,
  ridge trap, split approach, and old wall while still falling back to the best available accepted
  candidates when a theme is sparse

This milestone does not add new mechanics or themes. It makes generated content more dependable by
showing which themes are over/under-accepting, why candidates fail, and whether a generated mission
pack has a sane difficulty and complexity progression.

## ProcGen 6 — Generated mission visual integration

Implemented in this drop.

- added `MissionVisualTheme` to mission specs with a SpriteGen style-profile path and
  high-oblique projection contract
- bound generated themes to visual profiles: dry road below, old wall, and split approach use
  `cozy_upland`; orchard uses `cozy_upland_lush`; dry wash and ridge trap use
  `cozy_upland_sparse`
- added a high-oblique mission visual renderer in `ground_game` that projects mission cells as
  diamond top surfaces using effective generated/override SpriteGen pieces
- visual rendering now maps roads/paths, trenches, berms, stone/rock, grass/dirt, mission markers,
  trees, logs, stakes, wire, walls, rocks, fighting positions, and route overlays into one preview
- added `visual_asset_report.json` with sprite profile, projection, effective/generated/override
  counts, override issue counts, missing piece kinds, fallback sprite kinds, and warnings
- generated candidates now export `mission_visual_preview.png`, `mission_visual_routes.png`,
  `mission_visual_debug.png`, and `visual_asset_report.json`
- generated candidate batches now write visual contact sheets beside the schematic contact sheets
- all-theme batches now write `top_ranked_all_themes_visual.png`,
  `accepted_by_theme_visual.png`, and `rejected_by_reason_visual.png`
- mission packs now export `mission_pack_visual_sheet.png`
- added `cargo run -p ground_cli -- render-mission [out_dir] [mission_spec.ron|json]`
- Mission Lab now includes a `visual` map mode beside terrain, height, cover, resources, delay,
  pressure, actual, and hazards

This milestone does not add mechanics, themes, campaign progression, or a GPU renderer. It connects
the generated mission pipeline back to the effective SpriteGen asset pipeline so generated missions
can be scanned as high-oblique 2.5D tactical previews instead of only schematic grids.

## ProcGen 6.1 — Generated mission visual composition polish

Implemented in this drop.

- split generated mission visuals into `mission_visual_beauty.png`,
  `mission_visual_routes.png`, and `mission_visual_debug.png`
- kept `mission_visual_preview.png` as a compatibility alias for the beauty render
- increased high-oblique feature scale and height step so generated mission terrain reads less like
  a tiny schematic grid
- added a darker field backdrop, map drop shadow, terrain-feature contact shadows, and chunkier
  object silhouettes for trees, logs, stakes, rocks, walls, wire, and fighting positions
- route overlays are now thinner and separated from the beauty render so they remain useful without
  dominating the visual judgment image
- `visual_asset_report.json` now records effective SpriteGen piece ids used by the render and counts
  placeholder object sprite classes separately from missing/fallback terrain pieces
- added `generated_feature_map.json` with grass, path, trench, berm, and stone feature cell counts,
  component counts, topology-mask histograms, and object counts
- visual contact sheets now prefer beauty renders when available instead of route-overlaid images

This milestone does not add new gameplay, new themes, manual dressing, or a GPU renderer. It makes
ProcGen visual outputs better suited for scanning generated mission style while preserving route and
debug exports as tactical overlays.

## ProcGen 7 — Generated mission pack playtest pass

Implemented in this drop.

- generated mission pack export now runs a pack-level playtest pass after pack selection
- added `GeneratedMissionPackPlaytestReport` with average no-prep score, average best score,
  average plan spread, per-mission scenario spread, best/worst plan labels, and notes
- each selected pack mission exports a `per_mission_playtest/mission_##_{theme}/` bundle with the
  existing mission balance reports, scenario comparison, route-shift summary, hazard-effectiveness
  summary, visual renders, and `visual_qa.json`
- added visual readability QA metrics for terrain-feature coverage, fallback sprite count,
  placeholder object count, route overlay legibility, objective/spawn visibility, and terrain
  feature visibility
- added `cargo run -p ground_cli -- playtest-mission-pack [out_dir] [mission_pack.ron|json]` for
  replaying an existing generated pack through the same harness
- `generate-mission-pack` output now includes `pack_playtest_summary.json` and
  `per_mission_playtest/` alongside the pack manifest, curves, diversity report, and contact sheets
- Mission Lab's generated-content panel now includes a mission-pack loader that opens
  `mission_pack.ron`, steps previous/next through pack slots, shows pack slot score/difficulty/
  complexity, and loads the selected mission into the playable briefing/prep/assault loop

This milestone does not add new mechanics, themes, combat systems, or renderer work. It makes a
generated pack inspectable as a playable set instead of only a list of accepted candidates.

## ProcGen 7.1 — Full pack quality gate

Implemented in this drop.

- added `GeneratedMissionPackQualityGateReport`, pack-quality rows, theme-stability reports,
  difficulty/complexity curve reports, visual QA summaries, and weak-mission diagnostics
- added `cargo run -p ground_cli -- quality-gate-mission-packs [out_dir] [--seed N]
  [--seed-count N] [--missions N] [--candidates-per-theme N] [--curve balanced|tutorial]`
- the quality gate generates one mission pack per seed under `packs/seed_{seed}/`, preserving the
  normal pack manifest, visual sheet, playtest summary, and per-mission playtest folders
- exports `seed_matrix_summary.json`, `pack_quality_report.json`, `theme_stability_report.json`,
  `difficulty_curve_report.json`, `complexity_curve_report.json`, and `visual_qa_summary.json`
- copies each generated pack contact sheet and visual sheet into `generated_pack_contact_sheets/`
  for quick scanning across the seed matrix
- exports one JSON file per weak mission under `weak_mission_reports/` with reasons and
  recommendations for low plan spread, no-prep outperforming prep, route-overlay legibility,
  terrain-feature visibility, missing/fallback sprites, and invalid objective/spawn visibility
- aggregates per-theme acceptance drift and top rejection reasons across the generated seed matrix

This milestone does not add new gameplay, new themes, campaign progression, or renderer work. It
turns ProcGen packs into a repeatable quality gate so generator reliability can be judged across
many seeds before adding campaign/set packaging.

## ProcGen 8 — Generated campaign / mission set packaging

Implemented in this drop.

- added `MissionSet`, `MissionSetSlot`, lesson roles, unlock records, save-data templates, and a
  generated campaign-set summary
- added `cargo run -p ground_cli -- generate-campaign-set [out_dir] [--seed N] [--missions N]
  [--candidates-per-theme N] [--curve balanced|tutorial] [--render-visuals]`
- campaign-set generation builds a normal generated mission pack under `source_pack/`, then copies
  selected missions into ordered `missions/###_{theme}/` folders with `mission.ron`, `mission.json`,
  `mission_visual_beauty.png`, and `mission_pack_entry.json`
- exports `mission_set.ron`, `mission_set_summary.json`, `mission_set_contact_sheet.png`,
  `mission_set_debug_contact_sheet.png`, `mission_set_save_template.json`, `unlock_curve.json`,
  `difficulty_curve.json`, and `complexity_curve.json`
- tutorial sets assign mission-slot lessons for route/prep basics, tree/material dilemmas,
  trench/berm shaping, rolling hazards, split approaches, and mixed final tests
- generated unlock curves grant capability-style kits such as saw kit, survey kit, winch, and brace
  kit after early mission slots instead of stat upgrades
- added `cargo run -p ground_cli -- playtest-campaign-set [out_dir] [mission_set.ron|json]` to
  replay packaged mission sets through the existing pack playtest harness
- Mission Lab's generated-content panel now includes a mission-set loader that opens
  `mission_set.ron`, steps previous/next through mission slots, shows each slot's lesson/unlocks/
  difficulty/complexity, and loads the selected mission into the playable briefing/prep/assault loop

This milestone does not add new mechanics, enemy types, themes, campaign economy, or renderer work.
It packages the generated-content pipeline into playable mission sets with lesson sequencing,
capability unlock metadata, retry/play navigation, deterministic playtest exports, and save-state
templates.

## ProcGen 8.1 — Campaign set quality gate + unlock validation

Implemented in this drop.

- added `GeneratedCampaignSetQualityGateReport`, per-campaign quality rows, lesson-role reports,
  unlock-curve reports, campaign weak reports, and campaign-level aggregate notes
- added `cargo run -p ground_cli -- quality-gate-campaign-sets [out_dir] [--seed N]
  [--seed-count N] [--missions N] [--candidates-per-theme N] [--curve balanced|tutorial]
  [--render-visuals]`
- the quality gate generates one packaged mission set per seed under `campaign_sets/seed_{seed}/`,
  preserving mission-set manifests, visual sheets, playtest summaries, and per-mission playtest
  folders
- exports `campaign_set_matrix_summary.json`, `campaign_quality_report.json`,
  `lesson_role_report.json`, `unlock_curve_report.json`, `campaign_difficulty_curve_report.json`,
  `campaign_complexity_curve_report.json`, and `visual_qa_summary.json`
- copies mission-set contact sheets into `campaign_contact_sheets/` for quick visual scanning across
  the campaign seed matrix
- exports one JSON file per weak campaign under `weak_campaign_reports/` with missing lesson roles,
  missing or underused unlocks, curve issues, weak mission diagnostics, and tuning recommendations
- validates tutorial lesson coverage for route/prep basics, tree/material dilemmas, trench/berm
  shaping, rolling hazards, split approaches, and mixed final tests
- validates capability unlock metadata for saw kit, survey kit, winch, and brace kit, including
  whether each unlock has a later mission role where it can matter

This milestone does not add mechanics, themes, UI skinning, campaign economy, or renderer work. It
turns generated mission sets into a repeatable campaign-level quality gate before adding presentation
or progression systems on top.

## Visual Lock 1 — Generated mission art direction benchmark

Implemented in this drop.

- added `cargo run -p ground_cli -- visual-lock-benchmark [out_dir] [--theme ridge_trap]
  [--seed N] [--count N]`
- the benchmark command generates a fixed candidate batch under `source_candidates/`, selects the
  first accepted candidate, and preserves its mission/evaluation inputs
- exports `benchmark_visual_beauty.png`, `benchmark_visual_routes.png`,
  `benchmark_visual_debug.png`, `benchmark_visual_preview.png`,
  `benchmark_visual_asset_report.json`, and `benchmark_feature_map.json`
- applies the selected candidate's best known prep script when available and exports prepared-state
  beauty/routes/debug renders plus order validation and material ledger artifacts
- exports `benchmark_visual_audit.json` with dominant scene features, sprite role summaries,
  placeholder/fallback/override counts, and visual priority notes
- improves the mission visual renderer's benchmark framing, scale, contact shadows, perimeter
  treatment, objective prop rendering in beauty mode, and feature accents for paths, trenches,
  berms, and stone

This milestone intentionally freezes new mechanics, themes, campaign packaging, UI shell work, and
SpriteGen material-family expansion. Its purpose is to judge one generated mission in visual context
and expose the highest-impact art/composition problems before more systems are added.

## Visual Lock 2 — High-impact terrain piece polish

Implemented in this drop.

- keeps `visual-lock-benchmark` as the fixed art-direction export path, but uses it for the second
  lock pass rather than more systems work
- exports `benchmark_prepared_diff.png` to make the exact prepared-state visual change visible
- exports `benchmark_prepared_feature_overlay.png` to show prepared terrain features without route
  clutter
- extends `benchmark_visual_audit.json` with estimated visible sprite impact by terrain piece and
  estimated placeholder object impact
- updates visual priority notes to identify the largest visible sprite, largest placeholder class,
  and top prepared-feature sprite impacts
- softens path accents with lower-contrast center passes and grass edge blending
- adds warmer trench blending, broken exposed lips, grass intrusion, and soil speckles so trenches
  read less like black rectangular stamps
- adds berm top/face blending, softer exposed edges, grass intrusion, and soil speckles so berms
  read less like continuous wall slabs

This milestone still does not add mechanics, themes, campaign UI, campaign packaging, new
SpriteGen material families, or a renderer rewrite. It only improves the high-impact prepared
terrain layer and the benchmark artifacts used to judge that layer.

## Milestone 0 — Project seed

Implemented.

- Rust workspace
- Pure core crate with no UI dependencies
- Workbench shell crate
- CLI export crate
- Default dry upland art recipe

## Milestone 1 — Live pixel terrain asset lab

Implemented.

- deterministic tile generation from `TilesetRecipe`
- fixed palette ramps
- terrain materials: grass, dirt, mud, rock, trench floor, trench wall, berm top, berm face
- tile variants
- contact sheet and atlas export
- editable terrain grid preview
- brushes: paint, trench, berm, ditch, flatten
- overlays: material, height, slope, movement, route, line of sight
- custom A* route preview
- custom grid-based LOS query

## Milestone 1.1 — Erected terrain preview

Implemented.

- first software-rendered 2.5D pass
- top surfaces displaced upward by height
- exposed faces between height deltas
- workbench face fading for inspectability
- approximate hit testing in erected view

## Milestone 2 — Better asset pipeline

Implemented.

- external RON recipe loading/saving
- external RON palette loading/saving
- auto-reload polling for recipe/palette edits
- material transition tiles
- generated height masks
- generated normal maps
- generated shadow masks
- generated occlusion masks
- seam-test sheet
- validation report
- metadata-rich export bundle
- CLI `export` and `validate` commands

## Milestone 3 — Terrain extrusion and occlusion workbench

Implemented.

- generated structure-face tiles: front, left, right, and lip
- structure-face metadata in `TileMetadata`
- structure face masks for height, normal, shadow, and occlusion outputs
- validation count coverage for structure faces
- seam-test/contact-sheet display of structure faces
- 2.5D terrain preview uses generated face art instead of flat debug rectangles
- terrain lips/cut-edges are rendered using generated lip art
- projected route overlay draws in erected terrain mode
- hover-driven local cutaway lens fades occluding faces near the inspected cell
- global face fade remains as a workbench-only debug option

## Milestone 4 — Angled projection pivot

Implemented.

- `ProjectionSpec` added to `TilesetRecipe`
- default tile source size changed to `64 px`
- new `PreviewMode::AngledTerrain`
- new `ViewOrientation` enum with NE/SE/SW/NW views
- workbench rotate-left / rotate-right controls
- orientation-aware inverse picking in the angled preview
- angled diamond top-surface renderer using existing generated material tiles
- angled exposed-face renderer using generated structure-face/lip art
- local cutaway/selection support in angled view
- flat material view preserved as command/debug map
- export bundle writes default angled preview, cutaway preview, and all four orientation previews

## Milestone 4.1 — Faux-perspective 2D terrain renderer

Implemented in this drop.

- `ProjectionKind::FauxPerspective2D` added and made the default
- `PreviewMode::FauxPerspectiveTerrain` added and made the default workbench view
- default screen cell footprint set to `64x64 px`
- default faux height step set to `18 px` initially
- rectangular top-down renderer using sprite-stacked terrain faces/lips/shadows
- orientation-aware picking and 90-degree rotation retained in the faux view
- hover cutaway/selection support in faux view
- UI texture uploads downscale large previews while CLI exports stay full resolution
- CLI/app export target defaults to `exports/milestone_04_1`
- export bundle writes default faux preview, cutaway preview, and all four orientation previews

This milestone is a visual-direction pivot, not a gameplay expansion. It preserves the terrain data,
asset pipeline, pathing, LOS, validation, and experimental angled renderer while changing the main
visual projection to “actually 2D, but drawn to look 3D.”

## Milestone 4.2 — Terrain feature sprite system

Implemented in this drop.

- default demo terrain replaced with an art-directed preview map
- noisy renderer stress-test terrain preserved as `TerrainMap::stress_test`
- `TerrainFeatureMap` derives material, ledge, trench, and berm edge masks
- faux renderer uses generated transition tiles in the actual map preview
- top-tile sampling crops generated tile edges to reduce debug-grid appearance
- stronger faux-perspective front faces and contact shadows
- dedicated trench top, trench lip, berm top, berm lip, and feature-detail passes
- optional feature-mask overlay in the workbench
- CLI/app export target defaults to `exports/milestone_04_2`
- export bundle writes comparison views: `terrain_preview_faux_debug.png`,
  `terrain_preview_faux_art.png`, and `terrain_preview_faux_features.png`

This milestone is still software-preview rendering, but it moves the visual model from cell-by-cell
height strips toward coherent terrain features.

## Milestone 4.3 — Perspective sprite scene prototype

Implemented in this drop.

- `PreviewMode::PerspectiveSpriteScene` added and made the default workbench view
- `TerrainMap::visual_target` adds a small hand-composed outpost/approach scene
- `VisualScene` and `VisualTerrainForm` derive larger visual scene forms from the terrain grid
- renderer draws broad floor regions rather than one obvious square per cell
- continuous cliff-face, trench-run, berm-run, shadow, and dressing passes
- larger visual footprint defaults: `96x80 px` cells and `32 px` faux height steps
- debug overlay can outline exported visual forms
- CLI/app export target defaults to `exports/milestone_04_3`
- export bundle writes `terrain_preview_visual_target.png`, `terrain_preview_visual_target_debug.png`, and `terrain_forms.json`

This milestone intentionally demotes the previous cell-feature renderer to a diagnostic view. The
new target is an illustrated 2D scene whose sprites imply physical terrain while the simulation grid
remains hidden underneath.

## Milestone 4.4 — Terrain art-kit renderer

Implemented in this drop.

- `TerrainArtKit` and `TerrainArtPiece` added as the sprite-piece composition layer
- generated local art kit exports `terrain_artkit_atlas.png` and `terrain_artkit_manifest.json`
- perspective sprite scene consumes named pieces for floor regions, roads, trench runs, berms, cliff faces, shadows, and dressing
- art-kit pieces include irregular alpha edges, textured faces, lips, soft shadows, corner caps, and debris
- CLI/app export target defaults to `exports/milestone_04_4`

## Milestone 4.5 — External art kit + hero scene

Implemented in this drop.

- `assets/artkits/dry_upland_outpost/manifest.ron` added as the external source art-kit contract
- `assets/artkits/dry_upland_outpost/pieces/*.png` added as replaceable source sprite pieces
- `TerrainArtPiece` now includes footprint, z-bias, opacity, and occlusion metadata
- perspective sprite scene prefers the external art kit and falls back to generated placeholders
- art-kit validation reports missing required pieces, duplicate ids, bad footprints, bad opacity, and manifest/image size mismatches
- `TerrainMap::visual_target` now defaults to a smaller 16x12 hero scene for visual judgment
- CLI/app export target defaults to `exports/milestone_04_5`
- export bundle writes `terrain_artkit_validation.json` in addition to the packed atlas and manifest

The next pass should replace the generated placeholder PNGs with authored or AI-assisted art pieces:
stronger slope/ramp silhouettes, trench and berm corners/caps, prop silhouettes, cast shadows, and
scene dressing.

## Milestone 4.6 — Hero art pass

Implemented in this drop.

- external dry-upland art kit expanded from 15 pieces to 34 source pieces
- added art-directed variants for grass, road, mud, stone, trench, berm, shadow, corner, and debris kinds
- `TerrainArtKit::piece_variant(kind, seed)` added for deterministic variant selection
- perspective sprite scene now chooses stable variants for repeated art-piece kinds
- CLI/app export target defaults to `exports/milestone_04_6`

This milestone intentionally leaves the renderer architecture alone. It tests whether the external
art-kit contract can absorb a stronger source-art pass without touching simulation, visual-form
derivation, pathing, LOS, or export structure.

## Milestone 4.7 — Hero scene art direction lock

Implemented in this drop.

- `HeroScene` and `HeroPlacement` added as a hand-placed visual overlay layer
- default hero-scene manifest added at `assets/heroscenes/dry_upland_outpost_hero_01.ron`
- dry-upland art kit expanded from 34 pieces to 50 source pieces
- added prop/decal/cap/shadow kinds: grass tufts, loose rocks, dirt scrapes, trench spoil, broken berm edge, fallen log, stakes, sandbags, tool marks, large cast shadow, trench end caps, berm corners, broken ledge corner, and worn road edge patch
- perspective sprite scene draws hero placements after terrain forms and before debug/route overlays
- visual target reduced to a 14x9 scene for art-direction judgment
- export bundle writes `terrain_preview_visual_target_no_overlay.png`
- CLI/app export target defaults to `exports/milestone_04_7`

This milestone is allowed to cheat visually. The simulation grid and visual forms remain intact, but
the art-direction pass can place non-rectangular props, caps, silhouettes, and shadows to prove the
look before the renderer becomes more procedural.

## Milestone 4.8R — Target-look editable scene renderer

Implemented in this drop.

- `TerrainStampResolver` added to derive target-style stamps from `TerrainMap` and `TerrainFeatureMap`
- `TerrainStampDefinition` and `StampPiece` added as the feature-to-art bridge
- default perspective preview now renders connected terrain features as organic stamp groups instead of row-merged rectangles
- grass, road, mud, stone, trench, and berm components draw from editable terrain with software masks, lip/edge passes, shadows, pebbles, planks, and deterministic dressing
- hero-scene placements still draw on top, while route/LOS/debug overlays remain functional
- export bundle writes `terrain_stamps.json`
- CLI/app export target defaults to `exports/milestone_04_8r`

The target image is now treated as a style reference, not a backdrop. Editable GroundLab terrain
remains the source of truth.

## Milestone 4.9 — Target-look terrain composition

Implemented in this drop.

- `target_look.rs` added as the active perspective-scene renderer
- `PreviewMode::PerspectiveSpriteScene` now calls the target-look composition pass directly
- superseded 4.8R perspective-scene helper code removed rather than kept as dormant logic
- target-look picking added through `target_look_pixel_to_cell`
- roads, trenches, berms, stone platforms, mud, grass, hero dressing, and final lighting are composed from editable terrain
- CLI/app export target defaults to `exports/milestone_04_9`

The generated target image remains a style reference. GroundLab terrain data remains the editable
source of truth for rendering, pathing, LOS, and debug overlays.

## Milestone 4.10 — Target-derived editable scene

Implemented in this drop.

- `assets/visual_targets/dry_upland_outpost_01/visual_target.png` added as the committed source art
- `assets/visual_targets/dry_upland_outpost_01/manifest.ron` added for image/grid alignment
- `VisualTarget` added to load the source image and map preview pixels back to semantic cells
- `TerrainMap::target_derived(16, 12, seed)` added as the semantic map matching the image composition
- default perspective preview now draws the source image first and local replacement patches only after edits
- route, marker, grid, selection, and stamp debug overlays still draw over the target-derived scene
- export bundle writes `visual_target_source.png`
- CLI/app export target defaults to `exports/milestone_04_10`

This is the source-art correction: the target image is no longer only a reference, and the renderer
does not try to recreate the entire scene from placeholder art-kit pieces.

## Milestone 4.11 — Target-derived local edit patches

Implemented in this drop.

- `edit_patch.rs` added to compare editable terrain against the target-derived semantic baseline
- changed cells are grouped into connected dirty regions by terrain patch kind
- each patch records dirty cells, neighboring cells, image-space bounds, and old/new terrain signatures
- default perspective preview now renders local edits from patch records instead of implicit cell scanning
- patch drawing samples the target image under each edited cell so replacement art inherits local color
- target-grid / patch debug overlay shows material swatches, height marks, dirty cells, neighbor cells, and patch bounds
- export bundle writes `terrain_preview_target_base.png`
- export bundle writes `terrain_preview_target_with_edits.png`
- export bundle writes `terrain_preview_target_patch_debug.png`
- export bundle writes `terrain_edit_patches.json`
- CLI/app export target defaults to `exports/milestone_04_11`

This is the editability checkpoint for the target-derived source-art path. The base image keeps the
scene cohesive, while brush edits now produce explicit visual deltas that can be inspected, exported,
and replaced with stronger target-style patch pieces.

## Milestone 4.12 — Edit patch stress test and cover patches

Implemented in this drop.

- edit patches now record old/new patch kind, operation, cover requirement, and baked-feature touch
- patch metrics summarize dirty cells, patch count, cover patch count, patch kinds, operations, and patch bounds area
- target renderer adds a cover pass for subtractive edits before drawing replacement detail
- workbench exposes dirty-cell, patch-bounds, terrain-signature, and cover-only inspection toggles
- `edit_scenario.rs` adds scripted new-trench, new-berm, new-road, remove-trench, remove-road, flatten-trench, and paint-stone scenarios
- CLI adds `edit-scenarios` export command
- app adds `Export edit stress tests`
- bundle export writes `edit_scenarios/base.png`, edited previews, patch-debug previews, cover-only previews, per-scenario patch JSON, and `summary.json`
- CLI/app export target defaults to `exports/milestone_04_12`

This is the first visual stress-test milestone for target-derived editing. It proves the unedited
scene is no longer the only export and gives additive/removal edits concrete artifacts for judging
patch blending quality.

## ArtGen 2.1b — Trench topology polish

Implemented in this drop.

- active visual work remains in Pixel Terrain Forge instead of the full-scene renderer
- trench mask generation still exports `trench_mask_00` through `trench_mask_15`
- connected trench openings suppress shared internal lips and walls
- endpoint caps draw only on true dead ends and are blended into the trench body
- T-junctions and crosses receive a central floor resolver before edge/lip detail
- corner, dead-end, dense-clean, and single-mask preview exports added
- neighbor-seam, lip-continuity, and floor-continuity edge heatmaps added
- `trench_neighbor_pairs.json` lists the worst connected mask pairs for follow-up polish
- sprite CLI/app default export target is `exports/artgen_02_1b`

This is a narrow topology pass. It does not add berms, stone, props, scene rendering, or terrain
editor integration.

## ArtGen 3.0 — Oblique berm / mound sprite kit

Implemented in this drop.

- active visual work remains in Pixel Terrain Forge instead of the full-scene renderer
- berm rules are data-driven through the swappable sprite style profiles
- generated berm pieces include top, front face, front/back lips, end caps, inner/outer corners, contact shadow, spoil pile, and grass fringe
- berm pieces export with sprite role, anchor, footprint, z-bias, occlusion intent, and projection metadata
- `berm_contact_sheet.png` collects the generated raised-earth kit
- `berm_preview_oblique_straight.png` stages top, face, lip, shadow, spoil, and fringe as a raised mound
- `berm_preview_oblique_caps.png`, `berm_preview_oblique_corner.png`, and `berm_preview_oblique_shadow.png` provide focused visual checks
- `berm_mask_debug.png` documents the basic straight/cap/corner/shadow preview grammar
- berm validation reports piece coverage, role coverage, face/top contrast, shadow continuity, cap presence, and anchor validity
- sprite CLI/app default export target is `exports/artgen_03_0`

This is the raised-terrain counterpart to the trench work. It does not add full berm topology,
stone, props, scene rendering, or terrain editor integration.

## ArtGen 3.0b — Berm visual polish

Implemented in this drop.

- keeps the 3.0 berm piece categories and metadata contract intact
- does not add berm masks, berm topology, stone, props, scene rendering, or terrain editor integration
- front-face generation now uses an irregular mound silhouette instead of a full rectangular strip
- face shading adds a darker lower band, crevice under the top lip, and stronger base grounding
- end caps taper more naturally into the surrounding grass instead of reading as pasted wedges
- lips use chunkier broken segments with small vertical slumps
- contact shadow is stronger and broader under the mound
- berm validation now reports rectangularity, silhouette variance, base shadow strength, cap taper, and corner continuity
- sprite CLI/app default export target is `exports/artgen_03_0b`

This is the raised-terrain equivalent of the trench 2.0b polish pass. Berm topology should wait
until the straight/cap/corner previews read as mounded earth rather than a flat retaining wall.

## ArtGen 3.0c — Art override / replacement workflow

Implemented in this drop.

- each sprite style profile can now declare an `overrides/` folder
- generated sprites remain the deterministic source layer
- compatible `overrides/{sprite_id}.png` files replace generated pixels while preserving sprite id, kind, role, anchor, footprint, z-bias, occlusion, and projection metadata
- missing overrides fall back to generated sprites and are reported as `Missing override`, not validation failures
- invalid override sizes and load errors fall back to generated sprites and are reported in `override_report.json`
- override warnings report suspicious alpha changes and unused override PNGs
- `sprite_manifest.ron` and `sprite_manifest.json` now record whether each effective sprite source is generated or overridden
- exports now include `generated_pieces/`, effective `pieces/`, `generated_contact_sheet.png`, `effective_contact_sheet.png`, `override_contact_sheet.png`, `override_diff_sheet.png`, and `override_report.json`
- Pixel Terrain Forge adds generated/effective/override/diff preview panels and selected-sprite override status
- sprite CLI adds `promote-overrides [style_profile]` to copy the current generated sprites into a profile override folder as editable starting art
- sprite CLI/app default export target is `exports/artgen_03_0c`

This is a workflow milestone, not a new terrain feature. It makes generated art replaceable without
breaking the metadata contract that later berm topology, stone, props, and editable-scene integration
will consume.

## ArtGen 3.1 — Berm autotile / mound topology

Implemented in this drop.

- generated `berm_mask_00` through `berm_mask_15` using the same 4-bit cardinal mask contract as paths and trenches
- connected berm openings suppress shared internal lips, faces, and caps so adjacent mound cells read as one feature
- dead-end caps draw only on true endpoints
- corners, T-junctions, and crosses receive mound-aware center/top composition instead of overlapping straight pieces
- `berm_autotile_sheet.png` exports the full mask set
- sparse, dense, loop, junction, dead-end, and corner berm topology previews added
- neighbor-seam, lip-continuity, face-continuity, and shadow-continuity heatmaps added
- `berm_neighbor_pairs.json` lists the worst connected mask pairs for follow-up polish
- berm validation now reports mask coverage, missing masks, neighbor seam score, face/lip/shadow continuity, cap coverage, corner coverage, and junction coverage
- generated/effective/override/diff contact sheets and override reports remain part of the export bundle
- sprite CLI/app default export target is `exports/artgen_03_1`

This is the raised-terrain topology counterpart to the trench 2.1b pass. It keeps rough generated art
replaceable through the 3.0c override workflow while proving that berms can connect into straights,
corners, dead ends, loops, T-junctions, and crosses without baking internal faces into shared edges.

## ArtGen 3.2 — Primitive style tuning studio

Implemented in this drop.

- Pixel Terrain Forge now presents native primitive panels for grass, dirt, path/transition, trench, berm, and projection/global tuning
- grass and dirt panels expose their color ramps plus density/detail controls
- path tuning exposes path width, core width, corner rounding, edge noise, edge jitter, softness, grass intrusion, and dirt speckle controls
- trench tuning exposes floor darkness, inner/contact/wall shadows, floor/wall detail, wood plank/knot density, lip irregularity, spoil, and grass intrusion
- berm tuning exposes mound height, face/contact shadow, top grass blend, lip highlight, edge irregularity, spoil, and grass intrusion
- projection/global tuning exposes seed, tile size, variant count, export scale, cluster discipline, and high-oblique cell/face/shadow sizing
- Forge can save the current style profile, reload/revert it, or clone it by changing the profile save path
- Forge can ignore override PNGs while tuning generated style and can promote generated sprites into overrides from the same UI
- sprite CLI/app default export target is `exports/artgen_03_2`

This is a tool milestone, not a new sprite-family milestone. It makes the current primitive families
tunable without Rust edits or manual RON patching while preserving deterministic generation,
metadata, validation, overrides, and art-kit-compatible export.

## ArtGen 3.3 — Shared topology continuity polish

Implemented in this drop.

- added a common 4-bit topology resolver for connected, exposed, dead-end, straight, corner, T-junction, and cross masks
- trench and berm opening suppression now uses the shared topology helper instead of local bit checks
- trench and berm neighbor diagnostics compare all compatible connected neighbor masks instead of only the opposite dead-end mask
- worst-neighbor JSON entries now include the topology kind of each mask and a plain reason string
- exports now include `trench_worst_neighbor_pairs.png` and `berm_worst_neighbor_pairs.png`
- exports now include `terrain_engineering_topology_preview.png` with path, trench, and berm topology shown side by side over the same pattern
- Pixel Terrain Forge adds preview panels for the shared topology preview and worst-neighbor sheets
- sprite CLI/app default export target is `exports/artgen_03_3`

This is a continuity/diagnostics milestone, not a new material milestone. It keeps topology warnings
visible, but makes them based on the connected-mask space the generated sprites actually need to
support.

## ArtGen 4.0 — Stone platform / raised terrain kit

Implemented in this drop.

- added stone palette entries, stone style rules, and stone motif groups for cracks, chips, and moss
- generated the first hard raised-terrain piece family: stone floor tops, front faces, side face, bevel, step front, caps, corners, contact shadow, crack decal, and moss/grass edge
- stone pieces export with projection-aware sprite roles, anchors, footprints, z-bias, and occlusion metadata
- `stone_contact_sheet.png` shows the generated stone pieces alongside the existing effective sprite workflow
- oblique stone previews now export platform, steps, caps, corner, and mask-debug views
- Pixel Terrain Forge adds a Stone primitive tuning panel for ramp colors, top/face contrast, bevel/step/shadow strength, crack/chip density, slab jitter, and moss density
- Forge adds preview panels for the stone contact sheet and stone oblique previews
- validation now reports stone piece coverage, role coverage, top/face contrast, bevel contrast, step presence, shadow continuity, cap presence, and anchor validity
- sprite CLI/app default export target is `exports/artgen_04_0`

This is the hard raised-terrain equivalent of the trench and berm visual-piece milestones. It proves
stone/platform sprites can participate in the same 2.5D role metadata and override/export workflow,
but intentionally does not add stone topology yet.

## Milestone 5 — Custom renderer/runtime

- introduce `ground_render`
- `wgpu` native renderer
- sprite batching
- tilemap layer renderer
- nearest-neighbor sampling / pixel-perfect camera
- y + elevation sorting
- debug overlay render passes
- keep `eframe` either as editor shell or replace it with raw `winit + wgpu + egui`

## Milestone 6 — Terrain gameplay sandbox

- fixed-step simulation
- enemy agents
- route preview vs actual movement
- cover queries
- defender positions
- selected-defense LOS overlay
- objective and spawn definitions

## Milestone 7 — Rolling hazard sandbox

- custom rolling-log model based on height gradient
- controllable release triggers
- collision against enemies/obstacles at grid/sprite level
- path trace overlay
- impact/damage summary

## Milestone 8 — Prepared-ground vertical slice

- prep phase
- assault phase
- budget/labor model
- post-run explanation
- one polished dry upland outpost map
