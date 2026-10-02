**Status:** live

# M3 · L2 · `tfar_arma_extension_abi` and `tfar_game_link_client`

You are agent L2, the extension-side author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`.
Then read `/documentation/architecture/game_link_protocol.md` (authoritative) and
`/documentation/program/behaviour_specs/extensions.md`.

## `tfar_arma_extension_abi` (crates/game_link/tfar_arma_extension_abi)
This is one of the unsafe-allow-listed crates. Every `unsafe` block gets a `// SAFETY:` comment.
- **Export macro.** `arma_extension!(handler: fn(&str) -> String, version: &'static str)` expands in the app crate to:
  - `#[unsafe(no_mangle)] pub unsafe extern "system" fn RVExtension(output: *mut c_char, output_size: c_int, function: *const c_char)`
  - `RVExtensionVersion(output, output_size)`

  The expansion:
  - checks null and non-positive sizes;
  - reads the input as bytes up to the NUL and converts it lossily to `&str`, leaving the bytes untouched where already valid;
  - calls the handler;
  - writes the reply with `write_reply`.

  On x86_64 Windows `extern "system"` is the C ABI, so the export is undecorated. Document that no 32-bit build is supported.
- **`write_reply(output: &mut [u8], reply: &str)`:** a safe, testable core. It truncates to `len − 1` bytes, cuts back to a UTF-8 character boundary, then writes the NUL. An empty buffer writes nothing.
- **`arma_command_line_contains(flag: &str) -> bool`:** `GetCommandLineW` on Windows, `std::env::args` elsewhere. Used for `-tfdebug`.
- **`foreground_window_is_arma() -> bool`:** `GetForegroundWindow` + `GetClassNameW`; true when the class name contains `arma`, case-insensitive (the original's test at `reference/tfar_1_0/ts/src/Teamspeak.cpp:207`). Non-Windows returns `false`.
- Use `windows-sys` from the workspace dependencies with exactly the features you need, gated `[target.'cfg(windows)'.dependencies]`.
- **Tests** run on Linux: `write_reply` truncation at 0, 1, exact and multi-byte boundaries; the macro expanded in a test module and called through the raw function pointer with a Rust-allocated buffer.

## `tfar_game_link_client` (crates/game_link/tfar_game_link_client)
The extension side, std-only, used from Arma's calling thread. No background thread is needed except a
reader thread per session.
- **`GameLinkClient::new(ClientConfig { flavour, port, connect_timeout: 50 ms, hello_timeout: 250 ms, backoff: 1 s, sync_timeout: 1000 ms, session_dead_after: 1000 ms, write_timeout: 1000 ms })`.**
- **Connection:** lazy connect on the first call that needs it; the backoff applies after a failure or a drop. A reader thread decodes plugin frames, stores the latest `STATUS` needcfg and the last-frame time in atomics, and passes `SYNC_RESP` frames through a channel keyed by id (older ids discarded).
- **Results:**
  - `sync_request(text) -> SyncOutcome { Reply(Vec<u8>), NoSession, ClosedDuringCall, TimedOut }`.
  - `send_async(text) -> Result<()>`.
  - `send_focus(bool)`, `needcfg() -> bool`, `is_live()`.

  The **flavour-specific string mapping does not live here**: it lives in the two extension apps (L3), so this crate stays policy-free.
- **Tests** run against a real loopback listener written inside the tests (raw sockets plus `tfar_game_link_protocol`'s codec):
  - connect, handshake, sync;
  - timeout under 1.1 s;
  - closed during a call;
  - backoff (a second call within 1 s makes no new connect attempt; count accepts);
  - a stale id discarded;
  - the needcfg cache;
  - the dead-session detection;
  - a non-listening port failing within the connect timeout.

## Owned files
`crates/game_link/tfar_arma_extension_abi/**`, `crates/game_link/tfar_game_link_client/**`. Report needed root
manifest changes; don't edit the root manifest.

## Checks (each alone; `CARGO_TARGET_DIR=target-m3-l2`, `CARGO_BUILD_JOBS=4`; logs `target/agent-logs/l2-*.log`)
`cargo fmt` on both; `cargo clippy` on both `--all-targets -- -D warnings`; `cargo test` on both; `cargo clippy -p tfar_arma_extension_abi -p tfar_game_link_client --target x86_64-pc-windows-gnu -- -D warnings`;
`cargo build -p tfar_arma_extension_abi --target x86_64-pc-windows-gnu`; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; test names per spec rule.

Budget: M (250k tokens). Stop there and report what is done and what is not.
