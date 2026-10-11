//! Blocking frame reads and writes over any `std::io` stream.

use std::io::{ErrorKind, Read, Write};

use crate::protocol::error::{Error, Result};
use crate::protocol::frame::Frame;
use crate::protocol::frame_codec::{decode_frame_body, encode, frame_body_length};
use crate::protocol::protocol_constants::FRAME_LENGTH_PREFIX_SIZE;

#[cfg(test)]
#[path = "tests/frame_io.rs"]
mod tests;

/// Reads exactly one frame from `reader`, blocking until all of its bytes are in.
///
/// # Errors
///
/// [`Error::ConnectionClosed`] when the stream ends before the first byte of a frame,
/// [`Error::Io`] when it fails or ends inside one, and the length, kind and payload errors of
/// [`decode_frame_body`]; the body buffer is allocated only after the length is checked.
pub fn read_frame(reader: &mut impl Read) -> Result<Frame> {
    let mut prefix = [0_u8; FRAME_LENGTH_PREFIX_SIZE];
    let first_count = loop {
        match reader.read(&mut prefix) {
            Ok(count) => break count,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(Error::Io { source: error }),
        }
    };
    if first_count == 0 {
        return Err(Error::ConnectionClosed);
    }
    if let Some(rest) = prefix.get_mut(first_count..) {
        reader.read_exact(rest)?;
    }
    let mut body = vec![0_u8; frame_body_length(prefix)?];
    reader.read_exact(&mut body)?;
    decode_frame_body(&body)
}

/// Encodes `frame` and writes all of it to `writer` in one `write_all`, then flushes.
///
/// # Errors
///
/// The encode errors of [`encode`] and [`Error::Io`] when the write fails or times out.
pub fn write_frame(writer: &mut impl Write, frame: &Frame) -> Result<()> {
    let mut bytes = Vec::new();
    encode(frame, &mut bytes)?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}
