# DSP filters source

The source of `tfar_dsp_filters`: the filter designs, the cascade of second-order sections they
produce, and the filter that runs a cascade per channel, with the role constant, the error type and
the prelude.

## Contents

```text
crates/audio/tfar_dsp_filters/src/
├── channel_filter.rs         `ChannelFilter`, `MonoFilter`, `StereoFilter`: per-channel processing
├── crate_role.rs             `CRATE_ROLE`: what the crate is for, in one sentence
├── designs/                  the Butterworth and RBJ designs and their parameter checks
├── designs.rs                the `designs` module tree
├── error.rs                  `Error`, the `thiserror` enum, and `Result`
├── filter_design.rs          `FilterDesign`, `MAX_BUTTERWORTH_ORDER`: the designs TFAR uses
├── lib.rs                    the crate root: module header, `mod` lines and re-exports
├── prelude.rs                the public API except `Error` and `Result`, for glob import
├── second_order_sections.rs  `SecondOrderSection`, `SecondOrderSections`, the magnitude response
└── tests/                    one unit test file per module
```

## How it works

`FilterDesign::second_order_sections` dispatches each variant to `designs/`, which checks the
parameters and calls `sci-rs` or `biquad`; the result is a `SecondOrderSections` of normalised,
finite sections. `ChannelFilter::from_sections` turns each section into a `biquad`
`DirectForm2Transposed<f64>` and keeps one chain of them per channel; every processing method folds
a sample through its channel's chain. `lib.rs` re-exports the public items at the crate root and
`prelude.rs` the same items except `Error` and `Result`. Each module attaches its sibling test file
in `tests/` (or `designs/tests/`) with `#[path]`.

## Public surface

- `FilterDesign`, `MAX_BUTTERWORTH_ORDER`, `SecondOrderSections`, `SecondOrderSection`,
  `ChannelFilter`, `MonoFilter`, `StereoFilter`, `CRATE_ROLE`, `Error`, `Result` and `prelude`,
  re-exported at the crate root.

## Boundaries

- Depends on: `tfar_sample_buffer`, `sci-rs`, `biquad`, `thiserror`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_radio_effects_0_9`,
  `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  processing allocates nothing; tests live in `tests/` (`cargo xtask verify test-placement`).
