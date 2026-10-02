//! `distance_underwater`: how much of a straight path between two positions runs below the sea
//! surface.

use crate::position::Position;

#[cfg(test)]
#[path = "tests/underwater_distance.rs"]
mod tests;

/// The length, in metres, of the part of the straight segment from `from` to `to` that lies below
/// the sea surface (z < 0).
///
/// Both ends at or above the surface give 0; both at or below give the whole length (a segment
/// running along z = 0 gives 0); a segment crossing the surface gives the length of its
/// submerged part, whichever end is the submerged one. The result is symmetric in its arguments.
/// The rule follows `distanceUnderwater` in `reference/tfar_1_0/ts/src/datatypes.cpp`.
pub fn distance_underwater(from: Position, to: Position) -> f64 {
    let from_height = from.z();
    let to_height = to.z();
    if from_height >= 0.0 && to_height >= 0.0 {
        return 0.0;
    }
    let length = from.distance_to(to);
    if from_height <= 0.0 && to_height <= 0.0 {
        return length;
    }
    // The ends lie strictly on opposite sides of z = 0, so the heights differ and the fraction of
    // the segment below the surface is the submerged end's depth over the height difference.
    let (submerged_depth, height_difference) = if from_height < 0.0 {
        (-from_height, to_height - from_height)
    } else {
        (-to_height, from_height - to_height)
    };
    length * (submerged_depth / height_difference)
}
