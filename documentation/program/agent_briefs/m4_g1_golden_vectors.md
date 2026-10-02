**Status:** live

# M4 · G1 · `tfar_golden_vectors`: original-C++ oracles against our audio crates

You are agent G1, the oracle author for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md`
first (including the **Build directory rule**), then `/documentation/standards/coding_standards.md` and the public APIs
of `tfar_dsp_filters`, `tfar_dynamics`, `tfar_spatial_audio` and `tfar_sample_buffer`.

## Decision context
D9: C/C++ is allowed **only** here, as a test oracle compiled with `cc`. Allowed folders are
`tools/tfar_golden_vectors/oracle/` (vendored third-party C) and `tools/tfar_golden_vectors/shim/` (our tiny `extern "C"`
glue). The originals are compiled **straight from `reference/`**, read-only: never copy them into the repo, and never
modify them. Any include-path fix-ups (the `DspFilters/` vs `dspfilters/` case mismatch, backslash includes,
`pow10f`) happen in `OUT_DIR`: generated alias headers or copies there, plus `-D` defines. See
`/documentation/program/behaviour_specs/windows_dependencies.md` §3.

## Deliverables
1. **`build.rs`** compiles with `cc`, C++17:
   - DSPFilters sources from `reference/tfar_1_0/ts/include/dspfilters/source/*.cpp` (only those needed: Biquad, Butterworth, Cascade, Filter, Param, PoleFilter, RBJ, RootFinder, State, Design);
   - simpleSource `SimpleComp`/`SimpleEnvelope` from `reference/tfar_1_0/ts/include/simpleSource/`;
   - `oracle/F3DAudio.c` + headers, vendored from GitHub `FNA-XNA/FAudio` (src/F3DAudio.c, include/F3DAudio.h, and any minimal headers it needs), with `oracle/LICENSE` (zlib) and `oracle/README.md` recording the source commit;
   - `shim/oracle_shim.cpp`, which exposes `extern "C"` functions:
     - a Butterworth low-pass / band-pass of a given order, run over a mono or stereo f32 buffer via `SimpleFilter<..., N>` DirectFormII;
     - RBJ low-pass / high-pass with Q;
     - SimpleComp over interleaved stereo doubles with given settings;
     - the F3DAudio distance-curve evaluation and cone gain for given listener/emitter vectors. Use `F3DAudioCalculate` with a 1-channel emitter and stereo output, and the curve/cone set, and return the matrix coefficients and the curve-derived volume.

   The `build.rs` fails with a clear message if `reference/` is missing.
2. **`src/`:** safe Rust wrappers over the shim (the crate is not unsafe-allow-listed). Put the FFI declarations and calls in a small module guarded by `#![allow(unsafe_code)]`, with a reason line. If the `unsafe-allowlist` gate rejects that, report it so the orchestrator can add `tfar_golden_vectors` to the allow-list. Every `unsafe` block gets `// SAFETY:`.
3. **Comparison tests** (`tests/` integration tests of this crate, which may use our audio crates as normal dependencies):
   - **Filters:** every TFAR filter design (the table in `/documentation/program/agent_briefs/m4_p1_p2_filters_dynamics.md`) over white noise, a sine sweep and an impulse at 48 kHz. Compare against `tfar_dsp_filters` at RMS error ≤ 1e-4 relative to signal RMS, plus magnitude response at 64 frequencies within 0.1 dB in the passband and 1 dB elsewhere. P1 measured that their band-edge mapping matches DSPFilters' `BandPassTransform`, so expect a close match. Clipped samples may differ by 1–2 LSB (P1 finding: the original writes 32767/−32768 on clip, ours ±32766).
   - **Compressor:** `tfar_dynamics` presets 0.9 and 1.0 vs SimpleComp on speech-like test signals: an AM-modulated noise burst and steps. Gain-reduction curve within 1e-6.
   - **Spatial:** `DistanceCurve::tfar_voice` + `gain_at` vs F3DAudio's curve volume for distances 0..2× scaler; `EmitterCone::TFAR_VOICE.gain` vs F3DAudio's cone attenuation for 72 angles. Tolerance 1e-5.
   - Each mismatch gets a test named for the property it checks.
4. **No fixture files** are needed; the oracle runs live in the tests.

## Owned files
`tools/tfar_golden_vectors/**`. One root-manifest Edit allowed (for example `cc` into `[workspace.dependencies]`; re-read first), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/g1-*.log`)
fmt, clippy `--all-targets -D warnings`, test for the crate; `cargo xtask verify all` (report foreign violations only). Then
`cargo xtask verify reference-untouched` **must hold**, proving that nothing under `reference/` changed.

## Report additions
A results table per comparison (max error and the tolerance); the vendored FAudio commit; any real discrepancy in our crates, with numbers and a suggested fix (don't fix other crates yourself).

Budget: L (350k tokens). Stop there and report what is done and what is not.
