# Golden vectors

The `tfar_golden_vectors` crate: reference outputs of the original C++ audio code, used as test
fixtures for the audio crates. The audio crates' tests read its fixtures.

## Contents

```text
tools/tfar_golden_vectors/
├── Cargo.toml  the `tfar_golden_vectors` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The test oracle for the audio crates: builds the original DSPFilters, SimpleComp and clunk code from
`reference/` and FAudio's F3DAudio with `cc`, and turns their output into fixtures the Rust crates
must match within tolerance. The code is not written yet: `src/lib.rs` carries the module header and
re-exports, `src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds
the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_golden_vectors  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- golden fixtures per audio crate: input frames, settings and the reference output.
- tolerance helpers the audio crates' tests compare against.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: the audio crates' tests.
- Rules: the oracle code is built only here, never into a deliverable. Tier 0 in `tools`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
