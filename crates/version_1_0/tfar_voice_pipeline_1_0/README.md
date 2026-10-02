# Voice pipeline for 1.0

The `tfar_voice_pipeline_1_0` crate: the 1.0 audio path: one TeamSpeak voice frame through routing,
effects, compressor, HRTF and mixing. Used by `tfar_plugin_1_0`.

## Contents

```text
crates/version_1_0/tfar_voice_pipeline_1_0/
├── Cargo.toml  the `tfar_voice_pipeline_1_0` package (lib): tier 5, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 audio path for one voice frame: routes the speaker, applies the radio effect and the
compressor once, the distance and cone gains and the HRTF, and mixes radio sounds. The code is not
written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the
crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_voice_pipeline_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `process_voice_frame`: one sender's frame, edited in place.
- `process_playback_frame`: radio sounds mixed into the outgoing audio.

## Boundaries

- Depends on: `tfar_client_state_1_0` (the 1.0 per-client state); `tfar_radio_routing_1_0` (the 1.0
  reception decisions); `tfar_radio_effects_1_0` (the 1.0 radio effect chains);
  `tfar_plugin_settings_1_0` (the 1.0 settings); `tfar_dsp_filters` (the Butterworth and RBJ filters);
  `tfar_dynamics` (the compressor); `tfar_hrtf` (binaural direction rendering); `tfar_spatial_audio`
  (the distance curve and cone gains); `tfar_sample_buffer` (the shared sample frame type and
  saturating gain); `tfar_playback_mixer` (radio sound playback); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: the compressor runs once per frame. Tier 5 in `crates/version_1_0`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
