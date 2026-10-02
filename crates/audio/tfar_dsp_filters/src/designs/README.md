# Filter designs

The design side of `tfar_dsp_filters`: the parameter checks, the Butterworth designs through
`sci-rs` and the RBJ designs through `biquad`, each returning a cascade of second-order sections.

## Contents

```text
crates/audio/tfar_dsp_filters/src/designs/
├── butterworth_design.rs  Butterworth low-pass and band-pass through `sci-rs`, the band-edge mapping
├── parameter_checks.rs    the sample-rate, frequency, quality and order checks every design runs
├── rbj_design.rs          RBJ low-pass and high-pass through `biquad`
└── tests/                 the gains at DC, Nyquist and the corners, the analytic magnitudes, the checks
```

## How it works

Each design checks its parameters with `parameter_checks.rs` first, using the normalised frequency
`2 · f / fs` that `sci-rs` computes, so the `panic!`s inside `butter_dyn` (a frequency outside
(0, Nyquist), band edges out of order) cannot be reached. `butterworth_design.rs` derives the
band-pass edges as `centre ∓ width / 2`, DSPFilters' convention, and asks `butter_dyn` for
second-order sections; `rbj_design.rs` asks `biquad::Coefficients::from_params` for the cookbook
section DSPFilters' `RBJ` filters compute. Both normalise through
`SecondOrderSection::from_unnormalised`, which refuses a zero leading coefficient or a coefficient
that is not finite.

## Boundaries

- Depends on: `sci-rs` (`butter_dyn`), `biquad` (`Coefficients`, `Hertz`, `Type`), the crate's
  `second_order_sections`, `filter_design` (`MAX_BUTTERWORTH_ORDER`) and `error` modules.
- Used by: `filter_design.rs` (`FilterDesign::second_order_sections`, `band_edges_hz`) and
  `second_order_sections.rs` (the sample-rate check).
- Rules: no design library is called before its parameters pass the checks; a design returns
  finite, normalised sections or an `Error`, never a panic (the tests in `tests/`).
