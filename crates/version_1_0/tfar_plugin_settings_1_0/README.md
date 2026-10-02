# Plugin settings for 1.0

The `tfar_plugin_settings_1_0` crate: the 1.0 plugin's settings: the values `SETCFG` sets and the
defaults. Used by `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`,
`tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.

## Contents

```text
crates/version_1_0/tfar_plugin_settings_1_0/
├── Cargo.toml  the `tfar_plugin_settings_1_0` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 plugin's settings model: every value the game sets with `SETCFG`, its default and its type.
The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_plugin_settings_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `PluginSettings`: every setting with its default.
- `apply_setting`: one `SETCFG` key and value applied.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`,
  `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
- Rules: an unknown key or a malformed value is an `Error` and leaves the settings unchanged. Tier 0
  in `crates/version_1_0`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
