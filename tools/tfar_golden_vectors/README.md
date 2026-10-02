# Golden vectors

The `tfar_golden_vectors` crate: the original C++ audio code of the mods and FAudio's F3DAudio,
built as a live test oracle, and the comparison tests that hold `tfar_dsp_filters`,
`tfar_dynamics` and `tfar_spatial_audio` to it within stated tolerances.

## Contents

```text
tools/tfar_golden_vectors/
├── build.rs    compiles DSPFilters and simpleSource from `reference/`, F3DAudio and the shim with `cc`
├── Cargo.toml  the `tfar_golden_vectors` package (lib): tier 0, `cc` to build, the audio crates to test
├── oracle/     FAudio's F3DAudio, vendored unchanged with its zlib licence
├── shim/       the `extern "C"` glue over the originals and the stand-in FAudio internal header
├── src/        the safe oracle wrappers, the test signals and the error measures
└── tests/      the comparisons: filters, compressor, distance curve and cone against the oracle
```

## How it works

`build.rs` compiles the original DSPFilters sources (Biquad, Butterworth, Cascade, Design, Filter,
Param, PoleFilter, RBJ, RootFinder, State) and simpleSource's `SimpleComp` and `SimpleEnvelope`
in place from `reference/tfar_1_0/ts/include/`, the vendored `oracle/F3DAudio.c`, and
`shim/oracle_shim.cpp`, into two static libraries the crate links. The originals include
`"DspFilters/…"` while their folder is `dspfilters/`, so the build writes one alias header per
original header into `OUT_DIR/include_alias/DspFilters/`, each a single `#include` of the original
by its absolute path; `State.h` and `Filter.h` use `std::array` without including it, so every
C++ unit gets `-include array`. Nothing under `reference/` is written, and a missing `reference/`
stops the build with a message saying how to restore it.

The shim exposes one `extern "C"` function per oracle operation over plain structs and
caller-owned buffers, and catches every C++ exception. `src/oracles/oracle_bindings.rs` is the
one module that calls it; the filter, compressor and spatial oracles above it check parameters
and buffer shapes and offer safe methods. The integration tests in `tests/` run the same test
signals through the oracle and through the Rust crate and compare:

```text
test signal ──▶ OracleFilter / OracleCompressor / f3daudio_calculate ──▶ shim ──▶ original C/C++
           └──▶ tfar_dsp_filters / tfar_dynamics / tfar_spatial_audio
                         both outputs ──▶ relative RMS error, max difference, dB distance
```

| Comparison | Signals | Tolerance |
|---|---|---|
| every TFAR filter design, mono and stereo `f32` | white noise, sine sweep, impulse at 48 kHz | RMS error ≤ 1e-4 of the oracle's RMS |
| magnitude response | 64 log-spaced frequencies, 20 Hz to 23 kHz | 0.1 dB at or above −3 dB, 1 dB elsewhere |
| stereo `i16` filter path (`processFilterStereo`) | noise at gains 1 and 4 | 1 LSB, 2 LSB on clipped samples |
| compressor presets 0.9 and 1.0 | AM-modulated noise burst, level steps | gain within 1e-6; `i16` path exact |
| voice distance curve | 0 to twice the scaler, voice volumes 5 to 60 | 1e-5 |
| voice cone, and curve and cone together | 72 angles; a spiral round the emitter | 1e-5 |

## Getting started

Run from the repository root; a C and C++ compiler (`cc`, `c++`) must be installed:

```bash
cargo test -p tfar_golden_vectors                  # the unit tests and every comparison
cargo test -p tfar_golden_vectors -- --nocapture   # the same, printing each comparison's worst error
```

## Configuration

None: the crate reads no setting. The build reads `reference/tfar_1_0/ts/include/` at its fixed
place in the repository.

## Public surface

- `OracleFilter`, `OracleFilterDesign`, `ORACLE_MAX_BUTTERWORTH_ORDER`: DSPFilters' designs over
  mono and stereo `f32`, the stereo `i16` path and the magnitude response.
- `OracleCompressor`, `OracleCompressorSettings`, `OracleCompressorRun`: SimpleComp over stereo
  `f64` frames with each frame's gain, and over stereo `i16` blocks.
- `f3daudio_calculate`, `OracleSpatialScene`, `OracleCone`, `OracleSpatialResult`,
  `ORACLE_MAX_CURVE_POINTS`: F3DAudio's matrix, volume and failed parameter checks for a scene.
- The test signals (`white_noise`, `sine_sweep`, `impulse`, `modulated_noise_burst`,
  `level_steps`, `log_spaced_frequencies`, `NoiseGenerator`, `SAMPLE_RATE_HZ`) and the error
  measures (`root_mean_square`, `relative_rms_error`, `max_absolute_difference`, `decibels`,
  `decibel_difference`).
- `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: no workspace crate; `thiserror`; `cc` at build time; the originals under
  `reference/tfar_1_0/ts/include/` and a host C and C++ compiler. The tests use `tfar_dsp_filters`,
  `tfar_dynamics`, `tfar_spatial_audio`, `tfar_sample_buffer` and `tfar_geometry` as
  dev-dependencies.
- Used by: nothing; its own tests are the comparisons.
- Rules: the oracle is built only here, never into a deliverable; `reference/` is read in place
  and never written (`cargo xtask verify reference-untouched`); C and C++ live only in `oracle/`
  and `shim/` (`cargo xtask verify language-bans`); every `unsafe` call sits in
  `src/oracles/oracle_bindings.rs` with a `// SAFETY:` comment; every comparison names its
  tolerance, and each tolerance has a perturbation that breaks it. Tier 0 in `tools`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the
  portability notes for building the originals outside MSVC.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the filters, the
  compressor and the gains sit in the voice path.
