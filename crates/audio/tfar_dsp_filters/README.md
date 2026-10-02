# DSP filters

The `tfar_dsp_filters` crate: butterworth and RBJ filters in f64 Direct Form II Transposed, with
per-channel state. Used by `tfar_client_state_0_9`, `tfar_client_state_1_0`,
`tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_dsp_filters/
├── Cargo.toml  the `tfar_dsp_filters` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The filter set the radio effects use: Butterworth low-pass and band-pass designed by `sci-rs` as
second-order sections, RBJ high-pass and low-pass from `biquad`, all processed in `f64` Direct Form
II Transposed with state per channel. The code is not written yet: `src/lib.rs` carries the module
header and re-exports, `src/crate_role.rs` states the crate's role in one constant, and
`src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_dsp_filters  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `ButterworthLowPass` and `ButterworthBandPass`: designed once, applied per channel.
- `RbjHighPass` and `RbjLowPass` with a Q factor.
- `MultiChannelFilter`: one filter state per channel over a `SampleFrames` buffer.

## Boundaries

- Depends on: `tfar_sample_buffer` (the shared sample frame type and saturating gain); `thiserror`
  for the error type. Planned externals: `sci-rs`, `biquad`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_radio_effects_0_9`,
  `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: coefficients are designed once at construction; processing allocates nothing. Tier 1 in
  `crates/audio`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
