//! [`install_logging`]: makes the TeamSpeak log layer the process-wide `tracing` subscriber and
//! starts its logger thread; [`print_to_current_tab`]: a line in the chat tab in front.
//!
//! **Role:** the plugin-facing entry points of `logging`.
//!
//! **Position:** inside `logging`, over `log_layer`, `log_forwarder` and [`Ts3`].
//!
//! **Signals and state:** one process-wide queue, created and registered with `tracing` by the
//! first install and reused by every later one (TeamSpeak may stop and start a plugin without
//! unloading it, and `tracing` accepts a global default only once). Called on TeamSpeak's main
//! thread from `ts3plugin_init`.
//!
//! **Invariants:** the logger thread is the only thread that calls `logMessage`; a line is cleaned
//! of NUL bytes before it is passed on, so a log line never fails on its text.

use std::sync::{Arc, OnceLock};

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt as _;

use super::log_forwarder::{LoggingGuard, start_forwarder};
use super::log_layer::{LogLine, LogQueue, TeamSpeakLogLayer};
use crate::{Error, Result, ServerConnectionHandlerId, Ts3, Ts3Error};

/// Lines the queue holds before new ones are dropped: a burst of a few seconds of debug output.
const QUEUE_CAPACITY: usize = 1024;

/// The level [`install_logging`] forwards up to.
const DEFAULT_MAX_LEVEL: Level = Level::DEBUG;

/// The queue the global subscriber fills; `None` when another subscriber was already global.
static GLOBAL_QUEUE: OnceLock<Option<Arc<LogQueue>>> = OnceLock::new();

/// Forwards `tracing` events at `DEBUG` and above to TeamSpeak's log, under the channel
/// `plugin_name`, until the returned guard drops.
///
/// Call it in `init` once the function table is installed, keep the guard, drop it in `shutdown`.
///
/// # Errors
///
/// [`Error::TeamSpeak`] before `ts3plugin_setFunctionPointers`; [`Error::SubscriberAlreadySet`]
/// when another `tracing` subscriber is global; [`Error::LoggingAlreadyInstalled`] while an
/// earlier guard lives; [`Error::LoggerThreadSpawn`] when the thread cannot start.
pub fn install_logging(plugin_name: &str) -> Result<LoggingGuard> {
    install_logging_at(plugin_name, DEFAULT_MAX_LEVEL)
}

/// As [`install_logging`], forwarding events at `max_level` and everything more severe.
///
/// # Errors
///
/// As [`install_logging`].
pub fn install_logging_at(plugin_name: &str, max_level: Level) -> Result<LoggingGuard> {
    let ts3 = Ts3::installed_or_error()?;
    let queue = global_queue(max_level)?;
    queue.set_max_level(max_level);
    let channel = without_nul(plugin_name);
    let sink = Box::new(move |line: &LogLine| {
        // A failed log line has nowhere to be reported; it is dropped.
        let _ = ts3.log_message(
            &without_nul(&line.text),
            line.level,
            &channel,
            ServerConnectionHandlerId(0),
        );
    });
    start_forwarder(queue, sink)
}

/// The process-wide queue, registering the layer as the global subscriber on first use.
fn global_queue(max_level: Level) -> Result<Arc<LogQueue>> {
    let queue = GLOBAL_QUEUE.get_or_init(|| {
        let queue = Arc::new(LogQueue::new(QUEUE_CAPACITY, max_level));
        let subscriber =
            tracing_subscriber::registry().with(TeamSpeakLogLayer::new(Arc::clone(&queue)));
        tracing::subscriber::set_global_default(subscriber)
            .ok()
            .map(|()| queue)
    });
    queue.clone().ok_or(Error::SubscriberAlreadySet)
}

/// `text` with every NUL byte replaced by U+FFFD, so it always becomes a C string.
fn without_nul(text: &str) -> String {
    text.replace('\0', "\u{fffd}")
}

/// Prints `text` (BB-code allowed) in the chat of the server tab in front: the plugins' stand-in
/// for the originals' Windows message boxes.
///
/// # Errors
///
/// [`Ts3Error::FunctionsNotInstalled`] before `ts3plugin_setFunctionPointers`;
/// [`Ts3Error::InteriorNul`] when `text` holds a NUL byte.
pub fn print_to_current_tab(text: &str) -> core::result::Result<(), Ts3Error> {
    Ts3::installed_or_error()?.print_message_to_current_tab(text)
}
