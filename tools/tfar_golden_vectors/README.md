# Golden vectors

The `tfar_golden_vectors` crate: the original C++ audio code of the mods and FAudio's F3DAudio,
built as a live test oracle, and the comparison tests that hold `tfar_dsp_filters`,
`tfar_dynamics` and `tfar_spatial_audio` to it within stated tolerances.

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
| stereo `i16` filter path (`processFilterStereo`) | noise at gains 1 and 4 | 1 LSB, clipped samples included |
| compressor presets 0.9 and 1.0 | AM-modulated noise burst, level steps | gain within 1e-6; `i16` path exact |
| voice distance curve | 0 to twice the scaler, voice volumes 5 to 60 | 1e-5 |
| voice cone, and curve and cone together | 72 angles; a spiral round the emitter | 1e-5 |

`lib.rs` declares the private modules and re-exports their public items at the crate root;
`prelude.rs` re-exports the same items except `Error` and `Result`. The `oracles` module tree
holds everything that reaches the C++ code; `test_signals.rs` and `comparison_metrics.rs` are
pure Rust, so a comparison test builds its input with the first, runs it through an oracle and
through the crate under test, and judges the pair with the second. The noise comes from a seeded
xorshift64* generator, so every run and every thread sees the same samples. Each module attaches
its sibling test file in `tests/` with `#[path]`.

## Modules

### `oracle/`

The three files are byte-for-byte copies from <https://github.com/FNA-XNA/FAudio> at commit
`6839b88e304a046ae1a609ff14a085371e02a4e0` (tag `26.10`), taken from `src/F3DAudio.c`,
`include/F3DAudio.h` and `LICENSE`. They are not altered, so the zlib licence's marking rule
does not apply. `F3DAudio.c` also includes FAudio's `FAudio_internal.h`, which pulls in SDL; the
oracle does not vendor it and compiles against the stand-in in `tools/tfar_golden_vectors/shim/`
instead, which maps the few macros F3DAudio uses onto the C library and counts failed parameter
checks instead of asserting. `build.rs` compiles `F3DAudio.c` as C11 with `oracle/` and `shim/` on
the include path.

To update, copy the same three files from a newer FAudio commit, record the commit here, and run
`cargo test -p tfar_golden_vectors`.

### `shim/`

`oracle_shim.cpp` sets the originals up the way the plugins do and runs them over caller-owned
buffers:

- `oracle_filter_process_f32`, `oracle_filter_magnitudes` and `oracle_filter_process_stereo_i16`
  build a `Dsp::SimpleFilter<Design, Channels>` (DirectFormII state) for a Butterworth low-pass
  or band-pass of order 1 to 8 (the order is the template argument, as in `LowPass<2>` set up with
  order 2) or an RBJ low-pass or high-pass, as `reference/tfar_1_0/ts/src/clientData.hpp` and
  `RadioEffect.hpp` do; the `i16` path is `processFilterStereo` of `helpers.hpp`.
- `oracle_simple_comp_process` and `oracle_simple_comp_process_stereo_i16` run a
  `chunkware_simple::SimpleComp` set up in the order of `clientData.hpp`; the first also reads each
  frame's gain from a second, identical compressor fed the same stereo-linked key over unit
  samples; the second is `processCompressor` of `RadioEffect.hpp`.
- `oracle_f3daudio_calculate` runs `F3DAudioCalculate` for a stereo output and a one-channel
  emitter as `applyILD` in `helpers.cpp` sets them up, and returns both matrix coefficients,
  their sum, the emitter angle, the distance and the failed parameter checks.
- `oracle_shim_struct_sizes` reports the `sizeof` of each shared struct for the Rust layout test.

Every function checks its pointers, catches every C++ exception and returns a status: 0 for
success, 1 for an invalid argument, 2 for an exception. `FAudio_internal.h` replaces FAudio's own
header, which pulls in SDL: it maps the math, memset, min, max, clamp and alignment macros
F3DAudio uses onto the C library, and turns `FAudio_assert` into a call to
`oracle_shim_record_assertion`, which counts failures in a thread-local counter that
`oracle_f3daudio_calculate` resets and reports.

### `src/oracles/`

Each oracle checks its parameters with `parameter_checks.rs`, fills the `#[repr(C)]` struct
`oracle_bindings.rs` shares with `shim/oracle_shim.cpp`, and calls the matching safe function
there. `oracle_bindings.rs` is the only module that lifts `unsafe_code`: it checks every buffer
length, hands the shim pointers into live slices with the counts taken from them, and turns a
non-zero status into `Error::OracleRefused`. Every call builds the original object afresh (a
`SimpleFilter`, a `SimpleComp`, an F3DAudio handle) and drops it before returning, so calls are
independent and any test thread may make them; the shim's count of failed F3DAudio parameter
checks is thread-local. The layout test compares each shared struct's `size_of` with the
`sizeof` the shim reports.

```text
filter_oracle ─────┐
compressor_oracle ─┼──▶ oracle_bindings ──▶ shim/oracle_shim.cpp ──▶ DSPFilters, SimpleComp, F3DAudio
spatial_oracle ────┘
```

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
- `Error`, `Result`, `prelude`.

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

- [Windows dependencies](/documentation/behaviour_specs/windows_dependencies.md) — the
  portability notes for building the originals outside MSVC.
- [Audio pipeline](/documentation/architecture/audio_pipeline.md) — where the filters, the
  compressor and the gains sit in the voice path.
