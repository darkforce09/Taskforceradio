//! The one-line statement of what `tfar_playback_mixer` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_playback_mixer` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_playback_mixer: the radio sound player: loads the WAV files \
     shipped in the mod's radio-sounds folder through hound, checks \
     they are 48 kHz 16-bit stereo, caches them and mixes active \
     playbacks into a frame.";
