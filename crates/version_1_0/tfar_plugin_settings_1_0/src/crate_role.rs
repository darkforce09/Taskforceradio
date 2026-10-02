//! The one-line statement of what `tfar_plugin_settings_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_plugin_settings_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_plugin_settings_1_0: the 1.0 plugin's settings model: every \
     value the game sets with SETCFG, its default and its type, the key matching and conversions \
     of the original, and the refresh flag behind NEEDCFG.";
