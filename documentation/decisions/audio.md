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
