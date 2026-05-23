# Asset pipeline

GroundLab is recipe-first and export-oriented. The goal is not to create one-off pretty images;
it is to generate deterministic, metadata-rich terrain assets that the workbench and future runtime
can use for terrain shaping, pathing, LOS, cover, and occlusion.

## Inputs

```txt
recipes/dry_upland_outpost.ron
palettes/muted_field_32.ron
```

`TilesetRecipe` controls tile size, seed, variant count, lighting, transitions, masks, seam-warning
thresholds, structure-face generation, face shading/detail, and local cutaway tuning.
`PaletteFile` defines named ramps as hex colors. Ramp names must match the material ramp names used by
`GroundMaterial::ramp()`.

## Generation

`Tileset::generate_with_palette` creates:

- surface tiles for all `GroundMaterial` values
- transition tiles for selected material pairs and four edge directions
- structure-face tiles for exposed terrain bodies
- lip tiles for cut/ledge highlights
- deterministic variants based on seed/material/role/variant
- metadata for each tile

## Structure faces

Milestone 3 adds first-class structure faces:

```txt
StructureFaceKind::Front
StructureFaceKind::Left
StructureFaceKind::Right
StructureFaceKind::Lip
```

These are used by the 2.5D preview whenever height differences expose terrain body. Grass and dirt
surface cuts currently use a dirt face; rock uses rock; trench cuts use trench-wall material; berms
use berm-face material.

This is still a software preview, but it is a stronger asset contract for the future GPU renderer:
height is visual body, not just color.

## Masks

`ground_core::mask` creates atlas-aligned masks:

- `terrain_height_mask.png`
- `terrain_normal.png`
- `terrain_shadow_mask.png`
- `terrain_occlusion_mask.png`

Structure-face tiles produce stronger height/shadow/occlusion signals than regular surface tiles.
These maps are still generated approximations, not final hand-authored production maps.

## Validation

`ground_core::validation` checks:

- required palette ramps
- surface tile counts
- transition tile counts
- structure-face tile counts
- same-material seam deltas across variants
- average drift from palette anchor colors

The seam test image now includes structure-face tiles after surface and transition sections. The JSON
report is the machine-readable gate.

## Workbench hot reload

The app polls modification timestamps for the active recipe and palette files. It is intentionally
simple and dependency-free for now. A later editor pass can replace this with a richer file-watcher if needed.
