# Game commands for 1.0

The `tfar_game_commands_1_0` crate: the 1.0 messages Arma sends the plugin, parsed into typed
commands, and the answers the plugin gives sync calls. Used by `tfar_client_state_1_0`,
`tfar_plugin_1_0`.

## Contents

```text
crates/version_1_0/tfar_game_commands_1_0/
├── Cargo.toml  the `tfar_game_commands_1_0` package (lib): tier 2, workspace lints and dependencies
└── src/        the parser, the command payloads, the field readers, the replies and the error type
```

## How it works

`parse(text)` splits a message at its tabs, keeping every field as `helpers::split` does
(`reference/tfar_1_0/ts/src/helpers.cpp:245-262`; a trailing tab gives a trailing empty field),
finds the command by exact name in place of the original's FNV-1a hash switch, and reads its
fields by position with checked counts. The text is the message without the async `~`, which the
game link removes. An unknown name is `GameCommand::Unknown`; a known command with a wrong count
or a malformed field is an `Error` that rejects the whole message, never an out-of-bounds read or
a silent zero.

### Token counts

| Command | Tokens, name included | Notes |
|---|---|---|
| `TS_INFO` | 2 or more | only the second is read, as the sync split stops there |
| `POS` | exactly 14 | |
| `IS_SPEAKING` | 2 or more | only the second is read |
| `IS_SPEAKING_BULK` | any | every token after the name is a nickname, a trailing empty one too |
| `FREQ` | exactly 11 | |
| `KILLED`, `RELEASE_ALL_TANGENTS` | exactly 2 | |
| `TANGENT`, `TANGENT_LR`, `TANGENT_DD` | 5, or 6 with the class name | |
| `SETCFG` | 3 or more | exactly 4 carries the type tag; any other count sets the empty text |
| `SPEAKERS` | any | any count but 2 is the empty list, which clears the speakers |
| `RadioTwrAdd`, `RadioTwrDel` | exactly 2 | |
| `TRACK` | exactly 4 | |
| `DFRAME`, `MISSIONEND`, `collectDebugInfo` | any | no field is read |

`RECV_FREQS` is not a command: the original never maps it.

### Sync replies

`GameCommand::is_async_only` holds the original's rule: `TS_INFO`, `POS`, `IS_SPEAKING` and
`IS_SPEAKING_BULK` answer sync (`POS` is also queued); every other command answers
`ASYNC COMMAND SENT IN SYNC CONTEXT` when called sync.

| Situation | Reply | Source (`reference/tfar_1_0/ts/src/`) |
|---|---|---|
| unknown name | `UNKNOWN COMMAND` | `CommandProcessor.cpp:80` |
| async-only command called sync | `ASYNC COMMAND SENT IN SYNC CONTEXT` | `CommandProcessor.cpp:161` |
| `TS_INFO PING` | `PONG` | `CommandProcessor.cpp:625` |
| `TS_INFO VERSION` | the plugin version | `CommandProcessor.cpp:627` |
| `TS_INFO SERVER`, `SERVERUID` | the value, or `ERROR_GETTING_SERVER_NAME` | `Teamspeak.cpp:550-574` |
| `TS_INFO CHANNEL` | the channel name, or the empty text | `Teamspeak.cpp:526-539` |
| `TS_INFO CHANNELID` | the channel id in decimal | `CommandProcessor.cpp:623` |
| `TS_INFO` any other query | `FAIL` | `CommandProcessor.cpp:628` |
| `POS`, `IS_SPEAKING` | `speaking_code`: talking, receiving (`00`–`11`) | `CommandProcessor.cpp:89-101` |
| `POS`, `IS_SPEAKING` of an unknown client | `00` | `CommandProcessor.cpp:92-95` |
| `IS_SPEAKING_BULK` | one code and a tab per nickname, the last tab kept | `CommandProcessor.cpp:103-144` |
| a rejected message | `Error::sync_reply`: `00` for `POS` and `IS_SPEAKING`, `FAIL` for `TS_INFO`, the async text for the rest | |

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_commands_1_0  # the unit tests
```

## Configuration

None: the crate reads no setting. `SETCFG` is parsed here and applied by
`tfar_plugin_settings_1_0`.

## Public surface

- `parse`, `GameCommand`, `CommandName`.
- The payloads: `TsInfoQuery`, `PositionUpdate`, `FrequencyReport` (with `NICKNAMES_NEVER_ADOPTED`),
  `SpeakerRecord`, `TangentPress` and `TangentKind` (with `TANGENT_PRESSED`, `TANGENT_RELEASED`),
  `ConfigAssignment`, `TowerRecord`, `TrackingEvent`, `TrackerCustomVariable`,
  `VehicleDescriptor`, `NetworkObjectId`.
- `replies`: the reply strings, `speaking_code`, `speaking_code_text`, `bulk_speaking_reply`.
- `Error` (`WrongTokenCount`, `Malformed`, with `command` and `sync_reply`), `Result`,
  `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values` (numbers, frequency lists, nicknames, subtypes, separators);
  `tfar_geometry` (positions and directions); `tfar_plugin_settings_1_0` (the `SETCFG` type tag);
  `thiserror` for the error type.
- Used by: `tfar_client_state_1_0`, `tfar_plugin_1_0`.
- Rules: a wrong token count or a malformed field rejects the message
  (`a_short_pos_message_is_an_error_not_an_out_of_bounds_read`); a short speaker record is skipped
  and the rest kept (`a_short_speaker_record_is_skipped_and_the_rest_kept`); the reply strings are
  the original's (`the_reply_strings_are_the_originals`). Tier 2 in `crates/version_1_0`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — §2.2 the
  command table, §5 the vehicle descriptor, §8 subtypes and nicknames.
- [Bug fixes](/documentation/program/bug_fixes.md) — the original bugs this crate fixes.
