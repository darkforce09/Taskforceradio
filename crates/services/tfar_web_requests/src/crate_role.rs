//! The one-line statement of what `tfar_web_requests` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_web_requests` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_web_requests: the plugins' outbound HTTP: the update check \
     against the published version and the Piwik TRACK calls, over ureq \
     with rustls, off the audio thread.";
