**Status:** live

# Audio pipeline

The voice and radio processing chain of each plugin version as the Rust port builds it: which
TeamSpeak callback feeds it, which crate does each step, and where the port differs from the original
on purpose (marked with bug-fix ids from [bug fixes](/documentation/program/bug_fixes.md)). The
original chains, with every constant, are in the
[0.9 plugin spec](/documentation/program/behaviour_specs/tfar_0_9_plugin.md#3-audio-pipeline) and
the [1.0 plugin spec](/documentation/program/behaviour_specs/tfar_1_0_plugin.md#3-audio-pipeline).

## Building blocks

| Step | Crate | Library | Replaces |
|---|---|---|---|
| Interleaved i16 frames, saturating gain and mix, bounds-safe stereo view | `tfar_sample_buffer` | — | `SampleBuffer`, `applyGain`, the stereo scratch copy (F-004, F-006) |
| Butterworth LP/BP filters | `tfar_dsp_filters` | `sci-rs` designs second-order sections | DSPFilters `Butterworth::LowPass<N>`, `BandPass<N>` |
| RBJ high-pass and low-pass | `tfar_dsp_filters` | `biquad` coefficients | DSPFilters `RBJ::HighPass`, `RBJ::LowPass` |
| Filter processing | `tfar_dsp_filters` | f64 Direct Form II Transposed, one state per channel | DSPFilters f32 Direct Form II |
| Compressor | `tfar_dynamics` | our feed-forward compressor | simpleSource `SimpleComp` |
| Direction | `tfar_hrtf` | `hrtf` crate, IRCAM HRIR sphere, block adapter from TS frame sizes | clunk (an ear delay only, F-001) and X3DAudio panning |
| Distance curve, emitter cone, simple ILD | `tfar_spatial_audio` | small gain model on `glam` | X3DAudio curve and cone (1.0), `applyILD` (F-012) |
| Radio effects | `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0` | built on `tfar_dsp_filters` | `RadioEffect.h`, `RadioEffect.hpp` |
| WAV cache and playback mixing | `tfar_playback_mixer` | `hound`, `rtrb` queues | `PlaybackHandler`, the 0.9 playback deques (F-005) |
| The chain itself, and the distance volume | `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0` | — | the 0.9 stereo function and `volumeFromDistance`, 1.0 `processVoiceData` and `volumeAttenuation` |

Rules for every step:
- **One panner.** `hrtf` gives direction; the curve and cone give level. Nothing pans twice (D20).
- **No blocking on the audio thread.** Voice callbacks read client and game state from snapshots
  (`arc-swap`) and queues (`rtrb`); they never wait on the game-link thread (F-002).
- **State is owned.** Filters, effects, compressors and HRTF sources live per (sender, key) inside
  the pipeline's state, created on first use and dropped with the sender or the playback.
- **Denormals:** f64 processing replaces the FTZ/DAZ register tricks.
- **Format:** 48 kHz interleaved i16 in and out; processing in f64, except the HRTF, which the
  `hrtf` crate runs in f32; output clamped to i16.

## TeamSpeak callbacks

| Callback | 0.9 plugin | 1.0 plugin |
|---|---|---|
| `onEditPostProcessVoiceDataEvent` | Per sender: the voice chain below | Per sender: the voice chain below, in game only |
| `onEditCapturedVoiceDataEvent` | In game, my data present, alive: my voice through the chain as sender = me, queued as the `my_radio_voice` playback; the outgoing samples stay unchanged | Same, also requires speakers present |
| `onEditMixedPlaybackVoiceDataEvent` | `tfar_playback_mixer` adds every queued playback into channels 0–1, saturating | Same, current server only |
| `onCustom3dRolloffCalculationClientEvent` | `*volume = 1` (TeamSpeak's own 3D stays neutral) | Same |

A dormant plugin (see the [game link protocol](/documentation/architecture/game_link_protocol.md#dormant-plugin))
passes every callback through untouched.

## 0.9 voice chain (`tfar_voice_pipeline_0_9`)

```text
TS post-process samples (mono or more channels)
  -> stereo view (F-006)
  -> gate: sender and I have fresh data, sender's metadata says the plugin is on
       (serious channel and I am dead -> silence; gate fails for me -> silence)
  -> gain x voiceVolumeMultiplier, saturating (F-004)             --> copy "original"
  |
  |  DIRECT SPEECH (not me, distance <= sender's voiceVolume)
  |   hearing, same vehicle or no isolation: gain = 0.9 distance curve
  |   hearing, other vehicle:  Butterworth LP2 at 20000*(1-loss)/4, gain = curve*(1-loss)^1.2
  |   cannot speak:            Butterworth LP4 at 100 Hz, gain = curve(not hearing)*14
  |   -> tfar_hrtf at the sender's position, rotated by my view angle (F-001)
  |
  |  RADIO ("original")
  |   -> compressor (threshold 65 dB, ratio 0.1, attack 1 ms, release 300 ms)
  |   -> for each reception route (tfar_radio_routing_0_9):
  |        volume ((v+1)/10)^4, by the transmitter's subtype:
  |          digital            -> SW effect, error from distance (F-019), gain vol*0.35, stereo mode
  |          digital_lr/airborne-> LR effect, same
  |          dd                 -> underwater effect, error from raw distance, gain ddVol*0.6
  |          phone              -> Butterworth BP2 1850/1550, gain vol*10
  |        on a ground speaker:  Butterworth BP1 2000/1000 x4, speaker distance gain or vehicle LP,
  |                              waveZ < -1.1 -> LP4 100 Hz x14, tfar_hrtf at the speaker position
  |        -> saturating mix
  -> gain x globalVolume -> back into TS channels 0-1, others zeroed
```

0.9 radio effect inside `tfar_radio_effects_0_9`, per sample, in the original order: 2399-sample delay
line → ring modulation by the error level → foldback distortion → RBJ high-pass → RBJ low-pass →
×30. Filters per effect:

| Effect | Filters |
|---|---|
| SW (personal) | mic HP 900 Hz Q 0.85 and LP 3000 Hz Q 2.0 before the level mean, then HP 520/0.97 and LP 4300/2.0 |
| LR and airborne | HP 520 Hz Q 0.97, LP 1300 Hz Q 1.0 |
| Underwater (DD) | random sample drop with probability = error, Butterworth BP2 1000/400, ×30 |

The simple ILD of `tfar_spatial_audio` stays available as the fallback panner when the HRIR sphere
cannot be loaded; the plugin logs the fallback once.

## 1.0 voice chain (`tfar_voice_pipeline_1_0`)

```text
TS post-process samples
  -> stereo view (F-006)
  -> gates: in game; sender known; sender's plugin on (10 s cache that expires, F-016);
       spectator rules (muteSpectators, spectatorNotHearEnemies, spectatorCanHearFriendlies)
  -> gain x voiceVolumeMultiplier, saturating (F-004)             --> copy "original"
  |
  |  DIRECT SPEECH (not me, not a spectator, distance + 2*objectInterception <= voiceVolume+15)
  |   hearing, same vehicle or no isolation:
  |       objectInterception > 0 -> Butterworth LP2 at 2000 - min(n,5)*strength (one-frame look-back)
  |   hearing, other vehicle:  LP2 at 20000*(1-loss)/4, gain = attenuation*(1-loss)^1.2
  |   cannot speak:            LP4 100 Hz, gain = attenuation(not hearing)*14
  |   -> level: tfar_spatial_audio distance curve (the 25-point TFAR curve, scaled by the sender's
  |      voiceVolume) x emitter cone (inner 90 deg, outer 135 deg, 1.8 / 1.0) when voiceCone is on
  |   -> direction: tfar_hrtf at the unit direction to the sender (F-009) relative to my view
  |   -> direct-speech compressor (threshold 80 dB, ratio 0.1, 1 ms / 300 ms)  (F-013)
  |
  |  RADIO ("original")
  |   -> radio compressor, its own instance (F-013)
  |   -> for each reception route (tfar_radio_routing_1_0, antennas from tfar_antennas_1_0):
  |        volume ((v+1)/10)^4; headsetLowered x0.1; subtype matched by name (F-010):
  |          digital    -> SW effect (or underwater effect when carried under water)
  |          airborne   -> airborne effect;   digital_lr -> LR effect
  |          phone      -> Butterworth BP2 1850/1550, gain vol*10;   anything else -> silence
  |          error = calcErrorLevel(min(antenna loss, distance/range)) (F-011), gain vol*0.35
  |        ground speaker: BP1 2000/1000 x4, speaker distance gain or vehicle LP,
  |                        waveZ < -1.1 -> LP4 100 Hz x14, SW effect, tfar_hrtf at the speaker
  |        intercom: LR effect at error 0, gain intercomVolume, x0.1 headset lowered,
  |                  x(1-intercomDucking) while any LR route is louder than 2
  |        -> saturating mix
  -> gain x globalVolume -> back into TS channels 0-1, others zeroed
```

1.0 radio effect inside `tfar_radio_effects_1_0`, in the original order: foldback (threshold
0.3·(1−err)·mean/0.005) → ×30 → 2399-sample delay line → ring modulation → RBJ high-pass → RBJ
low-pass; mono sum over channels, out to both channels or one side by stereo mode (×1.5 when not
stereo).

| Effect | High-pass | Low-pass |
|---|---|---|
| LR | 520 Hz, Q 0.97 | 1300 Hz, Q 1.0 |
| Airborne | 1000 Hz, Q 1 | 4000 Hz, Q 1 |
| SW (personal) | 900 Hz, Q 0.85 | 3000 Hz, Q 2.0 |
| Underwater | random sample drop, Butterworth BP2 1000/400, ×30 | — |

## Sound playback (`tfar_playback_mixer`)

- Local and on/off sounds: `radio-sounds` WAVs played through TeamSpeak's `playWaveFile` from the
  plugin folder (the local start and end sounds per radio kind sw, lr, dd and ab, plus on and off).
- Remote tangent sounds and the `my_radio_voice` stream: decoded once (48 kHz, stereo, i16 only;
  anything else is refused with a log line), queued per playback, mixed in the mixed-playback
  callback. Positional sounds get the speaker band-pass, the vehicle low-pass, the underwater
  low-pass and direction from `tfar_hrtf`, exactly as speech on a ground speaker.
- Each playback owns its filters and is dropped when it ends; no thread per playback (F-005).

## HRTF rendering (`tfar_hrtf`)

- **Sample format:** the `hrtf` crate convolves in f32; every other step processes in f64. The
  block adapter converts i16 to f32 on the way in and back on the way out.
- **Block size and latency:** the processor renders 4 interpolation steps of 120 frames, 480 frames
  at a time, and glides direction and gain across the steps. The adapter adds a constant
  480 frames, 10 ms at 48 kHz (`ADDED_LATENCY_FRAMES`), to every source it renders.
- **Level:** the compensated IRCAM sphere is diffuse-field equalised, so a source comes out quieter
  than its input: about −4 dB straight ahead and −9 dB behind. Nothing compensates for it yet (see
  the open questions below).
- **Memory:** every `HrtfSource` holds its own copy of the prepared processor, about 2 MB, so a
  pipeline with one source per (sender, key) grows by about 2 MB per audible source.
- **The sphere file:** both plugins load it from `tfar_hrir/IRC_1002_C.bin` relative to
  TeamSpeak's plugins folder, beside `radio-sounds/`; `cargo xtask dist` copies the sphere and its
  `CREDITS.md` from `assets/hrir/` into `tfar_hrir/` of each version folder. When the file cannot
  be loaded, the simple ILD is the fallback panner.

## Spatial gains (`tfar_spatial_audio`)

- **ILD coefficients:** 0.375 for 0.9 (`reference/tfar_0_9_12/ts/src/plugin.cpp:621`) and
  21.5° in radians, 0.37525, for 1.0 (`reference/tfar_1_0/ts/src/datatypes.hpp:81`); both with the
  base gain 0.625.
- **The 1.0 cone call:** the original X3DAudio call swaps the roles on purpose: the X3DAudio
  listener sits at the sender's position facing my view direction, and the emitter sits at my
  position facing the sender's view direction with its x and y negated
  (`reference/tfar_1_0/ts/src/helpers.cpp:75`). `tfar_voice_pipeline_1_0` honours both the swap and
  the x/y flip when it calls `EmitterCone::gain`, so a sender facing me is the one at the inner
  volume, as in the original.
- **Distance volume:** `volumeAttenuation` (1.0) and `volumeFromDistance` (0.9) belong to the voice
  pipeline crates; `tfar_spatial_audio` builds the 25-point curve from the function it is given and
  owns no distance rule of its own.

## Open questions

- **Makeup gain for the HRTF:** whether the voice pipelines add a fixed gain after `tfar_hrtf`
  (about +4 dB, the frontal loss of the diffuse-field EQ) so that direct speech keeps the
  original's loudness, or accept the quieter output. Pending the owner.
- **HRTF memory:** about 2 MB per source; whether many simultaneous senders need a shared
  processor or a cap on HRTF sources. Pending a measurement in M6 and M7.

## Differences from Windows players

D3 accepts that sound differs: real HRTF instead of an ear delay (F-001), unit directions (F-009),
the corrected error-level curves (F-011, F-019), one compressor run per path (F-013) and saturation
instead of wrap (F-004). Wire formats do not change, so mixed sessions with Windows players work;
only what each listener hears locally differs. Whether the `hrtf` crate's clicks on fast-moving
sources are acceptable is an open question for M4 in the [roadmap](/documentation/roadmap.md).
