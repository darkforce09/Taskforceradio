# HRTF

The `tfar_hrtf` crate: binaural direction rendering over the IRCAM HRIR sphere through the hrtf
crate. Used by `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_hrtf/
├── Cargo.toml  the `tfar_hrtf` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

Real binaural direction rendering for both plugin versions: loads the IRCAM HRIR sphere through the
`hrtf` crate and adapts TeamSpeak's frame sizes to its block size, keeping state per sound source.
The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_hrtf  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `HrirSphere`: the loaded and resampled HRIR data.
- `HrtfSource`: per-source processor state.
- `HrtfBlockAdapter`: renders a mono TeamSpeak frame into stereo for a direction.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `tfar_sample_buffer` (the
  shared sample frame type and saturating gain); `thiserror` for the error type. Planned externals:
  `hrtf`.
- Used by: `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: a source's state belongs to one speaker and is never shared between two. Tier 1 in
  `crates/audio`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
