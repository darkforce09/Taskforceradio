# Plugin commands for 0.9

The `tfar_plugin_commands_0_9` crate: the 0.9.12 plugin-to-plugin commands sent through TeamSpeak,
byte-compatible with the original. Used by `tfar_plugin_0_9`.

## Contents

```text
crates/version_0_9/tfar_plugin_commands_0_9/
├── Cargo.toml  the `tfar_plugin_commands_0_9` package (lib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 0.9.12 plugin commands exchanged between TeamSpeak clients and the metadata block, encoded byte
for byte like the original so Windows players still interoperate. The code is not written yet:
`src/lib.rs` carries the module header and re-exports, `src/crate_role.rs` states the crate's role
in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_commands_0_9  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `PluginCommand`: the 0.9.12 commands (`CURRENT_CHANNEL` and the rest).
- encode and decode of each command and of the metadata block.

## Boundaries

- Depends on: `tfar_arma_values` (the parsers for Arma's field, array and position strings);
  `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`.
- Rules: the encoded bytes match what the original 0.9.12 plugin sends. Tier 2 in
  `crates/version_0_9`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
