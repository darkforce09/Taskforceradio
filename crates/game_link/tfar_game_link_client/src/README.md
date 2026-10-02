# Game link client source

The source of `tfar_game_link_client`: the client, one session with its reader thread, the config,
the sync outcome and the error type.

## Contents

```text
crates/game_link/tfar_game_link_client/src/
├── client_config.rs     `ClientConfig`: flavour, port and the spec's timeouts
├── crate_role.rs        `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs             `Error`, the `thiserror` enum, and `Result`
├── game_link_client.rs  `GameLinkClient`: lazy connect, backoff, sync, async, focus, needcfg, liveness
├── lib.rs               the crate root: module header, `mod` lines and re-exports
├── prelude.rs           the client, the config, the outcome, `Flavour` and `port` for glob import
├── session.rs           one session: connect, handshake, writes, the wait for one response id
├── session_reader.rs    the reader thread: decodes plugin frames, publishes needcfg, time, responses
├── sync_outcome.rs      `SyncOutcome`: how one sync request ended
└── tests/               unit tests of the config, the role constant and the plugin-free client paths
```

## How it works

`game_link_client.rs` keeps the session, the backoff deadline and the latest focus value behind one
mutex and opens a `Session` (`session.rs`) on demand. `Session::open` connects, completes the
`HELLO` exchange and hands the socket's second handle, with any bytes already buffered after
`HELLO`, to the reader thread (`session_reader.rs`). The reader writes the shared atomics and the
response channel; it marks the session closed before its channel sender drops, so a disconnected
channel always means a closed session. Dropping a `Session` asks the reader to stop and shuts the
socket down; the reader polls every 100 ms, so it ends soon after.

## Public surface

- `GameLinkClient`, `ClientConfig`, `SyncOutcome`, `CRATE_ROLE`, `Error`, `Result` and `prelude`,
  re-exported at the crate root.

## Boundaries

- Depends on: `tfar_game_link_protocol`, `thiserror`, `tracing`.
- Used by: `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`).
