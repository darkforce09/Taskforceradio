//! The `tracing` layer that queues every event for the logger thread, and the queue it fills.
//!
//! **Role:** formats an event into one line and hands it to a bounded queue without blocking, so a
//! `tracing` call on a TeamSpeak audio thread never waits and never reaches TeamSpeak itself.
//!
//! **Position:** inside `logging`; `log_forwarder` drains the queue, `install` registers the layer.
//!
//! **Signals and state:** the [`LogQueue`] is shared between every emitting thread (any thread,
//! audio threads included) and the one logger thread; its flags are atomics, its channel is
//! `crossbeam-channel`'s bounded MPMC queue.
//!
//! **Invariants:** `on_event` never blocks and never calls TeamSpeak: a full queue drops the line
//! and counts it; nothing is queued while no logger thread runs.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender, TrySendError};
use tracing::field::{Field, Visit};
use tracing::subscriber::Interest;
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

use crate::LogLevel;

/// One formatted line on its way to `logMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogLine {
    /// The TeamSpeak severity the `tracing` level maps to.
    pub(crate) level: LogLevel,
    /// `target: message key=value …`.
    pub(crate) text: String,
}

/// What travels through the queue.
#[derive(Debug)]
pub(crate) enum LogMessage {
    /// A line to forward.
    Line(LogLine),
    /// The logger thread stops after the lines before it.
    Stop,
}

/// The bounded queue between the threads that log and the logger thread.
#[derive(Debug)]
pub(crate) struct LogQueue {
    /// The emitting side; cloned by nothing, every thread uses this one (`Sender` is `Sync`).
    pub(crate) sender: Sender<LogMessage>,
    /// The logger thread's side.
    pub(crate) receiver: Receiver<LogMessage>,
    /// Whether a logger thread is running; while false, events are dropped at once.
    pub(crate) is_forwarding: AtomicBool,
    /// The most verbose level forwarded, as [`level_rank`] numbers it.
    max_level_rank: AtomicU8,
    /// Lines dropped because the queue was full.
    pub(crate) dropped_lines: AtomicU64,
}

impl LogQueue {
    /// A queue of `capacity` lines that forwards `max_level` and everything more severe.
    pub(crate) fn new(capacity: usize, max_level: Level) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(capacity);
        Self {
            sender,
            receiver,
            is_forwarding: AtomicBool::new(false),
            max_level_rank: AtomicU8::new(level_rank(max_level)),
            dropped_lines: AtomicU64::new(0),
        }
    }

    /// Changes the most verbose level forwarded.
    pub(crate) fn set_max_level(&self, max_level: Level) {
        self.max_level_rank
            .store(level_rank(max_level), Ordering::Relaxed);
    }

    /// Whether an event at `level` is forwarded.
    fn admits(&self, level: Level) -> bool {
        level_rank(level) <= self.max_level_rank.load(Ordering::Relaxed)
    }
}

/// Orders levels from most severe (`ERROR` = 0) to most verbose (`TRACE` = 4).
fn level_rank(level: Level) -> u8 {
    match level {
        Level::ERROR => 0,
        Level::WARN => 1,
        Level::INFO => 2,
        Level::DEBUG => 3,
        // `Level` is a struct of constants, so the match needs a catch-all; only TRACE is left.
        _ => 4,
    }
}

/// The TeamSpeak severity for a `tracing` level.
pub(crate) fn teamspeak_level(level: Level) -> LogLevel {
    match level {
        Level::ERROR => LogLevel::Error,
        Level::WARN => LogLevel::Warning,
        Level::INFO => LogLevel::Info,
        Level::DEBUG => LogLevel::Debug,
        // Only TRACE is left.
        _ => LogLevel::Devel,
    }
}

/// The `tracing` layer that forwards events to TeamSpeak's log through a [`LogQueue`].
pub struct TeamSpeakLogLayer {
    /// The queue the layer fills.
    queue: std::sync::Arc<LogQueue>,
}

impl TeamSpeakLogLayer {
    /// A layer over `queue`.
    pub(crate) fn new(queue: std::sync::Arc<LogQueue>) -> Self {
        Self { queue }
    }
}

impl<S: Subscriber> Layer<S> for TeamSpeakLogLayer {
    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        // The level can change at run time, so every event asks `enabled` again.
        Interest::sometimes()
    }

    fn enabled(&self, metadata: &Metadata<'_>, _context: Context<'_, S>) -> bool {
        self.queue.admits(*metadata.level())
    }

    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        if !self.queue.is_forwarding.load(Ordering::Acquire) {
            return;
        }
        let metadata = event.metadata();
        let mut text = TextVisitor {
            message: String::new(),
            fields: String::new(),
        };
        event.record(&mut text);
        let line = LogLine {
            level: teamspeak_level(*metadata.level()),
            text: format!("{}: {}{}", metadata.target(), text.message, text.fields),
        };
        match self.queue.sender.try_send(LogMessage::Line(line)) {
            Ok(()) | Err(TrySendError::Disconnected(_)) => {}
            Err(TrySendError::Full(_)) => {
                self.queue.dropped_lines.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Collects an event's `message` and its other fields as ` key=value` pairs.
struct TextVisitor {
    /// The `message` field.
    message: String,
    /// Every other field, each with a leading space.
    fields: String,
}

impl Visit for TextVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        } else {
            let _ = write!(self.fields, " {}={value}", field.name());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }
}
