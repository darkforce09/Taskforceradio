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

`GameLinkServer::start` binds `127.0.0.1` on each listen port, the normal one and, when
`debug_port` is set, the `-tfdebug` one, and starts one accept thread per port and the async
worker. Each port retries its own bind while it is busy. Every port feeds the one session slot, so
the newest valid `HELLO` on either port replaces the open session. Each accepted connection gets a
session thread that checks its `HELLO`, answers with `HELLO` and one `STATUS`, then reads frames:
`SYNC_REQ` is answered on that thread through the plugin's `GameLinkHandler`, `ASYNC` goes to the
worker's FIFO queue, `FOCUS` is forwarded, and `STATUS` goes out every 500 ms. The plugin's own
async messages join the same queue through `queue_async`, so a sync `POS` answer can schedule its
async apply behind the `ASYNC` frames already queued. The crate has no TeamSpeak dependency and no
game-connected timer: the plugin derives both from the handler callbacks, `is_session_up` and
`last_game_message`.

```text
extension ──TCP──> accept thread (one per port) ──> session thread ──SYNC_REQ──> handler.sync_request ──> SYNC_RESP
                                          │  ──ASYNC──> FIFO queue ──> async worker ──> handler.async_message
                                          │                 ^
                                          │                 └── GameLinkServer::queue_async (plugin)
                                          │  ──FOCUS──> handler.focus_changed
                                          └─ STATUS every 500 ms, and at once from set_needcfg
```

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_server  # unit tests and the loopback session tests (about 4 s)
```

## Configuration

`ServerConfig`, built by the plugin: `flavour`, `port` and `debug_port`
(`ServerConfig::for_plugin_on_both_ports` sets 47810 and 47811, or 47820 and 47821;
`ServerConfig::new` and `ServerConfig::for_plugin` listen on one port), `status_interval`
(500 ms), `bind_retry_interval` (5 s), `write_timeout` (1000 ms), `hello_timeout` (1000 ms) and
`initial_needcfg` (true for 1.0). The crate reads no setting of its own.

## Public surface

- `GameLinkServer`: `start`, `local_addr` (the normal port), `local_addrs` (every bound port),
  `set_needcfg`, `needcfg`, `queue_async`, `is_session_up`, `last_game_message`, `shutdown`;
  dropping it shuts it down too.
- `GameLinkHandler`: `sync_request`, `async_message`, `focus_changed`, `session_opened`,
  `session_closed`, implemented by the plugin.
- `ServerConfig` (`new`, `for_plugin`, `for_plugin_on_both_ports`, `with_debug_port`,
  `listen_ports`) and the `DEFAULT_*` periods; `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: `tfar_game_link_protocol` (the frames, the codec and the ports); `crossbeam-channel`
  for the async queue, `parking_lot` for the locks, `tracing` for log lines, `thiserror` for the error
  type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: binds loopback only; async messages run first in, first out, `queue_async` messages
  included, and `MISSIONEND` drops the queued ones before running itself
  (`tests/sync_and_async.rs`, `tests/queued_async.rs`); a busy port leaves the other one working and
  both ports share one session slot (`tests/listen_ports.rs`); a refused `HELLO` gets no reply and a
  malformed frame closes the session (`tests/handshake_and_sessions.rs`); shutdown joins every thread
  in well under 1 s (`tests/status_and_shutdown.rs`). Tier 1 in `crates/game_link`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the authoritative frame
  table, handshake, plugin semantics, timeouts and ports.
- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions the protocol replaces the transport of.
