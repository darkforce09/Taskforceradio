# Vendored F3DAudio

FAudio's F3DAudio, the open reimplementation of X3DAudio, vendored unchanged so the oracle can
compute the distance curve, cone and speaker matrix the 1.0 plugin got from X3DAudio on Windows.

## Contents

```text
tools/tfar_golden_vectors/oracle/
├── F3DAudio.c  FAudio's `src/F3DAudio.c`: `F3DAudioInitialize` and `F3DAudioCalculate`
├── F3DAudio.h  FAudio's `include/F3DAudio.h`: the X3DAudio-compatible structs and constants
└── LICENSE     FAudio's zlib licence, which covers both files
```

## How it works

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

## Boundaries

- Depends on: the C library (`math.h`, `string.h`) and `shim/FAudio_internal.h`.
- Used by: `tools/tfar_golden_vectors/shim/oracle_shim.cpp` (`oracle_f3daudio_calculate`), built
  by `tools/tfar_golden_vectors/build.rs`.
- Rules: the files stay byte-identical to the recorded commit; C stays in this folder and
  `shim/` (`cargo xtask verify language-bans`).
