# Arma values

The `tfar_arma_values` crate: parsing of the strings Arma sends: tab-split fields, Arma arrays,
frequency lists, positions. Used by `tfar_antennas_1_0`, `tfar_game_commands_0_9`,
`tfar_game_commands_1_0`, `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`,
`tfar_plugin_commands_1_0`.

## How it works

The parsers for every value shape Arma's `callExtension` strings carry. Tab-split fields are split
exactly like the original getline loop. Arma array literals become positions or string lists, and
frequency lists become typed entries in the shape each mod version accepts. Numbers, booleans,
radio subtypes and nicknames have their own parsers. Every parser returns a typed `Error` instead of
indexing past the end or crashing on a malformed value.

Every parser takes the text as sent and returns a typed value or an `Error` that carries the
failing text. `split_fields` cuts a message into fields with `std::getline` semantics.
`parse_number` reads one Arma number and `parse_int` rounds it. `arrays.rs` strips the brackets of
an Arma array: `parse_position_array` reads three numbers into a `tfar_geometry::Position`, and
`parse_string_array` reads quoted strings, splitting only at commas outside quotes.
`parse_frequency_entries` builds on `parse_string_array`. It splits each string at `|` with the
split of the version the caller names in `FrequencyEntryShape`, then reads volume and stereo mode
with `parse_int` and `StereoMode::from_wire_value`. `RadioSubtype::parse` matches exact names only.
Every module attaches its sibling test file in `tests/` with `#[path]`.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_arma_values  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

- `split_fields` and the `separators` constants (`FIELD`, `SPEAKER_RECORD`, `SPEAKER_PART`,
  `FREQUENCY_PART`, `VEHICLE_PART`, `ASYNC_SUFFIX`).
- `parse_number`, `parse_int` and `parse_bool`.
- `parse_position_array`, `parse_optional_position` and `parse_string_array`.
- `parse_frequency_entries`, `FrequencyEntry`, `FrequencyEntryShape` and `StereoMode`.
- `RadioSubtype` and `sanitize_nickname`.
- `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: `tfar_geometry` (positions, directions and distances); `thiserror` for the error type.
- Used by: `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
- Rules: a parser never indexes past the tokens it was given; a short message is an `Error`, never a
  panic. Tier 1 in `crates/foundation`: every workspace dependency has a lower tier and follows the category edge matrix (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
