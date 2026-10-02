# Game link server

The `tfar_game_link_server` crate: the plugin side of the game link: a localhost TCP listener that
answers the extension. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/game_link/tfar_game_link_server/
├── Cargo.toml  the `tfar_game_link_server` package (lib): tier 1, workspace lints and dependencies
├── src/        the server handle, its threads, the session handling and the async queue
└── tests/      loopback integration tests with a raw-socket extension stand-in
```

## How it works

`GameLinkServer::start` binds `127.0.0.1:<port>` and starts two threads: the accept thread and the
async worker. Each accepted connection gets a session thread that checks its `HELLO`, answers with
`HELLO` and one `STATUS`, then reads frames: `SYNC_REQ` is answered on that thread through the
plugin's `GameLinkHandler`, `ASYNC` goes to the worker's FIFO queue, `FOCUS` is forwarded, and
`STATUS` goes out every 500 ms. The crate has no TeamSpeak dependency and no game-connected timer:
the plugin derives both from the handler callbacks, `is_session_up` and `last_game_message`.

```text
extension ──TCP──> accept thread ──> session thread ──SYNC_REQ──> handler.sync_request ──> SYNC_RESP
                                          │  ──ASYNC──> FIFO queue ──> async worker ──> handler.async_message
                                          │  ──FOCUS──> handler.focus_changed
                                          └─ STATUS every 500 ms, and at once from set_needcfg
```

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_server  # unit tests and the loopback session tests (about 3 s)
```

## Configuration

`ServerConfig`, built by the plugin: `flavour`, `port` (`ServerConfig::for_plugin` picks 47810 or
47820, or the `-tfdebug` port), `status_interval` (500 ms), `bind_retry_interval` (5 s),
`write_timeout` (1000 ms), `hello_timeout` (1000 ms) and `initial_needcfg` (true for 1.0). The crate
reads no setting of its own.

## Public surface

- `GameLinkServer`: `start`, `local_addr`, `set_needcfg`, `needcfg`, `is_session_up`,
  `last_game_message`, `shutdown`; dropping it shuts it down too.
- `GameLinkHandler`: `sync_request`, `async_message`, `focus_changed`, `session_opened`,
  `session_closed`, implemented by the plugin.
- `ServerConfig` and the `DEFAULT_*` periods; `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_game_link_protocol` (the frames, the codec and the ports); `crossbeam-channel`
  for the async queue, `parking_lot` for the locks, `tracing` for log lines, `thiserror` for the error
  type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: binds loopback only; async messages run first in, first out, and `MISSIONEND` drops the
  queued ones before running itself (`tests/sync_and_async.rs`); a refused `HELLO` gets no reply and a
  malformed frame closes the session (`tests/handshake_and_sessions.rs`); shutdown joins every thread
  in well under 1 s (`tests/status_and_shutdown.rs`). Tier 1 in `crates/game_link`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the authoritative frame
  table, handshake, plugin semantics, timeouts and ports.
- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions the protocol replaces the transport of.
