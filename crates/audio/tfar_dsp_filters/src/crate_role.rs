//! The one-line statement of what `tfar_dsp_filters` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_dsp_filters` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_dsp_filters: the filter set the radio effects use: \
     Butterworth low-pass and band-pass designed by sci-rs as \
     second-order sections, RBJ high-pass and low-pass from biquad, all \
     processed in f64 Direct Form II Transposed with state per channel.";
