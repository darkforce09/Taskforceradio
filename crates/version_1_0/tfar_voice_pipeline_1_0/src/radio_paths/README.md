# Radio paths

How each radio reception of a 1.0 sender is processed: the plan of every reception and its
audio-path half.

## Contents

```text
crates/version_1_0/tfar_voice_pipeline_1_0/src/radio_paths/
├── plan.rs    `RoutePlan` per reception: effect and error, speaker muffling and placement, intercom
└── render.rs  one reception's copy through its effect, band-pass, low-passes and placement
```

## How it works

`plan_routes` turns each `HeardVia` of `is_over_radio` into a `RoutePlan`, as the loop of
`reference/tfar_1_0/ts/src/plugin.cpp:600-737` decides it. A carried radio plays at
`((volume + 1) / 10)^4` (a tenth with `headsetLowered`) through the effect of the sender's subtype:
`digital` the personal effect (the underwater effect when heard as a diver radio), `airborne` and
`digital_lr` their effects, `phone` the phone band-pass at ×10, anything else nothing. The error is
the routing crate's distance error capped by the antenna loss (`None` read as 7). A speaker radio
gets the band-pass at ×4, its distance gain over `volume / 10 · speakerDistance` (or the vehicle
low-pass when a vehicle stands between it and the listener), the 100 Hz low-pass at ×14 under
water, the personal effect and its placement. The intercom plays the long-range effect at error 0
and the `intercomVolume`, ducked by `intercomDucking` while a long-range transmission above volume
2 is received. `render_route` runs a plan over the compressed copy of the frame and says whether to
mix it in.

## Boundaries

- Depends on: `tfar_client_state_1_0` (receptions, effects, the radio volume curve),
  `tfar_radio_routing_1_0` (error levels), `tfar_radio_effects_1_0` (the effect mixdown); the crate's
  `distance_gain`, `placement`, `scene` and `filter_step`.
- Used by: the crate's `frame_plan`, `voice_processor` and `dsp_needs`.
- Rules: a reception whose effect or band-pass was never ensured stays out of the mix; the intercom
  is never heard from the microphone path
  (`the_captured_voice_skips_the_intercom_and_direct_speech`).
