//! The one-line statement of what `tfar_debug_window` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_debug_window` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_debug_window: the 1.0 plugin's debug window: one long-lived \
     UI thread running an eframe window on X11 with the panels that \
     show client and radio state.";
