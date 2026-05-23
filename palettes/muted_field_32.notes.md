# muted_field_32

Milestone 2 promotes the palette from notes-only to a live RON file:

- `palettes/muted_field_32.ron` is loaded by the workbench and CLI.
- Ramps are named to match `GroundMaterial::ramp()`.
- Generated tiles are still allowed to interpolate inside ramps, but validation reports average drift from palette anchors.
