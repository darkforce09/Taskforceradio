//! The 0.9.12 distance gain of a voice and the radio volume curve.

#[cfg(test)]
#[path = "tests/distance_gain.rs"]
mod tests;

/// The distance in metres a muffled voice carries when either side cannot speak
/// (`CANT_SPEAK_DISTANCE`, `reference/tfar_0_9_12/ts/src/plugin.cpp:70`).
pub const CANT_SPEAK_DISTANCE: f64 = 5.0;

/// The gain below which a voice counts as silent (`volumeFromDistance`, plugin.cpp:707).
pub const MINIMUM_AUDIBLE_GAIN: f64 = 0.001;

/// The distance up to which a voice keeps its full level, in metres.
const FULL_LEVEL_DISTANCE: f64 = 1.0;

/// The exponent of the near-field fall-off, `d^-0.3`.
const DISTANCE_EXPONENT: f64 = -0.3;

/// The gain of a voice heard `distance` metres away (`volumeFromDistance`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:698`).
///
/// Up to 1 m the gain is 1. Beyond, with `max = client_distance · multiplier` when
/// `should_player_hear` and [`CANT_SPEAK_DISTANCE`] otherwise, the gain is
/// `d^-0.3 · max(0, max − d) / max`, 0 below [`MINIMUM_AUDIBLE_GAIN`] and at most 1.
/// `client_distance` is a whole number of metres, as the original's `int` parameter.
///
/// A reach of zero or less gives 0 beyond 1 m; the original divides by it there and computes
/// `NaN`, which its sample conversion turns into silence.
pub fn volume_from_distance(
    distance: f64,
    should_player_hear: bool,
    client_distance: i32,
    multiplier: f64,
) -> f64 {
    if distance <= FULL_LEVEL_DISTANCE {
        return 1.0;
    }
    let reach = if should_player_hear {
        f64::from(client_distance) * multiplier
    } else {
        CANT_SPEAK_DISTANCE
    };
    if reach.is_nan() || reach <= 0.0 {
        return 0.0;
    }
    let gain = distance.powf(DISTANCE_EXPONENT) * ((reach - distance).max(0.0) / reach);
    if gain < MINIMUM_AUDIBLE_GAIN {
        0.0
    } else {
        gain.min(1.0)
    }
}

/// The gain of a radio at volume `volume` (`volumeMultiplifier`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:2569`): `((volume + 1) / 10)^4`, so volume 9 is 1.
pub fn radio_volume_multiplier(volume: i32) -> f64 {
    ((f64::from(volume) + 1.0) / 10.0).powi(4)
}
