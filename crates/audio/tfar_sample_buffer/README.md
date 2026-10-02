# Sample buffer

The `tfar_sample_buffer` crate: interleaved 16-bit sample frames as TeamSpeak hands them out, with
saturating gain. Used by `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
`tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_sample_buffer/
├── Cargo.toml  the `tfar_sample_buffer` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The sample frame type the audio crates pass around: interleaved `i16` channels as TeamSpeak delivers
them, conversion to and from `f32`/`f64`, and gain that saturates instead of wrapping. The code is
not written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states
the crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_sample_buffer  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `SampleFrames`: a borrowed view of interleaved samples with a channel count.
- saturating gain and mix helpers on `i16` and `f32`.
- sample-format conversions used by the filters, the HRTF and the mixer.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
  `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: gain saturates at the `i16` limits and never wraps. Tier 0 in `crates/audio`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
