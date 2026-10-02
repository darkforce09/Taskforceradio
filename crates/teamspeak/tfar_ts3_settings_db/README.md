# TeamSpeak settings database

The `tfar_ts3_settings_db` crate: reads TeamSpeak's settings.db for the values the plugins need.
Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/teamspeak/tfar_ts3_settings_db/
├── Cargo.toml  the `tfar_ts3_settings_db` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The reader of TeamSpeak's settings.db SQLite file, for the hotkey and capture settings the plugins
look up. The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_settings_db  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `TeamSpeakSettings::open`: opens settings.db under TeamSpeak's config path read-only.
- typed lookups of the settings the plugins read.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `rusqlite`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: the database is never written. Tier 0 in `crates/teamspeak`: every workspace dependency has
  a lower tier and follows the edge matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
