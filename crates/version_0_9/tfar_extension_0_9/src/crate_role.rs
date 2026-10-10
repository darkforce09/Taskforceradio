//! The one-line statement of what `tfar_extension_0_9` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_extension_0_9` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_extension_0_9: the Arma extension for the 0.9.12 mod: every \
     callExtension is a sync round trip to the 0.9 plugin over the game \
     link.";
