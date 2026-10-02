# Playback mixer

The `tfar_playback_mixer` crate: loading and mixing of the radio sound WAV files into the outgoing
audio. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`,
`tfar_voice_pipeline_1_0`.

## Contents

```text
crates/audio/tfar_playback_mixer/
├── Cargo.toml  the `tfar_playback_mixer` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The radio sound player: loads the WAV files shipped in the mod's `radio-sounds` folder through
`hound`, checks they are 48 kHz 16-bit stereo, caches them and mixes active playbacks into a frame.
The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_playback_mixer  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `SoundCache`: loaded and validated WAV files by name.
- `PlaybackMixer`: active playbacks mixed into a `SampleFrames` buffer, with per-playback gain.

## Boundaries

- Depends on: `tfar_sample_buffer` (the shared sample frame type and saturating gain); `thiserror`
  for the error type. Planned externals: `hound`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`,
  `tfar_voice_pipeline_1_0`.
- Rules: a playback's filters and buffers are freed when it ends. Tier 1 in `crates/audio`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
