# TeamSpeak host layer

The `tfar_ts3_host` crate: the safe layer over the TeamSpeak 3 plugin SDK (API 26): exports,
callbacks, function table, logging. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/teamspeak/tfar_ts3_host/
├── Cargo.toml  the `tfar_ts3_host` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The boundary to TeamSpeak: the `ts3plugin_*` export layer over `ts3plugin-sys`, a safe wrapper
around the `Ts3Functions` table, C-string ownership, and a `tracing` layer that forwards to
TeamSpeak's log. The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_host  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `export_teamspeak_plugin!`: generates every `ts3plugin_*` export a plugin app needs.
- `TeamSpeakPlugin`: the trait a plugin app implements for the callbacks it handles.
- `TeamSpeakFunctions`: safe calls into the host function table.
- a `tracing` layer that forwards to `logMessage` from a non-audio thread.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals:
  `ts3plugin-sys`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no panic or unwind crosses into TeamSpeak; every C string handed over stays alive for the
  call. Tier 0 in `crates/teamspeak`: every workspace dependency has a lower tier and follows the edge
  matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
