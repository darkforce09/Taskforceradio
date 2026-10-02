**Status:** live

# M3 · L1 · `tfar_game_link_protocol` and `tfar_game_link_server`

You are agent L1, the game-link server author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`.
Then read **`/documentation/architecture/game_link_protocol.md`, which is the authoritative spec**. Every
number, kind code, timeout and rule in it is binding.

## `tfar_game_link_protocol` (crates/game_link/tfar_game_link_protocol)
- **Types:** `Flavour { V0_9 = 9, V1_0 = 10 }`, plus `Frame`, an enum with `Hello`, `SyncRequest { id, text }`, `SyncResponse { id, text }`, `Async { text }`, `Focus { arma_focused }` and `Status { needcfg }`. Text fields are `Vec<u8>`.
- **Constants:** `MAGIC`, `PROTOCOL_VERSION = 1`, `MAX_FRAME_LENGTH = 1_048_576`, and the ports via `fn port(flavour, debug: bool) -> u16`.
- **Codec:** a `u32` LE length, then the `u8` kind, then the postcard payload.
  - `encode(&Frame, &mut Vec<u8>)`.
  - A streaming `FrameDecoder` that takes arbitrary byte chunks and yields complete frames. Errors: zero length, an over-length frame, an unknown kind, a bad payload.
  - `read_frame(&mut impl Read)` / `write_frame(&mut impl Write)` helpers.
  - Add `postcard` (with `use-std`) and `serde` (`derive`) from the workspace dependencies.
- **Tests:**
  - a round trip for every kind;
  - a decoder fed one byte at a time;
  - each error class;
  - a golden byte vector for `Hello` and `SyncRequest{1,"TS_INFO\tPING"}`, so wire drift fails a test.

## `tfar_game_link_server` (crates/game_link/tfar_game_link_server)
The plugin side, std-only (threads + `std::net`). There is no TeamSpeak dependency: the plugin passes in
handlers.
- **API:**
  - `GameLinkServer::start(config: ServerConfig, handler: Arc<dyn GameLinkHandler>) -> Result<GameLinkServer>`.
  - `ServerConfig { flavour, port, status_interval: 500 ms, … }`.
  - `trait GameLinkHandler: Send + Sync` with:
    - `fn sync_request(&self, text: &[u8]) -> Vec<u8>`
    - `fn async_message(&self, text: &[u8])`
    - `fn focus_changed(&self, focused: bool)`
    - `fn session_opened(&self)`
    - `fn session_closed(&self)`
  - `GameLinkServer::set_needcfg(bool)`: sends `STATUS` at once when the value changes.
  - `GameLinkServer::shutdown(self)`: stops and joins every thread. It must finish within 1 s, so the TeamSpeak plugin can unload cleanly.
- **Threads:**
  - **Accept thread:** binds `127.0.0.1:<port>` only. A busy port is logged with `tracing`, then the bind is retried every 5 s.
  - **Session thread:** handshake, frame reading, sync answers on this thread, heartbeat `STATUS` every 500 ms.
  - **Async worker:** one thread over a `crossbeam-channel`, strictly FIFO (F-007).
- **MISSIONEND:** an async text whose command name is `MISSIONEND` empties the not-yet-processed queue, then is delivered itself. Implement this with a generation counter so the worker can skip stale items.
- **Session handling:**
  - A new valid HELLO replaces the previous session.
  - A mismatched magic, protocol or flavour closes the connection without a reply, with a log line.
  - Frames over the limit or that fail to decode close the session.
  - `TCP_NODELAY`; a write timeout of 1000 ms.
- **Liveness:** game-connected timing (pluginTimeout and the debounce) is **not** this crate's job; the plugin derives it from the callbacks. This crate exposes `last_game_message() -> Option<Instant>`, where HELLO and FOCUS don't count and MISSIONEND resets it to `None`, plus `is_session_up()`.
- **Tests** (real loopback sockets on ephemeral ports via `ServerConfig.port = 0`, with `local_addr()` exposed):
  - the handshake ok and each mismatch;
  - a sync round trip;
  - async FIFO order across 1000 messages;
  - MISSIONEND dropping queued items;
  - session replacement;
  - the heartbeat cadence;
  - the needcfg push;
  - a malformed frame closing the session;
  - shutdown finishing under 1 s.

  Write a tiny raw-socket test client inside the tests. Don't depend on `tfar_game_link_client`.

## Owned files
`crates/game_link/tfar_game_link_protocol/**`, `crates/game_link/tfar_game_link_server/**`. If the root
`[workspace.dependencies]` lacks a needed feature, report it and do not edit the root manifest.

## Checks (each alone; `CARGO_TARGET_DIR=target-m3-l1`, `CARGO_BUILD_JOBS=4`; logs `target/agent-logs/l1-*.log`)
`cargo fmt` on both crates; `cargo clippy -p tfar_game_link_protocol -p tfar_game_link_server --all-targets -- -D warnings`;
`cargo test -p tfar_game_link_protocol -p tfar_game_link_server`; `cargo clippy -p tfar_game_link_protocol --target x86_64-pc-windows-gnu -- -D warnings`;
`cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the test names per spec rule; the measured shutdown time.

Budget: M (250k tokens). Stop there and report what is done and what is not.
