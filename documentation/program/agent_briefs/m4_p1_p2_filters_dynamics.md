**Status:** live

# M4 · P1 + P2 · `tfar_dsp_filters` and `tfar_dynamics`

You are agent P1, the DSP author for the TFAR Rust port. You own both crates in this prompt. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`
(including the audio-thread rules) and `/documentation/architecture/audio_pipeline.md`.

## Decision context (D21)
- Butterworth designs come from `sci-rs` (a scipy port), converted to second-order sections.
- RBJ coefficients come from `biquad`.
- All processing is f64 Direct Form II Transposed.
- The C++ originals are DSPFilters and simpleSource, under `reference/tfar_1_0/ts/include/`. A later agent (G1)
  compares your output against them through a C++ oracle, so keep the public API simple and deterministic.

## `tfar_dsp_filters` (crates/audio/tfar_dsp_filters)
- **Filter designs used by TFAR** (exact call sites: `/documentation/program/behaviour_specs/tfar_*_plugin.md` §3):

  | Design | Use |
  |---|---|
  | Butterworth low-pass, order 2 | vehicle muffling and object interception |
  | Butterworth low-pass, order 4 | can't-speak 100 Hz and underwater |
  | Butterworth band-pass, order 1 (centre 2000, width 1000) | speaker |
  | Butterworth band-pass, order 2 (centre 1850 / 1550 phone; centre 1000 / 400 underwater) | phone and underwater |
  | RBJ high-pass and low-pass with Q | radio effects |

  - Band-pass "centre + width" follows DSPFilters' `setup(order, sampleRate, centerFrequency, widthFrequency)` convention. Derive the band edges so the −3 dB points match DSPFilters, and document the mapping you chose. G1 measures it.
- **Types:**
  - `FilterDesign` (an enum of the designs above, with parameters).
  - `SecondOrderSections` (coefficients).
  - `ChannelFilter<const CHANNELS: usize>`, which holds the per-channel DF2T state and processes interleaved `f32`/`f64` frames in place.
- **Convenience:** `process_stereo_i16(buffer: &mut StereoBuffer, gain: f32)`, matching the original `processFilterStereo` (`reference/tfar_1_0/ts/src/helpers.hpp:152`): i16 → float via `tfar_sample_buffer`'s `INPUT_SCALE`, filter, × gain, back via `OUTPUT_SCALE` with clipping.
- **Mono:** a `MonoFilter` for the radio-effect chains.
- **Construction** (sci-rs design, ~ms) is allowed off the audio thread only; document that. `ChannelFilter::reset()` clears the state.
- Validate parameters: cutoff in (0, Nyquist) and a positive width. Bad parameters return `Error`.
- **Tests:**
  - DC and Nyquist gain of each low-pass / high-pass;
  - band-pass peak near the centre;
  - impulse response stability (decays);
  - multichannel independence;
  - a sci-rs `sosfilt` cross-check if sci-rs offers one;
  - the stereo i16 path with clipping.

## `tfar_dynamics` (crates/audio/tfar_dynamics)
- **Port of `SimpleComp`** (feed-forward compressor, ChunkWare SimpleSource 1.12, MIT-style permission notice; keep a credit line in the README). Sources: `reference/tfar_1_0/ts/include/simpleSource/SimpleComp*.{h,cpp,inl}` and `SimpleEnvelope.*`.
  - Stereo-linked: key = max(|l|, |r|) + DC_OFFSET 1e-25.
  - dB conversion via `20/ln10 · ln`.
  - `over = max(0, key_dB − threshold_dB)`.
  - Attack/release envelope with `coef = exp(−1000 / (ms · sample_rate))`, plus the envelope's "over" smoothing exactly as the `.inl` does.
  - Gain reduction `dB2lin(over · (ratio − 1))`.
- **Constructor:** `Compressor::new(CompressorSettings { sample_rate, threshold_db, ratio, attack_ms, release_ms })`, with the presets:
  - `CompressorSettings::VERSION_0_9`: threshold 65 dB, ratio 0.1, attack 1 ms, release 300 ms, 48 kHz.
  - `CompressorSettings::VERSION_1_0`: threshold 80 dB, the rest the same.
- **Input scale:** the original runs on raw i16 values as doubles, so the threshold is on that scale. Keep that and document it.
- **Methods:** `process_stereo_i16(&mut StereoBuffer)` and `process_frame(l: f64, r: f64) -> (f64, f64)`.
- **Tests:**
  - quiet input passes unchanged;
  - loud input is reduced by the expected steady-state amount;
  - attack/release time constants measured on step input;
  - a stereo-linked gain.

## Owned files
`crates/audio/tfar_dsp_filters/**`, `crates/audio/tfar_dynamics/**`. External crates come from `[workspace.dependencies]`;
report any missing features instead of editing the root manifest.

## Checks (each alone; `CARGO_TARGET_DIR=target-m4-p1`, `CARGO_BUILD_JOBS=4`; logs `target/agent-logs/p1-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the band-edge mapping you chose and its −3 dB check; the measured compressor attack and release.

Budget: M (250k tokens). Stop there and report what is done and what is not.
