# Voice pipeline for 0.9

The `tfar_voice_pipeline_0_9` crate: the 0.9.12 stereo function per sender, the path one
TeamSpeak voice frame takes through the gates, the direct voice, the radio effects, the
loudspeakers, the HRTF and the mix, plus the local player's own voice for loudspeakers and the
positional remote push-to-talk sound. Used by `tfar_plugin_0_9`.

## Contents

```text
crates/version_0_9/tfar_voice_pipeline_0_9/
├── Cargo.toml  the `tfar_voice_pipeline_0_9` package (lib): tier 5, workspace lints and dependencies
└── src/        the processor and its gates, the voice chain, the distance gain, the scratch, own voice
```

## How it works

The plugin owns one `VoiceProcessor` per server connection beside its `ServerRadioData`, both
behind the connection's lock. In the post-process callback it copies the frame to stereo
(`tfar_sample_buffer::copy_to_stereo`), reads what only TeamSpeak knows into a
`VoiceFrameContext` (the sender's and the listener's client ids, whether the sender's metadata
shows the plugin, the sender's channel name, the in-game flag), `try_lock`s the state, calls
`process_sender`, and writes the frame back (`write_back_from_stereo`). The context carries client
ids, not a nickname, because the original gates on TeamSpeak's client id (`hasClientData`,
`reference/tfar_0_9_12/ts/src/plugin.cpp:584`); the metadata and the channel name come in as values
so the processor makes no TeamSpeak call under the lock.

### The gates

`process_sender` follows `ts3plugin_onEditPostProcessVoiceDataEventStereo`
(`reference/tfar_0_9_12/ts/src/plugin.cpp:2653-2828`) and answers `VoiceOutcome`:

- sender and listener have current data and the plugin is enabled for the sender
  (`ClientData::refresh_plugin_enabled`, with its 10 s grace): `Silenced` when the sender sits in
  the [serious mode](/documentation/glossary.md#serious-mode) channel and the listener is dead,
  `Processed` otherwise;
- else the local player's own frame is `Silenced`; an alive, in-game listener hears a serious-channel
  TFAR player without current data as `Silenced`, unless the channel is another radio plugin's;
- everyone else is `PassThrough`, untouched. The original's "no info" debug line, every 4 s, is
  left to the plugin, which sees the `PassThrough`.

### The chain

Each frame is planned first (`src/voice_chain/sender_plan.rs`): with the state borrowed read-only it
computes the direct voice's branch and level, asks `tfar_radio_routing_0_9::is_over_radio_into` for
the receptions in the scratch reception list and plans each one's effect gain, effect error and
loudspeaker stages into the scratch route list. Then the sender's entry is borrowed mutably and the frame renders over its `ClientDsp`
(`src/voice_chain/frame_render.rs`), in the original's order:

```text
frame ── × voice volume multiplier ──┬─▶ direct voice (in range, not me) ── level ── direction ─┐
                                     │   else silence                                           │
                                     └─▶ copy ── compressor (once) ──┬─ reception 1 ─▶ mix ◀────┤
                                                                     └─ reception n ─▶ mix ◀────┘
                                                                      ── × global volume ── out
```

- **Distance gain:** `volume_from_distance` (`volumeFromDistance`, plugin.cpp:698):
  `d^-0.3 · max(0, reach − d) / reach`, 1 within a metre, 0 below 0.001, at most 1; the reach is the
  sender's voice volume times the vehicle multiplier, or 5 m when either side cannot speak. A reach
  of zero or less gives 0, where the original divides by zero.
- **Direct voice:** both can speak and no vehicle muffles: the distance gain; a vehicle muffles
  (combined isolation of at least 0.01, different vehicles): the vehicle low-pass at
  `20000 · (1 − isolation) / 4` Hz with the gain `volume(d, reach · (1 − isolation)) ·
  (1 − isolation)^1.2`; either side cannot speak: the 100 Hz low-pass at ×14.
- **Radio:** one compressor pass over the copy (threshold 65 dB, ratio 0.1, 1 ms / 300 ms), then per
  reception, by the transmitter's subtype: `digital` the personal effect, `digital_lr` and
  `airborne` the long-range effect, both at `((v + 1) / 10)^4 · 0.35` with the error from the
  effective distance over the range; `dd` the underwater effect at the diver volume level `· 0.6`
  with the error from the raw distance over the diver reach; `phone` the phone band-pass at
  `· 10`; any other subtype nothing. The effect writes to both ears or one, as the receiving radio
  is set.
- **Loudspeaker:** the 2000/1000 Hz band-pass at ×4; the distance gain over the loudspeaker's reach
  (`volume / 10 · speaker distance`, truncated to whole metres, 5 m when the sender cannot speak), or
  the vehicle low-pass when a vehicle stands between; below the wave line the 100 Hz low-pass at ×14;
  then the direction of the loudspeaker.
- **Mix and output:** each reception is added with saturation; the global volume is the last gain.

### Choices where the port differs from the original

- **Direction (one panner).** The original runs clunk (an ear delay only) and then the simple ILD on
  the direct voice and on every loudspeaker. Here a `tfar_hrtf::HrtfSource`, the one the sound state
  holds per sender (direct voice) and per radio id (loudspeaker), places the sound from the
  listener's view angle, and no ILD follows. The ILD with the 0.9 coefficient 0.375 is kept only as
  the fallback for a sound without an HRTF source: no HRIR sphere was loaded, or the source is not
  made yet.
- **Direction after the level.** The original places the sound before the level stage; here the
  level stage (gain or filter) runs first and the HRTF last. Both are linear, so the order changes
  only where a stage clips.
- **HRTF makeup gain.** The IRCAM sphere is diffuse-field equalised and puts a source straight ahead
  about 4 dB below its input; `HRTF_MAKEUP_GAIN_DB` (+4 dB) goes into the HRTF stage of direct speech
  so speech straight ahead keeps the original's level. Sources at the side and behind stay quieter,
  as the sphere measures them. A loudspeaker's HRTF stage runs at unity, as the 1.0 pipeline renders
  its speakers: the simple ILD it replaces already puts a loudspeaker straight ahead at 0.625, about
  the sphere's front level (`a_loudspeaker_through_the_hrtf_gets_no_makeup_gain`). The ILD fallback
  gets no makeup.
- **Sound state is borrowed, never made.** The original makes every filter, effect and ear-delay
  stage on first use on the audio thread. Here they come from `ClientData::ensure_dsp`, which the
  plugin calls while handling commands with `tfar_radio_routing_0_9::dsp_requirements`. A path whose
  object is missing (a command not handled yet) stays silent for that frame, never unfiltered (a
  can't-speak voice without its low-pass would play at ×14), and `frames_missing_sound_state` counts
  the frame.
- **Plugin-enabled cache entry.** The cache is the sender's current entry, else any entry with the
  sender's client id, where the original takes the first entry by nickname order.
- **Saturation.** Every gain and the mix saturate at the `i16` limits (`tfar_sample_buffer`).

### Own voice

`process_own_voice_for_speakers` runs the local player's captured frame through the same chain with
the local player as sender (`ts3plugin_onEditCapturedVoiceDataEvent`, plugin.cpp:2852): their
direct voice is silent and their own radios are skipped, so only a loudspeaker of another player on
the frequency they transmit on sounds. It returns nothing outside a game, for a dead local player or
without current data; `append_own_voice` appends the block to the playback mixer's
`my_radio_voice` stream off the audio thread only (it takes the stream lock); the plugin's
captured-voice callback pushes the block through a `StreamProducer` for that stream instead. The
captured frame itself is never changed.

### Memory on the audio path

`VoiceScratch` holds every buffer the chain writes, sized once (`DEFAULT_MAX_FRAMES`, 100 ms) and
reused, so a frame within that size changes no capacity. That includes the reception list:
`tfar_radio_routing_0_9::is_over_radio_into` refills it in place, and each reception shares its
radio id and vehicle with the server state (`Arc`), so routing copies no text. The reception and
route lists are emptied at the end of each frame, still under the server lock, so the audio thread
never drops the last handle on a radio id and never frees one.

### Remote push-to-talk sound

`render_remote_tangent_sound` shapes a decoded `remote_start` or `remote_end` WAV as the positional
`playWavFile` does (plugin.cpp:711-785): the radio volume and global gain, then, with current
listener data, the vehicle low-pass, on a loudspeaker the distance gain, the speaker band-pass and
the under-water low-pass at `CANT_SPEAK_GAIN * 50`, and last the direction through a one-shot HRTF
source at the makeup gain (its latency cut off) or the simple ILD without a sphere. The
`RemoteSoundPlan` says what to apply. It builds filters and an HRTF source per sound, so the
plugin calls it while handling the incoming command, never on the audio thread.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_voice_pipeline_0_9  # the gates, every branch of the chain, the HRTF, own voice
```

The HRTF scenarios load the shipped sphere `assets/hrir/IRC_1002_C.bin`.

## Configuration

None: the crate reads no setting. The levels it uses (global volume, diver volume level, loudspeaker
distance, terrain coefficient) arrive in `ServerRadioData` from the game's `FREQ` message.

## Public surface

- `VoiceProcessor` with `new`, `process_sender`, `process_own_voice_for_speakers`, `scratch` and
  `frames_missing_sound_state`; used by `tfar_plugin_0_9` from the voice callbacks.
- `VoiceFrameContext`, `OwnVoiceContext`, `VoiceOutcome`.
- `VoiceScratch`, `ScratchCapacities`, `DEFAULT_MAX_FRAMES`, `ROUTE_CAPACITY`, `NICKNAME_CAPACITY`.
- `append_own_voice`, `OWN_VOICE_STREAM_NAME`.
- `render_remote_tangent_sound`, `RemoteSoundPlan`, `ListenerView`,
  `UNDER_WATER_SOUND_GAIN_FACTOR`; used by `tfar_plugin_0_9` for an incoming push-to-talk.
- `volume_from_distance`, `radio_volume_multiplier`, `CANT_SPEAK_DISTANCE`, `MINIMUM_AUDIBLE_GAIN`, also the
  volume curve of the remote push-to-talk sounds.
- The gain constants `CANT_SPEAK_GAIN`, `SPEAKER_GAIN`, `RADIO_EFFECT_GAIN`,
  `DIVER_RADIO_EFFECT_GAIN`, `PHONE_GAIN`, `VEHICLE_LOSS_EXPONENT`, `HRTF_MAKEUP_GAIN_DB`, with
  `hrtf_makeup_gain` and `vehicle_loss_gain`.
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_client_state_0_9` (the state and the sound state it borrows);
  `tfar_radio_routing_0_9` (the receptions, distances and serious-mode checks);
  `tfar_radio_effects_0_9` (`process_radio_effect` and the effects); `tfar_dsp_filters` (the filters
  the sound state holds); `tfar_dynamics` (the compressor); `tfar_hrtf` (the HRTF sources and the
  listener-relative direction); `tfar_spatial_audio` (the 0.9 ILD fallback); `tfar_sample_buffer`
  (the stereo buffer, saturating gain and mix); `tfar_playback_mixer` (the own-voice stream and the
  remote sound's playback); `thiserror` for the error type; `tracing` for a refused remote sound
  filter.
- Used by: `tfar_plugin_0_9`.
- Rules: no TeamSpeak call and no sound object made or dropped (the processor only borrows
  `ClientDsp`); no scratch capacity changes for frames within the scratch's size
  (`a_thousand_frames_change_no_scratch_capacity`, which also holds the reception list empty and the
  shared radio ids unchanged between frames); the compressor runs once per frame
  (`the_compressor_envelope_advances_once_per_frame_whatever_the_receptions`); a missing sound object
  silences its path (`a_frame_missing_a_needed_filter_silences_that_path_and_is_counted`). Tier 5 in
  `crates/version_0_9`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — the 0.9 voice chain and the
  building blocks.
- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original stereo function, section 3.
- [Bug fixes](/documentation/program/bug_fixes.md) — the fixes the chain carries.
