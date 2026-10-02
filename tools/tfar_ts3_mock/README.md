# Mock TeamSpeak host

The `tfar_ts3_mock` crate: a fake TeamSpeak host that loads a plugin .so and drives it through
scripted scenarios. The plugin apps' test suites and developers run it.

## Contents

```text
tools/tfar_ts3_mock/
├── Cargo.toml  the `tfar_ts3_mock` package (lib + bin): tier 2, workspace lints and dependencies
└── src/        the library root, the binary's `main`, the role constant, the error type and the prelude
```

## How it works

The test host for the plugins: loads a plugin `.so` with `libloading`, hands it a fake
`Ts3Functions` table, plays the game side over the game link and runs scripted scenarios. The code
is not written yet: `src/lib.rs` carries the module header and re-exports, `src/crate_role.rs`
states the crate's role in one constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_ts3_mock  # the unit tests
cargo run -p tfar_ts3_mock   # prints what the mock host is for
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- the library `tfar_ts3_mock`: `MockTeamSpeakHost` and the scenario runner, for the plugin apps'
  test suites.
- the binary `tfar_ts3_mock`: runs one scenario against a built plugin.

## Boundaries

- Depends on: `tfar_game_link_protocol` (the game link frames and ports); `tfar_game_link_client`
  (the extension-side link client); `thiserror` for the error type. Planned externals: `libloading`,
  `ts3plugin-sys`.
- Used by: the plugin apps' test suites and developers at the command line.
- Rules: a scenario's outcome depends only on the scenario and the plugin. Tier 2 in `tools`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
