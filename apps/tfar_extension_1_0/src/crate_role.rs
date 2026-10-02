//! The one-line statement of what `tfar_extension_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_extension_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_extension_1_0: the Arma extension for the 1.0 mod: ~-prefixed \
     calls go async, data frames answer NEEDCFG or OK, and MISSIONEND \
     and FOCUS frames reach the 1.0 plugin over the game link.";
