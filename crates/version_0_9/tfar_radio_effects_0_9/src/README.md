# Radio effects for 0.9 source

The source of `tfar_radio_effects_0_9`: the radio effects, the signal stages they chain, the
error-level curve, the stereo mixdown, and the role constant, error type and prelude.

## Contents

```text
crates/version_0_9/tfar_radio_effects_0_9/src/
├── crate_role.rs            `CRATE_ROLE`: what the crate is for, in one sentence
├── effects/                 the long-range, personal and underwater effects and their speaker chain
├── effects.rs               the `effects` module tree
├── error.rs                 `Error`, the `thiserror` enum, and `Result`
├── error_level.rs           `error_level_from_distance`: the 0.9.12 table, interpolated per tenth
├── lib.rs                   the crate root: module header, `mod` lines and re-exports
├── mono_mix_buffer.rs       `MonoMixBuffer`: the reusable mono block of the mixdown
├── prelude.rs               the public API except `Error` and `Result`, plus `StereoMode`
├── radio_effect.rs          `RadioEffect`, the effect interface, and `SAMPLE_RATE_HZ`
├── radio_effect_mixdown.rs  `process_radio_effect`: mono mixdown, effect, clip, ear routing
├── signal_stages/           the delay line, ring modulator, foldback, dropouts and f32 narrowing
├── signal_stages.rs         the `signal_stages` module tree
└── tests/                   one unit test file per module and the shared signal measurements
```

## How it works

`process_radio_effect` mixes each stereo frame to `trunc((left + right) / 2)`, scales it to
`sample / 32767 · gain` (×1.5 for one ear) in the `MonoMixBuffer`, calls `RadioEffect::process` on
the whole block, and writes `trunc(sample · 32766)` back, 32767 above 1 and −32768 below −1, to one or
both channels. The effects in `effects/` are built from `signal_stages/` and the `MonoFilter` of
`tfar_dsp_filters`; the speaker effects map their error level through `error_level.rs`, the
underwater effect uses the raw value. Each filter output is rounded back to `f32`, as the
original's float buffers are. `tests/signal_measurements.rs` holds the sines, RMS, tone share and
band-shape helpers the effect tests share.

## Public surface

- `RadioEffect`, `SAMPLE_RATE_HZ`, `LongRangeRadioEffect`, `PersonalRadioEffect`,
  `UnderWaterRadioEffect`, `error_level_from_distance`, `process_radio_effect`, `MonoMixBuffer`,
  `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_dsp_filters`, `tfar_sample_buffer`, `tfar_arma_values`, `fastrand`,
  `thiserror`.
- Used by: `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  `process` and `process_radio_effect` allocate nothing once the mix buffer holds the block; the
  mixdown's routing, gain and clipping are pinned by `tests/radio_effect_mixdown.rs`; tests live in
  `tests/` (`cargo xtask verify test-placement`).
