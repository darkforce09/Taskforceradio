# TFAR 0.9 Arma extension source

The source of `tfar_extension_0_9`: the `RVExtension` exports, the process-wide game link client
and the call handling that gives every outcome the original extension's reply string.

## Contents

```text
crates/version_0_9/tfar_extension_0_9/src/
├── call_handling.rs      `answer_call` over the `GameLinkCalls` trait, and the three failure strings
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── extension_exports.rs  the exports, the `OnceLock` client, the flavour, the port and the version
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── prelude.rs            the call handling, the strings and the crate constants for glob import
└── tests/                unit tests of the string mapping, the exports module and `CRATE_ROLE`
```

## How it works

`extension_exports.rs` expands `arma_extension!` with `answer_game_call` as the handler and
`EXTENSION_VERSION` as the version. `answer_game_call` takes the process-wide `GameLinkClient`
(made on the first call with `client_config(arma_command_line_contains("-tfdebug"))`) and hands it
to `call_handling::answer_call`. `answer_call` is a pure function over the `GameLinkCalls` trait,
which `GameLinkClient` implements, so `tests/call_handling.rs` checks every `SyncOutcome` against
its string with a fake. Each module attaches its sibling test file under `tests/` with `#[path]`.

## Public surface

- `call_handling`: `answer_call`, `GameLinkCalls`, `PIPE_NOT_OPENED`, `PIPE_CLOSED_BY_TEAMSPEAK`,
  `NOT_CONNECTED_TO_TEAMSPEAK`; the round-trip test drives `answer_call` with a real client.
- `FLAVOUR`, `DEBUG_SWITCH`, `EXTENSION_VERSION`, `client_config`, `CRATE_ROLE`, `Error`, `Result`
  and `prelude`, at the crate root.
- The C exports `RVExtension` and `RVExtensionVersion`, for Arma.

## Boundaries

- Depends on: `tfar_arma_extension_abi`, `tfar_game_link_client`, `thiserror`.
- Used by: Arma, which loads the cdylib, and the tests that link the rlib.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); the only `unsafe` is the code
  `arma_extension!` expands.
