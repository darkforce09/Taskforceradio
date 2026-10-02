//! The one-line statement of what `tfar_ts3_settings_db` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_ts3_settings_db` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_ts3_settings_db: the reader of TeamSpeak's settings.db SQLite \
     file, for the hotkey and capture settings the plugins look up.";
