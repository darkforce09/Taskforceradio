# Arma extension ABI

The `tfar_arma_extension_abi` crate: the `RVExtension` exports and the Win32 calls an Arma extension
DLL needs. Used by `tfar_extension_0_9`, `tfar_extension_1_0`.

## Contents

```text
crates/game_link/tfar_arma_extension_abi/
├── Cargo.toml  the `tfar_arma_extension_abi` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The Arma side of an extension DLL: the `RVExtension` and `RVExtensionVersion` exports, output writes
truncated to Arma's buffer size, and the focus query through `GetForegroundWindow`. The code is not
written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the
crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_arma_extension_abi  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `export_arma_extension!`: generates `RVExtension` and `RVExtensionVersion` for an extension app.
- `ArmaOutputBuffer`: writes a reply truncated to the size Arma passes, always NUL-terminated.
- `game_window_has_focus`: the `GetForegroundWindow` check behind the `FOCUS` frame.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `windows-sys`.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`.
- Rules: a reply never writes past the buffer Arma passes. Tier 0 in `crates/game_link`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
