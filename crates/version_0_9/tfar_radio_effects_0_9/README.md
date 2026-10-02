# Radio effects for 0.9

The `tfar_radio_effects_0_9` crate: the 0.9.12 radio sound chains: filters and noise per radio type.
Used by `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.

## Contents

```text
crates/version_0_9/tfar_radio_effects_0_9/
├── Cargo.toml  the `tfar_radio_effects_0_9` package (lib): tier 2, workspace lints and dependencies
└── src/        the effects, their signal stages, the error-level curve and the stereo mixdown
```

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
- `Error`, `Result`, `CRATE_ROLE` and `prelude` (which also re-exports `StereoMode`).

## Boundaries

- Depends on: `tfar_dsp_filters` (the Butterworth and RBJ filters); `tfar_sample_buffer` (the
  stereo block); `tfar_arma_values` (`StereoMode`); `fastrand` for the seeded dropouts;
  `thiserror` for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.
- Rules: the error level rises with distance, never falls, and stays inside its table
  (`src/tests/error_level.rs`); each effect keeps the original's processing order and filters
  (the tests in `src/effects/tests/`). Tier 2 in `crates/version_0_9`: every workspace dependency
  has a lower tier and follows the edge matrix in `crates/README.md` (`cargo xtask verify
  crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports, its radio effects included.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the effects sit in the
  0.9 voice chain.
- [Bug fixes](/documentation/program/bug_fixes.md) — the error-level curve this crate corrects.
