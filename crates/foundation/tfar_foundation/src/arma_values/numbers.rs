//! `parse_number` and `parse_int`: Arma numbers as `str()` writes them.

use crate::arma_values::error::{Error, Result};

#[cfg(test)]
#[path = "tests/numbers.rs"]
mod tests;

/// The number in `text`, as Arma's `str()` writes it (`12`, `-0.5`, `1.5e+006`), with leading and
/// trailing spaces allowed.
///
/// # Errors
///
/// [`Error::MalformedNumber`] when the text is empty, holds anything but one decimal number, or
/// names a non-finite value (`inf`, `nan`).
pub fn parse_number(text: &str) -> Result<f64> {
    let malformed = || Error::MalformedNumber {
        value: text.to_owned(),
    };
    let number: f64 = text.trim_matches(' ').parse().map_err(|_| malformed())?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(malformed())
    }
}

/// The number in `text` rounded to the nearest integer, halves away from zero: `round(atof(text))`
/// as `parseArmaNumberToInt` in `reference/tfar_0_9_12/ts/src/plugin.cpp` computes it.
///
/// # Errors
///
/// [`Error::MalformedNumber`] as [`parse_number`] reports it, and [`Error::IntegerOutOfRange`]
/// when the rounded value does not fit an `i32`.
pub fn parse_int(text: &str) -> Result<i32> {
    let rounded = parse_number(text)?.round();
    if rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
        return Err(Error::IntegerOutOfRange {
            value: text.to_owned(),
        });
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the value is integral and checked to lie within the i32 range"
    )]
    Ok(rounded as i32)
}
