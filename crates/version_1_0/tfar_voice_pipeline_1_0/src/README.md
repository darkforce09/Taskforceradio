# Voice pipeline for 1.0 source

The source of `tfar_voice_pipeline_1_0`: the voice processor, the gates, the planning of direct
speech and radio receptions, their audio-path halves, the distance gains and the DSP needs.

## Contents

```text
crates/version_1_0/tfar_voice_pipeline_1_0/src/
├── crate_role.rs       `CRATE_ROLE`: what the crate is for, in one sentence
├── direct_speech/      the direct-speech plan and its muffling, placement and compressor
├── direct_speech.rs    the module lines and re-exports of `direct_speech/`
├── distance_gain.rs    `volume_attenuation`, the vehicle loss and gain, the speaker reach, the fixed gains
├── dsp_needs.rs        `ensure_sender_dsp`, `ensure_every_sender_dsp`: the DSP objects frames will need
├── error.rs            `Error` (a refused DSP design or stream append, a ragged sound) and `Result`
├── filter_step.rs      a filter that may not exist yet: filter at a gain, or the gain alone
├── frame_plan.rs       `FramePlan`: everything decided about a frame before a sample is touched
├── lib.rs              the crate root: module header, `mod` lines and re-exports
├── placement.rs        `Placement`: direction and level, rendered by HRTF or the simple ILD; the makeup
├── prelude.rs          the public names for glob import
├── radio_paths/        the plan of every radio reception and its audio-path half
├── radio_paths.rs      the module lines and re-exports of `radio_paths/`
├── remote_tangent_sound.rs  `render_remote_tangent_sound`: the positional remote tangent sound
├── scene.rs            `Scene`: the sender and the local player read for planning
├── sender_gate.rs      the TFAR, spectator and serious-mode gates; the spectator view
├── tests/              unit tests of the distance gains, the placement and the remote sound
├── voice_context.rs    `VoiceFrameContext` and `VoiceOutcome`
├── voice_processor.rs  `VoiceProcessor`: the whole chain per frame, and the captured voice
└── voice_scratch.rs    `VoiceScratch`: the reusable buffers and lists; `VoiceScratchCapacities`
```

## How it works

`VoiceProcessor::process_sender` runs in two phases. With the session borrowed immutably,
`sender_gate` decides pass-through, silence or processing, and `FramePlan::build` reads a `Scene`,
plans direct speech (`direct_speech/`), lists the receptions with `is_over_radio_into` and plans
each (`radio_paths/`). Then, with the sender's `ClientEffects` borrowed mutably, the frame gets the voice
multiplier, is copied as the radio original, goes through direct speech (or is silenced when the
sender is not heard directly, unless both spectate), and every reception's copy of the
radio-compressed original goes through its path and is mixed in; the global volume comes last. All
blocks live in `VoiceScratch`. `dsp_needs` runs the same planning off the audio thread and hands
`ClientEffects::ensure_dsp` the objects the next frames will look up. `remote_tangent_sound.rs`
shapes a remote tangent WAV into a playback on the command thread.

## Public surface

- Every name the crate root re-exports, listed in the crate README.

## Boundaries

- Depends on: `tfar_client_state_1_0`, `tfar_radio_routing_1_0`, `tfar_radio_effects_1_0`,
  `tfar_plugin_settings_1_0`, `tfar_antennas_1_0`, `tfar_geometry`, `tfar_dsp_filters`,
  `tfar_dynamics`, `tfar_hrtf`, `tfar_spatial_audio`, `tfar_sample_buffer`, `tfar_playback_mixer`,
  `thiserror`.
- Used by: `tfar_plugin_1_0`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); the audio path allocates nothing once the
  scratch fits the frame (`a_thousand_frames_leave_every_scratch_capacity_unchanged`).
