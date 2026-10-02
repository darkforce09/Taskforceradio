# Command application for 0.9

The methods of `ServerRadioData` that apply each 0.9.12 game message and each incoming plugin
command, and return what the plugin does next.

## Contents

```text
crates/version_0_9/tfar_client_state_0_9/src/command_application/
├── frequencies.rs       `FREQ`: radios and levels, `RenameRequest` and `confirm_rename`
├── local_tangent.rs     the game's `TANGENT` messages: the local push-to-talk
├── mission_messages.rs  `VERSION`, `SPEAKERS` and `KILLED`
├── mod.rs               the module lines and the re-exports
├── plugin_commands.rs   the incoming `TANGENT`, `RELEASE_ALL_TANGENTS` and `VOLUME` plugin commands
├── positions.rs         `POS` and `IS_SPEAKING`: `PositionOutcome`, `SpeakingState`
└── tests/               unit tests per command
```

## How it works

Each method takes the typed command (`tfar_game_commands_0_9` for the game, the decoded fields of
`tfar_plugin_commands_0_9` for the plugins) and `now` where the original stamps a time, changes the
state as `processGameCommand`, `processUnitPosition` and `processPluginCommand` in
`reference/tfar_0_9_12/ts/src/plugin.cpp` do, and returns an outcome naming the TeamSpeak side
effect: `PositionOutcome` (centre the listener, check the mute, or refresh the client list),
`RenameRequest`, the tangent-changed flag, `RemoteTangentOutcome` (route, mute and play the start or
end sound) and `RemoteVolumeOutcome`. None of them fails: an unknown nickname changes nothing and
says so.

## Boundaries

- Depends on: the state in `crates/version_0_9/tfar_client_state_0_9/src/server_radio_data.rs`
  and `crates/version_0_9/tfar_client_state_0_9/src/client_data.rs`, the data frame lookups;
  `tfar_game_commands_0_9`.
- Used by: `tfar_plugin_0_9`, on the game link thread and TeamSpeak's plugin-command event.
- Rules: a `POS` for another player never changes their view angle; a remote push-to-talk asks for a
  sound only on a flip; the original nickname is saved once per game session
  (`the_original_nickname_after_two_renames`).
