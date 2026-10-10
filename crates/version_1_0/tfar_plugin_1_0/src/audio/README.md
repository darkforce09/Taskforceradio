# Audio

The plugin's sound: the four TeamSpeak audio callbacks and the radio sounds they mix in.

## Contents

```text
crates/version_1_0/tfar_plugin_1_0/src/audio/
├── mod.rs              `mod` lines and the re-export of `RadioSounds`
├── radio_sounds.rs     the WAV files, the HRIR sphere, the mixer and its own-voice stream, the tangent sounds
├── tests/              unit tests of the sound list and the own-voice stream
└── voice_callbacks.rs  post-process, captured, mixed playback and the neutral 3D rolloff
```

## How it works

The post-process callback takes the server state with `try_lock`, copies the block to stereo,
runs `VoiceProcessor::process_sender` and writes the result back; a busy lock, no game, a dormant
plugin or an unknown sender leaves the block untouched. The captured callback runs
`VoiceProcessor::process_own_voice_for_speakers` and pushes the result through the mixer's
`StreamProducer` for the stream `my_radio_voice`, which never blocks or allocates. The
mixed-playback callback adds the mixer's playbacks and streams into channels 0 and 1 of the
server in front. `RadioSounds` preloads `radio-sounds/` at init; a local tangent sound is queued
at its gain into its ear, a remote one is shaped by `render_remote_tangent_sound` of
`tfar_voice_pipeline_1_0` (gain, ear, vehicle and loudspeaker filters, under-water muffling,
simple ILD) on the calling thread before the mixer gets it.

## Boundaries

- Depends on: `tfar_voice_pipeline_1_0`, `tfar_playback_mixer`, `tfar_hrtf`, `tfar_sample_buffer`,
  `tfar_arma_values`, `parking_lot`; the crate's `runtime`.
- Used by: `plugin_lifecycle` (the callbacks), `runtime`, `game_link` and `events` (the sounds).
- Rules: the audio callbacks never block, allocate once the scratch fits, or call TeamSpeak.
