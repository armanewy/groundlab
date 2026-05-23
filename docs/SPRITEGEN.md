# Pixel Terrain Forge

GroundLab's active visual milestone is now a dedicated terrain sprite generator, not the large
editable scene renderer. The goal is to make simple, cozy, top-down pixel terrain primitives first:
grass, dirt, grass-to-dirt transitions, and connected dirt path masks.

The generator does not require reference images. It uses:

- `TerrainSpriteRecipe`
- `TerrainSpriteStyle`
- built-in cozy palette ramps
- grass, dirt, transition, and pixel-cluster rules
- deterministic seeds
- contact sheets and repeat previews
- seam/noise validation
- art-kit-compatible PNG piece export

ArtGen 1.2b polishes the first topology layer over the 1.1b material baseline: a 16-mask grass/dirt
path autotile set, more consistent path width, softer corners and junctions, sparse/dense/loop/
junction connected-path previews, mask-debug preview, path neighbor seam heatmap, path mask coverage
validation, single-tile and variant repeat previews, seam heatmaps, motif heatmaps, and validation
for visible repetition.

Run the fast workbench:

```bash
cargo run -p ground_sprite_app
```

Export the deterministic bundle:

```bash
cargo run -p ground_sprite_cli -- export exports/artgen_01_2b
```

Export output:

```txt
exports/artgen_01_2b/
  manifest.ron
  recipe.ron
  contact_sheet.png
  path_autotile_sheet.png
  path_preview_random.png
  path_preview_random_sparse.png
  path_preview_random_dense.png
  path_preview_loop.png
  path_preview_junctions.png
  path_preview_mask_debug.png
  path_neighbor_seam_heatmap.png
  repeat_preview_grass_single.png
  repeat_preview_grass_variants.png
  repeat_preview_dirt_single.png
  repeat_preview_dirt_variants.png
  repeat_preview_transition.png
  repeat_preview_transition_edges.png
  seam_heatmap.png
  motif_heatmap.png
  palette_preview.png
  validation.json
  pieces/
    grass_tile_01.png
    grass_tile_02.png
    grass_tile_03.png
    grass_tile_04.png
    dirt_tile_01.png
    dirt_tile_02.png
    dirt_tile_03.png
    dirt_tile_04.png
    grass_dirt_edge_north_01.png
    grass_dirt_edge_south_01.png
    grass_dirt_edge_east_01.png
    grass_dirt_edge_west_01.png
    path_mask_00.png
    ...
    path_mask_15.png
```

The full-scene GroundLab renderer remains in the repository as downstream infrastructure for terrain
data, pathing, LOS, and art-kit composition. It is not the current art feedback loop.
