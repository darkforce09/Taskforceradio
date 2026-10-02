//! The logger thread: drains the [`LogQueue`] and hands each line to a sink (in the plugin,
//! TeamSpeak's `logMessage`), and the [`LoggingGuard`] that stops and joins it.
//!
//! **Role:** the only thread that turns `tracing` events into TeamSpeak calls.
//!
//! **Position:** inside `logging`; started by `install`, and by the tests with a recording sink.
//!
//! **Signals and state:** one named thread, `tfar-teamspeak-log`, per guard; it blocks on the
//! queue and owns its sink. The guard lives in the plugin's state between `ts3plugin_init` and
//! `ts3plugin_shutdown`.
//!
//! **Invariants:** at most one logger thread drains a queue; dropping the guard stops forwarding,
//! lets the thread finish the lines already queued and joins it before returning.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;

use super::log_layer::{LogLine, LogMessage, LogQueue};
use crate::{Error, Result};

/// The name of the logger thread, as shown in a debugger or `top -H`.
const LOGGER_THREAD_NAME: &str = "tfar-teamspeak-log";

/// Where the logger thread delivers lines.
pub(crate) type LogSink = Box<dyn FnMut(&LogLine) + Send>;

/// Keeps the logger thread running; dropping it stops the thread and joins it.
///
/// Hold it in the plugin state from `init` to `shutdown`.
#[must_use = "dropping the guard stops forwarding logs to TeamSpeak"]
pub struct LoggingGuard {
    /// The queue the thread drains.
    queue: Arc<LogQueue>,
    /// The thread, taken when it is joined.
    thread: Option<JoinHandle<()>>,
}

/// Starts the logger thread over `queue`, delivering every line to `sink`.
///
/// Fails with [`Error::LoggingAlreadyInstalled`] while another guard drains the same queue.
pub(crate) fn start_forwarder(queue: Arc<LogQueue>, sink: LogSink) -> Result<LoggingGuard> {
    if queue
        .is_forwarding
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(Error::LoggingAlreadyInstalled);
    }
    let thread_queue = Arc::clone(&queue);
    let spawned = std::thread::Builder::new()
        .name(LOGGER_THREAD_NAME.to_owned())
        .spawn(move || forward_lines(&thread_queue, sink));
    match spawned {
        Ok(thread) => Ok(LoggingGuard {
            queue,
            thread: Some(thread),
        }),
        Err(error) => {
            queue.is_forwarding.store(false, Ordering::Release);
            Err(Error::LoggerThreadSpawn {
                reason: error.to_string(),
            })
        }
    }
}

/// The logger thread's loop: deliver lines until the stop message, then the stragglers.
fn forward_lines(queue: &LogQueue, mut sink: LogSink) {
    while let Ok(message) = queue.receiver.recv() {
        match message {
            LogMessage::Line(line) => sink(&line),
            LogMessage::Stop => break,
        }
    }
    // Lines an emitter queued between the forwarding flag turning off and the stop message.
    while let Ok(LogMessage::Line(line)) = queue.receiver.try_recv() {
        sink(&line);
    }
}

impl LoggingGuard {
    /// Lines dropped so far because the queue was full.
    pub fn dropped_lines(&self) -> u64 {
        self.queue.dropped_lines.load(Ordering::Relaxed)
    }
}

impl Drop for LoggingGuard {
    fn drop(&mut self) {
        self.queue.is_forwarding.store(false, Ordering::Release);
        // The thread is draining, so a full queue frees a slot and this send completes.
        let _ = self.queue.sender.send(LogMessage::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
