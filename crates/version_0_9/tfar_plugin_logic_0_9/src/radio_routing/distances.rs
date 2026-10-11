//! The distances the reception rules compare: raw, effective (terrain and multiplier), the diver
//! radio's reach and the effect error.

use crate::client_state::prelude::TangentKind;
use crate::client_state::{ClientData, ServerRadioData};

#[cfg(test)]
#[path = "tests/distances.rs"]
mod tests;

/// The diver radio's reach on a calm sea, in metres (`DD_MAX_DISTANCE`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:58`).
pub const DIVER_RADIO_MAX_DISTANCE: f64 = 300.0;

/// The diver radio's reach on the roughest sea, in metres (`DD_MIN_DISTANCE`, plugin.cpp:57).
pub const DIVER_RADIO_MIN_DISTANCE: f64 = 70.0;

/// The distance over which terrain interception counts once more (plugin.cpp:962).
const TERRAIN_DISTANCE_SCALE: f64 = 2000.0;

/// The straight-line distance in metres between the local player and `client`.
pub fn raw_distance(server: &ServerRadioData, client: &ClientData) -> f64 {
    server.my_position.distance_to(client.position)
}

/// The distance a radio signal from `client` travels to the local player
/// (`effectiveDistance`, plugin.cpp:954): with `d` the raw distance, `ti` the client's terrain
/// interception and `c` the terrain coefficient, `(d + ti·c + ti·c·d / 2000)` times the
/// receiving distance multiplier.
pub fn effective_distance(server: &ServerRadioData, client: &ClientData) -> f64 {
    let distance = raw_distance(server, client);
    let terrain = f64::from(client.terrain_interception) * server.terrain_intersection_coefficient;
    (distance + terrain + terrain * distance / TERRAIN_DISTANCE_SCALE)
        * server.receiving_distance_multiplier
}

/// The diver radio's reach at sea state `waves_level` (`distanceForDiverRadio`, plugin.cpp:932):
/// `70 + 230 · (1 − waves_level)` metres.
pub fn diver_radio_distance(waves_level: f64) -> f64 {
    DIVER_RADIO_MIN_DISTANCE
        + (DIVER_RADIO_MAX_DISTANCE - DIVER_RADIO_MIN_DISTANCE) * (1.0 - waves_level)
}

/// The raw effect error of a signal over the radio family `over` that travelled `distance`
/// metres (`effectErrorFromDistance`, plugin.cpp:941): the distance over the sender's radio
/// range `transmitter_range` for short and long range, over the diver radio's reach at
/// `waves_level` for a diver radio. A zero range gives an infinite (or, at distance 0, NaN)
/// error, which the radio effects clamp.
pub fn effect_error_from_distance(
    over: TangentKind,
    distance: f64,
    transmitter_range: i32,
    waves_level: f64,
) -> f64 {
    let reach = match over {
        TangentKind::ShortRange | TangentKind::LongRange => f64::from(transmitter_range),
        TangentKind::DiverDigital => diver_radio_distance(waves_level),
    };
    distance / reach
}
