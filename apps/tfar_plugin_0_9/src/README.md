# TFAR 0.9 TeamSpeak plugin source

The source of `tfar_plugin_0_9`: the exports and lifecycle, the shared core and its per-connection
state, the game messages, the TeamSpeak events, the voice callbacks, push-to-talk, the radio sounds
and the service thread.

## Contents

```text
apps/tfar_plugin_0_9/src/
├── audio_callbacks.rs           the post-process, captured, mixed-playback and rolloff callbacks
├── channel_snapshot.rs          our channel read from TeamSpeak in one pass; the voice facts it leaves
├── crate_role.rs                `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs                     `Error`, the `thiserror` enum, and `Result`
├── game_messages/               the game link bridge and every game message: parse, apply, reply
├── game_session_transitions.rs  game start and end: on/off sounds, serious-mode move, nickname back
├── incoming_plugin_commands.rs  other players' TANGENT, RELEASE_ALL_TANGENTS and VOLUME commands
├── lib.rs                       the crate root: module header, `mod` lines and re-exports
├── listen_port.rs               the game link port from `TFAR_GAME_LINK_DEBUG` and `TFAR_GAME_LINK_PORT`
├── plugin_core.rs               `PluginCore`: the shared runtime, its flags, locks and helpers
├── plugin_exports.rs            the `PLUGIN` static and the `ts3_plugin!` expansion over it
├── plugin_lifecycle.rs          `TfarPlugin`: the metadata, `init`, `shutdown`, the callbacks
├── plugin_startup.rs            the plugin folder, the HRIR sphere and the push-to-talk delay at init
├── prelude.rs                   the plugin, the core, the TeamSpeak traits and the port choice
├── push_to_talk/                the local push-to-talk: state machine, VAD, microphone, release thread
├── radio_sounds.rs              the `radio-sounds/` and `tfar_hrir/` paths by subtype and moment
├── remote_sounds.rs             a remote push-to-talk sound placed and filtered for the mixer
├── running_plugin.rs            `RunningPlugin`: what `init` starts and the order `shutdown` stops it
├── server_connection.rs         `ServerConnection` and `ServerState`: one connection behind one lock
├── service_thread.rs            the 100 ms tick: game timeout, 4 s sweep, metadata, finished sounds
├── status_metadata.rs           the `<TFAR>` status in our metadata, and the disabled text
├── team_speak/                  the `TeamSpeak` traits and their function-table implementation
├── team_speak_events.rs         connection, channel and client events, the talking flag, info panel
└── tests/                       unit tests per module and the recording fake of TeamSpeak
```

## How it works

`plugin_exports.rs` expands `ts3_plugin!(PLUGIN)`; `TfarPlugin` (in `plugin_lifecycle.rs`) keeps a
`RunningPlugin` behind a mutex and the `PluginCore` behind a `parking_lot::RwLock`. Main-thread
callbacks clone the core's `Arc` and call it; the audio callbacks `try_read` the lock and pass
through while `init` or `shutdown` holds it.

`PluginCore` methods are spread by concern over the modules: `game_messages/` for the game link,
`incoming_plugin_commands.rs` and `team_speak_events.rs` for TeamSpeak's events,
`audio_callbacks.rs` for the voice blocks, `service_thread.rs` for the tick,
`game_session_transitions.rs` and `status_metadata.rs` for the game's start, end and status. Every
one follows the same pattern: read TeamSpeak through the `TeamSpeak` traits, lock the connection's
`ServerState` to apply and decide with the library crates, unlock, then call TeamSpeak
(`send_mute_decision` forgets a mute TeamSpeak refused). The voice callbacks read
`TeamSpeakFacts`, which `channel_snapshot.rs` fills on every channel or client event and sweep.

Threads: TeamSpeak's main thread (lifecycle, events, plugin commands), its audio threads (voice
callbacks), the game link server's session thread (game messages), `tfar-service` and
`tfar-push-to-talk-release`. The unit tests drive the core over `tests/fake_team_speak.rs`, a
recording fake of both traits; each module attaches its sibling test file with `#[path]`.

## Public surface

- `plugin_exports::PLUGIN` and `TfarPlugin`, `FLAVOUR`, `LOG_CHANNEL`; `RunningPlugin`,
  `StartError`, `SKIP_UPDATE_CHECK_VARIABLE`; `listen_port` (`server_config`,
  `server_config_from_environment`, `DEBUG_PORT_VARIABLE`, `PORT_OVERRIDE_VARIABLE`).
- `PluginCore` with its handlers (`handle_game_message`, `handle_plugin_command`, the event, audio
  and service methods), `PluginResources`, `MuteFacts`; `ServerConnection`, `ServerState`;
  `ChannelSnapshot`, `TeamSpeakFacts`; `PushToTalk`, `PushToTalkState`; `RadioSounds`;
  `RemoteSoundPlan`, `render_remote_sound`; `ServiceThread`.
- `team_speak` (`TeamSpeak`, `TeamSpeakQueries`, `TeamSpeakActions`, `Ts3TeamSpeak`).
- `CRATE_ROLE`, `Error`, `Result` and `prelude`; the C exports `ts3plugin_*`, for TeamSpeak.

## Boundaries

- Depends on: the crates of the app's manifest; TeamSpeak through `tfar_ts3_host` only.
- Used by: TeamSpeak, which loads the cdylib, and the tests that link the rlib or load the `.so`.
- Rules: no TeamSpeak call under a server lock and none on an audio thread (the audio paths take
  only `try_` locks and read `TeamSpeakFacts`); a dormant plugin sends nothing
  (`version_alone_keeps_the_plugin_dormant_and_writes_nothing`,
  `a_dormant_plugin_takes_in_remote_commands_without_any_request`); production files hold no
  `unwrap`, `expect`, `panic` or indexing (clippy deny list); the only `unsafe` is the code
  `ts3_plugin!` expands.
