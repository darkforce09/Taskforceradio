# Dynamics source

The source of `tfar_dynamics`: the compressor, its settings and presets, and the attack and
release envelope it smooths with, with the role constant, the error type and the prelude.

## Contents

```text
crates/audio/tfar_dynamics/src/
├── attack_release_envelope.rs  the one-pole attack and release follower and its coefficient
├── compressor.rs               `Compressor`, `DC_OFFSET` and the decibel conversions
├── compressor_settings.rs      `CompressorSettings`, the 0.9 and 1.0 presets and their checks
├── crate_role.rs               `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs                    `Error`, the `thiserror` enum, and `Result`
├── lib.rs                      the crate root: module header, `mod` lines and re-exports
├── prelude.rs                  the public API except `Error` and `Result`, for glob import
└── tests/                      one unit test file per module
```

## How it works

`Compressor::new` checks the `CompressorSettings` and builds an `AttackReleaseEnvelope` from the
attack and release time constants; `process_frame` turns the louder channel into a decibel excess
over the threshold, runs it through the envelope against the compressor's own envelope state, and
scales both channels by the resulting gain. `lib.rs` re-exports the public items at the crate root
and `prelude.rs` the same items except `Error` and `Result`. Each module attaches its sibling test
file in `tests/` with `#[path]`.

## Public surface

- `Compressor`, `CompressorSettings`, `DC_OFFSET`, `CRATE_ROLE`, `Error`, `Result` and `prelude`,
  re-exported at the crate root.

## Boundaries

- Depends on: `tfar_sample_buffer`, `thiserror`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
