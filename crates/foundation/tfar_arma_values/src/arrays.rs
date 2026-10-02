//! Arma array literals as `str()` writes them: positions (`[x,y,z]`) and string arrays
//! (`["a","b"]`).
//!
//! **Role:** turn the array text the SQF side sends into a `Position` or a list of strings, with a
//! typed error for every malformed shape.
//!
//! **Position:** a module of `tfar_arma_values`; the frequency list parser builds on
//! `parse_string_array`.
//!
//! **Signals and state:** none; pure functions, callable from any thread.
//!
//! **Invariants:** the text is walked by characters and `str` methods only, never by index; a
//! string element is read up to its matching closing quote, so a `,` inside quotes never splits.

use core::iter::Peekable;
use core::str::Chars;

use tfar_geometry::Position;

use crate::error::{Error, Result};
use crate::numbers::parse_number;

#[cfg(test)]
#[path = "tests/arrays.rs"]
mod tests;

/// The text between the outer `[` and `]` of `text`, with spaces around the brackets allowed.
fn array_interior(text: &str) -> Result<&str> {
    text.trim_matches(' ')
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| Error::NotAnArray {
            value: text.to_owned(),
        })
}

/// The position in `text`, an Arma array of three numbers such as `[1234.5,6789,12.25]`. Spaces
/// are allowed around the brackets and the numbers, and the numbers may use scientific notation
/// (`1.5e+006`).
///
/// # Errors
///
/// [`Error::NotAnArray`] without the brackets, [`Error::WrongElementCount`] for any count but 3
/// (`[]` included; [`parse_optional_position`] accepts it), and [`Error::MalformedNumber`] for a
/// coordinate that is not a finite number.
pub fn parse_position_array(text: &str) -> Result<Position> {
    let interior = array_interior(text)?;
    let elements: Vec<&str> = if interior.trim_matches(' ').is_empty() {
        Vec::new()
    } else {
        interior.split(',').collect()
    };
    let [x, y, z] = elements.as_slice() else {
        return Err(Error::WrongElementCount {
            expected: 3,
            found: elements.len(),
            value: text.to_owned(),
        });
    };
    let coordinates = [parse_number(x)?, parse_number(y)?, parse_number(z)?];
    Ok(Position::from_array(coordinates)?)
}

/// The position in `text` as [`parse_position_array`] reads it, or `None` for the empty array
/// `[]` the SQF side sends when there is no position.
///
/// # Errors
///
/// As [`parse_position_array`] for any text but an empty array.
pub fn parse_optional_position(text: &str) -> Result<Option<Position>> {
    if array_interior(text)?.trim_matches(' ').is_empty() {
        Ok(None)
    } else {
        parse_position_array(text).map(Some)
    }
}

/// The strings in `text`, an Arma array of strings such as `["a","b"]`: the outer brackets are
/// stripped, the elements split at the commas outside quotes, and each element's quotes removed.
/// A doubled quote inside a string (`""`, Arma's escape) becomes one quote. Spaces are allowed
/// around the brackets, the elements and the commas. `[]` gives an empty list.
///
/// # Errors
///
/// [`Error::NotAnArray`] without the brackets, [`Error::UnquotedElement`] for an element that is
/// not a quoted string (an empty element after a trailing comma included), and
/// [`Error::UnterminatedString`] for a string with no closing quote.
pub fn parse_string_array(text: &str) -> Result<Vec<String>> {
    let interior = array_interior(text)?;
    let mut characters = interior.chars().peekable();
    let mut strings = Vec::new();
    skip_spaces(&mut characters);
    if characters.peek().is_none() {
        return Ok(strings);
    }
    loop {
        let unquoted = || Error::UnquotedElement {
            index: strings.len(),
            value: text.to_owned(),
        };
        if characters.next() != Some('"') {
            return Err(unquoted());
        }
        let element =
            read_quoted_string(&mut characters).ok_or_else(|| Error::UnterminatedString {
                value: text.to_owned(),
            })?;
        skip_spaces(&mut characters);
        match characters.next() {
            None => {
                strings.push(element);
                return Ok(strings);
            }
            Some(',') => {
                strings.push(element);
                skip_spaces(&mut characters);
            }
            Some(_) => return Err(unquoted()),
        }
    }
}

/// Advances past any spaces.
fn skip_spaces(characters: &mut Peekable<Chars<'_>>) {
    while characters.next_if_eq(&' ').is_some() {}
}

/// The rest of a quoted string whose opening quote is already read, up to and past its closing
/// quote, with `""` read as one quote; `None` when the text ends first.
fn read_quoted_string(characters: &mut Peekable<Chars<'_>>) -> Option<String> {
    let mut element = String::new();
    loop {
        match characters.next()? {
            '"' if characters.next_if_eq(&'"').is_some() => element.push('"'),
            '"' => return Some(element),
            other => element.push(other),
        }
    }
}
