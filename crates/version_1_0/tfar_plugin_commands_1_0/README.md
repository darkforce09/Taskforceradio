# Plugin commands for 1.0

The `tfar_plugin_commands_1_0` crate: the 1.0 plugin-to-plugin commands sent through TeamSpeak,
byte-compatible with the original. Used by `tfar_plugin_1_0`.

## Contents

```text
crates/version_1_0/tfar_plugin_commands_1_0/
├── Cargo.toml  the `tfar_plugin_commands_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 plugin commands exchanged between TeamSpeak clients and the metadata block, encoded byte for
byte like the original so Windows players still interoperate. The code is not written yet:
`src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's role
in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_commands_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `PluginCommand`: the 1.0 commands.
- encode and decode of each command and of the metadata block.

## Boundaries

- Depends on: `tfar_arma_values` (the parsers for Arma's field, array and position strings);
  `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: the encoded bytes match what the original 1.0 plugin sends. Tier 2 in `crates/version_1_0`:
  every workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
