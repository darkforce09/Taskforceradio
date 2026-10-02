# Game link server source

The source of `tfar_game_link_server`: the server handle, the accept thread, the session threads, the
async worker and the state they share.

## Contents

```text
crates/game_link/tfar_game_link_server/src/
├── accept_loop.rs        the accept thread: loopback bind, retry of a busy port, a thread per connection
├── async_worker.rs       the async worker: FIFO delivery and the `MISSIONEND` generation check
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── game_link_handler.rs  `GameLinkHandler`, the plugin's callbacks
├── game_link_server.rs   `GameLinkServer`: start, needcfg, liveness accessors, shutdown
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── link_state.rs         the state every thread shares and the session swap
├── prelude.rs            the server, handler, config and `Flavour` names for glob import
├── server_config.rs      `ServerConfig` and the default periods
├── session_link.rs       the locked writing side of one session's socket
├── session_loop.rs       the session thread: handshake, frames, sync answers, heartbeat
└── tests/                unit tests of the config defaults and the `MISSIONEND` test
```

## How it works

`game_link_server.rs` builds the `link_state.rs` state, makes the first bind attempt and starts
`accept_loop.rs` and `async_worker.rs`. The accept loop polls a non-blocking listener and starts a
`session_loop.rs` thread per connection. After a valid `HELLO`, `LinkState::open_session` closes and
reports any session it replaces, then sends `HELLO` and `STATUS` through `session_link.rs` while
holding the write lock, so a `set_needcfg` push can never overtake them. The session thread reads
with a timeout no longer than 20 ms, so it sees the heartbeat time, a replacement and shutdown on
time. Each `ASYNC` is queued with the current mission generation; a `MISSIONEND` bumps the
generation first, so the worker skips every message queued before it.

## Public surface

- `GameLinkServer`, `GameLinkHandler`, `ServerConfig`, `DEFAULT_STATUS_INTERVAL`,
  `DEFAULT_BIND_RETRY_INTERVAL`, `DEFAULT_WRITE_TIMEOUT`, `DEFAULT_HELLO_TIMEOUT`, `Error`,
  `Result`, `CRATE_ROLE` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_game_link_protocol`, `crossbeam-channel`, `parking_lot`, `tracing`,
  `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no lock but the session-order lock is held across a handler call; `session_closed` always
  follows its `session_opened` and precedes the next one; production files hold no `unwrap`,
  `expect`, `panic` or indexing (clippy deny list); tests live in `tests/`
  (`cargo xtask verify test-placement`).
