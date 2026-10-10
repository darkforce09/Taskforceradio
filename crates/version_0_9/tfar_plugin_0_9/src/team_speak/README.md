# TeamSpeak access of the 0.9 plugin

The plugin's one door to TeamSpeak: the reads and the side effects as two traits, and their
implementation over TeamSpeak's function table.

## Contents

```text
crates/version_0_9/tfar_plugin_0_9/src/team_speak/
├── mod.rs                 the module lines and the re-exports
├── team_speak_actions.rs  `TeamSpeakActions`: mutes, nickname, metadata, commands, sounds, moves, 3D, VAD
├── team_speak_host.rs     `TeamSpeak`: both traits together, `Send + Sync`
├── team_speak_queries.rs  `TeamSpeakQueries`: our id, channels, clients, nicknames, metadata, flags
└── ts3_team_speak.rs      `Ts3TeamSpeak`: both traits over `tfar_ts3_host::Ts3`
```

## How it works

`PluginCore` holds an `Arc<dyn TeamSpeak>`: `Ts3TeamSpeak` in the plugin, the recording fake in the
unit tests. A failed read becomes the neutral answer the original falls back to (`None`, an empty
list, `false`) with a debug line; a failed write returns TeamSpeak's error so the caller can react
(a refused mute is forgotten by the tracker).

## Boundaries

- Depends on: `tfar_ts3_host` (`Ts3`, the newtypes, `Ts3Error`).
- Used by: every module of the crate that talks to TeamSpeak, through `PluginCore::team_speak`.
- Rules: nothing here runs on an audio thread or under a server lock (the callers' rule, held by the
  lock pattern of `crates/version_0_9/tfar_plugin_0_9/src/plugin_core.rs`).
