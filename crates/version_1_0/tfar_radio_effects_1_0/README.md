# Radio effects for 1.0

The `tfar_radio_effects_1_0` crate: the 1.0 radio sound chains: filters, compressor input and noise
per radio type. Used by `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_radio_effects_1_0/
├── Cargo.toml  the `tfar_radio_effects_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 radio effect chains: the filter settings per radio type and the error level that degrades a
signal with distance, with the original slope fixed. The code is not written yet: `src/lib.rs`
carries the module header and re-exports, `src/crate_role.rs` states the crate's role in one
constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_radio_effects_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `RadioEffect`: one effect chain per radio type.
- `error_level`: signal degradation by distance and range.

## Boundaries

- Depends on: `tfar_dsp_filters` (the Butterworth and RBJ filters); `tfar_sample_buffer` (the shared
  sample frame type and saturating gain); `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: the error level rises with distance, never falls. Tier 2 in `crates/version_1_0`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
