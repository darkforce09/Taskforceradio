//! The async worker: one thread that runs the 1.0 `~` messages strictly in arrival order.
//!
//! **Role:** takes [`AsyncMessage`]s off the one FIFO queue and hands each to
//! [`GameLinkHandler::async_message`](crate::server::GameLinkHandler::async_message), skipping the ones a
//! later `MISSIONEND` made stale.
//!
//! **Position:** fed by the session thread through the `crossbeam-channel` queue in the shared
//! state; started and joined by [`GameLinkServer`](crate::server::GameLinkServer).
//!
//! **Signals and state:** runs on the `tfar-game-link-async` thread; reads the stop flag and the
//! mission generation.
//!
//! **Invariants:** messages run first in, first out (one queue, one worker); a message queued
//! under an older mission generation than the current one never runs, so a `MISSIONEND` empties
//! the queue before it and then runs itself; the worker stops within one poll period of shutdown,
//! leaving the rest of the queue unrun.

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};

use crate::server::link_state::LinkState;

#[cfg(test)]
#[path = "tests/async_worker.rs"]
mod tests;

/// The command name of the async message that ends a mission.
const MISSION_END_COMMAND: &[u8] = b"MISSIONEND";

/// How often an idle worker checks the stop flag.
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// One queued `ASYNC` message and the mission generation it arrived in.
#[derive(Debug)]
pub(crate) struct AsyncMessage {
    /// The mission generation current when the message was queued.
    pub(crate) generation: u64,
    /// The call text without its `~`.
    pub(crate) text: Vec<u8>,
}

/// True when the command name of `text`, the bytes before the first tab, is `MISSIONEND`.
pub(crate) fn is_mission_end(text: &[u8]) -> bool {
    text.split(|byte| *byte == b'\t').next() == Some(MISSION_END_COMMAND)
}

/// The worker loop; returns once the server stops or the queue's senders are gone.
pub(crate) fn run_async_worker(state: &Arc<LinkState>, queue: &Receiver<AsyncMessage>) {
    while !state.is_stopping() {
        match queue.recv_timeout(WORKER_POLL_INTERVAL) {
            Ok(message) => {
                if state.is_stopping() {
                    break;
                }
                if message.generation == state.mission_generation() {
                    state.handler.async_message(&message.text);
                } else {
                    tracing::trace!(
                        generation = message.generation,
                        "game link: async message dropped by MISSIONEND"
                    );
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
