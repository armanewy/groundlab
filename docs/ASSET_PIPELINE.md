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

## Faux-perspective 2D assets

Milestone 4.2 keeps faux-perspective 2D as the default projection and adds feature-aware map rendering. The preview treats the existing
generated surface tiles as square top surfaces, then layers structure-face, lip, and shadow sprites
below them. This keeps the map top-down and editable while making height look physical.

The recipe now defaults to:

```ron
projection: (
    kind: FauxPerspective2D,
    source_tile_px: 64,
    tile_screen_width_px: 96,
    tile_screen_height_px: 48,
    faux_cell_width_px: 64,
    faux_cell_height_px: 64,
    faux_height_step_px: 24,
    faux_side_face_width_px: 16,
    height_step_px: 24,
    default_orientation: SouthEast,
    supports_four_way_rotation: true,
)
```

Milestone 4.2 now uses generated transition tiles in the map preview and exports feature-debug
comparison views. `Dimetric` remains available for experiments, but the next art-pipeline step should
improve continuous feature-run and corner sprites:

```txt
faux_top_grass
faux_top_dirt
cliff_face_front_left_corner
cliff_face_front_right_corner
trench_inner_corner
berm_outer_corner
ramp_up_front
ramp_up_side
contact_shadow_front
```

Rotation is handled by data first: terrain cells do not rotate; only the view projection does. Asset
coverage validation can later report missing orientation-specific placeables and props.


## Milestone 4.3 visual-target scene note

Milestone 4.3 separates the hidden simulation grid from the intended visual composition. The new
`PerspectiveSpriteScene` preview derives `VisualScene` / `VisualTerrainForm` records from the terrain
map and draws larger forms such as floor regions, cliff faces, trench runs, berm runs, shadows, and
field-engineering dressing. The older faux/angled/flat previews remain as diagnostic tools.

## Milestone 4.4 terrain art-kit note

Milestone 4.4 adds `TerrainArtKit`, `TerrainArtPiece`, and a generated art-kit export. The bundle now includes `terrain_artkit_atlas.png` and `terrain_artkit_manifest.json`. The manifest names pieces such as grass floors, dirt road edges, trench walls, trench lips, berm faces, stone walls, soft shadows, corner caps, and debris props. This is the first step toward replacing internal generated placeholders with imported art-kit atlases.

## Milestone 4.5 external art-kit note

Milestone 4.5 moves the art-kit source contract to `assets/artkits/dry_upland_outpost/`. The
`manifest.ron` file names each `TerrainArtPiece`, and the `pieces/` folder stores the replaceable
PNG source images. The renderer loads that folder when available, then packs the active kit into
`terrain_artkit_atlas.png` and `terrain_artkit_manifest.json` during export.

`TerrainArtPiece` now records `anchor_px`, `footprint_cells`, `repeat_mode`, `orientation`,
`z_bias`, `opacity`, `occlusion`, and tags. This keeps the simulation/form layer separate from the
actual art and gives future authored pieces enough metadata to compose long trenches, berms, ledges,
caps, shadows, and props without treating every piece as a stretched rectangle.

The export bundle also writes `terrain_artkit_validation.json`, which reports missing required
pieces, duplicate ids, invalid footprints or opacity, and manifest/image size mismatches.

## Milestone 4.6 hero art-pass note

Milestone 4.6 keeps the external art-kit contract and replaces the baseline placeholder set with 34
source PNG pieces. The manifest can now contain multiple entries with the same `TerrainArtPieceKind`,
such as several grass floors or trench walls. `TerrainArtKit::piece_variant(kind, seed)` selects a
stable variant from those duplicate kinds, so the scene renderer can add visual variety without
changing the simulation grid or visual-form derivation.

## Milestone 4.7 hero-scene overlay note

Milestone 4.7 adds `assets/heroscenes/dry_upland_outpost_hero_01.ron`. This manifest is deliberately
not simulation data. It places visual-only pieces such as fallen logs, stake clusters, sandbags,
trench spoil, broken ledge corners, road-edge patches, and large cast shadows over the derived
terrain forms.

The art kit now includes 50 source pieces. The new categories are meant to break the rectangular
strip look without requiring a renderer rewrite: `HeroScene` decides where the cinematic dressing
goes, `TerrainArtKit` supplies the sprites, and debug overlays can still reveal the hidden grid.

## Milestone 4.8R target-style stamp note

Milestone 4.8R adds `TerrainStampResolver`. It keeps editable terrain as source data, derives
connected feature components from `TerrainMap` + `TerrainFeatureMap`, and resolves those components
into `TerrainStampDefinition` records. The perspective preview then draws those stamps with organic
software masks and art-kit pieces instead of stretching visual forms into long rectangles.

The export bundle writes `terrain_stamps.json` so the stamp decomposition can be inspected alongside
`terrain_forms.json`. This is the bridge toward target-look assets such as road segments, trench
bodies, berm mounds, stone platforms, shadows, and dressing while preserving terrain brushes, pathing,
LOS, and debug overlays.

## Milestone 4.9 target-look composition note

Milestone 4.9 makes `target_look.rs` the active perspective-scene composition layer. It still consumes
editable terrain-derived stamps, but feature-specific code now composes the final scene: organic
field wash, worn dirt roads, dark plank trenches, dirt lips, berm mounds, stone platform faces and
steps, hero-scene dressing, and final lighting.

The superseded 4.8R perspective helper path was removed instead of retained as dormant code. Debug
previews remain explicit preview modes; the default visual target path is now the target-look
composition renderer.

## Milestone 4.10 target-derived source-art note

Milestone 4.10 adds `assets/visual_targets/dry_upland_outpost_01/visual_target.png` and a matching
`manifest.ron`. This is the first committed source image that the editable renderer treats as the
visual base rather than a comparison reference.

The manifest records image size, 16x12 semantic grid size, grid origin, cell size, spawn/objective
cells, and lighting notes. Export bundles now include `visual_target_source.png` so the rendered
workbench output can be compared against the exact source image.

The default perspective renderer draws the visual target first and then draws local replacement
patches only where the editable terrain map differs from `TerrainMap::target_derived`. This keeps
the art source cohesive while preserving terrain brushes, pathing, LOS, and debug overlays.

## Milestone 4.11 local edit patch note

Milestone 4.11 turns target-derived edits into exportable patch data. The active export bundle now
writes:

```txt
terrain_preview_target_base.png
terrain_preview_target_with_edits.png
terrain_preview_target_patch_debug.png
terrain_edit_patches.json
```

`terrain_preview_target_base.png` is the untouched source-art image. `terrain_preview_target_with_edits.png`
is the editable terrain state rendered without route and marker overlays. `terrain_preview_target_patch_debug.png`
shows the aligned semantic grid, material swatches, height marks, dirty cells, neighboring context,
and patch bounds. `terrain_edit_patches.json` is the machine-readable bridge for future target-style
patch pieces: it contains changed cells, old/new terrain signatures, neighbor cells, and image-space
bounds for every dirty region.

## Milestone 4.12 edit scenario and cover patch note

Milestone 4.12 adds repeatable edit-patch stress exports under `edit_scenarios/`:

```txt
base.png
edit_new_trench.png
edit_new_berm.png
edit_new_road.png
edit_remove_trench.png
edit_remove_road.png
edit_flatten_trench.png
edit_paint_stone.png
patch_debug_*.png
cover_only_*.png
terrain_edit_patches_*.json
summary.json
```

These exports intentionally include additive and subtractive cases. Subtractive cases use
`cover_required` patch metadata and a cover rendering pass to repaint the baked target feature before
new terrain detail is drawn. `summary.json` carries patch counts, dirty-cell counts, cover-patch
counts, target-baked feature touches, patch kinds, and operations so visual regressions can be judged
alongside image output.
