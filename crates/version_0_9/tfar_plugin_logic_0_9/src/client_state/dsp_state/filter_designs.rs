//! The four Butterworth filters a 0.9.12 client's sound state holds, with the original's setups.

use tfar_audio::dsp_filters::{FilterDesign, StereoFilter};

use crate::client_state::error::{Error, Result};

#[cfg(test)]
#[path = "tests/filter_designs.rs"]
mod tests;

/// The rate every filter is designed for: TeamSpeak's 48 kHz voice.
pub const FILTER_SAMPLE_RATE_HZ: f64 = 48_000.0;

/// The muffled voice of a player who cannot speak (under water, in an isolated vehicle): a
/// fourth-order low-pass at 100 Hz (`getFilterCantSpeak`, `setup(4, 48000, 100)`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:297`).
pub const CANT_SPEAK_LOW_PASS: FilterDesign = FilterDesign::ButterworthLowPass {
    order: 4,
    cutoff_hz: 100.0,
};

/// A loudspeaker radio's speaker: a first-order band-pass round 2000 Hz, 1000 Hz wide
/// (`getSpeakerFilter`, `setup(1, 48000, 2000, 1000)`, plugin.cpp:243).
pub const SPEAKER_BAND_PASS: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 1,
    centre_hz: 2000.0,
    width_hz: 1000.0,
};

/// A phone's earpiece: a second-order band-pass round 1850 Hz, 1550 Hz wide
/// (`getSpeakerPhone`, `setup(2, 48000, 1850, 1550)`, plugin.cpp:232).
pub const PHONE_BAND_PASS: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 2,
    centre_hz: 1850.0,
    width_hz: 1550.0,
};

/// The low-pass a vehicle of combined isolation `isolation` puts on a sound: second order, its
/// cut-off at `20000 · (1 − isolation) / 4` Hz (`getFilterVehicle`, plugin.cpp:307).
pub fn vehicle_low_pass(isolation: f64) -> FilterDesign {
    FilterDesign::ButterworthLowPass {
        order: 2,
        cutoff_hz: 20_000.0 * (1.0 - isolation) / 4.0,
    }
}

/// A stereo filter of `design` at [`FILTER_SAMPLE_RATE_HZ`], its failure naming `filter`.
/// Allocates and designs: call it off the audio thread.
///
/// # Errors
///
/// [`Error::FilterDesign`] when the filter library refuses the design.
pub(crate) fn stereo_filter(filter: &'static str, design: FilterDesign) -> Result<StereoFilter> {
    StereoFilter::new(design, FILTER_SAMPLE_RATE_HZ)
        .map_err(|source| Error::FilterDesign { filter, source })
}
