# Game commands for 0.9 source

The source of `tfar_game_commands_0_9`: the parser entry point, one parser per game message, the
field readers they share, and the reply strings.

## Contents

```text
crates/version_0_9/tfar_game_commands_0_9/src/
├── command_names.rs       the command names, the first field of each message
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error`: wrong field count, malformed field, short speaker record
├── field_values.rs        the shared field readers: rounding, truncation, `"true"`, exact counts
├── game_activity.rs       `message_counts_as_game_activity` and its exempt prefix `VERSION`
├── game_command.rs        `GameCommand`, its `parse` dispatch and `counts_as_game_activity`
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── message_parsers/       one parser per message with its payload type
├── prelude.rs             the command and payload types, with the value types they carry
├── replies.rs             the reply strings and `reply_for_unhandled`
├── tests/                 unit tests: dispatch, replies, field readers, activity, vehicle ids
└── vehicle_descriptor.rs  `VehicleDescriptor`: the vehicle name and isolation of a vehicle id
```

## How it works

```text
message ──split_fields(TAB)──▶ fields ──first field──▶ message_parsers::parse_* ──▶ GameCommand
                                                            │ field_values (numbers, "true", counts)
                                                            └─▶ Error (Malformed, WrongFieldCount, …)
GameCommand ──reply_for_unhandled──▶ &'static str        Error::reply ──▶ "FAIL"
```

`game_command.rs` splits the message with `tfar_arma_values::split_fields` and matches the first
field against `command_names.rs`; an unknown name is `GameCommand::Unknown` with the whole message.
Each parser in `message_parsers/` takes its fields by slice pattern, so a wrong count is an error
before any field is read, and reads its numbers through `field_values.rs`, which wraps
`tfar_arma_values`' errors with the command and field names. `vehicle_descriptor.rs` reads the
vehicle ids of `POS` and `SPEAKERS`. `replies.rs` maps a parsed command to the reply the original
gives it.

## Public surface

- `GameCommand`, the payload types from `message_parsers/`, `VehicleDescriptor`.
- `replies`, `reply_for_unhandled`, `command_names`, `IN_GAME_TRIGGER_EXEMPT_PREFIX`,
  `message_counts_as_game_activity`, `Error`, `Result`, `CRATE_ROLE`, `prelude`.

## Boundaries

- Depends on: `tfar_arma_values`, `tfar_geometry`, `thiserror`.
- Used by: `tfar_client_state_0_9`, `tfar_plugin_0_9`.
- Rules: production files hold no `unwrap`, `expect`, `panic` or indexing (clippy deny list);
  tests live in `tests/` (`cargo xtask verify test-placement`); a reply string or command name
  changes only with the original's, held by `tests/replies.rs`.
