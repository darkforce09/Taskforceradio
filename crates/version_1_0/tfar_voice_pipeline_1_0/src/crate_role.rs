//! The one-line statement of what `tfar_voice_pipeline_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_voice_pipeline_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_voice_pipeline_1_0: the 1.0 audio path for one voice frame: \
     routes the speaker, applies the radio effect and the compressor \
     once, the distance and cone gains and the HRTF, and mixes radio \
     sounds.";
