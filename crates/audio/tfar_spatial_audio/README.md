# Spatial audio

The `tfar_spatial_audio` crate: the distance curve and emitter cone gains that replace X3DAudio,
plus the 0.9 stereo fallback. Used by `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_spatial_audio/
├── Cargo.toml  the `tfar_spatial_audio` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The gain model that replaces X3DAudio: the piecewise distance curve, the emitter cone gain and the
simple interaural level difference the 0.9 stereo fallback uses. The code is not written yet:
`src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's role
in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_spatial_audio  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `DistanceCurve`: piecewise attenuation by distance.
- `EmitterCone`: inner and outer cone gain for a speaker direction.
- `interaural_level_difference`: the 0.9 stereo fallback.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `tfar_sample_buffer` (the
  shared sample frame type and saturating gain); `thiserror` for the error type.
- Used by: `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: every gain is finite and within 0 to 1. Tier 1 in `crates/audio`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
