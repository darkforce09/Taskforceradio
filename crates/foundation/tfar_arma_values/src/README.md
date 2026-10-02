# Arma values source

The source of `tfar_arma_values`: one module per value shape the game sends, from tab-split fields
to frequency lists, plus the separator constants they share.

## Contents

```text
crates/foundation/tfar_arma_values/src/
├── arrays.rs             `parse_position_array`, `parse_optional_position`, `parse_string_array`
├── booleans.rs           `parse_bool`: `true` or `false` in any letter case
├── crate_role.rs         `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs              `Error`, the `thiserror` enum, and `Result`
├── field_split.rs        `split_fields` (the `std::getline` split) and the crate-private 1.0 split
├── frequency_entries.rs  `parse_frequency_entries`, `FrequencyEntry` and `FrequencyEntryShape`
├── lib.rs                the crate root: module header, `mod` lines and re-exports
├── nickname.rs           `sanitize_nickname`: edge spaces of a nickname become `_`
├── numbers.rs            `parse_number` and `parse_int`: Arma numbers, rounded like `round(atof)`
├── prelude.rs            the public types and functions for glob import
├── radio_subtype.rs      `RadioSubtype`: exact-name subtypes and their sound folders
├── separators.rs         the wire separators: `FIELD`, `SPEAKER_RECORD`, `FREQUENCY_PART` and more
├── stereo_mode.rs        `StereoMode`: both, left or right ear, from the wire value
└── tests/                unit tests, one file per module
```

## How it works

Every parser takes the text as sent and returns a typed value or an `Error` that carries the
failing text. `split_fields` cuts a message into fields with `std::getline` semantics.
`parse_number` reads one Arma number and `parse_int` rounds it. `arrays.rs` strips the brackets of
an Arma array: `parse_position_array` reads three numbers into a `tfar_geometry::Position`, and
`parse_string_array` reads quoted strings, splitting only at commas outside quotes.
`parse_frequency_entries` builds on `parse_string_array`. It splits each string at `|` with the
split of the version the caller names in `FrequencyEntryShape`, then reads volume and stereo mode
with `parse_int` and `StereoMode::from_wire_value`. `RadioSubtype::parse` matches exact names only.
Every module attaches its sibling test file in `tests/` with `#[path]`.

## Public surface

- `split_fields(input, separator)` and the module `separators`: `FIELD`, `SPEAKER_RECORD`,
  `SPEAKER_PART`, `FREQUENCY_PART`, `VEHICLE_PART`, `ASYNC_SUFFIX`.
- `parse_number`, `parse_int`, `parse_bool`.
- `parse_position_array`, `parse_optional_position`, `parse_string_array`.
- `parse_frequency_entries(text, shape)`, `FrequencyEntry`, `FrequencyEntryShape`
  (`ThreeParts`, `ThreeOrFourParts`), `StereoMode` (`Both`, `Left`, `Right`, `from_wire_value`,
  `wire_value`).
- `RadioSubtype` (`parse`, `as_str`, `sound_folder`) and `sanitize_nickname`.
- `Error`, `Result`, `CRATE_ROLE` and `prelude`, re-exported at the crate root.

## Boundaries

- Depends on: `tfar_geometry` (`Position` and its `Error`); `thiserror`.
- Used by: `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
- Rules: `split_fields` keeps the getline table (`tests/field_split.rs`); a subtype matches only
  its exact name (`tests/radio_subtype.rs`); a malformed number is an `Error`, never a crash
  (`tests/numbers.rs`); production files hold no `unwrap`, `expect`, `panic` or indexing (clippy
  deny list); tests live in `tests/` (`cargo xtask verify test-placement`).
