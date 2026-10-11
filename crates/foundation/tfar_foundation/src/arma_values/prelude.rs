//! The names a caller of `tfar_foundation::arma_values` imports with `use crate::arma_values::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`;
//! the separator constants stay in [`crate::arma_values::separators`] so their short names keep their module.

pub use crate::arma_values::arrays::{
    parse_optional_position, parse_position_array, parse_string_array,
};
pub use crate::arma_values::booleans::parse_bool;
pub use crate::arma_values::field_split::split_fields;
pub use crate::arma_values::frequency_entries::{
    FrequencyEntry, FrequencyEntryShape, parse_frequency_entries,
};
pub use crate::arma_values::nickname::sanitize_nickname;
pub use crate::arma_values::numbers::{parse_int, parse_number};
pub use crate::arma_values::radio_subtype::RadioSubtype;
pub use crate::arma_values::stereo_mode::StereoMode;
