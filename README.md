# GroundLab

GroundLab is a custom Rust workbench/runtime seed for a terrain-first pixel-art defense game.
It intentionally avoids commercial or full game engines. The current shell uses `eframe/egui`
only as a desktop workbench UI, while the project-owned engine code lives in `ground_core`.

## Current status: Visual Lock 1 — generated mission art direction benchmark

GroundLab has paused new systems, campaign layers, mechanics, and SpriteGen family expansion to
lock a minimum generated-mission art bar. The primary product direction is still a 2.5D tactical
engineering defense game: the player is a commander / engineer who reads a compact level, issues
prep-phase work orders, transforms terrain and local objects, then tests those choices against
predictable enemy doctrine. The current benchmark takes one accepted generated mission, renders
beauty/routes/debug views, applies the best known prep script, and audits the visual asset usage so
art direction can be judged in gameplay context instead of isolated sprites or schematic boards.

SpriteGen remains in the repository as the terrain art forge. It still provides swappable style
profiles, override PNGs, sprite manifests, validation, and exportable grass/dirt/path/trench/berm/
stone pieces. It is now supporting infrastructure rather than the main roadmap driver.

GamePivot 8 and ProcGen 1-8.1 build on the `ground_game` crate with:

- `MissionSpec`, `MissionMap`, and `MissionCell`
- earth states such as normal, scraped, trench, deep trench, spoil pile, berm, unstable, and muddy
- environment object states for trees, logs, rocks, walls, wire, stakes, and fighting positions
- `ToolLoadout`, `CrewPool`, local material stock, and enemy group doctrine specs
- deterministic queued work orders for dig trench, raise berm, flatten, fell tree, cut into logs, and place stakes
- per-order crew requirements, labor cost, elapsed prep duration, tool requirements, material inputs/outputs, validation, and preview notes
- a material ledger that records spoil/log/timber/stake changes as orders complete
- deterministic A* route previews for enemy groups after terrain/object state changes
- doctrine-specific route cost profiles for rushers, cover-seekers, flankers, obstacle-avoiders, push-through troops, and clearers
- route explanations that report trench/berm/object/cover/road cells used by each projected route
- route delta exports comparing the initial mission terrain against the post-order prepared terrain
- mission phases for prep, assault, and debrief
- deterministic enemy agents that spawn from doctrine routes and advance step by step
- basic defender positions that apply range/line-of-sight pressure
- terrain and obstacle effects for trenches, berms, mud, unstable ground, stakes, wire, and logs
- typed assault timeline events with cause, magnitude, and human-readable explanation
- rolling-log hazard states, preparation work orders, deterministic release timing, and predicted hazard paths
- slope/direction-based rolling path prediction over discrete height cells
- typed rolling hazard timeline events for release, movement, enemy hits, obstacle destruction, blocking, and spent hazards
- rolling hazard summaries in the assault debrief
- mission briefing metadata with primary and optional objectives plus enemy intel
- outcome ratings that score objective survival, stopped attackers, health remaining, prep efficiency, friendly-risk hazards, unused defenses, and hazard impact
- built-in balance scenarios for no prep, trench line, berm/stakes, basic prep, rolling-log prep, ridge chokepoint, and an overbuilt bad plan
- scenario comparison exports that show whether different prep plans produce meaningfully different assault outcomes
- route-shift, hazard-effectiveness, and rating-breakdown summaries for the balance pass
- a playable Mission Lab lifecycle: briefing, start prep, player work orders, assault, debrief, retry, and reset
- a Road Below guide checklist that nudges the first playthrough through route preview, earthwork, local material, rolling log, assault, and debrief
- player prep-plan save/load/apply controls using `exports/gamepivot_08/player_plan.ron`
- a Mission Lab rating breakdown that surfaces objective health, enemies stopped/reached, prep time, friendly-risk penalties, unused defenses, hazard impact, and route accuracy
- a compact in-app balance dashboard that shows the scripted benchmark plans and their star/score outcomes
- deterministic `MissionGeneratorSpec` records with seed, theme, terrain archetype, difficulty band, objective kind, doctrine mix, material budget style, and required affordances
- a Road-Below-like ridge/road mission grammar that generates terrain height, road cells, objectives, enemy spawns, tree clusters, loose logs, defenders, tool loadouts, prep budgets, and local material opportunities
- generated affordance reports for road approach, ridge height interest, trenchable soil, tree/timber availability, rolling-log opportunity, spawn count, and route intersections
- batch mission generation that runs the existing route-preview, balance-scenario, assault, rating, and debrief harnesses against each candidate
- accepted/rejected candidate reports with tactical-interest scoring, structured rejection kinds, rejection reasons, score breakdowns, plan-sensitivity summaries, best-known prep plan, baseline rating, best rating, route diversity, height interest, material score, work-order opportunity score, rolling-hazard score, doctrine spread, and objective vulnerability
- generated mission fingerprints for objective/spawn/ridge/tree/route/hazard patterns, used to downrank near-duplicate candidates
- ProcGen preview exports including per-candidate mission/route previews, accepted/rejected contact sheets, and a top-ranked contact sheet with route overlays plus score/sensitivity bars
- six generated mission theme classes: dry road below, orchard approach, dry wash, ridge trap, old wall, and split approach
- all-theme generation that writes per-theme candidate batches plus combined `theme_summary.json`, `all_ranked_candidates.json`, `all_rejected_candidates.json`, and cross-theme contact sheets
- a generated mission browser index with seed, theme, score, accepted/rejected state, best plan, plan sensitivity, route diversity, hazard viability, local material affordance, difficulty, complexity, primary affordance, rejection reason, and mission path for every candidate
- a Mission Lab generated-mission browser that filters accepted/rejected candidates by theme and loads any candidate mission into the playable briefing/prep/assault loop
- auto-built mission packs that pick diverse accepted candidates across themes, sort them into difficulty/complexity curves, and export pack manifests/contact sheets without manual map curation
- theme calibration reports that track acceptance rate, target bands, average score, average difficulty, average complexity, plan sensitivity, route diversity, hazard usefulness, material affordance, top rejection reasons, and generator-tuning recommendations per theme
- pack diversity reports that check unique themes, tree/material coverage, rolling-hazard coverage, split-route coverage, and curve monotonicity
- visual theme bindings that map generated mission themes to SpriteGen style profiles
- high-oblique mission visual previews rendered from generated/effective SpriteGen terrain pieces
- per-candidate visual exports for beauty preview, compatibility preview, route overlay, debug grid, feature map, and sprite asset report
- visual contact sheets for generated batches and mission packs
- mission pack playtest reports that replay each selected mission through the scripted balance/debrief harness
- per-mission pack playtest bundles with scenario comparisons, route/hazard summaries, visual renders, and visual QA
- visual QA metrics for terrain-feature coverage, fallback sprite count, placeholder object count, route overlay legibility, objective/spawn visibility, and feature visibility
- a Mission Lab mission-pack panel for loading `mission_pack.ron`, stepping previous/next through pack slots, and loading any pack mission into the playable loop
- a multi-seed mission-pack quality gate that generates packs across seed matrices and aggregates pack stability, theme acceptance drift, difficulty/complexity curve quality, visual QA, and weak mission diagnostics
- generated campaign / mission-set manifests that package selected missions into ordered slots with lessons, relative mission paths, visual paths, difficulty, complexity, and best-plan metadata
- mission-set lesson roles for basic route/prep, tree-material dilemmas, trench/berm shaping, dead ground, rolling hazards, split approaches, hard-cover decisions, and mixed final tests
- capability-based unlock curves for saw kit, survey kit, winch, and brace kit milestones
- minimal mission-set save templates for completed missions, ratings, and unlocked kits
- a campaign-set playtest command that replays packaged mission sets through the existing pack playtest harness
- a Mission Lab mission-set panel for loading `mission_set.ron`, stepping previous/next through slots, and playing/retrying packaged missions
- a campaign-set quality gate that generates mission sets across seed matrices and validates lesson-role coverage, unlock sanity, campaign difficulty/complexity curves, visual QA, and weak campaign diagnostics
- lesson-role reports that check route/prep basics, tree/material dilemmas, trench/berm shaping, rolling hazards, split approaches, and mixed-final slots
- unlock-curve reports that check whether saw kit, survey kit, winch, and brace kit appear and have later mission roles where they matter
- weak campaign reports that preserve actionable reasons and recommendations per failed seed
- a Visual Lock benchmark exporter that selects one accepted generated mission from a fixed seed/theme batch
- benchmark beauty/routes/debug visual exports for the initial mission and the prepared mission state
- a visual audit that records generated/effective/override/fallback sprite usage, placeholder object counts, dominant scene features, sprite role summaries, and highest-priority visual notes
- a Mission Lab file loader for opening generated `mission.ron` candidates directly from disk
- assault summary and debrief exports for debugging why the plan worked or failed
- per-cell influence summaries for crossed cells, delayed cells, damaging cells, defender pressure, breach cells, effective obstacles, and unused defenses
- prediction-vs-actual route comparison for doctrine preview accuracy
- assault delay, pressure, and prediction-vs-actual PNG exports
- a seed mission, `The Road Below`, with a small road/ridge/tree terrain problem, a briefing card, optional objectives, and balance scripts
- CLI export of mission spec, order script, before/after mission state, work log, material ledger, validation, route previews, route delta, ASCII maps, PNG mission previews, and a summary
- a `Mission Lab` tab in `ground_app` beside the older terrain forge controls, now organized as a tactical prep screen with mission status, order mode toolbar, selected-cell context actions, work-order queue, route overlay controls, assault controls, debrief panel, delay/pressure/actual/hazard map modes, enemy intel, objective panel, minimap, notifications, material ledger, and validation feedback

Run the sprite workbench:

```bash
cargo run -p ground_sprite_app
```

Export the sprite bundle:

```bash
cargo run -p ground_sprite_cli -- export exports/artgen_04_0 assets/sprite_styles/cozy_upland/style.ron
```

Promote the current generated sprites into a profile's override folder as editable starting art:

```bash
cargo run -p ground_sprite_cli -- promote-overrides assets/sprite_styles/cozy_upland/style.ron
```

Export the GamePivot mission seed:

```bash
cargo run -p ground_cli -- mission-seed exports/gamepivot_03_seed
```

Run the scripted work-order and route-preview scenario:

```bash
cargo run -p ground_cli -- mission-routes exports/gamepivot_05_1_routes
```

Run the deterministic GamePivot 5.1 assault readability export:

```bash
cargo run -p ground_cli -- mission-assault exports/gamepivot_05_1
```

Run the deterministic GamePivot 6 rolling hazard sandbox:

```bash
cargo run -p ground_cli -- mission-hazards exports/gamepivot_06
```

Run the GamePivot 7 mission balance pass:

```bash
cargo run -p ground_cli -- mission-balance exports/gamepivot_07
```

Generate a batch of ProcGen 7 all-theme mission candidates:

```bash
cargo run -p ground_cli -- generate-missions exports/procgen_07_batch --theme all --count 20 --seed 99418113 --render-visuals
```

ProcGen output includes:

```txt
exports/procgen_07_batch/
  browser_index.json
  theme_summary.json
  all_ranked_candidates.json
  all_rejected_candidates.json
  per_theme/
    orchard_approach/
    dry_wash/
    ridge_trap/
    old_wall/
    split_approach/
  contact_sheets/
    accepted_by_theme.png
    accepted_by_theme_visual.png
    rejected_by_reason.png
    rejected_by_reason_visual.png
    top_ranked_all_themes.png
    top_ranked_all_themes_visual.png
  per_theme/*/candidates/*/
    mission_visual_beauty.png
    mission_visual_preview.png
    mission_visual_routes.png
    mission_visual_debug.png
    generated_feature_map.json
    visual_asset_report.json
```

Build a generated mission pack from the accepted candidates:

```bash
cargo run -p ground_cli -- generate-mission-pack exports/procgen_07_pack --seed 99418113 --missions 6 --candidates-per-theme 20 --curve tutorial --render-visuals
```

Mission pack output includes:

```txt
exports/procgen_07_pack/
  mission_pack.ron
  mission_pack_summary.json
  mission_pack_contact_sheet.png
  mission_pack_visual_sheet.png
  difficulty_curve.json
  complexity_curve.json
  pack_diversity_report.json
  pack_playtest_summary.json
  per_mission_playtest/
  source_candidates/
    browser_index.json
    per_theme/
```

Replay an existing generated mission pack through the playtest harness:

```bash
cargo run -p ground_cli -- playtest-mission-pack exports/procgen_07_playtest exports/procgen_07_pack/mission_pack.ron
```

Pack playtest output includes:

```txt
exports/procgen_07_playtest/
  pack_playtest_summary.json
  per_mission_playtest/
    mission_01_*/
      mission_balance_summary.json
      scenario_comparison.json
      route_shift_summary.json
      hazard_effectiveness.json
      visual/
        mission_visual_beauty.png
        mission_visual_routes.png
        mission_visual_debug.png
      visual_qa.json
```

Run the ProcGen 7.1 multi-seed pack quality gate:

```bash
cargo run -p ground_cli -- quality-gate-mission-packs exports/procgen_07_1 --seed 99418113 --seed-count 3 --missions 6 --candidates-per-theme 20 --curve tutorial --render-visuals
```

Quality-gate output includes:

```txt
exports/procgen_07_1/
  seed_matrix_summary.json
  pack_quality_report.json
  theme_stability_report.json
  difficulty_curve_report.json
  complexity_curve_report.json
  visual_qa_summary.json
  weak_mission_reports/
  generated_pack_contact_sheets/
  packs/
    seed_*/
      mission_pack.ron
      pack_playtest_summary.json
      per_mission_playtest/
```

Generate a ProcGen 8 campaign / mission set from the pack builder:

```bash
cargo run -p ground_cli -- generate-campaign-set exports/procgen_08_campaign --seed 99418113 --missions 6 --candidates-per-theme 20 --curve tutorial --render-visuals
```

Campaign-set output includes:

```txt
exports/procgen_08_campaign/
  mission_set.ron
  mission_set_summary.json
  mission_set_contact_sheet.png
  mission_set_debug_contact_sheet.png
  mission_set_save_template.json
  unlock_curve.json
  difficulty_curve.json
  complexity_curve.json
  missions/
    001_*/
      mission.ron
      mission.json
      mission_visual_beauty.png
      mission_pack_entry.json
  source_pack/
    mission_pack.ron
    pack_playtest_summary.json
    per_mission_playtest/
```

Replay a generated campaign / mission set through the playtest harness:

```bash
cargo run -p ground_cli -- playtest-campaign-set exports/procgen_08_playtest exports/procgen_08_campaign/mission_set.ron
```

Campaign playtest output includes:

```txt
exports/procgen_08_playtest/
  mission_set_playtest_summary.json
  pack_playtest_summary.json
  per_mission_playtest/
```

Run the ProcGen 8.1 campaign-set quality gate:

```bash
cargo run -p ground_cli -- quality-gate-campaign-sets exports/procgen_08_1 --seed 99418113 --seed-count 3 --missions 6 --candidates-per-theme 20 --curve tutorial --render-visuals
```

Campaign quality-gate output includes:

```txt
exports/procgen_08_1/
  campaign_set_matrix_summary.json
  campaign_quality_report.json
  lesson_role_report.json
  unlock_curve_report.json
  campaign_difficulty_curve_report.json
  campaign_complexity_curve_report.json
  visual_qa_summary.json
  weak_campaign_reports/
  campaign_contact_sheets/
  campaign_sets/
    seed_*/
      mission_set.ron
      mission_set_summary.json
      mission_set_contact_sheet.png
      mission_set_playtest_summary.json
      per_mission_playtest/
```

Export the Visual Lock 1 generated-mission benchmark:

```bash
cargo run -p ground_cli -- visual-lock-benchmark exports/visual_lock_01 --theme ridge_trap --seed 99418113 --count 8
```

Visual Lock output includes:

```txt
exports/visual_lock_01/
  benchmark_mission.ron
  benchmark_mission.json
  benchmark_candidate_evaluation.json
  benchmark_visual_beauty.png
  benchmark_visual_routes.png
  benchmark_visual_debug.png
  benchmark_visual_preview.png
  benchmark_visual_asset_report.json
  benchmark_feature_map.json
  benchmark_prepared_visual_beauty.png
  benchmark_prepared_visual_routes.png
  benchmark_prepared_visual_debug.png
  benchmark_prepared_visual_asset_report.json
  benchmark_prepared_feature_map.json
  benchmark_prepared_order_script.ron
  benchmark_prepared_order_validation.json
  benchmark_prepared_material_ledger.json
  benchmark_visual_audit.json
  source_candidates/
```

Render one mission directly as a high-oblique visual preview:

```bash
cargo run -p ground_cli -- render-mission exports/procgen_06_visual path/to/mission.ron
```

Visual output includes:

```txt
exports/procgen_06_visual/
  mission_visual_beauty.png
  mission_visual_preview.png
  mission_visual_routes.png
  mission_visual_debug.png
  generated_feature_map.json
  visual_asset_report.json
```

Calibrate every theme with the same evaluator:

```bash
cargo run -p ground_cli -- calibrate-themes exports/procgen_05 --count 200 --seed 99418113
```

Calibration output includes:

```txt
exports/procgen_05/
  theme_calibration_report.json
  theme_calibration_summary.png
  rejection_reason_histogram.png
  difficulty_complexity_scatter.png
  browser_index.json
  per_theme/
```

Road Below player plans saved from Mission Lab are written to:

```txt
exports/gamepivot_08/player_plan.ron
```

Run the mission workbench:

```bash
cargo run -p ground_app
```

The full-scene terrain renderer and ArtGen outputs remain downstream infrastructure for terrain
data, pathing, LOS, and art-kit composition. ProcGen 6 reconnects that asset pipeline to generated
missions through deterministic visual previews, ProcGen 6.1 separates beauty/routes/debug outputs,
ProcGen 7 and 7.1 validate generated packs, and ProcGen 8/8.1 package and quality-gate campaign
sets. The active work is now visually narrower: use the generated-content pipeline only as a source
of fixed benchmark missions, then improve the high-oblique beauty render until it no longer reads as
a debug board.

## Previous status: ProcGen 8.1 — campaign set quality gate + unlock validation

ProcGen 8.1 added multi-seed campaign-set quality gates, lesson-role coverage, unlock usefulness
diagnostics, campaign-level difficulty/complexity curves, visual QA aggregation, and weak-campaign
recommendation exports.

## Previous status: ProcGen 8 — generated campaign / mission set packaging

ProcGen 8 added `MissionSet`, ordered mission slots, lesson roles, unlock metadata, save templates,
`generate-campaign-set`, `playtest-campaign-set`, and Mission Lab mission-set loading/navigation.

## Previous status: ProcGen 7.1 — full pack quality gate

ProcGen 7.1 added multi-seed pack quality gates, stability/theme drift reports,
difficulty/complexity curve diagnostics, visual QA aggregation, weak-mission reports, and generated
pack contact-sheet preservation.

## Previous status: ProcGen 7 — generated mission pack playtest pass

ProcGen 7 added `pack_playtest_summary.json`, per-mission playtest bundles, visual QA metrics,
`playtest-mission-pack`, and Mission Lab pack loading so generated packs can be evaluated as
playable sets instead of only as accepted candidate lists.

## Previous status: ProcGen 6.1 — generated mission visual composition polish

ProcGen 6.1 split generated mission visuals into beauty/routes/debug renders, kept
`mission_visual_preview.png` as a compatibility alias, increased feature scale, improved backdrop
and object silhouettes, added `generated_feature_map.json`, expanded `visual_asset_report.json`,
and made visual contact sheets prefer beauty renders.

## Previous status: ProcGen 6 — generated mission visual integration

ProcGen 6 added visual theme bindings, high-oblique generated mission previews backed by effective
SpriteGen art, per-candidate visual previews/routes/debug images, visual contact sheets for batches
and packs, `visual_asset_report.json`, `render-mission`, and Mission Lab's `visual` map mode.

## Previous status: ProcGen 5 — theme calibration and difficulty curves

ProcGen 5 added theme calibration reports, per-theme acceptance target bands, difficulty and
complexity scoring, tutorial/balanced pack curves, pack diversity reports, and candidate-card
difficulty/complexity metadata.

## Previous status: ProcGen 4 — generated mission browser and mission packs

ProcGen 4 added `browser_index.json`, Mission Lab generated-mission browsing, candidate cards,
theme filters, accepted-only browsing, direct candidate loading, and automatic mission-pack export
with pack manifests, difficulty curves, and contact sheets.

## Previous status: ProcGen 3 — theme classes

ProcGen 3 added six mission theme grammars: dry road below, orchard approach, dry wash, ridge trap,
old wall, and split approach. All-theme generation writes per-theme candidate batches plus combined
ranked/rejected reports and cross-theme contact sheets.

## Previous status: ProcGen 2 — candidate evaluation and batch ranking

ProcGen 2 added structured rejection kinds, score breakdowns, plan-sensitivity metrics, generated
mission fingerprints, near-duplicate filtering, richer candidate contact sheets, and Mission Lab
loading for generated `mission.ron` files.

## Previous status: ProcGen 1 — terrain / mission generator seed

ProcGen 1 added deterministic Road-Below-like mission generation, affordance reports, candidate
evaluation through the existing balance harness, initial accepted/rejected ranking, and basic batch
contact sheets.

## Previous status: GamePivot 8 — first playable Road Below slice

GamePivot 8 added the playable briefing-to-debrief Mission Lab loop, player prep-plan save/load,
Road Below guide checklist, lifecycle controls, rating breakdown, retry flow, and in-app balance
dashboard.

## Previous status: GamePivot 7 — first balanced mission pass

GamePivot 7 added mission briefing metadata, mission ratings, Road Below balance scripts, the
`mission-balance` export, scenario comparison reports, and in-app briefing/rating surfaces.

## Previous status: GamePivot 6 — rolling hazard sandbox

GamePivot 6 added prepared rolling-log states, deterministic path prediction, manual/scripted
release, typed rolling-hazard timeline events, hazard impact summaries, friendly-risk reporting,
hazard preview PNG exports, and Mission Lab hazard overlay/actions.

## Previous status: GamePivot 5.1 — assault readability and debrief

GamePivot 5.1 added typed assault event causes and magnitudes, `AssaultDebrief`, influence
summaries, unused-defense reporting, prediction-vs-actual route accuracy, delay/pressure heatmaps,
actual path traces, and Mission Lab delay/pressure/actual map modes.

## Previous status: GamePivot 5 — assault sandbox

GamePivot 5 added mission phases, deterministic enemy agents, route-following assault movement,
terrain/obstacle delay and damage effects, defender range/LOS pressure, assault summaries, start/end
previews, and Mission Lab start/step/run/reset controls.

## Previous status: GamePivot 4 — tactical prep UI

GamePivot 4 reorganized Mission Lab into a prep screen with mission status, action modes, map modes,
route controls, enemy intel, objective panel, minimap, work-order queue, notifications, and
validation feedback.

## Previous status: GamePivot 3 — doctrine route preview

GamePivot 3 added doctrine-specific A* route previews, route delta exports, multiple Road Below
enemy groups, `mission-routes`, route explanation JSON, route preview PNGs, Mission Lab route
overlay modes, and route-preview regression coverage.

## Previous status: GamePivot 2 — work orders and local materials

GamePivot 2 added queued work orders, per-order validation, crew/time/material costs, local
material ledger entries, context-sensitive Mission Lab order buttons, preview cards, and the
`mission-orders` CLI export.

## Previous status: GamePivot 1 — mission workbench seed

GamePivot 1 added the first mission/prep data layer: `ground_game`, mission specs, mission maps,
earth/object state machines, tools, crew, local materials, deterministic immediate work orders, the
Road Below seed mission, `mission-seed` export, and the initial Mission Lab tab.

## Previous status: ArtGen 4.0 — stone platform / raised terrain kit

ArtGen 4.0 added the first hard raised-terrain sprite family: stone platform sprites. The generator
now produces stone top surfaces, front/side faces, bevels, steps, caps, corners, contact shadow,
crack decals, and moss/grass edge pieces with role, anchor, footprint, z-bias, occlusion, override,
and validation metadata.

## Previous status: Milestone 4.12 — edit patch stress tests and cover patches

Milestone 4.12 tests whether the target-derived scene survives editing. It adds scripted edit
scenarios, cover/erase patch handling for subtractive edits, patch quality metrics, and workbench
toggles for inspecting dirty cells, patch bounds, terrain signatures, and cover passes.

Milestone 4.11 built on the target-derived source image and made terrain edits explicit local
patches. The generated target image is still the base scene, but every divergence from the aligned
semantic terrain baseline becomes a tracked dirty region with cell changes, neighbor context, pixel
bounds, old/new terrain signatures, and now operation metadata.

Milestone 4.10 made the generated target image a real source asset instead of a comparison image.
The default renderer draws `assets/visual_targets/dry_upland_outpost_01/visual_target.png` as the
base scene, aligns a 16x12 semantic terrain grid to it, and renders only local replacement patches
when editable terrain diverges from that target-derived baseline.

New in this milestone:

- `assets/visual_targets/dry_upland_outpost_01/visual_target.png` is the committed visual source
- `assets/visual_targets/dry_upland_outpost_01/manifest.ron` defines the grid/image alignment
- `VisualTarget` loads the image and maps pixels to terrain cells for editing
- `TerrainMap::target_derived(16, 12, seed)` creates the matching semantic terrain map
- `edit_patch.rs` records dirty cells, neighbor cells, patch bounds, and old/new terrain signatures
- edit patches now classify additive, cover, replacement, height-only, and mixed operations
- cover patches repaint baked target-scene features before drawing replacement details
- `edit_scenario.rs` defines scripted new-trench, new-berm, new-road, remove-trench, remove-road, flatten-trench, and paint-stone scenarios
- `target_look.rs` draws the target image first, then patch-record-driven local edits and overlays
- `terrain_stamps.json` exports the resolved stamp list for debugging/decomposition
- `assets/artkits/dry_upland_outpost/manifest.ron` now describes 50 source art pieces
- `assets/artkits/dry_upland_outpost/pieces/*.png` contains authored-looking variants, props, caps, and shadows
- `assets/heroscenes/dry_upland_outpost_hero_01.ron` places hero-scene dressing over the terrain forms
- `TerrainArtKit::piece_variant(kind, seed)` picks stable variants for duplicate piece kinds
- `HeroScene` / `HeroPlacement` keep hand-placed visual direction separate from simulation
- `TerrainArtPiece` now carries footprint, z-bias, opacity, and occlusion metadata
- art-kit validation reports missing pieces, duplicate ids, bad footprints, and size mismatches
- `terrain_artkit_atlas.png`, `terrain_artkit_manifest.json`, and `terrain_artkit_validation.json` are exported with the bundle
- `terrain_preview_visual_target_no_overlay.png` is exported for judging art without route/marker overlays
- `terrain_preview_target_base.png` exports the untouched source-image base
- `terrain_preview_target_with_edits.png` exports the current editable scene without route/marker overlays
- `terrain_preview_target_patch_debug.png` exports semantic grid, dirty cells, patch bounds, material swatches, and height marks
- `terrain_edit_patches.json` exports the active dirty-region records
- `edit_scenarios/` exports stress-test comparison images, cover-only images, debug images, per-scenario patch JSON, and summary metrics
- perspective scene rendering now uses the target-derived source image instead of trying to repaint the whole scene procedurally
- roads, trenches, berms, grass, stone, and mud edits draw as local replacement patches
- `PreviewMode::PerspectiveSpriteScene` remains the default view, now labeled target-derived editable scene
- `VisualScene` and `VisualTerrainForm` still export larger forms derived from the terrain grid
- CLI/app export target defaults to `exports/milestone_04_12`

The prior faux, angled, erected, and flat previews remain available as debug views. The design goal
remains terrain-first: trenches, berms, elevation, line of sight, route shaping, movement cost, and
rolling-hazard scaffolding are still the core systems.

## Run

```bash
cargo run -p ground_app
```

The workbench loads these by default:

```txt
recipes/dry_upland_outpost.ron
palettes/muted_field_32.ron
assets/artkits/dry_upland_outpost/manifest.ron
assets/heroscenes/dry_upland_outpost_hero_01.ron
assets/visual_targets/dry_upland_outpost_01/manifest.ron
```

The UI can reload, save, and auto-reload the recipe/palette files while the app is running. The art
kit and hero-scene manifest are loaded by the perspective sprite renderer and can be edited on disk
between runs.

## CLI export

```bash
cargo run -p ground_cli -- export exports/milestone_04_12
```

Optional explicit files:

```bash
cargo run -p ground_cli -- export exports/milestone_04_12 recipes/dry_upland_outpost.ron palettes/muted_field_32.ron
```

Edit scenario suite only:

```bash
cargo run -p ground_cli -- edit-scenarios exports/milestone_04_12/edit_scenarios
```

Validation only:

```bash
cargo run -p ground_cli -- validate
```

## Export bundle

Milestone 4.12 writes:

```txt
terrain_atlas.png
terrain_height_mask.png
terrain_normal.png
terrain_shadow_mask.png
terrain_occlusion_mask.png
contact_sheet.png
seam_validation.png
terrain_preview.png
terrain_preview_2_5d.png
terrain_preview_cutaway.png
terrain_preview_visual_target.png
terrain_preview_visual_target_no_overlay.png
terrain_preview_visual_target_debug.png
visual_target_source.png
terrain_preview_target_base.png
terrain_preview_target_with_edits.png
terrain_preview_target_patch_debug.png
terrain_edit_patches.json
edit_scenarios/base.png
edit_scenarios/edit_new_trench.png
edit_scenarios/patch_debug_new_trench.png
edit_scenarios/cover_only_new_trench.png
edit_scenarios/terrain_edit_patches_new_trench.json
edit_scenarios/summary.json
terrain_forms.json
terrain_stamps.json
terrain_artkit_atlas.png
terrain_artkit_manifest.json
terrain_artkit_validation.json
terrain_preview_faux.png
terrain_preview_faux_cutaway.png
terrain_preview_faux_debug.png
terrain_preview_faux_art.png
terrain_preview_faux_features.png
terrain_preview_faux_ne.png
terrain_preview_faux_se.png
terrain_preview_faux_sw.png
terrain_preview_faux_nw.png
terrain_preview_angled.png
terrain_preview_angled_cutaway.png
terrain_preview_angled_ne.png
terrain_preview_angled_se.png
terrain_preview_angled_sw.png
terrain_preview_angled_nw.png
tileset_metadata.json
validation_report.json
recipe.ron
palette.ron
terrain_demo.json
```

## Project shape

```txt
crates/
  ground_core/   # engine-owned data, terrain, art kits, visual forms, generation, masks, validation, preview, path/LOS, export
  ground_app/    # desktop workbench shell; not a game engine
  ground_cli/    # deterministic asset/export/validation command
  ground_sprite_app/ # fast terrain sprite generator workbench
  ground_sprite_cli/ # deterministic sprite export command
```

## Important scope note

This is still not the final game runtime. Milestone 4.12 makes the edit layer testable: the target
image is the visual base, while editable GroundLab terrain remains the source of truth for gameplay
data, local modifications, patch records, scripted edit stress tests, pathing, LOS, and debug
overlays.
