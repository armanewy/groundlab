# Art Pivot

GroundLab is now being steered back toward a daily-use art workflow. The goal is not to add more
game systems first. The goal is to make the app useful for producing, comparing, approving, and
exporting sprite art.

## North Star

GroundLab is an art-generation workbench first.

It helps produce usable sprites for terrain, objects, props, and tactical map readability. Mission
generation, route preview, visual lock reports, and gameplay simulation remain useful support
systems, but the daily product should begin with sprite creation.

## Immediate Non-Goals

- No new gameplay systems.
- No new procgen mission or campaign expansion.
- No new balance-gate complexity.
- No renderer rewrite.
- No giant refactor that changes behavior.

## Daily-Use Loop

1. Choose sprite family.
2. Generate variants.
3. View a contact sheet.
4. Pick or approve a variant.
5. Save it as an override/source sprite.
6. Re-render a small scene to judge readability.
7. Repeat.

## First Target Sprite Families

- grass/dirt/path base tiles
- trench/recessed terrain
- berm/raised terrain
- tree/log/stump objects
- rock/wall/stakes/wire props
- objective/spawn/readability markers

## Staged Plan

### Stage A: Document and isolate the Art Lab goal

Keep the systems freeze in place and make the product direction explicit. Art Lab should be the
default place to start, while Mission Lab and Terrain Forge remain available.

### Stage B: Make sprite generation APIs easier to call

Expose a small core API for deterministic sprite variants. The app and CLI should not need to know
which older generator internals are involved.

### Stage C: Add an Art Lab panel in the app

Make Art Lab the primary app panel. The first UI can be basic: family, seed, count, generate,
preview, select, export.

### Stage D: Add variant/contact-sheet/approve flow

The user should be able to generate many variants, inspect them together, export the selected
variant, and export a contact sheet.

### Stage E: Use approved art in mission visual previews

Approved Art Lab sprites should be usable in a small preview scene or an override profile so art can
be judged in context.

### Stage F: Only then return to gameplay/procgen work

Gameplay and procedural mission work should resume after the sprite loop is useful in one sitting.

## First Useful Version

The first useful version succeeds when this is possible in one app session:

1. Open GroundLab.
2. Select Trench in Art Lab.
3. Generate 12 trench variants.
4. Compare the variants in a grid.
5. Approve one variant.
6. Save it as an override/source sprite.
7. See it available for a terrain or mission preview path.

That is the bar before large refactors or more systems work.
