# GroundLab

GroundLab is currently an art-generation workbench for terrain-first tactical game sprites.

The app still contains Mission Lab, Terrain Forge, procgen, visual lock, and assault/debrief systems,
but the active product focus is Art Lab: generate sprite variants, compare them, approve useful
pieces, export contact sheets, and preview assigned overrides in a small scene.

See [docs/ART_PIVOT.md](docs/ART_PIVOT.md) for the current product direction.

## Current Focus

Art Lab should make this loop useful in one sitting:

1. Choose a sprite family.
2. Generate variants.
3. Tune simple style controls.
4. Pick or mutate a promising variant.
5. Export the selected sprite as an approved override.
6. Export a contact sheet for review.
7. Assign approved sprites to preview roles.
8. Render a small preview scene with Art Lab overrides.

The current sprite families are:

- terrain base
- path
- trench
- berm
- tree
- log
- rock
- wall
- stakes
- wire
- objective marker
- spawn marker

## Quick Start

Run the desktop app:

```bash
cargo run -p ground_app
```

In the app:

1. Open `Art Lab`.
2. Choose `Trench`, `Berm`, or `Path`.
3. Set a seed and variant count.
4. Adjust `roughness`, `contrast`, `edge emphasis`, `noise`, and `warmth`.
5. Click `Generate variants`.
6. Select a variant.
7. Use `Mutate selected` to branch from a promising sprite.
8. Use `Export selected override` or `Export contact sheet`.
9. Assign selected variants to override roles.
10. Click `Render preview with Art Lab overrides`.

Art Lab writes outputs under:

```txt
exports/art_lab/
  approved/
  contact_sheets/
  previews/
```

The override profile is saved to:

```txt
exports/art_lab/approved/art_lab_overrides.json
```

The fixed preview scene is saved to:

```txt
exports/art_lab/previews/art_lab_preview.png
```

## CLI

Generate sprite variants and a contact sheet from the CLI:

```bash
cargo run -p ground_cli -- art-variants trench 123 12 exports/art_lab/demo
```

The command exports individual PNGs, metadata JSON files, and:

```txt
exports/art_lab/demo/contact_sheet.png
```

## Paused On Purpose

These areas are intentionally not the current focus:

- new gameplay systems
- new campaign or procgen expansion
- new balance gates
- renderer rewrites
- broad refactors that do not make Art Lab more useful

## Existing Systems

Mission Lab, Terrain Forge, procgen, visual lock, and campaign-set tools remain in the repo. They are
supporting infrastructure for judging sprites in context and should not drive the near-term roadmap.

Useful examples:

```bash
cargo run -p ground_sprite_app
cargo run -p ground_sprite_cli -- export exports/artgen assets/sprite_styles/cozy_upland/style.ron
cargo run -p ground_cli -- visual-lock-benchmark exports/visual_lock_benchmark --theme ridge_trap --seed 99418113 --count 8
```

The next work should keep improving the Art Lab loop before returning to larger game/procgen systems.
