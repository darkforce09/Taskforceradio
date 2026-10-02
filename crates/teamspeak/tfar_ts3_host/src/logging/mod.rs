//! Logging into TeamSpeak: a `tracing` layer, its bounded queue and the logger thread that calls
//! `logMessage`, plus the chat-tab printer.

mod install;
mod log_forwarder;
mod log_layer;

pub use install::{install_logging, install_logging_at, print_to_current_tab};
pub use log_forwarder::LoggingGuard;
pub use log_layer::TeamSpeakLogLayer;

#[cfg(test)]
#[path = "tests/logging_queue.rs"]
mod logging_queue_tests;
