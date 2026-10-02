# Radio effects for 1.0 source

The source of `tfar_radio_effects_1_0`: the radio effects, the signal stages they chain, the
error-level curve, the stereo mixdown, and the role constant, error type and prelude.

## Contents

```text
crates/version_1_0/tfar_radio_effects_1_0/src/
├── crate_role.rs            `CRATE_ROLE`: what the crate is for, in one sentence
├── effects/                 the long-range, airborne, personal and underwater effects and their speaker chain
├── effects.rs               the `effects` module tree
├── error.rs                 `Error`, the `thiserror` enum, and `Result`
├── error_level.rs           `error_level_from_distance`: the 1.0 table, interpolated per tenth
├── lib.rs                   the crate root: module header, `mod` lines and re-exports
├── mono_mix_buffer.rs       `MonoMixBuffer`: the reusable mono block of the mixdown
├── prelude.rs               the public API except `Error` and `Result`, plus `StereoMode`
├── radio_effect.rs          `RadioEffect`, the effect interface, and `SAMPLE_RATE_HZ`
├── radio_effect_mixdown.rs  `process_radio_effect`: mono mixdown, effect, clamp, ear routing
├── signal_stages/           the delay line, ring modulator, foldback, dropouts and f32 narrowing
├── signal_stages.rs         the `signal_stages` module tree
└── tests/                   one unit test file per module and the shared signal measurements
```

## How it works

`process_radio_effect` multiplies the gain by 1.5 for one ear, mixes each stereo frame to
`(left + right) · (1/2 · gain · 1/32766)` in `f32` in the `MonoMixBuffer`, calls
`RadioEffect::process` on the whole block, and writes `trunc(clamp(sample, −1, 1) · 32766)` back
through `tfar_sample_buffer::float_to_sample` to one or both channels. The effects in `effects/` are
built from `signal_stages/` and the `MonoFilter` of `tfar_dsp_filters`; the speaker effects map
their error level through `error_level.rs`, the underwater effect uses the raw value. Each filter
output is rounded back to `f32`, as the original's float buffers are.
`tests/signal_measurements.rs` holds the sines, RMS, tone share and band-shape helpers the effect
tests share.

## Public surface

- `RadioEffect`, `SAMPLE_RATE_HZ`, `LongRangeRadioEffect`, `AirborneRadioEffect`,
  `PersonalRadioEffect`, `UnderWaterRadioEffect`, `error_level_from_distance`,
  `process_radio_effect`, `MonoMixBuffer`, `CRATE_ROLE`, `Error`, `Result` and `prelude`,
  re-exported at the crate root.

## Boundaries

- Depends on: `tfar_dsp_filters`, `tfar_sample_buffer`, `tfar_arma_values`, `fastrand`,
  `thiserror`.
- Used by: `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  `process` and `process_radio_effect` allocate nothing once the mix buffer holds the block; the
  mixdown's routing, gain and clamping are pinned by `tests/radio_effect_mixdown.rs`; tests live in
  `tests/` (`cargo xtask verify test-placement`).
