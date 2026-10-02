//! The one-line statement of what `tfar_dynamics` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_dynamics` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_dynamics: the feed-forward compressor that replaces \
     SimpleComp: threshold, ratio, attack and release over a sample \
     buffer, run exactly once per frame.";
