//! [`MockHost`](crate::MockHost) and its drivers: the lifecycle, the events and the audio.

mod audio_drivers;
mod event_drivers;
pub(crate) mod host_lifecycle;

pub use audio_drivers::{CapturedVoiceOutcome, VoiceBlockOutcome};
pub use event_drivers::ServerSetup;
pub use host_lifecycle::{
    CommandOutcome, HOSTED_API_VERSION, HostOptions, InfoKind, MOCK_PLUGIN_ID, MockHost,
};

#[cfg(test)]
#[path = "tests/in_process_plugin.rs"]
mod in_process_plugin;
