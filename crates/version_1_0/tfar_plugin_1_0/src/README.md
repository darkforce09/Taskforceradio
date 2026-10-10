# TFAR 1.0 TeamSpeak plugin source

The source of `tfar_plugin_1_0`: the `ts3plugin_*` exports, the plugin's lifecycle, the runtime
with its per-server state, the game link handler, the TeamSpeak events, the audio callbacks,
push-to-talk, the service thread and the diagnostics.

## Contents

```text
crates/version_1_0/tfar_plugin_1_0/src/
├── audio/                the audio callbacks and the radio sounds
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── diagnostics/          `/tfar` commands, reports, the debug dump and the debug window snapshot
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── events/               plugin commands from other clients and the own talking flag
├── game_link/            the game link handler, sync answers, async messages, tangent, SETCFG
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── listen_port.rs        the game link ports: both spec ports, or the test override `TFAR_GAME_LINK_PORT`
├── plugin_exports.rs     the `PLUGIN` static and the `ts3_plugin!` expansion over it
├── plugin_lifecycle.rs   `TfarPlugin`: the original metadata, `init`, `shutdown`, callback routing
├── prelude.rs            the plugin, the runtime, the port choice and the constants for glob import
├── push_to_talk/         the tangent's push-to-talk, voice activation and delayed release
├── runtime/              `PluginRuntime`, the server states, lifecycle, serious mode, mutes, roster
├── service/              the 100 ms service thread and its periods
├── teamspeak_actions.rs  `TeamSpeakActions`: every TeamSpeak query and side effect, over `Ts3`
└── tests/                unit tests of the metadata, the port choice and `CRATE_ROLE`
```

## How it works

`plugin_exports.rs` expands `ts3_plugin!(PLUGIN)`. `TfarPlugin::init` installs the log forwarder,
loads the radio sounds and the HRIR sphere, reads TeamSpeak's push-to-talk delay, builds the
`PluginRuntime`, starts the `GameLinkServer` on both ports with a `LinkHandler`, the service
thread and the cancellable update check, and publishes the runtime in an `ArcSwapOption`. Every
callback loads it: the event callbacks call runtime methods, the audio callbacks the functions of
`audio::voice_callbacks`, which wait only for a server lock whose holders never call TeamSpeak. All TeamSpeak traffic goes through `TeamSpeakActions`; side effects only
through `PluginRuntime::act`, which is `None` while the plugin is dormant. `TfarPlugin::shutdown`
takes the runtime out, runs `PluginRuntime::shut_down` (web requests cancelled, game end, metadata
cleared, mutes lifted, link server, service, release thread and debug window stopped and joined,
web requests joined within 1 s) and drops the log forwarder last, all within 2 s.

## Public surface

- `plugin_exports::PLUGIN`, `TfarPlugin`, `FLAVOUR`, `LOG_CHANNEL`, `PLUGIN_VERSION`,
  `SKIP_UPDATE_CHECK_VARIABLE`; `listen_port` (`server_config`, `server_config_from_environment`,
  `PORT_OVERRIDE_VARIABLE`); `service::ServicePeriods` and `SERVICE_TICK_VARIABLE`;
  `runtime::PluginRuntime`; `teamspeak_actions::TeamSpeakActions`; the modules `audio`, `diagnostics`, `events`,
  `game_link`, `push_to_talk`, `runtime`, `service`; `CRATE_ROLE`, `Error`, `Result`, `prelude`.
- The C exports `ts3plugin_*`, for TeamSpeak.

## Boundaries

- Depends on: every crate the manifest lists.
- Used by: TeamSpeak, which loads the cdylib, and the tests that load the `.so` in the mock host.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list); tests
  live in `tests/` folders (`cargo xtask verify test-placement`); the only `unsafe` is the code
  `ts3_plugin!` expands; no TeamSpeak call while a server lock is held; the audio callbacks wait
  only for a server lock.
