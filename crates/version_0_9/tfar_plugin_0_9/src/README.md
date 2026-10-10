# TFAR 0.9 TeamSpeak plugin source

The source of `tfar_plugin_0_9`: the exports and lifecycle, the shared core and its per-connection
state, the game messages, the TeamSpeak events, the voice callbacks, push-to-talk, the radio sounds
and the service thread.

## Contents

```text
crates/version_0_9/tfar_plugin_0_9/src/
├── audio_callbacks.rs           the post-process, captured, mixed-playback and rolloff callbacks
├── background_web_requests.rs   the running update check and Piwik requests, joined at shutdown
├── channel_snapshot.rs          our channel read from TeamSpeak in one pass; the voice facts it leaves
├── crate_role.rs                `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs                     `Error`, the `thiserror` enum, and `Result`
├── game_messages/               the game link bridge and every game message: parse, apply, reply
├── game_session_transitions.rs  game start and end: on/off sounds, serious-mode move, nickname back
├── incoming_plugin_commands.rs  other players' TANGENT, RELEASE_ALL_TANGENTS and VOLUME commands
├── lib.rs                       the crate root: module header, `mod` lines and re-exports
├── listen_port.rs               the game link's two spec ports, or the test port of `TFAR_GAME_LINK_PORT`
├── plugin_core.rs               `PluginCore`: the shared runtime, its flags, locks and helpers
├── plugin_exports.rs            the `PLUGIN` static and the `ts3_plugin!` expansion over it
├── plugin_lifecycle.rs          `TfarPlugin`: the metadata, `init`, `shutdown`, the callbacks
├── plugin_startup.rs            the plugin folder, the HRIR sphere and the push-to-talk delay at init
├── prelude.rs                   the plugin, the core, the TeamSpeak traits and the port choice
├── push_to_talk/                the local push-to-talk: state machine, VAD, microphone, release thread
├── radio_sounds.rs              the `radio-sounds/` and `tfar_hrir/` paths by subtype and moment
├── running_plugin.rs            `RunningPlugin`: what `init` starts and the order `shutdown` stops it
├── server_connection.rs         `ServerConnection` and `ServerState`: one connection behind one lock
├── service_thread.rs            the 100 ms tick: game timeout and dormancy, 4 s sweep, metadata, sounds
├── status_metadata.rs           the `<TFAR>` status in our metadata, and the disabled text
├── team_speak/                  the `TeamSpeak` traits and their function-table implementation
├── team_speak_events.rs         connection, channel, client and talk events, the talking flag, info panel
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
A remote push-to-talk sound is shaped by the voice pipeline's `render_remote_tangent_sound`; the
own voice reaches the mixer through the `StreamProducer` each `ServerState` holds.

Threads: TeamSpeak's main thread (lifecycle, events, plugin commands), its audio threads (voice
callbacks), the game link server's accept and session threads (game messages), `tfar-service`,
`tfar-push-to-talk-release`, the web request threads (`tfar-update-check`, `tfar-piwik-track`)
and the log forwarder. The unit tests drive the core over `tests/fake_team_speak.rs`, a recording
fake of both traits, and the service tick with the time injected (`ServiceSchedule`); each module
attaches its sibling test file with `#[path]`.

## Public surface

- `plugin_exports::PLUGIN` and `TfarPlugin`, `FLAVOUR`, `LOG_CHANNEL`; `RunningPlugin`,
  `StartError`, `SKIP_UPDATE_CHECK_VARIABLE`, `SHUTDOWN_BUDGET`, `WEB_REQUEST_JOIN_DEADLINE`;
  `listen_port` (`server_config`, `server_config_from_environment`, `PORT_OVERRIDE_VARIABLE`).
- `PluginCore` with its handlers (`handle_game_message`, `handle_plugin_command`,
  `handle_talk_status_change`, the event, audio and service methods, `record_game_activity`,
  `return_to_dormant`), `PluginResources`, `MuteFacts`; `ServerConnection`, `ServerState`;
  `ChannelSnapshot`, `TeamSpeakFacts`; `PushToTalk`, `PushToTalkState`; `RadioSounds`;
  `PlannedRemoteSound`; `BackgroundWebRequests`; `ServiceThread`, `ServiceSchedule`.
- `team_speak` (`TeamSpeak`, `TeamSpeakQueries`, `TeamSpeakActions`, `Ts3TeamSpeak`).
- `CRATE_ROLE`, `Error`, `Result` and `prelude`; the C exports `ts3plugin_*`, for TeamSpeak.

## Boundaries

- Depends on: the crates of the app's manifest; TeamSpeak through `tfar_ts3_host` only.
- Used by: TeamSpeak, which loads the cdylib, and the tests that link the rlib or load the `.so`.
- Rules: no TeamSpeak call under a server lock and none on an audio thread (the audio paths take
  only `try_` locks and read `TeamSpeakFacts`); a dormant plugin sends nothing
  (`version_alone_keeps_the_plugin_dormant_and_writes_nothing`,
  `a_dormant_plugin_takes_in_remote_commands_without_any_request`), also for 10 s of ticks after
  a game end (`after_a_game_times_out_the_plugin_is_dormant_for_ten_seconds_of_ticks_until_the_game_resumes`);
  a shutdown never waits past its deadline for a web request
  (`a_request_in_flight_never_holds_the_shutdown_past_its_deadline`); production files hold no
  `unwrap`, `expect`, `panic` or indexing (clippy deny list); the only `unsafe` is the code
  `ts3_plugin!` expands.
