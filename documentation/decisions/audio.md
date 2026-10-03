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
