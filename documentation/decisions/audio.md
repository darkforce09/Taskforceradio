**Status:** live

# Audio decisions

How the plugins place a voice in space and shape its sound: the HRTF, the distance and cone gains that
replace X3DAudio, and the filter libraries. The chain each version builds from them is in the
[audio pipeline](/documentation/architecture/audio_pipeline.md).

### 2026-10-02 — HRTF through the hrtf crate with the IRCAM sphere, in both versions (D19)

**Context:** Both original plugins position voices with clunk, whose KEMAR HRTF filter is disabled (a
TODO in its `hrtf.cpp`), so only the ear delay remains. No other HRTF code exists in the originals.

**Decision:** HRTF uses the `hrtf` crate, a real binaural HRTF, in both versions. The IRCAM HRIR sphere
ships in assets/hrir/ with its credit.

**Consequences:** Direction sounds different from the originals (D3 accepts this), and better: the
disabled filter is a bug fixed in both versions. `tfar_hrtf` writes the block adapter from TeamSpeak's
frame sizes to the HRTF block and the per-source state. Whether the crate's artefacts on fast-moving
sources are acceptable is an open question for the audio milestone.

**Supersedes:** none.

### 2026-10-02 — X3DAudio is replaced by hrtf for direction and a small gain model (D20)

**Context:** The 1.0 plugin uses Windows X3DAudio for the distance curve, the emitter cone and panning;
it is not available on Linux, and no standalone Rust X3DAudio calculator exists.

**Decision:** X3DAudio is replaced by `hrtf` for direction, plus a small Rust gain model on `glam` for the
distance curve and the emitter cone. There is no full X3DAudio port and no double panning. FAudio's
F3DAudio is used only as an oracle for the curve and cone maths.

**Consequences:** `tfar_spatial_audio` holds the piecewise distance curve and the cone gain, about 150
lines, tested against golden vectors from F3DAudio built in `tfar_golden_vectors`. Direction comes from
one place only, so a voice is never panned twice.

**Supersedes:** none.

### 2026-10-02 — Filters come from sci-rs and biquad, processed in f64 DF2T (D21)

**Context:** The originals use the DSPFilters library: Butterworth low-pass and band-pass filters of
several orders, and RBJ high-pass and low-pass sections, in many radio and speaker effects.

**Decision:** `sci-rs` designs the Butterworth low-pass and band-pass filters as second-order sections,
`biquad` provides the RBJ high-pass and low-pass coefficients, and everything processes in f64 Direct
Form II Transposed.

**Consequences:** `tfar_dsp_filters` wraps both crates per setup and holds the multi-channel state; no
filter design is written by hand. Its output is compared with DSPFilters built from `reference/` in
`tfar_golden_vectors`, within a stated tolerance.

**Supersedes:** none.

### 2026-10-03 — Direct speech keeps a 4 dB makeup gain after the HRTF (D23)

**Context:** The compensated IRCAM sphere is diffuse-field equalised, so a source leaves `tfar_hrtf`
about 4 dB below its input straight ahead and about 9 dB below behind (measured in `tfar_hrtf`). The
originals' X3DAudio panning and clunk ear delay lost no level, so direct speech would come out quieter
than a Windows player hears it. The options were a fixed makeup gain after the HRTF, or accepting the
quieter output (OF-022).

**Decision:** Both voice pipelines keep `HRTF_MAKEUP_GAIN_DB`, +4 dB, on direct speech rendered through
the HRTF, so speech straight ahead keeps the original's loudness. Sources at the side and behind stay
as much quieter as the sphere measures them. The simple-ILD fallback gets no makeup. Speaker radios get
the HRTF without the makeup: the simple ILD they replace is itself 0.625 (−4.1 dB) straight ahead, about
what the HRTF leaves.

**Consequences:** The constant and its linear factor `hrtf_makeup_gain` live in each voice pipeline
(`tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`), not in `tfar_hrtf`, which stays a neutral
renderer. The gain is a fixed number, not a setting. The 1.0 pipeline renders speaker radios at the
plain level; the 0.9 pipeline still passes its loudspeakers through the same placement as direct
speech, with the makeup, until it follows this entry. Tests: `the_makeup_gain_is_four_decibels` in
both pipelines, `the_makeup_gain_keeps_speech_straight_ahead_at_its_input_level` (0.9) and
`straight_ahead_the_hrtf_with_its_makeup_keeps_about_the_input_level` (1.0).

**Supersedes:** none.

### 2026-10-03 — HRTF memory of about 2 MB per source is accepted, with no pooling (D26)

**Context:** Each `HrtfSource` holds its own copy of the prepared HRTF processor, about 2 MB, so memory
grows with every audible source: one per sender for direct speech and one per sender and speaker radio.
A shared processor or a cap on sources would bound it at the cost of a pool and its locking on the
audio path (OF-023).

**Decision:** The memory is accepted. Every source keeps its own processor; there is no pool, no shared
processor and no cap on sources.

**Consequences:** A session with a few dozen audible sources costs tens of megabytes. Sources are made
off the audio thread by the client-state crates and dropped with the sender or its sound state; the
1.0 voice source is ensured only while the sender is within direct-speech range.

**Supersedes:** none.

### 2026-10-10 — Voice callbacks wait for the state and sound state is kept across resets (D27)

**Context:** The owner heard clicks and stutters in other players' voices with the 0.9 plugin, only
while the game ran. The voice callbacks took the connection state with `try_lock` and passed a block
through unprocessed when a command thread held it. That block came out at full level, with no
distance gain, no effect and not delayed by the HRTF's 480 frames like the blocks around it. With the
game sending a `POS` for each nearby player at least every 25 ms, the lock was busy often. Expensive
work under the lock made it worse: in serious mode every `POS` of a silent player dropped and rebuilt
a two-megabyte HRTF source. Both originals wait on their critical section instead. A reset of the
direction sources on a talk start or a tangent press ran on another thread and could cut into blocks
already playing. Neither plugin set flush-to-zero, which the originals do.

**Decision:** The post-process and captured voice callbacks of both plugins wait for the connection
state lock. Its holders never call TeamSpeak, never take another lock and keep their work short. The
server map is still taken with `try_read`. The voice reset (`resetVoices`) clears the memory of the
filters it covers and frees nothing. An `HrtfSource` restarts itself on the first frame after more
than `IDLE_RESET_AFTER` (200 ms) without input. No other thread resets a direction source. Expired
0.9 entries are freed after the lock is released. Every voice block runs with FTZ and DAZ set and
the caller's MXCSR restored afterwards.

**Consequences:** The audio thread can wait for one short decision on a command thread. The coding
standard's audio-thread rule says when it may wait. `AudioDiagnostics` in `tfar_ts3_host` counts the
waits and the blocks passed through while the server map changes. Each plugin's service thread logs
them at debug level once a second when any occurred. The 0.9 log also shows voice frames without
sound state. Under D26, direction sources now go only with their sender, or with their 0.9 radio
entry once `reset_radio_effects` empties it. Tests:
- `an_audio_callback_waits_for_a_held_state_and_counts_the_wait` (0.9 plugin);
- `a_frame_after_a_long_gap_starts_from_silence` and
  `a_frame_after_a_short_gap_continues_the_utterance` (`tfar_hrtf`);
- the voice-reset tests in both client-state crates and in `tfar_radio_routing_0_9`;
- the denormal guard tests (`tfar_ts3_host`);
- the mock-host scenarios `a_sender_silent_past_the_idle_gap_never_replays_the_old_tail` (0.9) and
  `a_sender_silent_past_the_idle_gap_gets_a_fresh_hrtf_source` (1.0).

**Supersedes:** none.
