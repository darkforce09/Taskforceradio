# Sample buffer source

The source of `tfar_sample_buffer`: the stereo buffer, the TeamSpeak channel adapters, saturating
gain and mixing, and float conversion, with the role constant, the error type and the prelude.

## Contents

```text
crates/audio/tfar_sample_buffer/src/
├── channel_views.rs      `copy_to_stereo`, `write_back_from_stereo`, `mono_to_stereo`
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── float_conversion.rs   the scale constants and the `i16` and planar `f32` conversions
├── gain.rs               `apply_gain`, `apply_stereo_gain`
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── mixing.rs             `mix_into_additive`
├── prelude.rs            the public API except `Error` and `Result`, for glob import
├── sample_saturation.rs  the one truncating, saturating `f32` to `i16` cast
├── stereo_buffer.rs      `StereoBuffer`, the owned interleaved stereo block
└── tests/                one unit test file per module
```

## How it works

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never shadows
`std`'s `Result`. `StereoBuffer` exposes its frames to the other modules as `[i16; 2]` pairs, so
gain, the adapters and the conversions walk frames without indexing. Every `f32` to `i16` step
goes through `sample_saturation.rs`, whose `as` cast truncates toward zero and saturates. Each
module attaches its sibling test file in `tests/` with `#[path]`.

## Public surface

- `StereoBuffer`, `copy_to_stereo`, `write_back_from_stereo`, `mono_to_stereo`, `apply_gain`,
  `apply_stereo_gain`, `mix_into_additive`, `INPUT_SCALE`, `OUTPUT_SCALE`, `sample_to_float`,
  `float_to_sample`, `to_float_planar`, `from_float_planar`, `CRATE_ROLE`, `Error`, `Result` and
  `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `thiserror`.
- Used by: `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
  `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
