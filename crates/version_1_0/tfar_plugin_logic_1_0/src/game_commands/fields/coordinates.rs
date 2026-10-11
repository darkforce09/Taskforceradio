//! Coordinate triples as the 1.0 plugin reads them: `[x,y,z]`, with two or four elements accepted.
//!
//! **Role:** read the position, view direction and velocity texts of the game messages with the
//! element counts `Vector3D(std::string_view)` accepts
//! (`reference/tfar_1_0/ts/src/datatypes.cpp:31-51`), but reject what it silently reads as zero.
//!
//! **Position:** a module of the crate's `fields`; built on `tfar_foundation::arma_values::parse_number` and
//! `tfar_foundation::geometry::Position`; used by the `POS`, `SPEAKERS` and `RadioTwrAdd` parsers and the vehicle
//! descriptor.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** two elements give `z = 0`; three or four give the first three (older position
//! overrides send a fourth); any other count, or an element that is not a finite number, is
//! `None`, never the zero vector.

use tfar_foundation::arma_values::parse_number;
use tfar_foundation::geometry::Position;

#[cfg(test)]
#[path = "tests/coordinates.rs"]
mod tests;

/// The coordinates in `text`: `[x,y]`, `[x,y,z]` or `[x,y,z,w]`, the brackets optional, spaces
/// around the numbers allowed. `None` for any other shape or a malformed number.
pub(crate) fn parse_coordinates(text: &str) -> Option<[f64; 3]> {
    let interior = match text.strip_prefix('[') {
        Some(rest) => rest.strip_suffix(']')?,
        None => text,
    };
    let elements: Vec<&str> = interior.split(',').collect();
    let (x, y, z) = match elements.as_slice() {
        [x, y] => (x, y, None),
        [x, y, z] | [x, y, z, _] => (x, y, Some(z)),
        _ => return None,
    };
    let z = match z {
        Some(z) => parse_number(z).ok()?,
        None => 0.0,
    };
    Some([parse_number(x).ok()?, parse_number(y).ok()?, z])
}

/// The position of a speaker record (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:517`):
/// `Ok(None)` for `[]` and for the all-zero position, both of which the original reads as "on its
/// owner" (`Vector3D::isNull`, `datatypes.cpp:116-120`); `Err(())` for a malformed text.
pub(crate) fn parse_speaker_position(text: &str) -> Result<Option<Position>, ()> {
    if text.trim_matches(' ') == "[]" {
        return Ok(None);
    }
    let coordinates = parse_coordinates(text).ok_or(())?;
    #[expect(
        clippy::float_cmp,
        reason = "the original tests each coordinate for exactly zero (`Vector3D::isNull`)"
    )]
    let is_null = coordinates == [0.0; 3];
    if is_null {
        return Ok(None);
    }
    Position::from_array(coordinates).map(Some).map_err(|_| ())
}
