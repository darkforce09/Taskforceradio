# TFAR 1.0 Arma extension source

The source of `tfar_extension_1_0`: the `RVExtension` exports, the process-wide game link client,
the focus watch and the call handling that gives every situation the original extension's reply.

## Contents

```text
crates/version_1_0/tfar_extension_1_0/src/
├── call_handling.rs      `answer_call` over the `GameLinkCalls` trait: sync, `~` async, focus, the strings
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── extension_exports.rs  the exports, the `OnceLock` client, the focus mutex, the flavour, port, version
├── focus_watch.rs        `FocusWatch`: read the foreground window at most every 250 ms, report changes
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── prelude.rs            the call handling, the strings, the focus watch and the constants for glob import
└── tests/                unit tests of the string mapping, the focus watch, the exports module and `CRATE_ROLE`
```

## How it works

`extension_exports.rs` expands `arma_extension!` with `answer_game_call` as the handler and
`EXTENSION_VERSION` as the version. `answer_game_call` locks the process-wide `FocusWatch`, takes
the process-wide `GameLinkClient` (made on the first call with
`client_config(arma_command_line_contains("-tfdebug"))`) and calls `call_handling::answer_call`
with `Instant::now()` and `foreground_window_is_arma`. `answer_call` is a pure function over the
`GameLinkCalls` trait, which `GameLinkClient` implements, with the time and the focus read passed
in, so `tests/call_handling.rs` checks every situation and the focus order with a fake, and
`tests/focus_watch.rs` checks the 250 ms rule with chosen instants.
Each module attaches its sibling test file under `tests/` with `#[path]`.

## Public surface

- `call_handling`: `answer_call`, `GameLinkCalls`, `NOT_CONNECTED_TO_TEAMSPEAK`, `ASYNC_ACCEPTED`,
  `CONFIGURATION_NEEDED`, `ASYNC_MARKER`, `DATA_FRAME_COMMAND`, `MISSION_END_COMMAND`; the
  round-trip test drives `answer_call` with a real client.
- `FocusWatch` and `FOCUS_READ_INTERVAL`, `FLAVOUR`, `DEBUG_SWITCH`, `EXTENSION_VERSION`,
  `client_config`, `CRATE_ROLE`, `Error`, `Result` and `prelude`, at the crate root.
- The C exports `RVExtension` and `RVExtensionVersion`, for Arma.

## Boundaries

- Depends on: `tfar_arma_extension_abi`, `tfar_game_link_client`, `tracing`, `thiserror`.
- Used by: Arma, which loads the cdylib, and the tests that link the rlib.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); the only `unsafe` is the code
  `arma_extension!` expands.
