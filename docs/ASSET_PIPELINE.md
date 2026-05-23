# Milestone 2 asset pipeline

Milestone 2 makes the asset pipeline recipe-first and export-oriented.

## Inputs

```txt
recipes/dry_upland_outpost.ron
palettes/muted_field_32.ron
```

`TilesetRecipe` controls tile size, seed, variant count, lighting, transitions, masks, and seam-warning thresholds.
`PaletteFile` defines named ramps as hex colors. Ramp names must match the material ramp names used by `GroundMaterial::ramp()`.

## Generation

`Tileset::generate_with_palette` creates:

- surface tiles for all `GroundMaterial` values
- transition tiles for selected material pairs and four edge directions
- deterministic variants based on seed/material/variant
- metadata for each tile

## Masks

`ground_core::mask` creates atlas-aligned masks:

- `terrain_height_mask.png`
- `terrain_normal.png`
- `terrain_shadow_mask.png`
- `terrain_occlusion_mask.png`

These are first-pass masks. They are good enough to define the export contract and give the future renderer useful inputs. They are not final hand-authored production maps.

## Validation

`ground_core::validation` checks:

- required palette ramps
- surface tile counts
- transition tile counts
- same-material seam deltas across variants
- average drift from palette anchor colors

The seam test image is a visual tool; the JSON report is the machine-readable gate.

## Workbench hot reload

The app polls modification timestamps for the active recipe and palette files. It is intentionally simple and dependency-free for now. A later editor pass can replace this with a richer file-watcher if needed.
