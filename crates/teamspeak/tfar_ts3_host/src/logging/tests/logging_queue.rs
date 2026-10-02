//! Unit tests for the log layer, its queue and the logger thread, with a recording sink in place
//! of TeamSpeak's `logMessage` and a scoped subscriber in place of the global one.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt as _;

use super::log_forwarder::start_forwarder;
use super::log_layer::{LogLine, LogQueue, TeamSpeakLogLayer};
use crate::{Error, LogLevel};

/// What the sink received: the thread it ran on and the line.
type Delivered = Arc<Mutex<Vec<(ThreadId, LogLine)>>>;

fn recording_sink(delivered: &Delivered) -> super::log_forwarder::LogSink {
    let delivered = Arc::clone(delivered);
    Box::new(move |line: &LogLine| {
        delivered
            .lock()
            .unwrap()
            .push((std::thread::current().id(), line.clone()));
    })
}

/// Runs `emit` with a subscriber whose only layer feeds `queue`.
fn with_layer(queue: &Arc<LogQueue>, emit: impl FnOnce()) {
    let subscriber = tracing_subscriber::registry().with(TeamSpeakLogLayer::new(Arc::clone(queue)));
    tracing::subscriber::with_default(subscriber, emit);
}

#[test]
fn lines_reach_the_sink_on_the_logger_thread_never_on_the_emitting_one() {
    let queue = Arc::new(LogQueue::new(16, Level::DEBUG));
    let delivered = Delivered::default();
    let guard = start_forwarder(Arc::clone(&queue), recording_sink(&delivered)).unwrap();
    let emitter = std::thread::current().id();
    with_layer(&queue, || {
        tracing::warn!(frequency = "45.3", "radio jammed");
        tracing::debug!("ticked");
    });
    drop(guard);
    let delivered = delivered.lock().unwrap();
    assert_eq!(delivered.len(), 2);
    for (thread, _) in delivered.iter() {
        assert_ne!(*thread, emitter, "the sink ran on the emitting thread");
    }
    let target = module_path!();
    assert_eq!(
        delivered[0].1,
        LogLine {
            level: LogLevel::Warning,
            text: format!("{target}: radio jammed frequency=45.3")
        }
    );
    assert_eq!(
        delivered[1].1,
        LogLine {
            level: LogLevel::Debug,
            text: format!("{target}: ticked")
        }
    );
}

#[test]
fn an_audio_thread_event_is_queued_and_delivered_elsewhere() {
    let queue = Arc::new(LogQueue::new(16, Level::TRACE));
    let delivered = Delivered::default();
    let guard = start_forwarder(Arc::clone(&queue), recording_sink(&delivered)).unwrap();
    let audio_queue = Arc::clone(&queue);
    let audio_thread = std::thread::spawn(move || {
        with_layer(&audio_queue, || tracing::trace!("frame"));
        std::thread::current().id()
    });
    let audio_thread_id = audio_thread.join().unwrap();
    drop(guard);
    let delivered = delivered.lock().unwrap();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].1.level, LogLevel::Devel);
    assert_ne!(delivered[0].0, audio_thread_id);
    assert_ne!(delivered[0].0, std::thread::current().id());
}

#[test]
fn events_above_the_level_or_without_a_logger_thread_are_dropped() {
    let queue = Arc::new(LogQueue::new(16, Level::INFO));
    with_layer(&queue, || tracing::error!("nobody forwards this yet"));
    assert!(
        queue.receiver.is_empty(),
        "queued with no logger thread running"
    );

    let delivered = Delivered::default();
    let guard = start_forwarder(Arc::clone(&queue), recording_sink(&delivered)).unwrap();
    with_layer(&queue, || {
        tracing::debug!("too verbose");
        tracing::info!("kept");
    });
    drop(guard);
    let texts: Vec<String> = delivered
        .lock()
        .unwrap()
        .iter()
        .map(|(_, line)| line.text.clone())
        .collect();
    assert_eq!(texts, [format!("{}: kept", module_path!())]);
}

#[test]
fn a_full_queue_drops_and_counts_instead_of_blocking() {
    let queue = Arc::new(LogQueue::new(2, Level::INFO));
    // Forwarding on, but no thread draining: the emitter must still never wait.
    queue.is_forwarding.store(true, Ordering::Release);
    with_layer(&queue, || {
        for index in 0..5 {
            tracing::info!(index, "burst");
        }
    });
    assert_eq!(queue.receiver.len(), 2);
    assert_eq!(queue.dropped_lines.load(Ordering::Relaxed), 3);
}

#[test]
fn a_second_forwarder_on_a_running_queue_is_refused() {
    let queue = Arc::new(LogQueue::new(4, Level::INFO));
    let delivered = Delivered::default();
    let guard = start_forwarder(Arc::clone(&queue), recording_sink(&delivered)).unwrap();
    let second = start_forwarder(Arc::clone(&queue), recording_sink(&delivered));
    assert_eq!(second.err(), Some(Error::LoggingAlreadyInstalled));
    drop(guard);
    let again = start_forwarder(Arc::clone(&queue), recording_sink(&delivered)).unwrap();
    assert_eq!(again.dropped_lines(), 0);
}
