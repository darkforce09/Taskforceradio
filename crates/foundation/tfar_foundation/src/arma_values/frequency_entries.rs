//! Frequency lists: the arrays of `frequency|volume|stereo[|classname]` strings the SQF side
//! sends for the radios a player has on.
//!
//! **Role:** turn a frequency list into typed entries, with the part split and the accepted part
//! counts of the mod version the caller names.
//!
//! **Position:** a module of `tfar_foundation::arma_values`, built on `parse_string_array`, `parse_int` and
//! `StereoMode`; the version crates call it with their own `FrequencyEntryShape`.
//!
//! **Signals and state:** none; pure functions, callable from any thread.
//!
//! **Invariants:** an entry with the wrong number of parts is skipped, as both originals skip it;
//! a malformed number in an entry of the right shape is an error for the whole list, never a
//! silent 0.

use crate::arma_values::arrays::parse_string_array;
use crate::arma_values::error::Result;
use crate::arma_values::field_split::{split_fields, split_keeping_every_field};
use crate::arma_values::numbers::parse_int;
use crate::arma_values::separators::FREQUENCY_PART;
use crate::arma_values::stereo_mode::StereoMode;

#[cfg(test)]
#[path = "tests/frequency_entries.rs"]
mod tests;

/// The entry shapes a mod version accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrequencyEntryShape {
    /// TFAR 0.9.12: exactly `frequency|volume|stereo`, its parts split like `std::getline` (a
    /// trailing empty part is dropped), as `parseFrequencies` in
    /// `reference/tfar_0_9_12/ts/src/plugin.cpp`.
    ThreeParts,
    /// TFAR 1.0: `frequency|volume|stereo` or `frequency|volume|stereo|classname`, every part
    /// kept, as `parseFrequencies` in `reference/tfar_1_0/ts/src/helpers.cpp`.
    ThreeOrFourParts,
}

/// One radio a player has on, from a frequency list.
#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyEntry {
    /// The frequency as the game writes it (`"87.6"`, or `"87.6_2"` with a channel suffix); kept
    /// as text because it is compared, never computed with.
    pub frequency: String,
    /// The radio volume, rounded to the nearest integer as both originals round it.
    pub volume: f64,
    /// The ear the radio plays into.
    pub stereo_mode: StereoMode,
    /// The radio's class name, sent by 1.0 only; `None` when absent or empty.
    pub radio_classname: Option<String>,
}

/// The entries of the frequency list `text` (`["87.6|7|0","45|5|1|tf_rt1523g"]`), in list order,
/// keeping only entries of `shape`.
///
/// # Errors
///
/// The errors of [`parse_string_array`](crate::arma_values::parse_string_array) for a list that is not an
/// array of strings, and those of [`parse_int`](crate::arma_values::parse_int) for a volume or stereo mode
/// that is not a number.
pub fn parse_frequency_entries(
    text: &str,
    shape: FrequencyEntryShape,
) -> Result<Vec<FrequencyEntry>> {
    let mut entries = Vec::new();
    for element in parse_string_array(text)? {
        let parts = match shape {
            FrequencyEntryShape::ThreeParts => split_fields(&element, FREQUENCY_PART),
            FrequencyEntryShape::ThreeOrFourParts => {
                split_keeping_every_field(&element, FREQUENCY_PART)
            }
        };
        let (frequency, volume, stereo, classname) = match (shape, parts.as_slice()) {
            (_, [frequency, volume, stereo]) => (frequency, volume, stereo, None),
            (FrequencyEntryShape::ThreeOrFourParts, [frequency, volume, stereo, classname]) => {
                (frequency, volume, stereo, Some(classname))
            }
            _ => continue,
        };
        entries.push(FrequencyEntry {
            frequency: (*frequency).to_owned(),
            volume: f64::from(parse_int(volume)?),
            stereo_mode: StereoMode::from_wire_value(parse_int(stereo)?),
            radio_classname: classname
                .filter(|name| !name.is_empty())
                .map(|name| (*name).to_owned()),
        });
    }
    Ok(entries)
}
