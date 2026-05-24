# Pixel Terrain Forge

GroundLab's active visual milestone is now a dedicated terrain sprite generator, not the large
editable scene renderer. The generator first produces cozy top-surface terrain primitives: grass,
dirt, grass-to-dirt transitions, and connected dirt path masks. ArtGen 2.1b extends that foundation
with polished connected oblique trench masks; ArtGen 3.0 adds the first raised-terrain counterpart:
an oblique berm/mound sprite kit for the desired high-oblique 2.5D terrain grammar.

The generator does not require reference images. It uses:

- `TerrainSpriteRecipe`
- `TerrainSpriteStyle`
- swappable `TerrainSpriteStyleProfile` files in `assets/sprite_styles/`
- external `motifs.ron` pixel-cluster motif libraries
- cozy palette ramps
- grass, dirt, transition, path, trench, berm, and pixel-cluster rules
- a high-oblique 2.5D projection profile
- per-piece sprite role, anchor, footprint, z-bias, and occlusion metadata
- deterministic seeds
- contact sheets and repeat previews
- seam/noise validation
- art-kit-compatible PNG piece export

ArtGen 3.0 keeps the ArtGen 1.2b path topology, ArtGen 1.3 style profiles, ArtGen 1.4
projection-aware sprite contract, ArtGen 2.0b trench polish, and ArtGen 2.1b connected trench
topology. Grass/dirt/path pieces are still top-surface material primitives, trench pieces include
both the base role pieces and `trench_mask_00` through `trench_mask_15`, and berm pieces now test
the raised-earth side of the same 2.5D contract with top, front face, lips, caps, corners, contact
shadow, spoil, and grass fringe:

```txt
sprite role
anchor_px
footprint_cells
z_bias
occludes
projection:
  kind: HighOblique2D
  cell_width_px
  cell_height_px
  face_height_px
  light_direction
  shadow_offset_px
```

The generator still loads palette ramps, grass/dirt/transition/path/trench/berm rules, projection
settings, and motif libraries from profile folders:

```txt
assets/sprite_styles/
  cozy_upland/
    style.ron
    motifs.ron
  cozy_upland_lush/
    style.ron
    motifs.ron
  cozy_upland_sparse/
    style.ron
    motifs.ron
```

This keeps future terrain materials from baking the cozy look or the projection assumptions directly
into Rust code.

Run the fast workbench:

```bash
cargo run -p ground_sprite_app
```

Export the deterministic bundle:

```bash
cargo run -p ground_sprite_cli -- export exports/artgen_03_0 assets/sprite_styles/cozy_upland/style.ron
```

Export output:

```txt
exports/artgen_03_0/
  manifest.ron
  sprite_manifest.ron
  sprite_manifest.json
  recipe.ron
  contact_sheet.png
  oblique_material_preview.png
  berm_contact_sheet.png
  berm_preview_oblique_straight.png
  berm_preview_oblique_caps.png
  berm_preview_oblique_corner.png
  berm_preview_oblique_shadow.png
  berm_mask_debug.png
  trench_contact_sheet.png
  trench_preview_oblique_straight.png
  trench_preview_oblique_caps.png
  trench_preview_oblique_corner.png
  trench_preview_oblique_shadow.png
  trench_autotile_sheet.png
  trench_preview_sparse.png
  trench_preview_dense.png
  trench_preview_dense_clean.png
  trench_preview_loop.png
  trench_preview_junctions.png
  trench_preview_single_masks.png
  trench_preview_dead_ends.png
  trench_preview_corners.png
  trench_mask_debug.png
  trench_neighbor_seam_heatmap.png
  trench_lip_continuity_heatmap.png
  trench_floor_continuity_heatmap.png
  trench_neighbor_seam_heatmap_edges.png
  trench_lip_continuity_heatmap_edges.png
  trench_floor_continuity_heatmap_edges.png
  trench_neighbor_pairs.json
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
    trench_floor_top_01.png
    trench_floor_top_02.png
    trench_wall_front_01.png
    trench_wall_front_02.png
    trench_lip_front_01.png
    trench_lip_back_01.png
    trench_end_cap_left_01.png
    trench_end_cap_right_01.png
    trench_corner_inner_01.png
    trench_corner_outer_01.png
    trench_contact_shadow_01.png
    trench_spoil_pile_01.png
    trench_mask_00.png
    ...
    trench_mask_15.png
    berm_top_01.png
    berm_top_02.png
    berm_face_front_01.png
    berm_face_front_02.png
    berm_lip_front_01.png
    berm_lip_back_01.png
    berm_end_cap_left_01.png
    berm_end_cap_right_01.png
    berm_corner_inner_01.png
    berm_corner_outer_01.png
    berm_contact_shadow_01.png
    berm_spoil_pile_01.png
    berm_grass_fringe_01.png
```

The full-scene GroundLab renderer remains in the repository as downstream infrastructure for terrain
data, pathing, LOS, and art-kit composition. It is not the current art feedback loop.
