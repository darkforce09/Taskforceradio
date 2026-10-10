# Sound state for 0.9

The sound objects one sender's voice passes through: the compressor, the direct voice's direction
source and low-passes, and per radio id the radio effect and the phone, loudspeaker, under-water
and vehicle filters with the loudspeaker's direction source.

## Contents

```text
crates/version_0_9/tfar_client_state_0_9/src/dsp_state/
├── client_dsp.rs        `ClientDsp`: the compressor, the direct voice and the radios of one sender
├── direct_voice_dsp.rs  `DirectVoiceDsp`: the direct voice's direction source and low-passes
├── dsp_requirements.rs  `DspRequirements` and `RadioEffectKind`: what to make for the next frames
├── filter_designs.rs    the four Butterworth designs of the original and the vehicle cut-off
├── mod.rs               the module lines and the re-exports
├── radio_dsp.rs         `RadioDsp`: the state of one sender heard over one radio id
├── tests/               unit tests for the designs and for making and resetting the state
└── vehicle_filter.rs    `VehicleFilter`: a vehicle low-pass with the isolation it was designed for
```

## How it works

The original makes every object on first use, on the TeamSpeak audio thread, under keys such as
`voice_clunk`, `local_cantspeak`, `local_vehicle` and the radio id
(`reference/tfar_0_9_12/ts/src/plugin.cpp:232-341`). Here `ClientDsp::ensure` makes them from a
`DspRequirements` while the plugin handles a command, so the audio thread only borrows: the direct
voice's objects sit in `DirectVoiceDsp`, the radio-keyed ones in a `RadioDsp` per radio id, and the
lookups by `&str` allocate nothing. The [HRTF](/documentation/glossary.md#hrtf) sources replace the
clunks: one for the direct voice and one per loudspeaker radio, made only when the plugin passes the
HRIR sphere. An existing object is never replaced, except a vehicle low-pass whose isolation
changed. `reset_radio_effects` drops the effects and the phone and loudspeaker band-passes;
`reset_voices` clears the memory of the can't-speak, under-water and vehicle low-passes and frees
nothing: in serious mode it runs on every `POS` of a silent player, under the server lock the
audio thread waits for, so it must not free or rebuild a two-megabyte direction source. The
direction sources restart themselves after 200 ms without input (`tfar_hrtf`); the compressor stays.

## Boundaries

- Depends on: `tfar_dsp_filters` (`StereoFilter`, `FilterDesign`), `tfar_radio_effects_0_9` (the
  three effects), `tfar_dynamics` (`Compressor` with `VERSION_0_9`), `tfar_hrtf` (`HrtfSource`),
  `tfar_game_commands_0_9` (`RadioSubtype`).
- Used by: `crates/version_0_9/tfar_client_state_0_9/src/client_data.rs` (`ClientData::ensure_dsp`
  and the resets); `tfar_radio_routing_0_9`,
  which builds the requirements; `tfar_voice_pipeline_0_9`, which borrows the objects.
- Rules: objects are made only by `ensure` and dropped only by `reset_radio_effects` and expiry,
  never on the audio thread; `ensure` keeps the memory of what exists
  (`ensure_keeps_existing_objects_and_their_memory`); `reset_radio_effects` drops exactly the
  original's set and `reset_voices` keeps every object (`tests/client_dsp.rs`).
