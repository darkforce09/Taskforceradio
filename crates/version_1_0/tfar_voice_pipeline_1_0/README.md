# Voice pipeline for 1.0

The `tfar_voice_pipeline_1_0` crate: the 1.0 audio path: one TeamSpeak voice frame through routing,
effects, compressor, HRTF and mixing. Used by `tfar_plugin_1_0`.

## Contents

```text
crates/version_1_0/tfar_voice_pipeline_1_0/
├── Cargo.toml  the `tfar_voice_pipeline_1_0` package (lib): tier 5, workspace lints and dependencies
├── src/        the voice processor, the gates, direct speech, the radio paths and the DSP needs
└── tests/      voice scenarios over synthetic sessions: gates, direct speech, radios, invariants
```

## How it works

The crate is `processVoiceData` of `reference/tfar_1_0/ts/src/plugin.cpp:455-743` for a frame the
plugin app has already copied to stereo, with the session locked. Step by step:

1. **Gates.** Outside a game, for an unknown sender, or for a sender (or listener) without TFAR the
   frame passes through untouched; the local player's own voice without TFAR is silence;
   `muteSpectators` silences one spectator for another; a sender the spectator rules keep away is
   silence in serious mode when the listener is dead or the sender is not alive.
2. **Voice multiplier**, saturating; the result is kept as the radio original.
3. **Direct speech** (not the local player, not a spectator, within voice range + 15 m, 2 m per
   object): clear, the object low-pass with the previous frame as look-back, the vehicle low-pass,
   or the 100 Hz can't-speak low-pass, each at the original's gain; then the placement; then the
   direct-speech compressor. A sender not heard directly is silenced unless both spectate.
4. **Radios.** The radio compressor (its own instance) runs once over the original; each
   reception of `is_over_radio` gets a copy through its path (carried radio by subtype, speaker
   radio, intercom) and is mixed in, saturating.
5. **Global volume.**

`process_own_voice_for_speakers` runs the same chain on the captured voice as the local player's,
microphone path (no direct speech, no intercom), and returns the block for the `my_radio_voice`
stream; `append_own_voice_for_speakers` appends it to a `PlaybackMixer`. `ensure_sender_dsp` plans
a sender exactly as the audio path does and creates the objects its frames will borrow, off the
audio thread.

### Level and direction (replacing X3DAudio and clunk)

- **Direct speech:** the level is the X3DAudio part kept as gains: the 25-point voice curve
  (`DistanceCurve::tfar_voice` over `volume_attenuation`) at the straight distance, scaled by the
  sender's voice range, times `EmitterCone::TFAR_VOICE` when `voiceCone` is on. The original's
  swapped listener and emitter and its negated facing x and y (`helpers.cpp:46-110`) are kept, so
  the cone measures the sender's facing towards the listener. The direction is the HRTF source
  `voice_clunk` of the sender (`tfar_hrtf`), which replaces both clunk's ear delay and X3DAudio's
  panning; the simple ILD is not used for direct speech.
- **Makeup gain:** `HRTF_MAKEUP_GAIN_DB` (+4 dB) follows the HRTF on direct speech, compensating
  the diffuse-field EQ of the IRCAM sphere (about −4 dB straight ahead), so speech straight ahead
  keeps its original loudness. Whether to keep it is the roadmap's open question for the owner.
- **Speaker radios:** the original ran clunk and the simple ILD; here the HRTF source of the
  speaker's radio id gives direction and the ILD is dropped. They get no makeup: the ILD they
  replace is 0.625 (−4.1 dB) straight ahead, about what the HRTF leaves.
- **No HRTF source** (sphere missing, or not yet ensured): the 1.0 simple ILD
  (`ILD_COEFFICIENT_1_0`) pans, at the same level without the makeup.
- **Distance gain:** `volume_attenuation` is `10^(−1.5 · d / max)` (the original's
  `(d / (2 · max)) · −60 / 20`), 1 within 1 m, 0 below 0.001, `max` = 5 m for a player who cannot
  speak.

### Where DSP objects are missing

The audio path never creates anything: `ClientEffects`' accessors answer `None` for an object
never ensured. A missing low-pass leaves its gain, a missing HRTF source falls back to the ILD, a
missing compressor leaves the frame uncompressed, and a reception whose radio effect, phone
band-pass or speaker band-pass is missing stays out of the mix (a radio without its effect would
sound like direct speech from nowhere). The voice HRTF source (about 2 MB) is ensured only while
the sender is within direct-speech range.

### Fixed original bugs

| Id | Test |
|---|---|
| F-013 | `direct_speech_and_radio_each_advance_their_own_compressor_once_per_frame` |
| proposed F-061: the first frame behind objects reads an empty look-back buffer | `the_first_frame_behind_objects_has_no_look_back_and_is_filtered_once` |
| proposed F-062: a speaker's muffling skipped when the sender shares the listener's vehicle | `a_speaker_outside_the_listeners_vehicle_is_muffled_even_when_the_sender_sits_inside` |
| proposed F-063: the intercom never ducks | `the_intercom_is_ducked_while_a_long_range_transmission_is_received` |

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_voice_pipeline_1_0  # the unit tests and the voice scenarios
```

## Configuration

Read from `tfar_plugin_settings_1_0::PluginSettings` through `VoiceFrameContext`:

- `voiceCone` (default true): the emitter cone on direct speech.
- `headsetLowered` (default false): carried radios and the intercom at a tenth.
- `intercomVolume` (default 0.1), `intercomDucking` (default 0.2): the intercom level and ducking.
- `muteSpectators`, `spectatorNotHearEnemies`, `spectatorCanHearFriendlies`: the spectator gates.
- `objectInterceptionStrength` (default 400): read by the caller into `DspContext`.
- Through the routing: `intercomEnabled`, `full_duplex`.

## Public surface

- `VoiceProcessor`: `new(max_frames)`, `process_sender`, `process_own_voice_for_speakers`,
  `append_own_voice_for_speakers`, `scratch_capacities`; `VoiceScratchCapacities`;
  `OWN_RADIO_VOICE_STREAM`.
- `VoiceFrameContext` (settings, antennas, in game, serious mode, sender metadata, now) and
  `VoiceOutcome` (`Processed`, `Silenced`, `PassThrough`).
- `ensure_sender_dsp`, `ensure_every_sender_dsp`.
- `volume_attenuation`, `vehicle_volume_loss`, `vehicle_loss_gain`, `speaker_audible_distance`,
  `CANT_SPEAK_DISTANCE`, `CANT_SPEAK_GAIN`, `SPEAKER_GAIN`, `UNDERWATER_LEVEL`, `MAX_VEHICLE_LOSS`,
  `NO_VEHICLE_LOSS`; `HRTF_MAKEUP_GAIN_DB`, `hrtf_makeup_gain`.
- `Error` (`Dsp`, `Mixer`), `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_client_state_1_0` (the session, the effects and their keys);
  `tfar_radio_routing_1_0` (`is_over_radio`, the error levels); `tfar_radio_effects_1_0` (the
  effect mixdown); `tfar_plugin_settings_1_0` (the settings); `tfar_antennas_1_0` (the towers the
  routing reads); `tfar_geometry` (positions and directions); `tfar_dsp_filters` (the filters);
  `tfar_dynamics` (the compressors); `tfar_hrtf` (the HRTF sources and directions);
  `tfar_spatial_audio` (curve, cone, simple ILD); `tfar_sample_buffer` (the frame, gain and mixing);
  `tfar_playback_mixer` (the `my_radio_voice` stream); `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: each compressor runs once per frame
  (`direct_speech_and_radio_each_advance_their_own_compressor_once_per_frame`); the scratch never
  grows (`a_thousand_frames_leave_every_scratch_capacity_unchanged`); nothing pans twice. Tier 5
  in `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix
  in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports, §3 for the audio pipeline.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the chain of both versions.
- [Bug fixes](/documentation/program/bug_fixes.md) — the original bugs fixed here.
