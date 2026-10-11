//! `parse_bool`: Arma booleans as `str()` writes them.

use crate::arma_values::error::{Error, Result};

#[cfg(test)]
#[path = "tests/booleans.rs"]
mod tests;

/// The boolean in `text`: `true` or `false` in any letter case, with leading and trailing spaces
/// allowed.
///
/// # Errors
///
/// [`Error::NotABoolean`] for any other text. The originals compare with `"true"` and treat
/// everything else as false, so a caller that follows them maps this error to `false`
/// (`parse_bool(text).unwrap_or(false)`); a caller that must reject a malformed message keeps the
/// error.
pub fn parse_bool(text: &str) -> Result<bool> {
    let trimmed = text.trim_matches(' ');
    if trimmed.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(Error::NotABoolean {
            value: text.to_owned(),
        })
    }
}
