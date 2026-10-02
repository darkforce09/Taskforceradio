//! The filter designs of the per-sender filters and the original's effect keys.

use tfar_dsp_filters::{FilterDesign, StereoFilter};
use tfar_radio_effects_1_0::SAMPLE_RATE_HZ;

use crate::error::Result;

/// The key of the direct-speech HRTF source (`"voice_clunk"`, `plugin.cpp:582`).
pub const VOICE_HRTF_KEY: &str = "voice_clunk";

/// The key of the direct-speech can't-speak low-pass (`"local_cantspeak"`, `plugin.cpp:574`).
pub const CANT_SPEAK_FILTER_KEY: &str = "local_cantspeak";

/// The key of the direct-speech vehicle low-pass (`"local_vehicle"`, `plugin.cpp:569`).
pub const VEHICLE_FILTER_KEY: &str = "local_vehicle";

/// The can't-speak and underwater-speaker low-pass: Butterworth, 4 poles, 100 Hz.
pub(crate) const CANT_SPEAK: FilterDesign = FilterDesign::ButterworthLowPass {
    order: 4,
    cutoff_hz: 100.0,
};

/// The loudspeaker band-pass: Butterworth order 1, centre 2000 Hz, width 1000 Hz.
pub(crate) const SPEAKER: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 1,
    centre_hz: 2000.0,
    width_hz: 1000.0,
};

/// The phone band-pass: Butterworth order 2, centre 1850 Hz, width 1550 Hz.
pub(crate) const PHONE: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 2,
    centre_hz: 1850.0,
    width_hz: 1550.0,
};

/// The vehicle low-pass at `vehicle_volume_loss`: Butterworth, 2 poles, cutoff
/// `20000 · (1 − loss) / 4` (`clientData.hpp:192-203`).
pub(crate) fn vehicle(vehicle_volume_loss: f64) -> FilterDesign {
    FilterDesign::ButterworthLowPass {
        order: 2,
        cutoff_hz: 20_000.0 * (1.0 - vehicle_volume_loss) / 4.0,
    }
}

/// The object-interception cutoff for `object_count` objects at `strength`:
/// `2000 − min(count, 5) · strength` (`clientData.cpp:53-74`), with the count clamped to 0..=5
/// instead of the original's cast to `uint8_t`. At or below 0 Hz there is no low-pass to design
/// and the sound is silence ([`crate::ObjectInterceptionFilter::Silence`]).
pub fn object_interception_cutoff_hz(object_count: i32, strength: f32) -> f64 {
    2000.0 - f64::from(object_count.clamp(0, 5)) * f64::from(strength)
}

/// `design` built as a stereo filter at the voice sample rate.
pub(crate) fn stereo(design: FilterDesign) -> Result<StereoFilter> {
    Ok(StereoFilter::new(design, SAMPLE_RATE_HZ)?)
}
