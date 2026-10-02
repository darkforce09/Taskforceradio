//! The one-line statement of what `tfar_sample_buffer` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_sample_buffer` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_sample_buffer: the sample frame type the audio crates pass \
     around: interleaved i16 channels as TeamSpeak delivers them, \
     conversion to and from f32/f64, and gain that saturates instead of \
     wrapping.";
