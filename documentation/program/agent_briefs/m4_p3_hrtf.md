**Status:** live

# M4 · P3 · `tfar_hrtf` with the IRCAM sphere

You are agent P3, the HRTF author for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md`
first (including the **Build directory rule**), then `/documentation/standards/coding_standards.md` (audio-thread
rules) and `/documentation/architecture/audio_pipeline.md`.

## Decision context
- D19: real binaural HRTF through the `hrtf` crate (0.9.0, MIT) replaces clunk's ear-delay-only behaviour in both versions. This fixes F-001: the disabled KEMAR filter at `reference/tfar_1_0/ts/include/clunk/hrtf.cpp:216`.
- **The owner approved downloading the HRIR data.**

## Steps
1. **HRIR data.**
   - Download one IRCAM sphere `.bin` built by `mrDIMAS/hrir_sphere_builder` (GitHub; the `hrtf_base/IRCAM/` folder). Pick the subject the `hrtf` crate's own examples or tests use (check its repo/docs), else `IRC_1002_C.bin`.
   - Store it under `assets/hrir/` with assets/hrir/CREDITS.md, which states:
     - the source URL and commit;
     - the IRCAM LISTEN database credit ("LISTEN HRTF database, IRCAM, http://recherche.ircam.fr/equipes/salles/listen/") and its terms of use;
     - the SHA-256 of the file.
   - Update `assets/hrir/README.md` (Contents, format, why this subject).
   - The file is binary; `.gitattributes` already marks `*.bin` binary.
2. **`tfar_hrtf` (crates/audio/tfar_hrtf):**
   - **`HrirLibrary::load(path) -> Result<Arc<HrirLibrary>>`** loads the sphere once, resampled to 48 kHz by the `hrtf` crate. Construction is heavy, so it happens off the audio thread.
   - **`HrtfSource`:** per-source state (previous sample vector, previous left/right tails, distance gain) plus a **block adapter**. TeamSpeak delivers frames of variable length (commonly 480 or 960 samples per channel at 48 kHz); the `hrtf` processor needs `interpolation_steps · block_len` input. Choose block_len and interpolation_steps for ≤ 10 ms added latency, and document the latency.
     - The adapter keeps an input FIFO and an output FIFO. It outputs silence only during the very first fill.
     - It never drops audio when the frame size changes.
     - It allocates nothing per frame after warm-up (pre-size the buffers).
   - **`HrtfSource::process(&mut self, input: &StereoBuffer or mono slice, output: &mut StereoBuffer, direction: ListenerRelativeDirection, distance_gain: f32)`.** The input is down-mixed to mono: the original used only the left channel, and we mix L+R/2 deliberately, with a note.
   - **`ListenerRelativeDirection::from_arma(listener_position, listener_view_angle: Angle, source_position) -> Self`** converts Arma world axes (x east, y north, z up; the view angle is the compass heading) into the `hrtf` crate's coordinate convention. **Check the crate's convention in its source/docs** and pin it with tests: a source straight ahead gives equal ears; a source at +90° right gives right-ear dominant energy; behind and left accordingly; a source above stays roughly symmetric.
   - **`reset()`** when a source jumps (the caller decides).
3. **Tests**, using the real sphere from `assets/hrir/` via `CARGO_MANIFEST_DIR`:
   - the four directions' ear-energy ratios;
   - zero input → zero output;
   - variable frame sizes 480/960/441/1 over 2 seconds giving continuous output with exactly the documented latency;
   - no allocation after warm-up, checked by a capacity-stability assertion;
   - loading a missing file → `Error`.

## Owned files
`crates/audio/tfar_hrtf/**`, `assets/hrir/**`. If `hrtf` needs features or a dependency missing from `[workspace.dependencies]`, make one
Edit to the root `Cargo.toml` (re-read first) and report it.

## Checks (each alone, with your target dir; logs `target/agent-logs/p3-*.log`)
fmt, clippy `--all-targets -D warnings`, test for the crate; `cargo xtask verify all` (report foreign violations only;
markdown-placement allows `assets/**/CREDITS.md`).

## Report additions
The chosen sphere and its SHA-256; block_len, interpolation_steps and the latency; the coordinate mapping with its test names.

Budget: L (350k tokens). Stop there and report what is done and what is not.
