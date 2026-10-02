//! The one-line statement of what `tfar_spatial_audio` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_spatial_audio` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_spatial_audio: the gain model that replaces X3DAudio: the \
     piecewise distance curve, the emitter cone gain and the simple \
     interaural level difference both versions use as their stereo fallback.";
