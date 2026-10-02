# Dynamics

The `tfar_dynamics` crate: the feed-forward compressor the radio effects run once per frame. Used by
`tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_dynamics/
├── Cargo.toml  the `tfar_dynamics` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The feed-forward compressor that replaces SimpleComp: threshold, ratio, attack and release over a
sample buffer, run exactly once per frame. The code is not written yet: `src/lib.rs` carries the
module header and re-exports, `src/crate_role.rs` states the crate's role in one constant, and
`src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_dynamics  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `Compressor` with `CompressorSettings` (threshold, ratio, attack, release, makeup gain).
- `Compressor::process`: one pass over a `SampleFrames` buffer.

## Boundaries

- Depends on: `tfar_sample_buffer` (the shared sample frame type and saturating gain); `thiserror`
  for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: one `process` call is one pass over the frame; the caller never runs it twice. Tier 1 in
  `crates/audio`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
