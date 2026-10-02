//! The radio volume curve and the remote radio sounds.

use tfar_arma_values::RadioSubtype;

#[cfg(test)]
#[path = "tests/radio_volume.rs"]
mod tests;

/// The gain of a radio at knob `volume` (0 to 10): `((volume + 1) / 10)^4`, as
/// `helpers::volumeMultiplifier` (`reference/tfar_1_0/ts/src/helpers.cpp:302-305`).
pub fn radio_volume_gain(volume: f64) -> f64 {
    ((volume + 1.0) / 10.0).powi(4)
}

/// The plugin-relative sound (without `.wav`) played when a remote player presses
/// (`is_start`) or releases a radio of `subtype`, as `SoundDirectory::getRemoteRadioSound`
/// (`reference/tfar_1_0/ts/src/PlaybackHandler.hpp:304-316`): only `digital`, `digital_lr` and
/// `airborne` radios have one.
pub const fn remote_radio_sound(subtype: &RadioSubtype, is_start: bool) -> Option<&'static str> {
    match (subtype, is_start) {
        (RadioSubtype::DigitalLongRange, true) => Some("radio-sounds/lr/remote_start"),
        (RadioSubtype::DigitalLongRange, false) => Some("radio-sounds/lr/remote_end"),
        (RadioSubtype::Digital, true) => Some("radio-sounds/sw/remote_start"),
        (RadioSubtype::Digital, false) => Some("radio-sounds/sw/remote_end"),
        (RadioSubtype::Airborne, true) => Some("radio-sounds/ab/remote_start"),
        (RadioSubtype::Airborne, false) => Some("radio-sounds/ab/remote_end"),
        _ => None,
    }
}
