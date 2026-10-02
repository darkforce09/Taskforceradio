# Radio effects for 1.0

The `tfar_radio_effects_1_0` crate: the 1.0 radio sound chains: filters, compressor input and noise
per radio type. Used by `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_radio_effects_1_0/
├── Cargo.toml  the `tfar_radio_effects_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the effects, their signal stages, the error-level curve and the stereo mixdown
```

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
- `Error`, `Result`, `CRATE_ROLE` and `prelude` (which also re-exports `StereoMode`).

## Boundaries

- Depends on: `tfar_dsp_filters` (the Butterworth and RBJ filters); `tfar_sample_buffer` (the
  stereo block and the `f32` to `i16` conversion); `tfar_arma_values` (`StereoMode`); `fastrand`
  for the seeded dropouts; `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`). Tier 2 in `crates/version_1_0`: every workspace dependency
  has a lower tier and follows the edge matrix in `crates/README.md` (`cargo xtask verify
  crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, its radio effects included.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the effects sit in the
  1.0 voice chain.
- [Bug fixes](/documentation/program/bug_fixes.md) — the error-level curve this crate corrects.
