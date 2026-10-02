//! The one-line statement of what `tfar_voice_pipeline_0_9` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_voice_pipeline_0_9` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_voice_pipeline_0_9: the 0.9.12 audio path for one voice \
     frame: routes the speaker, applies the radio effect, the distance \
     and stereo gains and the HRTF, and mixes radio sounds.";
