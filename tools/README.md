# Tools

The repository tooling: the xtask runner, the mock TeamSpeak host, the Arma extension probe and the
golden-vector oracle. Nothing here ships to a player.

## Contents

```text
tools/
├── tfar_extension_probe/  a Windows executable that loads an extension DLL and calls it, crate `tfar_extension_probe`
├── tfar_golden_vectors/   the original C++ audio code as a live test oracle for the audio crates, crate `tfar_golden_vectors`
├── tfar_ts3_mock/         a fake TeamSpeak host that loads a plugin and runs scenarios, crate `tfar_ts3_mock`
└── xtask/                 the repository tasks: verify gates, CI lane, dist, new-crate, reference manifest
```

## How it works

`xtask` runs every gate and repository task as Rust, through the `cargo xtask` alias. The mock host
loads a built plugin and plays both TeamSpeak and the game side of the link, so plugin behaviour is
tested without TeamSpeak or Arma. The probe stands in for Arma under Proton's wine. The golden
vectors build the original C++ DSP code as an oracle, the only C/C++ the repository compiles.

## Getting started

Run from the repository root:

```bash
cargo build -p tfar_extension_probe --target x86_64-pc-windows-gnu   # tfar_extension_probe.exe
cargo run -p tfar_ts3_mock                                           # prints what the mock host is for
cargo test -p tfar_ts3_mock -p tfar_golden_vectors -p tfar_extension_probe
```

## Boundaries

- Depends on: the library crates in `crates/`; the built deliverables at run time (the mock loads
  a plugin `.so`, the probe loads an extension DLL).
- Used by: developers and the gate runner.
- Rules: tools may depend on library crates under `crates/`, never on the four deliverables
  (`cargo xtask verify crate-tiers`); C/C++ appears only under `tfar_golden_vectors/`
  (`cargo xtask verify language-bans`); `cc` stays in `tfar_golden_vectors` and `libloading` in
  `tfar_ts3_mock`.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the crates the tools use.
- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the ABI the probe calls.
