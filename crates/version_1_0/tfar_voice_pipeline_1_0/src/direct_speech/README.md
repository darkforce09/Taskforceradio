# Direct speech

How a 1.0 sender is heard without radio: the plan of one frame and its audio-path half.

## Contents

```text
crates/version_1_0/tfar_voice_pipeline_1_0/src/direct_speech/
├── plan.rs    `DirectSpeechPlan`: the range gate, the muffling and the placement at curve and cone level
└── render.rs  the frame through the object, vehicle or can't-speak low-pass, HRTF or ILD and compressor
```

## How it works

`plan_direct_speech` reads a `Scene` and decides, as `reference/tfar_1_0/ts/src/plugin.cpp:547-597`
does, whether the sender is heard directly (not the local player, not a spectator, not kept away by
the spectator rules, within the voice range plus 15 m with 2 m per object) and how: clear, behind
objects (the object low-pass with a one-frame look-back), from another vehicle (the vehicle low-pass
at `volumeAttenuation(d, …, 1 − loss) · (1 − loss)^1.2`) or muffled because one of the two cannot
speak (the 100 Hz low-pass at `volumeAttenuation(d, false, …) · 14`). The placement's level is the
X3DAudio replacement: the 25-point voice curve of `tfar_spatial_audio` at the straight distance,
scaled by the sender's voice range, times the voice cone when `voiceCone` is on, with the original's
swapped roles and negated facing so a sender facing the listener gets the inner volume.
`render_direct_speech` runs that plan over the frame with the sender's DSP objects and ends with the
direct-speech compressor.

## Boundaries

- Depends on: `tfar_client_state_1_0` (the effects and their keys), `tfar_spatial_audio` (curve and
  cone), `tfar_geometry`; the crate's `distance_gain`, `placement`, `scene`, `sender_gate` and
  `filter_step`.
- Used by: the crate's `frame_plan`, `voice_processor` and `dsp_needs`.
- Rules: a step whose DSP object was never ensured is skipped, never allocated
  (`without_the_dsp_objects_the_voice_still_passes_at_its_level`); the first frame behind objects
  has no look-back (`the_first_frame_behind_objects_has_no_look_back_and_is_filtered_once`).
