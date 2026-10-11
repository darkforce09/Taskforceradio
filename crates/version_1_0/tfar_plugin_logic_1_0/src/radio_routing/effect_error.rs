//! The error level a radio effect gets from distance and antenna loss.

use crate::antennas::loss_or_no_connection;
use crate::client_state::SendingRadio;
use tfar_foundation::geometry::{Position, distance_underwater};

use crate::radio_routing::underwater::underwater_weighted_distance;

#[cfg(test)]
#[path = "tests/effect_error.rs"]
mod tests;

/// The distance error of a radio of `range` metres heard at `distance` over `over`: the
/// distance over the range for a short- or long-range radio, the distance itself for a diver
/// radio (`effectErrorFromDistance`, `reference/tfar_1_0/ts/src/plugin.cpp:58-66`).
pub fn effect_error_from_distance(over: SendingRadio, distance: f64, range: i32) -> f64 {
    let maximum = match over {
        SendingRadio::ShortRange | SendingRadio::LongRange => f64::from(range),
        SendingRadio::DiverDigital => 1.0,
    };
    distance / maximum
}

/// The error level of a radio effect: the smaller of the distance error and the antenna loss,
/// a missing connection read as the original's 7 (`std::min(antennaConnection.getLoss(), …)`,
/// `plugin.cpp:636`, `:652`, `:656`, `:700`).
pub fn capped_by_antenna(distance_error: f64, antenna_loss: Option<f64>) -> f64 {
    distance_error.min(loss_or_no_connection(antenna_loss))
}

/// The error level of a diver radio heard from `sender` at `listener` with `range` metres: the
/// path's underwater part weighed at the diver range, over the range, capped by the antenna
/// loss (`plugin.cpp:643-650`).
pub fn underwater_effect_error(
    listener: Position,
    sender: Position,
    range: i32,
    waves_level: f64,
    antenna_loss: Option<f64>,
) -> f64 {
    let range = f64::from(range);
    let weighted = underwater_weighted_distance(
        listener.distance_to(sender),
        distance_underwater(listener, sender),
        range,
        waves_level,
    );
    capped_by_antenna(weighted / range, antenna_loss)
}
