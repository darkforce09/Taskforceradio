# Events

The TeamSpeak events beyond the client roster: plugin commands from other TFAR clients and the own
talking flag.

## Contents

```text
apps/tfar_plugin_1_0/src/events/
├── mod.rs              `mod` lines
└── plugin_commands.rs  remote tangents, `VOLUME`, `RELEASE_ALL_TANGENTS`, `REQVOL`, the own `VOLUME`
```

## How it works

`plugin_command_received` accepts only commands of the server in front from a plugin whose name
starts with `TFAR`, decodes them with `tfar_plugin_commands_1_0::decode` and applies them under the
server lock: a remote tangent through `tfar_radio_routing_1_0::apply_remote_tangent`, which yields
the remote sounds, the reception count and the heard list; a real press resets the sender's
direction sources. The sounds are shaped and queued and the mute sent after the lock is released.
`own_variable_changed` sends `VOLUME` to the channel when the own talking flag changes in game.

## Boundaries

- Depends on: `tfar_plugin_commands_1_0`, `tfar_client_state_1_0`, `tfar_radio_routing_1_0`,
  `tfar_ts3_host`; the crate's `runtime`.
- Used by: `plugin_lifecycle`.
- Rules: no TeamSpeak call under the server lock; a dormant plugin sends nothing.
