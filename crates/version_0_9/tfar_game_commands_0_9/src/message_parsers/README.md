# Game message parsers

One parser per 0.9.12 game message, each with the payload type it fills and the field-count rule
`processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp` applies.

## Contents

```text
crates/version_0_9/tfar_game_commands_0_9/src/message_parsers/
├── frequencies.rs          `FREQ` into `FrequencyUpdate`; field 11 is never read
├── mod.rs                  the module lines and the re-exports
├── speakers.rs             `SPEAKERS` into `SpeakerRecord`s, records split at VT, parts at LF
├── tangent.rs              `TANGENT`, `TANGENT_LR`, `TANGENT_DD` and `TangentKind`
├── tests/                  unit tests per message: fields, counts, malformed values
├── text_field_messages.rs  `KILLED`, `RELEASE_ALL_TANGENTS`, `VERSION`, `TRACK`, `IS_SPEAKING`
├── ts_info.rs              `TS_INFO` and `TsInfoQuery`
└── unit_position.rs        `POS` into `PositionUpdate` and `UsableRadios`
```

## How it works

`GameCommand::parse` hands each parser the message's fields, the command name first. A parser
destructures them with a slice pattern of the exact count (`field_values::exact_fields`), so a
short or long message is `Error::WrongFieldCount` before a field is read; `KILLED` takes 2 or
more and `SPEAKERS` 1 or more. Numbers go through `field_values` (rounded like
`parseArmaNumberToInt`, truncated like `std::stoi`, or plain), booleans are the exact text `true`,
and the frequency lists, positions and vehicle ids go through `tfar_arma_values` and
`VehicleDescriptor`. Every failure names the command and the field.

## Boundaries

- Depends on: `field_values`, `command_names`, `error`, `game_command` (the `GameCommand` they
  build), `vehicle_descriptor`; `tfar_arma_values`, `tfar_geometry`.
- Used by: `game_command.rs`, which dispatches to the parsers; `replies.rs` reads `TsInfoQuery`.
- Rules: the field counts and field order are the original's, held by the tests in `tests/`; no
  parser indexes a slice.
