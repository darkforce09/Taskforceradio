# Game link client

The `tfar_game_link_client` crate: the extension side of the game link: a lazy, time-bounded
localhost TCP client. Used by `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.

## Contents

```text
crates/game_link/tfar_game_link_client/
├── Cargo.toml  the `tfar_game_link_client` package (lib): tier 1, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The extension side of the game link: connects lazily with a connect timeout of at most 50 ms and a 1
s backoff, sends a sync request and waits at most 1000 ms, and returns the original game-facing
strings when the plugin is not there. The code is not written yet: `src/lib.rs` carries the module
header and re-exports, `src/crate_role.rs` states the crate's role in one constant, and
`src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_client  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `GameLinkClient::call`: one sync round trip with a bounded wait.
- `GameLinkClient::send_async` and `send_focus`.
- the game-facing replies `Not connected to TeamSpeak` and `Pipe not opened from TS plugin`.

## Boundaries

- Depends on: `tfar_game_link_protocol` (the game link frames and ports); `thiserror` for the error
  type.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
- Rules: a call never blocks the game for longer than the sync timeout. Tier 1 in
  `crates/game_link`: every workspace dependency has a lower tier and follows the edge matrix in
  `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
