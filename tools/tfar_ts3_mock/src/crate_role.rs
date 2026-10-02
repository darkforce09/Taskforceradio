//! The one-line statement of what `tfar_ts3_mock` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_ts3_mock` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_ts3_mock: the test host for the plugins: loads a plugin .so \
     with libloading, hands it a fake Ts3Functions table, plays the \
     game side over the game link and runs scripted scenarios.";
