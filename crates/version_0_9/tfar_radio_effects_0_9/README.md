# Radio effects for 0.9

The `tfar_radio_effects_0_9` crate: the 0.9.12 radio sound chains: filters and noise per radio type.
Used by `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.

## How it works

The crate ports `reference/tfar_0_9_12/ts/src/RadioEffect.h` and the `processRadioEffect` mixdown of
`reference/tfar_0_9_12/ts/src/plugin.cpp:2476`. A voice pipeline owns one effect per sender and radio:
[SW](/documentation/glossary.md) radios use `PersonalRadioEffect`, [LR](/documentation/glossary.md)
and airborne radios `LongRangeRadioEffect`, and [DD](/documentation/glossary.md) radios carried under
water `UnderWaterRadioEffect`. It sets the error level from the distance, then hands
`process_radio_effect` the block copied for that radio:

```text
stereo i16 block --mono mixdown, /32767, x gain--> mono f32 block --effect--> clip, x32766 --> one or both ears
```

The two speaker effects run, per sample, a 2399-sample delay, ring modulation by the error level,
foldback distortion, an [RBJ](/documentation/glossary.md) high-pass and low-pass and ×30; the
personal effect filters its microphone band first. The underwater effect drops samples at random
with about the error level as probability, band-passes round 1000 Hz and multiplies by 30. Effects
are built off the TeamSpeak audio thread; processing runs on it and allocates nothing.

`process_radio_effect` mixes each stereo frame to `trunc((left + right) / 2)`, scales it to
`sample / 32767 · gain` (×1.5 for one ear) in the `MonoMixBuffer`, calls `RadioEffect::process` on
the whole block, and writes `trunc(sample · 32766)` back, 32767 above 1 and −32768 below −1, to one or
both channels. The effects in `effects/` are built from `signal_stages/` and the `MonoFilter` of
`tfar_dsp_filters`; the speaker effects map their error level through `error_level.rs`, the
underwater effect uses the raw value. Each filter output is rounded back to `f32`, as the
original's float buffers are. `tests/signal_measurements.rs` holds the sines, RMS, tone share and
band-shape helpers the effect tests share.

## Modules

### `src/effects/`

`speaker_chain.rs` computes the foldback threshold `0.3 · (1 − error) · mean(|x|) / 0.005` from the
block as it arrives, then runs each sample through the delay line, the ring modulator, the foldback,
the RBJ high-pass, the RBJ low-pass and ×30. `long_range.rs` gives it its filters; `personal.rs`
first runs its microphone high-pass and low-pass over the whole block, so the threshold comes from
the microphone-filtered signal. `under_water.rs` draws one dropout decision per sample, then
band-passes and multiplies by 30. 0.9.12 has no airborne effect: airborne radios use the long-range
one.

### `src/signal_stages/`

Each stage is a small value with the state the original keeps in its effect object: the delay
line's slots and position, the ring modulator's phase, the dropout's generator. The stages hold no
filters; the effects in `effects/` add those. The dropouts replace the original's global, unseeded
`rand()` with one `fastrand` generator per effect, seeded by its creator, so runs are reproducible
and two effects never share a sequence; the draw keeps the 0..=32767 range of the Windows `rand()`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_effects_0_9  # the unit tests
```

## Configuration

None: the crate reads no setting. The underwater dropouts draw from a generator the creator seeds.

## Public surface

- `RadioEffect`: `set_error_level` and `process`, implemented by `LongRangeRadioEffect`,
  `PersonalRadioEffect` and `UnderWaterRadioEffect`; `SAMPLE_RATE_HZ`.
- `error_level_from_distance`: the 0.9.12 error-level curve.
- `process_radio_effect` and its reusable `MonoMixBuffer`.
- `Error`, `Result` and `prelude` (which also re-exports `StereoMode`).

## Boundaries

- Depends on: `tfar_dsp_filters` (the Butterworth and RBJ filters); `tfar_sample_buffer` (the
  stereo block); `tfar_arma_values` (`StereoMode`); `fastrand` for the seeded dropouts;
  `thiserror` for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`). Tier 2 in `crates/version_0_9`: every workspace dependency
  has a lower tier and follows the category edge matrix (`cargo xtask verify
  crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports, its radio effects included.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the effects sit in the
  0.9 voice chain.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the error-level curve this crate corrects.
