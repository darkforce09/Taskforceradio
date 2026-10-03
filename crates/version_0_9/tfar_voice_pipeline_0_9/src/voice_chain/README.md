# Voice chain for 0.9

The steps of the 0.9.12 stereo function for one sender's frame: the plan read from the server
state, and the render of the direct voice, the compressor and every radio reception over the
sender's sound state.

## Contents

```text
crates/version_0_9/tfar_voice_pipeline_0_9/src/voice_chain/
├── direct_voice.rs     the direct voice: plain gain, vehicle low-pass or can't-speak low-pass, direction
├── frame_render.rs     `render_frame`: multiplier, direct voice, compressor, receptions, global volume
├── mod.rs              the module lines and the crate-internal re-exports
├── radio_reception.rs  one reception: the effect of the subtype, then the loudspeaker stages
├── sender_plan.rs      `plan_sender`: every branch, gain, error and direction of the frame
├── sound_placement.rs  the direction: the HRTF source (makeup for speech only), or the ILD fallback
└── tests/              scenario tests per step
```

## How it works

`sender_plan.rs` reads the server state once: the direct voice's branch and level, and per
reception from `tfar_radio_routing_0_9::is_over_radio_into` (into the scratch reception list) its
effect gain, its effect error and its loudspeaker stages, each with a `SoundPlacement` (the HRTF
direction from the listener's view angle and the 0.9 ILD gains). `frame_render.rs` then runs the
plan over the sender's `ClientDsp`: `direct_voice.rs` for the voice in range, one compressor pass
over the copy, and `radio_reception.rs` for each reception on its own copy, mixed in with
saturation. `sound_placement.rs` is the one panner both use: the direct voice renders through the
HRTF at the makeup gain, a loudspeaker at unity. A step whose sound object is not made yet reports
it and contributes silence.

## Boundaries

- Depends on: `tfar_client_state_0_9` (`ClientDsp`, `DirectVoiceDsp`, `RadioDsp`, the state),
  `tfar_radio_routing_0_9` (the receptions and distances), `tfar_radio_effects_0_9`, `tfar_hrtf`,
  `tfar_spatial_audio`, `tfar_sample_buffer`; the crate's distance gain, gains and scratch.
- Used by: `crates/version_0_9/tfar_voice_pipeline_0_9/src/voice_processor.rs`.
- Rules: the compressor runs once per frame; every reception starts from the same compressed copy;
  the can't-speak and vehicle branches need their filter, or the path is silent
  (`a_frame_missing_a_needed_filter_silences_that_path_and_is_counted`); one panner per sound; no
  makeup gain on a loudspeaker (`a_loudspeaker_through_the_hrtf_gets_no_makeup_gain`).
