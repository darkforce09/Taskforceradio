//! The field readers every message parser shares: numbers with the original's rounding or
//! truncation, the `"true"` test, and the error that names the field at fault.

use tfar_foundation::arma_values::{parse_int, parse_number};

use crate::game_commands::error::{Error, Result};

#[cfg(test)]
#[path = "tests/field_values.rs"]
mod tests;

/// The number in `text` (`atof` or `std::stof` in the original), or [`Error::Malformed`] naming
/// `field` of `command`.
pub(crate) fn number(command: &'static str, field: &'static str, text: &str) -> Result<f64> {
    parse_number(text).map_err(|source| Error::Malformed {
        command,
        field,
        source,
    })
}

/// The number in `text` rounded half away from zero (`parseArmaNumberToInt`, `round(atof)`), or
/// [`Error::Malformed`] naming `field` of `command`.
pub(crate) fn rounded_integer(
    command: &'static str,
    field: &'static str,
    text: &str,
) -> Result<i32> {
    parse_int(text).map_err(|source| Error::Malformed {
        command,
        field,
        source,
    })
}

/// The number in `text` truncated toward zero (`std::stoi` and `(int)atof` in the original), or
/// [`Error::Malformed`] naming `field` of `command`, also for a value outside the `i32` range.
pub(crate) fn truncated_integer(
    command: &'static str,
    field: &'static str,
    text: &str,
) -> Result<i32> {
    let truncated = number(command, field, text)?.trunc();
    if truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX) {
        return Err(Error::Malformed {
            command,
            field,
            source: tfar_foundation::arma_values::Error::IntegerOutOfRange {
                value: text.to_owned(),
            },
        });
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the value is integral and checked to lie within the i32 range"
    )]
    Ok(truncated as i32)
}

/// Whether `text` is exactly `true`, as `isTrue` in `reference/tfar_0_9_12/ts/src/plugin.cpp`
/// tests it: any other text, `True` included, is false.
pub(crate) fn is_true(text: &str) -> bool {
    text == "true"
}

/// The `N` fields of a message of `command` that needs exactly `N`, its name included, or
/// [`Error::WrongFieldCount`].
pub(crate) fn exact_fields<'message, const N: usize>(
    command: &'static str,
    fields: &[&'message str],
) -> Result<[&'message str; N]> {
    <[&str; N]>::try_from(fields).map_err(|_| Error::WrongFieldCount {
        command,
        expected: N,
        found: fields.len(),
    })
}
