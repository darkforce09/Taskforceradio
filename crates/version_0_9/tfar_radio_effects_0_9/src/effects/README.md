# Radio effects 0.9

The 0.9.12 effects of `tfar_radio_effects_0_9`, one file each, and the speaker chain the long-range
and personal effects share (`reference/tfar_0_9_12/ts/src/RadioEffect.h`).

## Contents

```text
crates/version_0_9/tfar_radio_effects_0_9/src/effects/
├── long_range.rs     `LongRangeRadioEffect`: speaker band 520 Hz Q 0.97 to 1300 Hz Q 1.0
├── personal.rs       `PersonalRadioEffect`: microphone band 900/0.85 to 3000/2.0, then 520/0.97 to 4300/2.0
├── speaker_chain.rs  the delay, ring modulation, foldback, speaker filters and ×30 both share
├── under_water.rs    `UnderWaterRadioEffect`: seeded dropouts, Butterworth band-pass 1000/400, ×30
└── tests/            silence, the 2399-sample delay, error 0 against 1, the band shape, the seed
```

## How it works

`speaker_chain.rs` computes the foldback threshold `0.3 · (1 − error) · mean(|x|) / 0.005` from the
block as it arrives, then runs each sample through the delay line, the ring modulator, the foldback,
the RBJ high-pass, the RBJ low-pass and ×30. `long_range.rs` gives it its filters; `personal.rs`
first runs its microphone high-pass and low-pass over the whole block, so the threshold comes from
the microphone-filtered signal. `under_water.rs` draws one dropout decision per sample, then
band-passes and multiplies by 30. 0.9.12 has no airborne effect: airborne radios use the long-range
one.

## Boundaries

- Depends on: the crate's `signal_stages`, `error_level`, `radio_effect` and `error` modules;
  `tfar_dsp_filters` (`FilterDesign`, `MonoFilter`).
- Used by: the crate root's re-exports and `radio_effect_mixdown.rs`.
- Rules: the processing order and the filter table are the original's, pinned by the impulse and
  band-shape tests in `tests/`; silence in gives silence out at every error level.
