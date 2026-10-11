# Foundation

The crate every other crate builds on: `geometry` (positions, directions and distances in Arma's
frame, on `glam`) and `arma_values` (the parsers for the strings Arma sends). Pure functions and
value types, callable from any thread; nothing here knows TeamSpeak, audio or a mod version.

```bash
cargo test -p tfar_foundation
```

Boundaries: depends on no workspace crate (`glam` and `thiserror` only); used by every other
category, both version tracks and their deliverables.

## `geometry`

The `tfar_foundation::geometry` crate: positions, directions and distances in Arma's frame, built on glam. Used
by `tfar_plugin_logic_1_0::antennas`, `tfar_foundation::arma_values`, `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`,
`tfar_plugin_logic_0_9::game_commands`, `tfar_plugin_logic_1_0::game_commands`, `tfar_audio::hrtf`, `tfar_plugin_0_9`,
`tfar_plugin_1_0`, `tfar_plugin_logic_0_9::radio_routing`, `tfar_plugin_logic_1_0::radio_routing`, `tfar_audio::spatial_audio`.

### How it works

The vector and angle helpers every positional crate shares, over `glam`'s `DVec3`: `Position` and
`Direction` in Arma's frame (x east, y north, z up, metres), `Angle` in radians, the underwater
distance rule and the rotation about the vertical axis. Constructors reject non-finite coordinates
with `Error::NonFinite`; every direction is normalised to unit length or kept at zero.

`Position` and `Direction` wrap a `glam::DVec3` in Arma's frame (x east, y north, z up, metres).
Their constructors from `[f64; 3]` reject NaN and infinity with `Error::NonFinite`.
`Direction::from_vector` normalises with `glam`'s `normalize_or_zero`, so every direction has length
1 or is zero. `Position::direction_to` builds on it. `Direction::view_angle` (`atan2(x, y)`, the
compass heading) and `Direction::polar_angle` (`atan2(y, x)`) return an `Angle`, which
`normalized_signed` wraps into (-π, π]. `rotate_about_z` applies the 2D rotation matrix to a
position, and `distance_underwater` clips a segment at z = 0. Every module attaches its sibling test
file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting.

### Public surface

- `Position`: a point in metres, built from `[f64; 3]`, with `distance_to` and `direction_to`.
- `Direction`: a unit vector or zero, from `from_vector` or `from_array`, with `view_angle`
  (`atan2(x, y)`) and `polar_angle` (`atan2(y, x)`).
- `Angle`: radians, with `from_degrees`, `to_degrees` and the (-π, π] wrap `normalized_signed`.
- `distance_underwater(from, to)`: the length of a segment below z = 0.
- `rotate_about_z(position, angle)`: a position turned about the z axis.
- `Error` (`NonFinite`), `Result` and `prelude`.

### Boundaries

- Depends on: no workspace crate; `glam` for the vectors and `thiserror` for the error type.
- Used by: `tfar_plugin_logic_1_0::antennas`, `tfar_foundation::arma_values`, `tfar_plugin_logic_0_9::client_state`,
  `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::game_commands`, `tfar_plugin_logic_1_0::game_commands`, `tfar_audio::hrtf`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_logic_0_9::radio_routing`, `tfar_plugin_logic_1_0::radio_routing`,
  `tfar_audio::spatial_audio`.
- Rules: every direction it returns has unit length or is the zero vector, never NaN.

## `arma_values`

The `tfar_foundation::arma_values` crate: parsing of the strings Arma sends: tab-split fields, Arma arrays,
frequency lists, positions. Used by `tfar_plugin_logic_1_0::antennas`, `tfar_plugin_logic_0_9::game_commands`,
`tfar_plugin_logic_1_0::game_commands`, `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_logic_0_9::plugin_commands`,
`tfar_plugin_logic_1_0::plugin_commands`.

### How it works

The parsers for every value shape Arma's `callExtension` strings carry. Tab-split fields are split
exactly like the original getline loop. Arma array literals become positions or string lists, and
frequency lists become typed entries in the shape each mod version accepts. Numbers, booleans,
radio subtypes and nicknames have their own parsers. Every parser returns a typed `Error` instead of
indexing past the end or crashing on a malformed value.

Every parser takes the text as sent and returns a typed value or an `Error` that carries the
failing text. `split_fields` cuts a message into fields with `std::getline` semantics.
`parse_number` reads one Arma number and `parse_int` rounds it. `arrays.rs` strips the brackets of
an Arma array: `parse_position_array` reads three numbers into a `tfar_foundation::geometry::Position`, and
`parse_string_array` reads quoted strings, splitting only at commas outside quotes.
`parse_frequency_entries` builds on `parse_string_array`. It splits each string at `|` with the
split of the version the caller names in `FrequencyEntryShape`, then reads volume and stereo mode
with `parse_int` and `StereoMode::from_wire_value`. `RadioSubtype::parse` matches exact names only.
Every module attaches its sibling test file in `tests/` with `#[path]`.

### Configuration

None: the crate reads no setting.

### Public surface

- `split_fields` and the `separators` constants (`FIELD`, `SPEAKER_RECORD`, `SPEAKER_PART`,
  `FREQUENCY_PART`, `VEHICLE_PART`, `ASYNC_SUFFIX`).
- `parse_number`, `parse_int` and `parse_bool`.
- `parse_position_array`, `parse_optional_position` and `parse_string_array`.
- `parse_frequency_entries`, `FrequencyEntry`, `FrequencyEntryShape` and `StereoMode`.
- `RadioSubtype` and `sanitize_nickname`.
- `Error`, `Result` and `prelude`.

### Boundaries

- Depends on: `tfar_foundation::geometry` (positions, directions and distances); `thiserror` for the error type.
- Used by: `tfar_plugin_logic_1_0::antennas`, `tfar_plugin_logic_0_9::game_commands`, `tfar_plugin_logic_1_0::game_commands`,
  `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_plugin_logic_0_9::plugin_commands`, `tfar_plugin_logic_1_0::plugin_commands`.
- Rules: a parser never indexes past the tokens it was given; a short message is an `Error`, never a
  panic.
