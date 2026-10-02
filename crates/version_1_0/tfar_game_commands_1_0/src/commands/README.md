# Game command payloads

The typed payload of each 1.0 game command with fields, and the parser that reads it from a
message's tokens.

## Contents

```text
crates/version_1_0/tfar_game_commands_1_0/src/commands/
├── frequency_report.rs  `FrequencyReport` (`FREQ`): the local player's radios and state
├── mod.rs               the module lines and re-exports
├── position_update.rs   `PositionUpdate` (`POS`): one player's position and state
├── radio_towers.rs      `TowerRecord` (`RadioTwrAdd`) and the ids of `RadioTwrDel`
├── set_config.rs        `ConfigAssignment` (`SETCFG`): key, value and type tag
├── speaker_record.rs    `SpeakerRecord` (`SPEAKERS`): the radios playing over loudspeakers
├── tangent_press.rs     `TangentPress` and `TangentKind` (`TANGENT`, `TANGENT_LR`, `TANGENT_DD`)
├── tests/               unit tests of each parser with the messages the addon builds
├── tracking_event.rs    `TrackingEvent` (`TRACK`): action, player id and custom variables
└── ts_info.rs           `TsInfoQuery` (`TS_INFO`): the query and its fixed answers
```

## How it works

Each parser takes the `MessageFields` of one message, checks its token count and reads the fields
at the original's indices (`tokens[i]` of `reference/tfar_1_0/ts/src/CommandProcessor.cpp`), so a
field's index in the code is its index in the C++. Lists inside a field (frequency entries,
speaker records, tower records, custom variables) are split with their own separators; a record
of the wrong shape is skipped where the original skips it (and, for speakers, where the original
abandons the rest of the list), and a malformed number rejects the message.

## Boundaries

- Depends on: the crate's `fields/` readers, `CommandName`, `Error` and `replies`;
  `tfar_arma_values` (frequency entries, subtypes, numbers, separators); `tfar_geometry`;
  `tfar_plugin_settings_1_0` (`TypeTag`).
- Used by: the crate's `parse.rs` and `game_command.rs`; the payload types are public.
- Rules: the range of a tangent is kept as sent beside its rounded value, so a forwarded press
  carries the game's bytes (`a_press_with_a_class_name_reads_six_tokens`); subtypes match by exact
  name (`the_subtype_is_matched_by_exact_name_not_by_length`).
