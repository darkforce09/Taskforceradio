//! The diver radio's range and the underwater part of a path.

#[cfg(test)]
#[path = "tests/underwater.rs"]
mod tests;

/// The diver radio range at the shortest, in metres (`DD_MIN_DISTANCE`).
const DIVER_RADIO_MINIMUM_DISTANCE: f64 = 70.0;

/// The diver radio range at the longest, in metres (`DD_MAX_DISTANCE`).
const DIVER_RADIO_MAXIMUM_DISTANCE: f64 = 300.0;

/// How far a diver radio carries under water at `waves_level` (0 calm to 1 rough):
/// `70 + 230 · (1 − waves)` (`helpers::distanceForDiverRadio`,
/// `reference/tfar_1_0/ts/src/helpers.cpp:352-354`).
pub fn diver_radio_distance(waves_level: f64) -> f64 {
    (DIVER_RADIO_MAXIMUM_DISTANCE - DIVER_RADIO_MINIMUM_DISTANCE)
        .mul_add(1.0 - waves_level, DIVER_RADIO_MINIMUM_DISTANCE)
}

/// `distance` with its underwater part `underwater` stretched to a radio of `range` metres:
/// `underwater · range / diver_distance + (distance − underwater)`, how the original weighs a
/// path that runs under water (`clientData.cpp:199-200`, `:210-212`, `plugin.cpp:647`).
pub fn underwater_weighted_distance(
    distance: f64,
    underwater: f64,
    range: f64,
    waves_level: f64,
) -> f64 {
    underwater.mul_add(
        range / diver_radio_distance(waves_level),
        distance - underwater,
    )
}
