# TFAR 1.0 Arma extension

The `tfar_extension_1_0` crate: the `task_force_radio_pipe` Arma extension for the 1.0 mod, a
Windows x64 DLL run under Proton. Arma 3 loads it through `callExtension` under Proton.

## Contents

```text
apps/tfar_extension_1_0/
├── Cargo.toml  the `tfar_extension_1_0` package (cdylib + rlib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The Arma extension for the 1.0 mod: `~`-prefixed calls go async, data frames answer `NEEDCFG` or
`OK`, and `MISSIONEND` and `FOCUS` frames reach the 1.0 plugin over the game link. The code is not
written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the
crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_extension_1_0                                  # the host unit tests
cargo build -p tfar_extension_1_0 --target x86_64-pc-windows-gnu  # tfar_extension_1_0.dll under target/x86_64-pc-windows-gnu/debug/
```

## Configuration

It reads the `-tfdebug` launch switch, which selects the plugin's debug port 47821 instead of 47820.

## Exports

- Symbols: none yet, because the code is not written. The crate is for `RVExtension` and
  `RVExtensionVersion`, the two exports Arma resolves.
- Artifact: task_force_radio_pipe_x64.dll, which `cargo xtask dist` renames from
  tfar_extension_1_0.dll into target/dist/tfar_1_0/.
- Installed by: the owner, by hand, into the root folder of the matching mod; Arma (BattlEye off)
  then answers `"task_force_radio_pipe" callExtension` through it.

## Boundaries

- Depends on: `tfar_arma_extension_abi` (the Arma extension exports); `tfar_game_link_client` (the
  extension-side link client); `thiserror` for the error type.
- Used by: Arma 3 under Proton, which loads the DLL as `task_force_radio_pipe`;
  `tools/tfar_extension_probe`.
- Rules: async calls reach the plugin in the order the game made them. Tier 2 in `apps`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`). `unsafe` is allowed here for the export glue only
  (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
