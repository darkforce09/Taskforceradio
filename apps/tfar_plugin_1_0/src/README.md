# TFAR 1.0 TeamSpeak plugin source

The source of `tfar_plugin_1_0`: the `ts3plugin_*` exports, the plugin's lifecycle, the choice of
the game link port and the stub handler that answers the game's ping.

## Contents

```text
apps/tfar_plugin_1_0/src/
├── crate_role.rs        `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs             `Error`, the `thiserror` enum, and `Result`
├── game_link_stub.rs    `StubGameLinkHandler`: `PONG` for the ping, `UNKNOWN COMMAND` otherwise
├── lib.rs               the crate root: module header, `mod` lines and re-exports
├── listen_port.rs       the game link port from `TFAR_GAME_LINK_DEBUG` and `TFAR_GAME_LINK_PORT`
├── plugin_exports.rs    the `PLUGIN` static and the `ts3_plugin!` expansion over it
├── plugin_lifecycle.rs  `TfarPlugin`: the original metadata, `init` and `shutdown`
├── prelude.rs           the plugin, the port choice and the stub handler for glob import
└── tests/               unit tests of the metadata, the port choice, the stub replies and `CRATE_ROLE`
```

## How it works

`plugin_exports.rs` expands `ts3_plugin!(PLUGIN)`. `TfarPlugin::init` (in `plugin_lifecycle.rs`)
stops anything an earlier `init` started, installs the log forwarder, asks `listen_port.rs` for the
server settings and starts `GameLinkServer` with `StubGameLinkHandler`; the server and the logging
guard live behind one mutex until `shutdown` takes them, joins the server and drops the guard. The
port choice is a pure function over the two variable values, so `tests/listen_port.rs` checks it
without touching the environment. Each module attaches its sibling test file under `tests/` with
`#[path]`.

## Public surface

- `plugin_exports::PLUGIN` and `TfarPlugin`, `FLAVOUR`, `LOG_CHANNEL`; `listen_port`
  (`server_config`, `server_config_from_environment`, `DEBUG_PORT_VARIABLE`,
  `PORT_OVERRIDE_VARIABLE`);
  `game_link_stub` (`StubGameLinkHandler`, `PING_REQUEST`, `PING_REPLY`, `UNKNOWN_COMMAND_REPLY`);
  `CRATE_ROLE`, `Error`, `Result` and `prelude`.
- The C exports `ts3plugin_*`, for TeamSpeak.

## Boundaries

- Depends on: `tfar_ts3_host`, `tfar_game_link_server`, `tracing`, `thiserror`.
- Used by: TeamSpeak, which loads the cdylib, and the tests that link the rlib or load the `.so`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` (`cargo xtask verify test-placement`); the only `unsafe` is the code
  `ts3_plugin!` expands.
