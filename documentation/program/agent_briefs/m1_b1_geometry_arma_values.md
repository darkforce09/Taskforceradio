**Status:** live

# M1 · B1 · `tfar_geometry` and `tfar_arma_values`

You are agent B1, the foundations author for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first, then `/documentation/standards/coding_standards.md`.

## Goal
Replace each skeleton with the real crate. Every function is pure, documented and tested. No panics and no
indexing, as the workspace lints require. Add `glam` (already pinned in `[workspace.dependencies]`) to
`tfar_geometry`. Every number format below has to match what the SQF side sends; the behaviour specs in
`/documentation/program/behaviour_specs/` are the source.

## `tfar_geometry` (crates/foundation/tfar_geometry)
- **Types:** the newtypes `Position` (Arma world or relative metres, `glam::DVec3`) and `Direction` (unit vector, `DVec3`), both `Copy`, `Debug`, `PartialEq`.
  - Constructors from `[f64; 3]`.
  - `Direction::from_vector`, which normalises and keeps zero as zero (F-009). The original normalisation is at `reference/tfar_1_0/ts/src/datatypes.cpp:105`.
- **Distances and directions:**
  - `Position::distance_to`.
  - `Position::direction_to(other) -> Direction`, normalised correctly.
  - `distance_underwater(from, to) -> f64`: the length of the straight segment below z = 0 (`reference/tfar_1_0/ts/src/datatypes.cpp:150`). Cover the cases: both above, both below, a crossing either way, and a horizontal segment at z < 0.
- **Angles:** an `Angle` newtype in radians, with `from_degrees` and `to_degrees`, plus:
  - `Direction::view_angle()` = `atan2(x, y)`, the compass heading the 1.0 view direction uses (`reference/tfar_1_0/ts/src/datatypes.cpp`);
  - `polar_angle()` = `atan2(y, x)`;
  - a `normalized_signed` wrap into (-π, π], for the corrected ILD wrap (the 1.0 original wraps only above 2π: `helpers.cpp:12`).
- **Rotation:** `rotate_about_z(position, angle)` with `x' = x cos θ − y sin θ`, `y' = x sin θ + y cos θ` (`reference/tfar_0_9_12/ts/src/Clunk.h`).
- **Errors and tests:** keep `error.rs` with the variants actually needed (likely none fallible; then the `Error` enum holds `NonFinite` for constructors given NaN/inf, and the constructors return `Result`). Tests use sibling files, one per module.

## `tfar_arma_values` (crates/foundation/tfar_arma_values)
- **`split_fields(input, separator) -> Vec<&str>`** with exact `std::getline` semantics (`reference/tfar_0_9_12/ts/src/plugin.cpp:801`):
  - an empty input gives 0 fields;
  - a trailing empty field is dropped;
  - inner empty fields are kept.

  Test against the C++ semantics table: `""`, `"a"`, `"a\t"`, `"a\t\tb"`, `"\ta"`, `"a\t\t"`.
- **Separator constants:** `FIELD` `\t`, `SPEAKER_RECORD` `\x0B`, `SPEAKER_PART` `\n`, `FREQUENCY_PART` `|`, `VEHICLE_PART` `\x10`, `ASYNC_SUFFIX` `~`.
- **Numbers:**
  - `parse_number(&str) -> Result<f64>`. Arma `str()` numbers, with leading/trailing spaces allowed. A malformed value is an `Error`; it never throws or crashes (F-003, 0.9 `std::stof`).
  - `parse_int` rounds like `round(atof)` (`reference/tfar_0_9_12/ts/src/plugin.cpp:112-120`).
- **Booleans:** `parse_bool` takes `true`/`false` case-insensitively. For any other value: return an `Error` and document it. The original 0.9 code compared to `"true"`, so callers default to false.
- **Arrays and lists:**
  - `parse_position_array("[x,y,z]") -> Result<Position>` (Arma `str` of a 3-array, spaces allowed, scientific notation allowed). `"[]"` gives `Ok(None)` in a separate `parse_optional_position`.
  - `parse_string_array(r#"["a","b"]"#) -> Result<Vec<String>>`: strip the outer brackets, split on `,` outside quotes, strip the quotes (`reference/tfar_1_0/ts/src/helpers.cpp:307`). An empty array gives an empty vec.
  - `parse_frequency_entries(list) -> Result<Vec<FrequencyEntry>>`, where `FrequencyEntry { frequency: String, volume: f64, stereo_mode: StereoMode, radio_classname: Option<String> }`. Each string is split on `|` into 3 parts (0.9) or 4 parts (1.0, classname). The caller passes `FrequencyEntryShape::{ThreeParts, ThreeOrFourParts}`, so the version crates choose; entries of the wrong shape are skipped, matching the originals.
  - `StereoMode` from the rounded number: 0 = both, 1 = left, 2 = right, anything else = both.
- **`RadioSubtype`:** an enum `Digital` (`digital`), `DigitalLongRange` (`digital_lr`), `DiverDigital` (`dd`), `Airborne` (`airborne`), `Phone` (`phone`), `Unknown(String)`.
  - `parse` matches the **exact name** (F-010; the original matched by length at `reference/tfar_1_0/ts/src/plugin.cpp:116`).
  - Also `as_str()` and the sound-folder mapping `sound_folder() -> Option<&'static str>`: `digital` → `sw`, `digital_lr` → `lr`, `dd` → `dd`, `airborne` → `ab`, else `None`.
- **Nicknames:** `sanitize_nickname`: leading and trailing spaces become `_` (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:654`).

## Owned files
`crates/foundation/tfar_geometry/**`, `crates/foundation/tfar_arma_values/**`. Keep the READMEs' Contents
trees exact, and update "Public surface" to the real API.

## Checks (each alone, `CARGO_BUILD_JOBS=4`, logs to `target/agent-logs/b1-*.log`)
`cargo fmt -p tfar_geometry -p tfar_arma_values`, `cargo clippy -p tfar_geometry -p tfar_arma_values --all-targets -- -D warnings`,
`cargo test -p tfar_geometry -p tfar_arma_values`. If `tools/xtask` exists, also run `cargo xtask verify all`; report
violations outside your files instead of fixing them.

## Report additions
Public API list (one line per item) and which F-ids your tests pin, by test name.

Budget: M (250k tokens). Stop there and report what is done and what is not.
