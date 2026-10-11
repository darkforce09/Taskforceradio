//! Parsing of the strings Arma sends: tab-split fields, Arma arrays, frequency lists, positions.
//!
//! **Role:** the parsers for every value shape Arma's `callExtension` strings carry: tab-split
//! fields split exactly like the original getline loop, Arma array literals, frequency lists and
//! positions.
//!
//! **Position:** `crates/foundation`, tier 1, depending on `tfar_geometry`; used by
//! `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_plugin_0_9`,
//! `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
//!
//! **Signals and state:** none; pure parsing, callable from any thread.
//!
//! **Invariants:** a parser never indexes past the tokens it was given; a short message is an
//! `Error`, never a panic.

mod arrays;
mod booleans;
mod error;
mod field_split;
mod frequency_entries;
mod nickname;
mod numbers;
pub mod prelude;
mod radio_subtype;
pub mod separators;
mod stereo_mode;

pub use arrays::{parse_optional_position, parse_position_array, parse_string_array};
pub use booleans::parse_bool;
pub use error::{Error, Result};
pub use field_split::split_fields;
pub use frequency_entries::{FrequencyEntry, FrequencyEntryShape, parse_frequency_entries};
pub use nickname::sanitize_nickname;
pub use numbers::{parse_int, parse_number};
pub use radio_subtype::RadioSubtype;
pub use stereo_mode::StereoMode;
