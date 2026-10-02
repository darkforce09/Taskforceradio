//! `error_level_from_distance`: the 0.9.12 curve from a raw distance error to the effect's error
//! level (`calcErrorLevel`, `reference/tfar_0_9_12/ts/src/RadioEffect.h:144`).
//!
//! **Role:** maps the distance error (distance over range) through the 0.9.12 table by linear
//! interpolation between its entries, one entry per tenth.
//!
//! **Position:** called by the speaker effects in `effects/` when their error level is set, and
//! exported for the client state that wants the level a radio is heard at.
//!
//! **Signals and state:** a pure function; called from whichever thread sets the error level.
//!
//! **Invariants:** the result rises with the distance error, never falls; it is continuous; it
//! equals the table entry at every tenth; the input is clamped to the table's domain `[0, 1.2]`,
//! and `NaN` reads as 0, so the result always lies between the first and last entries.

#[cfg(test)]
#[path = "tests/error_level.rs"]
mod tests;

/// The 0.9.12 table, one entry per tenth of distance error from 0 to 1.2, as `RadioEffect.h`
/// writes it (its last entry is 0.999).
const ERROR_LEVEL_TABLE: [f64; 13] = [
    0.0,
    0.150_000_006,
    0.300_000_012,
    0.600_000_024,
    0.899_999_976,
    0.950_000_048,
    0.960_000_038,
    0.970_000_029,
    0.980_000_019,
    0.995_000_005,
    0.997_799_993,
    0.998_799_993,
    0.999,
];

/// The index of the last segment, the one between the last two table entries.
const LAST_SEGMENT: f64 = 11.0;

/// The highest distance error the table covers: its last entry sits at 1.2.
const TABLE_END: f64 = 1.2;

/// The effect error level for `distance_error`, the sender's distance over the radio's range.
///
/// Inside each tenth the level runs linearly from one table entry to the next:
/// `from + (to − from) · (10 · e − part)`, with `part` the tenth `e` falls in. The input is
/// clamped to `[0, 1.2]` and `NaN` reads as 0.
pub fn error_level_from_distance(distance_error: f64) -> f64 {
    let clamped = if distance_error.is_nan() {
        0.0
    } else {
        distance_error.clamp(0.0, TABLE_END)
    };
    let scaled = clamped * 10.0;
    let part = scaled.floor().min(LAST_SEGMENT);
    let Some(&[from, to]) = ERROR_LEVEL_TABLE.get(segment_index(part)..=segment_index(part) + 1)
    else {
        return 0.0;
    };
    from + (to - from) * (scaled - part)
}

/// The table index of a segment number already floored and clamped to `0..=11`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the segment number is a whole number in 0..=11, so the cast is exact"
)]
fn segment_index(part: f64) -> usize {
    part as usize
}
