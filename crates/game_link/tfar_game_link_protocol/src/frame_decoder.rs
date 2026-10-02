//! The streaming frame decoder: arbitrary byte chunks in, complete frames out.
//!
//! **Role:** buffers the bytes a socket delivers in whatever chunks it delivers them and yields
//! each frame once all of its bytes are in.
//!
//! **Position:** tier 0 of `crates/game_link`; fed by the session thread of `tfar_game_link_server`
//! and the reader thread of `tfar_game_link_client`.
//!
//! **Signals and state:** the bytes received and not yet consumed; owned by one thread.
//!
//! **Invariants:** the length prefix is checked as soon as its four bytes are in, before the body is
//! waited for; frames come out in the order their bytes went in; after an error the stream is out of
//! step and the caller closes it.

use crate::error::Result;
use crate::frame::Frame;
use crate::frame_codec::{decode_frame_body, frame_body_length};
use crate::protocol_constants::FRAME_LENGTH_PREFIX_SIZE;

#[cfg(test)]
#[path = "tests/frame_decoder.rs"]
mod tests;

/// Reassembles frames from a byte stream delivered in arbitrary chunks.
#[derive(Debug, Default)]
pub struct FrameDecoder {
    /// Bytes received and not yet consumed by a decoded frame.
    pending: Vec<u8>,
}

impl FrameDecoder {
    /// A decoder with nothing buffered.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends the bytes the stream delivered next.
    pub fn push(&mut self, chunk: &[u8]) {
        self.pending.extend_from_slice(chunk);
    }

    /// The next complete frame, `Ok(None)` while its bytes are still incomplete.
    ///
    /// # Errors
    ///
    /// Any length, kind or payload error of [`decode_frame_body`] and [`frame_body_length`]; the
    /// stream cannot be resynchronised afterwards.
    pub fn next_frame(&mut self) -> Result<Option<Frame>> {
        let Some(prefix) = self
            .pending
            .first_chunk::<FRAME_LENGTH_PREFIX_SIZE>()
            .copied()
        else {
            return Ok(None);
        };
        let body_length = frame_body_length(prefix)?;
        let frame_end = FRAME_LENGTH_PREFIX_SIZE + body_length;
        let Some(body) = self.pending.get(FRAME_LENGTH_PREFIX_SIZE..frame_end) else {
            return Ok(None);
        };
        let frame = decode_frame_body(body)?;
        self.pending.drain(..frame_end);
        Ok(Some(frame))
    }

    /// How many received bytes wait for the rest of their frame.
    pub fn buffered_len(&self) -> usize {
        self.pending.len()
    }
}
