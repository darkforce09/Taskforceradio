# TFAR 1.0 Arma extension

The `tfar_extension_1_0` crate: the `task_force_radio_pipe` Arma extension for the 1.0 mod, a
Windows x64 DLL run under Proton. Arma 3 loads it through `callExtension` under Proton.

## Contents

```text
apps/tfar_extension_1_0/
├── Cargo.toml  the `tfar_extension_1_0` package (cdylib + rlib): tier 2, workspace lints and dependencies
├── src/        the exports, the process-wide link client, the focus watch and the call handling
└── tests/      the call handling over the real client against a real server on an ephemeral port
```

## How it works

`src/extension_exports.rs` expands `arma_extension!`, so Arma's `RVExtension` reaches
`call_handling::answer_call` with the call text. On the first call the DLL creates its one
`GameLinkClient` (a `OnceLock`), on port 47820, or on 47821 when Arma's command line holds
`-tfdebug`. A call ending in `~` is sent as `ASYNC` with that one `~` removed and is answered at
once; any other call is a `SYNC_REQ` that waits up to 1000 ms. Before a non-empty call is sent, the
`FocusWatch` reads the foreground window (at most every 250 ms) and sends `FOCUS` when "is Arma in
front" changed. The replies are those of the original shared-memory extension:

```text
empty input                                  -> ""  (nothing sent, no focus read)
DFRAME~ while the latest STATUS says needcfg -> "NEEDCFG"
MISSIONEND~                                  -> ""
any other ~ call                             -> "OK"
a sync call the plugin answered              -> the plugin's text
a sync call with no answer within 1000 ms    -> ""
any call with no live session                -> "Not connected to TeamSpeak"
```

A sync call whose session the plugin closes mid-call also answers `Not connected to TeamSpeak`,
since no live session remains. The command name is the text up to the first TAB, so only `DFRAME`
and `MISSIONEND` themselves get their special replies. `RVExtensionVersion` writes
`tfar_extension_1_0 <version>`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_extension_1_0                                  # the string mapping, the focus watch and the round trips
cargo build -p tfar_extension_1_0 --target x86_64-pc-windows-gnu  # tfar_extension_1_0.dll under target/x86_64-pc-windows-gnu/debug/
cargo build -p tfar_extension_1_0                                 # libtfar_extension_1_0.so for host testing
cargo run -p tfar_extension_probe -- target/debug/libtfar_extension_1_0.so 'DFRAME~'  # calls it as Arma would
```

## Configuration

It reads the `-tfdebug` launch switch from Arma's command line (`src/extension_exports.rs`), which
selects the plugin's debug port 47821 instead of 47820. Nothing else.

## Exports

- Symbols: `RVExtension` and `RVExtensionVersion`, undecorated, written by `arma_extension!` in
  `src/extension_exports.rs`.
- Artifact: task_force_radio_pipe_x64.dll, which `cargo xtask dist` copies from
  tfar_extension_1_0.dll into target/dist/tfar_1_0/.
- Installed by: the owner, by hand, into the root folder of the matching mod; Arma (BattlEye off)
  then answers `"task_force_radio_pipe" callExtension` through it.

## Boundaries

- Depends on: `tfar_arma_extension_abi` (the exports, the `-tfdebug` test and the foreground-window
  test); `tfar_game_link_client` (the extension-side link client); `tracing` for debug lines Arma
  never collects; `thiserror` for the error type; in tests, `tfar_game_link_server`.
- Used by: Arma 3 under Proton, which loads the DLL as `task_force_radio_pipe`;
  `tools/tfar_extension_probe`, which calls it the same way.
- Rules: async calls reach the plugin in the order the game made them and never wait for it; each
  situation has the original's string (`src/tests/call_handling.rs`,
  `tests/game_link_round_trip.rs`). Tier 2 in `apps`: every workspace dependency has a lower tier
  and follows the edge matrix in `crates/README.md` (`cargo xtask verify crate-tiers`). `unsafe` is
  allowed here for the export glue only (`cargo xtask verify unsafe-allowlist`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the 1.0 extension
  semantics, the focus rule and the game-facing strings this crate returns.
- [Arma extensions](/documentation/program/behaviour_specs/extensions.md) — the original
  `task_force_radio_pipe` extensions and the Arma ABI facts.
- [Windows dependencies](/documentation/program/behaviour_specs/windows_dependencies.md) — the Win32
  objects the originals use and what replaces them.
