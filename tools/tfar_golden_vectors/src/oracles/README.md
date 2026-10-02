# Oracles

The safe side of the golden-vector oracle: one wrapper each for the original DSPFilters filters,
the SimpleComp compressor and FAudio's F3DAudio, over the one module that calls the C++ shim.

## Contents

```text
tools/tfar_golden_vectors/src/oracles/
├── compressor_oracle.rs  `OracleCompressor`: SimpleComp over stereo f64 frames with gains, and i16
├── filter_oracle.rs      `OracleFilter`: DSPFilters' Butterworth and RBJ designs, f32, i16, magnitude
├── oracle_bindings.rs    the `#[repr(C)]` structs, the `extern "C"` block and the safe calls
├── parameter_checks.rs   the positive-value and below-Nyquist checks the oracles run first
├── spatial_oracle.rs     `f3daudio_calculate`: matrix, volume, angle and failed checks for a scene
└── tests/                the layout, refusal and plain-property tests of each oracle
```

## How it works

Each oracle checks its parameters with `parameter_checks.rs`, fills the `#[repr(C)]` struct
`oracle_bindings.rs` shares with `shim/oracle_shim.cpp`, and calls the matching safe function
there. `oracle_bindings.rs` is the only module that lifts `unsafe_code`: it checks every buffer
length, hands the shim pointers into live slices with the counts taken from them, and turns a
non-zero status into `Error::OracleRefused`. Every call builds the original object afresh (a
`SimpleFilter`, a `SimpleComp`, an F3DAudio handle) and drops it before returning, so calls are
independent and any test thread may make them; the shim's count of failed F3DAudio parameter
checks is thread-local. The layout test compares each shared struct's `size_of` with the
`sizeof` the shim reports.

```text
filter_oracle ─────┐
compressor_oracle ─┼──▶ oracle_bindings ──▶ shim/oracle_shim.cpp ──▶ DSPFilters, SimpleComp, F3DAudio
spatial_oracle ────┘
```

## Boundaries

- Depends on: the crate's `error` module; the static libraries `build.rs` makes from `shim/`,
  `oracle/` and `reference/tfar_1_0/ts/include/`.
- Used by: `lib.rs` and `prelude.rs` (the re-exports), and through them the comparison tests.
- Rules: no pointer crosses the boundary without the length of the slice it points into
  (`tests/oracle_bindings.rs`); the shared structs keep the shim's layout (the size test in
  `tests/oracle_bindings.rs`); a parameter outside the original's range is an `Error` before the
  shim is called (`tests/filter_oracle.rs`, `tests/compressor_oracle.rs`,
  `tests/spatial_oracle.rs`).
