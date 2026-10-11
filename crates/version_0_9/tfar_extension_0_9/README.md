# TFAR 0.9 Arma extension

The `tfar_extension_0_9` crate: the `task_force_radio_pipe` Arma extension for the 0.9.12 mod, a
Windows x64 DLL run under Proton. Arma 3 loads it through `callExtension` under Proton.

## How it works

Every `callExtension` is a sync round trip to the 0.9 plugin over the game link.
`src/extension_exports.rs` expands `arma_extension!`, so Arma's `RVExtension` reaches
`call_handling::answer_call` with the call text. On the first call the DLL creates its one
`GameLinkClient` (a `OnceLock`), on port 47810, or on 47811 when Arma's command line holds
`-tfdebug`. `answer_call` sends the whole text as one `SYNC_REQ` and maps the outcome to the reply
the original named-pipe extension gave:

```text
empty input                               -> ""  (nothing sent)
the plugin answered                       -> the plugin's text
no session and none could be opened       -> "Pipe not opened from TS plugin"
the plugin closed the connection mid-call -> "Pipe was closed by TS"
no answer within the 1000 ms sync wait    -> "Not connected to TeamSpeak"
```

A trailing `~` has no meaning in 0.9 and is sent as is. The client connects lazily within 50 ms,
waits at most 250 ms for the plugin's `HELLO` and then backs off for 1 s after a failure, so a call
never holds Arma longer than about 1.3 s. `RVExtensionVersion` writes
`tfar_extension_0_9 <version>`.

`extension_exports.rs` expands `arma_extension!` with `answer_game_call` as the handler and
`EXTENSION_VERSION` as the version. `answer_game_call` takes the process-wide `GameLinkClient`
(made on the first call with `client_config(arma_command_line_contains("-tfdebug"))`) and hands it
to `call_handling::answer_call`. `answer_call` is a pure function over the `GameLinkCalls` trait,
which `GameLinkClient` implements, so `tests/call_handling.rs` checks every `SyncOutcome` against
its string with a fake. Each module attaches its sibling test file under `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_extension_0_9                                  # the string mapping and the round trips
cargo build -p tfar_extension_0_9 --target x86_64-pc-windows-gnu  # tfar_extension_0_9.dll under target/x86_64-pc-windows-gnu/debug/
cargo build -p tfar_extension_0_9                                 # libtfar_extension_0_9.so for host testing
cargo run -p tfar_extension_probe -- target/debug/libtfar_extension_0_9.so  # calls it as Arma would
```

## Configuration

It reads the `-tfdebug` launch switch from Arma's command line (`src/extension_exports.rs`), which
selects the plugin's debug port 47811 instead of 47810. Nothing else.

## Exports

- Symbols: `RVExtension` and `RVExtensionVersion`, undecorated, written by `arma_extension!` in
  `src/extension_exports.rs`.
- Artifact: task_force_radio_pipe_x64.dll, which `cargo xtask dist` copies from
  tfar_extension_0_9.dll into target/dist/tfar_0_9/.
- Installed by: the owner, by hand, into the root folder of the matching mod; Arma (BattlEye off)
  then answers `"task_force_radio_pipe" callExtension` through it.

## Boundaries

- Depends on: `tfar_arma_extension_abi` (the Arma extension exports and the `-tfdebug` test);
  `tfar_game_link::client` (the extension-side link client); `thiserror` for the error type; in
  tests, `tfar_game_link::server` and `tfar_game_link::protocol`.
- Used by: Arma 3 under Proton, which loads the DLL as `task_force_radio_pipe`;
  `tools/tfar_extension_probe`, which calls it the same way.
- Rules: every call returns within the sync timeout, connected or not, and each failure has the
  original's string (`src/tests/call_handling.rs`, `tests/game_link_round_trip.rs`). `unsafe` is allowed here for the export
  glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the 0.9 extension
  semantics and the game-facing strings this crate returns.
- [Arma extensions](/documentation/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
