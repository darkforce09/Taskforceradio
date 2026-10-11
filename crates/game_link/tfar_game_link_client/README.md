# Game link client

The `tfar_game_link_client` crate: the extension side of the game link: a lazy, time-bounded
localhost TCP client. Used by `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.

## How it works

`GameLinkClient::new(ClientConfig::new(flavour, port))` connects nothing. The first call that needs a
session connects to `127.0.0.1:<port>` within the connect timeout (50 ms), sends `HELLO`, and waits
up to the `HELLO` timeout (250 ms) for the plugin's matching `HELLO`. A reader thread per session then
decodes the plugin's frames: `STATUS` updates the needcfg flag, every frame refreshes the last-frame
time, and `SYNC_RESP` frames go through a channel to the waiting call, which discards any id but its
own.

```text
Arma thread ── sync_request ──► GameLinkClient ──SYNC_REQ──► plugin
     ▲                              │  (mutex)                  │
     └──── SyncOutcome ◄── channel ◄┴── reader thread ◄─SYNC_RESP┘
                                         ├─ STATUS → needcfg atomic
                                         └─ any frame → last-frame time
```

`sync_request` returns `Reply`, `NoSession`, `ClosedDuringCall` or `TimedOut` (sync timeout 1000
ms); `send_async` and `send_focus` return a `Result`. A failed connect or handshake and every dropped
session (closed, quiet for 1000 ms, or a failed write) start a 1 s backoff in which calls fail at once
without a connect attempt. The latest focus value is sent again after each new handshake. Which
string Arma sees for each outcome is the extension app's choice: this crate holds none.

`game_link_client.rs` keeps the session, the backoff deadline and the latest focus value behind one
mutex and opens a `Session` (`session.rs`) on demand. `Session::open` connects, completes the
`HELLO` exchange and hands the socket's second handle, with any bytes already buffered after
`HELLO`, to the reader thread (`session_reader.rs`). The reader writes the shared atomics and the
response channel; it marks the session closed before its channel sender drops, so a disconnected
channel always means a closed session. Dropping a `Session` asks the reader to stop and shuts the
socket down; the reader polls every 100 ms, so it ends soon after.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_link_client  # the unit tests and the loopback scenarios
```

## Configuration

None: the crate reads no setting. `ClientConfig` carries the port and the timeouts its caller passes.

## Public surface

- `GameLinkClient`: `new`, `config`, `sync_request`, `send_async`, `send_focus`, `needcfg`, `is_live`.
- `ClientConfig`: the flavour, the port and the six timeouts, with the spec's values as constants.
- `SyncOutcome`: `Reply(Vec<u8>)`, `NoSession`, `ClosedDuringCall`, `TimedOut`.
- `Error`, `Result` and `prelude` (which also re-exports `Flavour` and `port`).

## Boundaries

- Depends on: `tfar_game_link_protocol` (the frames, the `HELLO` check and the frame decoder);
  `thiserror` for the error type; `tracing` for the close and connect log lines.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
- Rules: a call blocks at most for the connect, `HELLO` and sync timeouts together, and no connect
  is attempted during the backoff (`tests/game_link_client_sessions.rs`). Tier 1 in
  `crates/game_link`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the frames, the
  handshake, the timeouts and the extension semantics this client implements.
- [Arma extensions](/documentation/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
