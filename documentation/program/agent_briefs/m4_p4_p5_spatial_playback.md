**Status:** live

# M4 · P4 + P5 · `tfar_spatial_audio` and `tfar_playback_mixer`

You are agent P4, the spatial and playback author for the TFAR Rust port. You own both crates in this prompt. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`
(audio-thread rules) and `/documentation/architecture/audio_pipeline.md`.

## `tfar_spatial_audio` (crates/audio/tfar_spatial_audio)
Decision D20: the `hrtf` crate gives direction (another crate). This crate gives the gain parts of what the 1.0
plugin got from X3DAudio, plus the simple ILD both versions use.

- **`DistanceCurve`:**
  - Piecewise-linear `(normalised_distance, gain)` points, sorted and validated.
  - `gain_at(distance, scaler)` normalises by `distance / scaler`, interpolates linearly, and holds the last value past the end. These are X3DAudio curve semantics; FAudio `F3DAudio.c` is the oracle G1 will use.
  - Constructor `DistanceCurve::tfar_voice(voice_volume, attenuation: impl Fn(f64) -> f64)` builds the 25-point 1.0 curve exactly as `reference/tfar_1_0/ts/src/helpers.cpp:113-140` does (points k·0.05 for k = 0..19, then 1.05 … 1.25), taking the version's `volumeAttenuation` function as a parameter.
- **`EmitterCone`:**
  - `{ inner_angle, outer_angle, inner_volume, outer_volume }`.
  - `gain(emitter_front: Direction, emitter_to_listener: Direction)`: full inner volume inside half the inner angle, outer volume beyond half the outer angle, linear in between. These are X3DAudio cone semantics.
  - `EmitterCone::TFAR_VOICE` = inner 90°, outer 135°, inner volume 1.8, outer volume 1.0 (`reference/tfar_1_0/ts/src/helpers.cpp:88-110`).
- **Simple ILD:**
  - `stereo_pan_gains(relative_direction: Direction, view_angle: Angle) -> (f32, f32)` implements `L = −c·cos(dir) + 0.625`, `R = c·cos(dir) + 0.625`, with `dir = polar_angle + view_angle` wrapped by `Angle::normalized_signed` (F-012 fix).
  - Per-version coefficient: `ILD_COEFFICIENT_0_9 = 0.375` (`reference/tfar_0_9_12/ts/src/plugin.cpp:608`) and `ILD_COEFFICIENT_1_0 = 0.37525` (`reference/tfar_1_0/ts/src/helpers.cpp:12`, ≈ sin(21.5°)). Use the exact values the originals compute and cite them.
  - `apply_pan(&mut StereoBuffer, gains)`.
- **Tests:**
  - curve interpolation, the ends, an unsorted-points error;
  - the TFAR voice curve shape (monotone non-increasing for a monotone attenuation);
  - cone gain inside, outside and in the middle;
  - ILD symmetry (front = equal gains), hard left/right values, the wrap for negative sums.

## `tfar_playback_mixer` (crates/audio/tfar_playback_mixer)
The radio-sound playback engine behind the mixed-playback callback (both behaviour specs, §3 "PlaybackHandler" / "WAV playback").

- **`WaveLibrary`:**
  - Loads WAVs with `hound`.
  - Accepts only 48 kHz, 2 channels, 16-bit PCM; anything else is an `Error` naming the file and its format, never a silent skip.
  - Caches by canonical path (`Arc<[i16]>`), with `preload(paths)`.
- **`Playback`:**
  - An owned interleaved buffer and a cursor.
  - Its processing (gain, filters, pan) is applied **when the playback is created, off the audio thread**. The caller supplies a `FnOnce(&mut StereoBuffer)`, so filters are owned and freed with the playback (F-005).
  - Named by `PlaybackId` (a counter), never `rand()`.
- **`PlaybackMixer`:**
  - `add(Playback)` from any thread, via a bounded lock-free queue (`rtrb`) or a short `parking_lot` lock.
  - `mix_into(samples: &mut [i16], channels: usize, frames: usize, channel_fill_mask: &mut u32)`, called on the TS audio thread, implements the original semantics (`reference/tfar_1_0/ts/src/PlaybackHandler.cpp:36`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2600`):
    - if `fill_mask & 3 == 0`, zero channels 0–1 first;
    - mix saturating into channels 0 and 1;
    - leave channels ≥2 untouched (1.0 behaviour; also correct for 0.9, which zeroed them only because it wrote them);
    - set `fill_mask |= 3` when anything was written;
    - drop finished playbacks.
  - No allocation in `mix_into`.
  - `append_stream(name, samples)` covers the "my_radio_voice" continuous stream that the captured-voice callback appends each frame. It is a named FIFO stream with a bounded length (drop oldest beyond 500 ms, and document why).
- **Tests:**
  - a WAV format rejection (write temp WAVs with hound in the test);
  - cache hit;
  - mixing two playbacks with saturation;
  - fill-mask semantics (pre-filled vs empty);
  - mono and 6-channel outputs;
  - a finished playback being removed;
  - a stream that stays bounded.

## Owned files
`crates/audio/tfar_spatial_audio/**`, `crates/audio/tfar_playback_mixer/**`. Report missing workspace dependency features;
don't edit the root manifest.

## Checks (each alone; `CARGO_TARGET_DIR=target-m4-p4`, `CARGO_BUILD_JOBS=4`; logs `target/agent-logs/p4-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the ILD coefficients with their source lines.

Budget: M (250k tokens). Stop there and report what is done and what is not.
