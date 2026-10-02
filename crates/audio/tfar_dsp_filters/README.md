# DSP filters

The `tfar_dsp_filters` crate: Butterworth and RBJ filters in f64 Direct Form II Transposed, with
per-channel state. Used by `tfar_client_state_0_9`, `tfar_client_state_1_0`,
`tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_dsp_filters/
├── Cargo.toml  the `tfar_dsp_filters` package (lib): tier 1, workspace lints and dependencies
└── src/        the filter designs, the coefficient cascade and the per-channel filter
```

## How it works

A caller describes a filter as a `FilterDesign` and builds a `ChannelFilter<CHANNELS>` from it
once, off the audio thread: the Butterworth designs come from `sci-rs` (`butter_dyn`, the port of
`scipy.signal.butter`) as second-order sections, the RBJ designs from `biquad`'s cookbook
coefficients, and both become a `SecondOrderSections` cascade of normalised sections. The filter
then runs every channel through its own copy of the cascade, each section a `biquad`
`DirectForm2Transposed<f64>`, on the TeamSpeak audio thread without allocating:
`process_frame`, `process_frames_f64` and `process_frames_f32` work on interleaved frames,
`MonoFilter::process_sample` on the radio effects' mono signal, and
`StereoFilter::process_stereo_i16` is the original `processFilterStereo`
(`reference/tfar_1_0/ts/src/helpers.hpp`): `sample / 32767`, filter, `× gain`; above 1.0 the sample
is `i16::MAX`, below -1.0 `i16::MIN`, otherwise `× 32766` and truncated, as the original clips.

### Designs and their originals

| `FilterDesign` | DSPFilters original | TFAR use |
|---|---|---|
| `ButterworthLowPass { order: 2, .. }` | `Butterworth::LowPass<2>` | vehicle muffling, object interception |
| `ButterworthLowPass { order: 4, .. }` | `Butterworth::LowPass<4>` | can't speak and underwater, 100 Hz |
| `ButterworthBandPass { order: 1, .. }` | `Butterworth::BandPass<1>` | speaker, 2000 / 1000 |
| `ButterworthBandPass { order: 2, .. }` | `Butterworth::BandPass<2>` | phone 1850 / 1550, underwater 1000 / 400 |
| `RbjHighPass`, `RbjLowPass` | `RBJ::HighPass`, `RBJ::LowPass` | the radio effects |

### Band-pass centre and width

DSPFilters' `BandPassBase::setup(order, sampleRate, centre, width)` places the digital band edges
at `centre ∓ width / 2` (`BandPassTransform` in
`reference/tfar_1_0/ts/include/dspfilters/source/PoleFilter.cpp`) and normalises the peak to 1 at
the warped geometric centre `2·atan(sqrt(tan(ω1/2)·tan(ω2/2)))`. This crate passes the same two
edges to `butter_dyn` as the band-pass critical frequencies, which puts the −3 dB points exactly
on them and the unit peak at the same warped centre. For TFAR's band-passes the edges are
1500–2500 Hz (speaker), 1075–2625 Hz (phone) and 800–1200 Hz (underwater); the tests check the
gain at both edges is −3.0103 dB: to 1e-9 in the designed magnitude, and to 0.01 dB when a sine is
measured through the filter.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_dsp_filters  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `FilterDesign` (`ButterworthLowPass`, `ButterworthBandPass`, `RbjLowPass`, `RbjHighPass`) with
  `second_order_sections` and `band_edges_hz`; `MAX_BUTTERWORTH_ORDER`.
- `SecondOrderSections` (`new`, `sections`, `sample_rate_hz`, `magnitude_at`) and
  `SecondOrderSection` (`b0`, `b1`, `b2`, `a1`, `a2`, `from_unnormalised`, `coefficients`).
- `ChannelFilter<CHANNELS>` (`new`, `from_sections`, `sections`, `reset`, `process_frame`,
  `process_frames_f64`, `process_frames_f32`); `MonoFilter` (`process_sample`) and
  `StereoFilter` (`process_stereo_i16`).
- `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_sample_buffer` (the stereo buffer and the `i16`/`f32` scales); `sci-rs`
  (Butterworth design), `biquad` (RBJ coefficients and the Direct Form II Transposed section) and
  `thiserror`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_radio_effects_0_9`,
  `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: coefficients are designed once at construction, off the audio thread; processing
  allocates nothing and never panics; every parameter is checked before a design library is
  called, so their panics are unreachable. Tier 1 in `crates/audio`: every workspace dependency
  has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where each filter sits in the
  0.9 and 1.0 voice chains.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
