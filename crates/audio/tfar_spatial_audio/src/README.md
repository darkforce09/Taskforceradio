# Spatial audio source

The source of `tfar_spatial_audio`: the distance curve, the emitter cone and the simple
interaural level difference, with the role constant, the error type and the prelude.

## Contents

```text
crates/audio/tfar_spatial_audio/src/
├── crate_role.rs                   `CRATE_ROLE`: what the crate is for, in one sentence
├── distance_curve.rs               `DistanceCurve`, `CurvePoint`: X3DAudio's volume curve
├── emitter_cone.rs                 `EmitterCone`: X3DAudio's emitter cone
├── error.rs                        `Error`, the `thiserror` enum, and `Result`
├── interaural_level_difference.rs  `stereo_pan_gains`, `apply_pan` and the coefficients
├── lib.rs                          the crate root: module header, `mod` lines and re-exports
├── prelude.rs                      the public API except `Error` and `Result`, for glob import
└── tests/                          one unit test file per module
```

## How it works

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never
shadows `std`'s `Result`. `DistanceCurve` keeps its points in a fixed array of
`DistanceCurve::MAX_POINTS`, so it is `Copy` and never allocates; its constructor checks every
point and refuses unsorted ones. `EmitterCone` works on the `glam` vectors inside
`tfar_geometry::Direction`. Each module attaches its sibling test file in `tests/` with `#[path]`.

## Public surface

- `DistanceCurve`, `CurvePoint`, `EmitterCone`, `stereo_pan_gains`, `apply_pan`,
  `ILD_COEFFICIENT_0_9`, `ILD_COEFFICIENT_1_0`, `ILD_BASE_GAIN`, `CRATE_ROLE`, `Error`, `Result`
  and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_geometry`, `tfar_sample_buffer`, `thiserror`.
- Used by: `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
