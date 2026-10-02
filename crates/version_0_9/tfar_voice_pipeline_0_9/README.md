# Voice pipeline for 0.9

The `tfar_voice_pipeline_0_9` crate: the 0.9.12 audio path: one TeamSpeak voice frame through
routing, effects, HRTF and mixing. Used by `tfar_plugin_0_9`.

## Contents

```text
crates/version_0_9/tfar_voice_pipeline_0_9/
├── Cargo.toml  the `tfar_voice_pipeline_0_9` package (lib): tier 5, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 0.9.12 audio path for one voice frame: routes the speaker, applies the radio effect, the
distance and stereo gains and the HRTF, and mixes radio sounds. The code is not written yet:
`src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's role
in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_voice_pipeline_0_9  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `process_voice_frame`: one sender's frame, edited in place.
- `process_playback_frame`: radio sounds mixed into the outgoing audio.

## Boundaries

- Depends on: `tfar_client_state_0_9` (the 0.9 per-client state); `tfar_radio_routing_0_9` (the 0.9
  reception decisions); `tfar_radio_effects_0_9` (the 0.9 radio effect chains); `tfar_dsp_filters`
  (the Butterworth and RBJ filters); `tfar_dynamics` (the compressor); `tfar_hrtf` (binaural direction
  rendering); `tfar_spatial_audio` (the distance curve and cone gains); `tfar_sample_buffer` (the
  shared sample frame type and saturating gain); `tfar_playback_mixer` (radio sound playback);
  `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`.
- Rules: the post-process never reads past the frame it was handed. Tier 5 in `crates/version_0_9`:
  every workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
