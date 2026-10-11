# Game link

The localhost TCP link between an Arma extension (a Windows DLL under Proton) and its TeamSpeak
plugin (a native `.so`): `protocol` defines the frames both sides exchange, the plugins run
`server`, and the extensions run `client`. The 0.9 plugin listens on port 47810, the 1.0 plugin on
47820.

```text
Arma (Proton) -- callExtension --> extension DLL -- TCP 127.0.0.1 --> TeamSpeak plugin (.so)
                                   client                              server
```

```bash
cargo test -p tfar_game_link
cargo clippy -p tfar_game_link --target x86_64-pc-windows-gnu -- -D warnings
```

Boundaries: depends on no workspace crate; used by all four deliverables, the version libraries
(the frame types) and `tools/tfar_ts3_mock`. `postcard` stays in the game_link category
(`cargo xtask verify dependency-firewalls`).

## `protocol`

The `tfar_game_link::protocol` crate: the frames the Arma extensions and the TeamSpeak plugins
exchange over localhost TCP. Used by `tfar_game_link::client`, `tfar_game_link::server`,
`tfar_ts3_mock`.

### How it works

A frame on the wire is a `u32` little-endian length, a kind byte and the `postcard` encoding of the
kind's fields in table order. `src/` holds the in-memory `Frame`, the encoder, a decoder for whole
bodies, a streaming `FrameDecoder` for sockets that deliver bytes in arbitrary chunks, and blocking
`read_frame` and `write_frame` helpers. Every malformed input (a zero or oversized length, an unknown
kind, a payload that does not decode exactly) is an `Error`, after which the receiver closes the
connection.

`frame_codec.rs` writes a placeholder length, the kind byte from `frame.rs` and the `postcard`
payload, then fills in the length, refusing a body above `MAX_FRAME_LENGTH`. Decoding checks the
length first (`frame_body_length`), then maps the kind byte and decodes the payload, refusing bytes
left over. `frame_decoder.rs` buffers chunks and runs the same two steps once a whole frame is in;
`frame_io.rs` reads one frame with `read_exact`, allocating the body only after its length passed.
Text fields are byte strings (`postcard` writes a varint length, then the bytes).

### Configuration

None: the crate reads no setting.

### Public surface

- `Frame` (`Hello`, `SyncRequest`, `SyncResponse`, `Async`, `Focus`, `Status`), `FrameKind`,
  `Hello` and `HelloMismatch`: the frames and the handshake check.
- `encode`, `decode_frame_body`, `frame_body_length`, `FrameDecoder`, `read_frame`, `write_frame`:
  the codec.
- `Flavour` (`V0_9 = 9`, `V1_0 = 10`), `port(flavour, debug)` and the four port constants: 47810 and
  47811 for 0.9, 47820 and 47821 for 1.0.
- `MAGIC`, `PROTOCOL_VERSION`, `MAX_FRAME_LENGTH`, `FRAME_LENGTH_PREFIX_SIZE`, `Error`, `Result`,
  `prelude`.

### Boundaries

- Depends on: no workspace crate; `postcard` (`use-std`) for the payloads, `serde` (`derive`) for
  the `HELLO` payload, `thiserror` for the error type.
- Used by: `tfar_game_link::client`, `tfar_game_link::server`, `tfar_ts3_mock`.
- Rules: every frame that encodes decodes to itself, and the `HELLO` and
  `SYNC_REQ{1, "TS_INFO\tPING"}` bytes match their golden vectors (`src/tests/frame_codec.rs`); a
  length of zero or above 1 MiB is refused before the body is read.

## `server`

The `tfar_game_link::server` crate: the plugin side of the game link: a localhost TCP listener that
answers the extension. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

### How it works

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

`game_link_server.rs` builds the `link_state.rs` state, makes the first bind attempt on each listen
port and starts one `accept_loop.rs` thread per port and `async_worker.rs`. Each accept loop polls
its own non-blocking listener, retries its own bind while the port is busy, and starts a
`session_loop.rs` thread per connection. After a valid `HELLO`, `LinkState::open_session` closes and
reports any session it replaces, then sends `HELLO` and `STATUS` through `session_link.rs` while
holding the write lock, so a `set_needcfg` push can never overtake them. The session thread reads
with a timeout no longer than 20 ms, so it sees the heartbeat time, a replacement and shutdown on
time. Each `ASYNC`, and each `GameLinkServer::queue_async` text, is queued through
`LinkState::queue_async_text` with the current mission generation; a `MISSIONEND` bumps the
generation first, so the worker skips every message queued before it.

### Configuration

`ServerConfig`, built by the plugin: `flavour`, `port` and `debug_port`
(`ServerConfig::for_plugin_on_both_ports` sets 47810 and 47811, or 47820 and 47821;
`ServerConfig::new` and `ServerConfig::for_plugin` listen on one port), `status_interval`
(500 ms), `bind_retry_interval` (5 s), `write_timeout` (1000 ms), `hello_timeout` (1000 ms) and
`initial_needcfg` (true for 1.0). The crate reads no setting of its own.

### Public surface

- `GameLinkServer`: `start`, `local_addr` (the normal port), `local_addrs` (every bound port),
  `set_needcfg`, `needcfg`, `queue_async`, `is_session_up`, `last_game_message`, `shutdown`;
  dropping it shuts it down too.
- `GameLinkHandler`: `sync_request`, `async_message`, `focus_changed`, `session_opened`,
  `session_closed`, implemented by the plugin.
- `ServerConfig` (`new`, `for_plugin`, `for_plugin_on_both_ports`, `with_debug_port`,
  `listen_ports`) and the `DEFAULT_*` periods; `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_game_link::protocol` (the frames, the codec and the ports); `crossbeam-channel`
  for the async queue, `parking_lot` for the locks, `tracing` for log lines, `thiserror` for the error
  type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: binds loopback only; async messages run first in, first out, `queue_async` messages
  included, and `MISSIONEND` drops the queued ones before running itself
  (`tests/sync_and_async.rs`, `tests/queued_async.rs`); a busy port leaves the other one working and
  both ports share one session slot (`tests/listen_ports.rs`); a refused `HELLO` gets no reply and a
  malformed frame closes the session (`tests/handshake_and_sessions.rs`); shutdown joins every thread
  in well under 1 s (`tests/status_and_shutdown.rs`).

## `client`

The `tfar_game_link::client` crate: the extension side of the game link: a lazy, time-bounded
localhost TCP client. Used by `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.

### How it works

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

### Configuration

None: the crate reads no setting. `ClientConfig` carries the port and the timeouts its caller passes.

### Public surface

- `GameLinkClient`: `new`, `config`, `sync_request`, `send_async`, `send_focus`, `needcfg`, `is_live`.
- `ClientConfig`: the flavour, the port and the six timeouts, with the spec's values as constants.
- `SyncOutcome`: `Reply(Vec<u8>)`, `NoSession`, `ClosedDuringCall`, `TimedOut`.
- `Error`, `Result` and `prelude` (which also re-exports `Flavour` and `port`).

### Boundaries

- Depends on: `tfar_game_link::protocol` (the frames, the `HELLO` check and the frame decoder);
  `thiserror` for the error type; `tracing` for the close and connect log lines.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
- Rules: a call blocks at most for the connect, `HELLO` and sync timeouts together, and no connect
  is attempted during the backoff (`tests/game_link_client_sessions.rs`).
