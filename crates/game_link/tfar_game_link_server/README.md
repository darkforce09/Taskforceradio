# Game link server

The `tfar_game_link_server` crate: the plugin side of the game link: a localhost TCP listener that
answers the extension. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/game_link/tfar_game_link_server/
├── Cargo.toml  the `tfar_game_link_server` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The plugin side of the game link: listens on `127.0.0.1`, accepts the extension's connection, hands
each sync request to the plugin and writes its answer, and queues async messages in arrival order.
The code is not written yet: `src/lib.rs` carries the module header and re-exports,
`src/crate_role.rs` states the crate's role in one constant, and `src/error.rs` holds the
`thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_server  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `GameLinkServer::bind`: listens on the plugin's port.
- `GameLinkRequestHandler`: the trait the plugin implements to answer a sync request.
- an async message queue drained first in, first out.

## Boundaries

- Depends on: `tfar_game_link_protocol` (the game link frames and ports); `thiserror` for the error
  type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: async messages are delivered first in, first out. Tier 1 in `crates/game_link`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
