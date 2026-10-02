//! The one-line statement of what `tfar_hrtf` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_hrtf` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_hrtf: real binaural direction rendering for both plugin \
     versions: loads the IRCAM HRIR sphere through the hrtf crate and \
     adapts TeamSpeak's frame sizes to its block size, keeping state \
     per sound source.";
