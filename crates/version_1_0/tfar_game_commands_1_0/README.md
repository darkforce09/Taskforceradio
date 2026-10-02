# Game commands for 1.0

The `tfar_game_commands_1_0` crate: the 1.0 messages Arma sends the plugin, parsed into typed
commands. Used by `tfar_client_state_1_0`, `tfar_plugin_1_0`.

## Contents

```text
crates/version_1_0/tfar_game_commands_1_0/
├── Cargo.toml  the `tfar_game_commands_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The typed form of every 1.0 game message (`POS`, `TANGENT`, `FREQ`, `SETCFG`, `MISSIONEND`, data
frames and the rest), parsed with checked token counts. The code is not written yet: `src/lib.rs`
carries the module header and re-exports, `src/crate_role.rs` states the crate's role in one
constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_commands_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `GameCommand`: one variant per 1.0 message.
- `parse_game_command`: a message string to a `GameCommand` or an `Error`.

## Boundaries

- Depends on: `tfar_arma_values` (the parsers for Arma's field, array and position strings);
  `tfar_geometry` (positions, directions and distances); `tfar_plugin_settings_1_0` (the 1.0
  settings); `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_plugin_1_0`.
- Rules: `TANGENT` parses by its own token layout, never by shifted indices. Tier 2 in
  `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
