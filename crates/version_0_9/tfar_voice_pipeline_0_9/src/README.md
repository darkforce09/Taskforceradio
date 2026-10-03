# Voice pipeline for 0.9 source

The source of `tfar_voice_pipeline_0_9`: the processor with the stereo function's gates, the voice
chain it runs, the distance gain and the fixed gains, the scratch it works in, the own-voice
path for loudspeakers and the positional remote push-to-talk sound.

## Contents

```text
crates/version_0_9/tfar_voice_pipeline_0_9/src/
├── crate_role.rs       `CRATE_ROLE`: what the crate is for, in one sentence
├── distance_gain.rs    `volume_from_distance` and `radio_volume_multiplier`
├── error.rs            `Error`: the own-voice stream refusing a block, a ragged remote sound
├── lib.rs              the crate root: module header, `mod` lines and re-exports
├── own_voice.rs        `process_own_voice_for_speakers`, `append_own_voice`, the stream name
├── prelude.rs          the processor, its context and outcome types and the distance gain
├── remote_tangent_sound.rs  `render_remote_tangent_sound`: the positional remote push-to-talk sound
├── tests/              unit and scenario tests per module, and the shared test scene
├── voice_chain/        the plan of a sender's frame and its render over the sender's sound state
├── voice_context.rs    `VoiceFrameContext`, `OwnVoiceContext` and `VoiceOutcome`
├── voice_gains.rs      the fixed gains of the stereo function and the HRTF makeup gain
├── voice_processor.rs  `VoiceProcessor`: the gates and the plugin-enabled check
└── voice_scratch.rs    `VoiceScratch`: the buffers reused from frame to frame
```

## How it works

```text
plugin ── VoiceFrameContext, &mut ServerRadioData, &mut StereoBuffer ──▶ VoiceProcessor
   voice_processor: gates ──▶ PassThrough | Silenced | run the chain
   voice_chain: plan_sender (read-only) ──▶ SenderPlan + routes in VoiceScratch
                render_frame (sender's ClientDsp) ──▶ Processed
own_voice: captured frame ──▶ same chain as the local player ──▶ block ──▶ PlaybackMixer stream
remote_tangent_sound: WAV + RemoteSoundPlan (command thread) ──▶ Playback ──▶ PlaybackMixer
```

`voice_processor.rs` holds the gates of the stereo function and the plugin-enabled check, and runs
the chain in two steps so that the read-only plan and the mutable render never borrow the state
together; the sender's nickname and the receptions travel between them in `VoiceScratch`.
`distance_gain.rs` and `voice_gains.rs` hold the numbers the plan uses. `own_voice.rs` adds the
captured-voice path to the processor. `remote_tangent_sound.rs` shapes a remote push-to-talk sound
off the audio thread with the same gains and filter designs. The tests share one synthetic scene
(`tests/test_scene.rs`, compiled only for tests).

## Public surface

- `VoiceProcessor`, `VoiceFrameContext`, `OwnVoiceContext`, `VoiceOutcome`, `VoiceScratch`,
  `ScratchCapacities` and the scratch constants.
- `append_own_voice`, `OWN_VOICE_STREAM_NAME`.
- `render_remote_tangent_sound`, `RemoteSoundPlan`, `ListenerView`, `UNDER_WATER_SOUND_GAIN_FACTOR`.
- `volume_from_distance`, `radio_volume_multiplier` and the gain constants.
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_client_state_0_9`, `tfar_radio_routing_0_9`, `tfar_radio_effects_0_9`,
  `tfar_dsp_filters` and `tfar_dynamics` (the filters and the compressor the sound state holds),
  `tfar_hrtf`, `tfar_spatial_audio`, `tfar_sample_buffer`, `tfar_playback_mixer`, `thiserror`,
  `tracing`.
- Used by: `tfar_plugin_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); no
  production file calls TeamSpeak or reads the clock, and none on the audio path makes a sound object; tests live in `tests/` folders
  (`cargo xtask verify test-placement`).
