# Radio effects for 1.0

The `tfar_radio_effects_1_0` crate: the 1.0 radio sound chains: filters, compressor input and noise
per radio type. Used by `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.

## How it works

The crate ports `reference/tfar_1_0/ts/src/RadioEffect.hpp`, its `processRadioEffect` mixdown
included. A voice pipeline owns one effect per sender and radio: [SW](/documentation/glossary.md)
radios use `PersonalRadioEffect` (or `UnderWaterRadioEffect` when carried under water), airborne
radios `AirborneRadioEffect`, and [LR](/documentation/glossary.md) radios and the intercom
`LongRangeRadioEffect`. It sets the error level from the distance, then hands
`process_radio_effect` the block copied for that radio:

```text
stereo i16 block --(left + right) x gain / (2 x 32766)--> mono f32 block --effect--> clamp, x32766 --> one or both ears
```

The three speaker effects run, per sample, foldback distortion, ×30, a 2399-sample delay and ring
modulation by the error level, then an [RBJ](/documentation/glossary.md) high-pass and low-pass. The
underwater effect drops samples at random with about the error level as probability, band-passes
round 1000 Hz and multiplies by 30. Effects are built off the TeamSpeak audio thread; processing
runs on it and allocates nothing.

`process_radio_effect` multiplies the gain by 1.5 for one ear, mixes each stereo frame to
`(left + right) · (1/2 · gain · 1/32766)` in `f32` in the `MonoMixBuffer`, calls
`RadioEffect::process` on the whole block, and writes `trunc(clamp(sample, −1, 1) · 32766)` back
through `tfar_sample_buffer::float_to_sample` to one or both channels. The effects in `effects/` are
built from `signal_stages/` and the `MonoFilter` of `tfar_dsp_filters`; the speaker effects map
their error level through `error_level.rs`, the underwater effect uses the raw value. Each filter
output is rounded back to `f32`, as the original's float buffers are.
`tests/signal_measurements.rs` holds the sines, RMS, tone share and band-shape helpers the effect
tests share.

## Modules

### `src/effects/`

`speaker_chain.rs` computes the foldback threshold `0.3 · (1 − error) · mean(|x|) / 0.005` from the
block as it arrives, then runs each sample through the foldback, ×30, the delay line and the ring
modulator, and the result through the RBJ high-pass and low-pass. The three speaker effects differ
only in their filters; the personal effect has no microphone filters. `under_water.rs` draws one
dropout decision per sample, then band-passes and multiplies by 30.

### `src/signal_stages/`

Each stage is a small value with the state the original keeps in its effect object: the delay
line's slots and position, the ring modulator's phase, the dropout's generator. The stages hold no
filters; the effects in `effects/` add those. The dropouts replace the original's global, unseeded
`rand()` with one `fastrand` generator per effect, seeded by its creator, so runs are reproducible
and two effects never share a sequence; the draw keeps the 0..=32767 range of the Windows `rand()`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_effects_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting. The underwater dropouts draw from a generator the creator seeds.

## Public surface

- `RadioEffect`: `set_error_level` and `process`, implemented by `LongRangeRadioEffect`,
  `AirborneRadioEffect`, `PersonalRadioEffect` and `UnderWaterRadioEffect`; `SAMPLE_RATE_HZ`.
- `error_level_from_distance`: the 1.0 error-level curve.
- `process_radio_effect` and its reusable `MonoMixBuffer`.
- `Error`, `Result` and `prelude` (which also re-exports `StereoMode`).

## Boundaries

- Depends on: `tfar_dsp_filters` (the Butterworth and RBJ filters); `tfar_sample_buffer` (the
  stereo block and the `f32` to `i16` conversion); `tfar_arma_values` (`StereoMode`); `fastrand`
  for the seeded dropouts; `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`). Tier 2 in `crates/version_1_0`: every workspace dependency
  has a lower tier and follows the category edge matrix (`cargo xtask verify
  crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, its radio effects included.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the effects sit in the
  1.0 voice chain.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the error-level curve this crate corrects.
