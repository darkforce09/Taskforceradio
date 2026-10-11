# Sample buffer

The `tfar_sample_buffer` crate: interleaved 16-bit sample frames as TeamSpeak hands them out, with
saturating gain. Used by `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
`tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## How it works

A TeamSpeak voice callback hands over an interleaved `i16` block of 1, 2 or more channels.
`copy_to_stereo` checks the block against its channel and frame counts and copies it into a
`StereoBuffer` (mono duplicated, channels 0 and 1 otherwise); the pipeline works on that buffer,
going through `to_float_planar` and `from_float_planar` around the `f32` filters and effects, with
`apply_gain`, `apply_stereo_gain` and `mix_into_additive` saturating at the `i16` limits; and
`write_back_from_stereo` folds the result back (mono as the average of left and right, channels 2
and up silenced). The scales follow the originals: `i16` to `f32` divides by 32767, `f32` to `i16`
clamps to [-1, 1] and multiplies by 32766. Once a `StereoBuffer` is reserved for the largest
block, nothing on these paths allocates.

`lib.rs` declares the modules privately and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`, so a glob import never shadows
`std`'s `Result`. `StereoBuffer` exposes its frames to the other modules as `[i16; 2]` pairs, so
gain, the adapters and the conversions walk frames without indexing. Every `f32` to `i16` step
goes through `sample_saturation.rs`, whose `as` cast truncates toward zero and saturates. Each
module attaches its sibling test file in `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_sample_buffer  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `StereoBuffer`: `with_frames`, `frames`, `capacity_frames`, `reserve_frames`, `set_frames`,
  `clear`, `as_slice`, `as_mut_slice`, `copy_from`, `CHANNELS`.
- `copy_to_stereo`, `write_back_from_stereo`, `mono_to_stereo`: the TeamSpeak channel adapters.
- `apply_gain`, `apply_stereo_gain`, `mix_into_additive`: saturating gain and mixing.
- `INPUT_SCALE`, `OUTPUT_SCALE`, `sample_to_float`, `float_to_sample`, `to_float_planar`,
  `from_float_planar`: float conversion.
- `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
  `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: gain and mixing saturate at the `i16` limits and never wrap; no adapter reads or writes
  past `channels * frames` samples; only `StereoBuffer::with_frames` and `reserve_frames` allocate
  by design. Tier 0 in `crates/audio`: every
  workspace dependency has a lower tier and follows the category edge matrix
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
