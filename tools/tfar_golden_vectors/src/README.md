# Golden vectors source

The source of `tfar_golden_vectors`: the safe oracle wrappers over the C++ shim, the
deterministic test signals and the error measures the comparison tests use, with the role
constant, the error type and the prelude.

## Contents

```text
tools/tfar_golden_vectors/src/
├── comparison_metrics.rs  RMS, relative RMS error, largest difference and decibel distance
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error`, the `thiserror` enum, and `Result`
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── oracles/               the filter, compressor and spatial oracles and the one FFI module
├── oracles.rs             the `oracles` module tree and its re-exports
├── prelude.rs             the public API except `Error` and `Result`, for glob import
├── test_signals.rs        seeded noise, sweep, impulse, modulated burst, level steps, frequencies
└── tests/                 unit tests of the signals, the measures and `CRATE_ROLE`
```

## How it works

`lib.rs` declares the private modules and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`. The `oracles` module tree
holds everything that reaches the C++ code; `test_signals.rs` and `comparison_metrics.rs` are
pure Rust, so a comparison test builds its input with the first, runs it through an oracle and
through the crate under test, and judges the pair with the second. The noise comes from a seeded
xorshift64* generator, so every run and every thread sees the same samples. Each module attaches
its sibling test file in `tests/` with `#[path]`.

## Public surface

- The oracles (`OracleFilter`, `OracleCompressor`, `f3daudio_calculate` and their types), the
  test signals, the error measures, `CRATE_ROLE`, `Error`, `Result` and `prelude`, re-exported at
  the crate root.

## Boundaries

- Depends on: `thiserror`; the shim library the build script links (through `oracles/`).
- Used by: the comparison tests in `tools/tfar_golden_vectors/tests/`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  `unsafe` appears only in `oracles/oracle_bindings.rs` (`cargo xtask verify unsafe-allowlist`);
  tests live in `tests/` (`cargo xtask verify test-placement`).
