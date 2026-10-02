# Oracle shim

The `extern "C"` glue between the Rust oracle wrappers and the original code: DSPFilters and
simpleSource from `reference/` and the vendored F3DAudio.

## Contents

```text
tools/tfar_golden_vectors/shim/
├── FAudio_internal.h  the stand-in for FAudio's internal header that `oracle/F3DAudio.c` includes
└── oracle_shim.cpp    one `extern "C"` function per oracle operation, over plain structs
```

## How it works

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

## Boundaries

- Depends on: DSPFilters and simpleSource in `reference/tfar_1_0/ts/include/` (through the
  `DspFilters/` alias headers `build.rs` writes), `tools/tfar_golden_vectors/oracle/F3DAudio.h`,
  and the C and C++ standard libraries.
- Used by: `tools/tfar_golden_vectors/src/oracles/oracle_bindings.rs`, built by
  `tools/tfar_golden_vectors/build.rs`.
- Rules: the struct layouts and status codes match `oracle_bindings.rs` (the layout test in
  `src/oracles/tests/oracle_bindings.rs`); no exception and no pointer outlives a call; C and C++
  stay in this folder and `oracle/` (`cargo xtask verify language-bans`).
