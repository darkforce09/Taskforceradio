# Game commands for 0.9

The `tfar_game_commands_0_9` crate: the 0.9.12 messages Arma sends the plugin, parsed into typed
commands, and the reply strings the plugin answers with. Used by `tfar_client_state_0_9`,
`tfar_plugin_0_9`.

## Contents

```text
crates/version_0_9/tfar_game_commands_0_9/
├── Cargo.toml  the `tfar_game_commands_0_9` package (lib): tier 2, workspace lints and dependencies
└── src/        the parser entry point, one parser per message, the replies and the error type
```

## How it works

`GameCommand::parse` splits a message at tabs with `std::getline` semantics (a trailing empty field
is dropped), picks the parser by the first field and checks the field count the original checks.
Every number goes through `tfar_arma_values`; a malformed field is `Error::Malformed`, a wrong
count `Error::WrongFieldCount`, and both are answered `FAIL`. The crate has no state and calls no
TeamSpeak function: the plugin app does the side effect and picks the reply.

### Replies

`reply_for_unhandled` gives the reply a command earns from its kind; for the three replies that
depend on TeamSpeak it gives the original's fallback, and the app answers with the looked-up value
when it has one.

| Message | Field count, name included | Reply |
|---|---|---|
| `TS_INFO SERVER` | exactly 2 | the server name; `ERROR_GETTING_SERVER_NAME` without one |
| `TS_INFO CHANNEL` | exactly 2 | the channel name; the empty string without one |
| `TS_INFO PING` | exactly 2 | `PONG` |
| `TS_INFO` other | exactly 2 | `FAIL` |
| `KILLED` | 2 or more | `DONE` |
| `SPEAKERS` | 1 or more | `OK` |
| `RELEASE_ALL_TANGENTS` | exactly 2 | `OK` (the original falls through to `FAIL`) |
| `VERSION` | exactly 4 | `OK` |
| `TRACK` | exactly 2 | `OK` (the original falls through to `FAIL`) |
| `POS` | exactly 14 | `SPEAKING` when the player talks, else `NOT_SPEAKING` |
| `TANGENT`, `TANGENT_LR`, `TANGENT_DD` | exactly 5 | `OK` |
| `FREQ` | exactly 14 | `OK` |
| `IS_SPEAKING` | exactly 2 | `SPEAKING` when the player talks, else `NOT_SPEAKING` |
| unknown name, wrong count, malformed field | — | `FAIL` |

### Behaviour kept from the original

- `KILLED` accepts the 2 fields Arma sends; the original needs more than 2 and never runs.
- `SPEAKERS` reads records only from a message of exactly 2 fields; 1 field or more than 2 is an
  empty list, which clears the speakers.
- `VERSION` with an empty serious channel password has 3 fields after the split and is refused,
  as in the original.
- Booleans are `true` only for the exact text `true`; `POS` field 11 (`std::stoi`) and `FREQ`
  field 6 (`(int)atof`) truncate, the other integers round (`parseArmaNumberToInt`).
- `FREQ` field 11 (a second voice volume) is never read, so it is neither parsed nor checked.
- Game activity: every message but one starting with `VERSION` and longer than it counts
  (`message_counts_as_game_activity`, `GameCommand::counts_as_game_activity`).

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_game_commands_0_9  # every message, its counts, its malformed fields, the replies
```

## Configuration

None: the crate reads no setting.

## Public surface

- `GameCommand` and `GameCommand::parse`, `GameCommand::counts_as_game_activity`.
- The payload types `TsInfoQuery`, `SpeakerRecord`, `PositionUpdate`, `UsableRadios`,
  `TangentKind`, `FrequencyUpdate`, `VehicleDescriptor`.
- `replies` (`OK`, `FAIL`, `DONE`, `PONG`, `SPEAKING`, `NOT_SPEAKING`,
  `ERROR_GETTING_SERVER_NAME`, `NO_CHANNEL_NAME`) and `reply_for_unhandled`.
- `command_names`, `IN_GAME_TRIGGER_EXEMPT_PREFIX`, `message_counts_as_game_activity`.
- `Error` (with `Error::reply`), `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values` (field splitting, numbers, arrays, frequency lists, radio
  subtypes); `tfar_geometry` (`Position`, `Angle`); `thiserror` for the error type.
- Used by: `tfar_client_state_0_9`, `tfar_plugin_0_9`.
- Rules: the reply strings and field counts are those of `processGameCommand` in
  `reference/tfar_0_9_12/ts/src/plugin.cpp`, held by the tests in `src/tests/replies.rs` and
  `src/message_parsers/tests/`; every field access is length-checked, so a short message is an
  `Error`. Tier 2 in `crates/version_0_9`: every workspace dependency has a lower tier and follows
  the edge matrix in `crates/README.md` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports; section 2 lists the game messages.
- [Bug fixes](/documentation/program/bug_fixes.md) — F-003, F-020 and F-021, held by this crate.
