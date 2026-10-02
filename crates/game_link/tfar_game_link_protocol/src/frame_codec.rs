//! The length-prefixed frame of the game link.
//!
//! **Role:** encodes a frame as its length prefix, kind byte and payload, and decodes a frame body
//! (kind byte and payload) back into a frame, refusing anything malformed.
//!
//! **Position:** tier 0 of `crates/game_link`; called by the streaming decoder, the blocking read and
//! write helpers, `tfar_game_link_server` in the plugins and `tfar_game_link_client` in the
//! extensions, on either side of the localhost TCP socket.
//!
//! **Signals and state:** none; pure functions over byte slices, called from whichever thread owns
//! the socket.
//!
//! **Invariants:** a frame is `u32 LE length | u8 kind | payload`, and the length counts the kind
//! byte and the payload; a length of zero or above [`MAX_FRAME_LENGTH`] is rejected before any
//! allocation; the payload is the `postcard` encoding of the kind's fields in table order and must
//! decode with no byte left over; every frame that encodes decodes to itself.

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::frame::{Frame, FrameKind, Hello};
use crate::protocol_constants::{FRAME_LENGTH_PREFIX_SIZE, MAX_FRAME_LENGTH};

#[cfg(test)]
#[path = "tests/frame_codec.rs"]
mod tests;

/// Appends `frame` to `out` as one complete frame: length prefix, kind byte, payload.
///
/// # Errors
///
/// [`Error::FrameTooLong`] when the body would exceed [`MAX_FRAME_LENGTH`] (a text field near
/// 1 MiB), [`Error::EncodeFailed`] when `postcard` refuses a field; `out` is left as it was.
pub fn encode(frame: &Frame, out: &mut Vec<u8>) -> Result<()> {
    let start = out.len();
    let kind = frame.kind();
    out.extend_from_slice(&[0; FRAME_LENGTH_PREFIX_SIZE]);
    out.push(kind.code());
    let payload_result = match frame {
        Frame::Hello(hello) => postcard::to_io(hello, &mut *out).map(drop),
        Frame::SyncRequest { id, text } | Frame::SyncResponse { id, text } => {
            postcard::to_io(&(id, text.as_slice()), &mut *out).map(drop)
        }
        Frame::Async { text } => postcard::to_io(text.as_slice(), &mut *out).map(drop),
        Frame::Focus { arma_focused: flag } | Frame::Status { needcfg: flag } => {
            postcard::to_io(flag, &mut *out).map(drop)
        }
    };
    if let Err(source) = payload_result {
        out.truncate(start);
        return Err(Error::EncodeFailed { kind, source });
    }
    let body_length = out.len() - start - FRAME_LENGTH_PREFIX_SIZE;
    let Some(length) = u32::try_from(body_length)
        .ok()
        .filter(|length| *length <= MAX_FRAME_LENGTH)
    else {
        out.truncate(start);
        return Err(Error::FrameTooLong {
            length: body_length,
            limit: MAX_FRAME_LENGTH,
        });
    };
    if let Some(prefix) = out.get_mut(start..start + FRAME_LENGTH_PREFIX_SIZE) {
        prefix.copy_from_slice(&length.to_le_bytes());
    }
    Ok(())
}

/// The body length a frame's length prefix announces, checked against the limits.
///
/// # Errors
///
/// [`Error::ZeroLength`] for 0 and [`Error::FrameTooLong`] above [`MAX_FRAME_LENGTH`].
pub fn frame_body_length(prefix: [u8; FRAME_LENGTH_PREFIX_SIZE]) -> Result<usize> {
    let length = u32::from_le_bytes(prefix);
    if length == 0 {
        return Err(Error::ZeroLength);
    }
    let length_in_bytes = usize::try_from(length).unwrap_or(usize::MAX);
    if length > MAX_FRAME_LENGTH {
        return Err(Error::FrameTooLong {
            length: length_in_bytes,
            limit: MAX_FRAME_LENGTH,
        });
    }
    Ok(length_in_bytes)
}

/// Decodes a frame body, the bytes after the length prefix: the kind byte, then its payload.
///
/// # Errors
///
/// [`Error::ZeroLength`] for an empty body, [`Error::UnknownKind`] for a kind byte outside the
/// table, [`Error::BadPayload`] when the fields do not decode and
/// [`Error::TrailingPayloadBytes`] when bytes are left after them.
pub fn decode_frame_body(body: &[u8]) -> Result<Frame> {
    let Some((&kind_code, payload)) = body.split_first() else {
        return Err(Error::ZeroLength);
    };
    let kind = FrameKind::from_code(kind_code).ok_or(Error::UnknownKind { kind: kind_code })?;
    let frame = match kind {
        FrameKind::Hello => Frame::Hello(decode_payload::<Hello>(kind, payload)?),
        FrameKind::SyncRequest => {
            let (id, text) = decode_payload::<(u32, &[u8])>(kind, payload)?;
            Frame::SyncRequest {
                id,
                text: text.to_vec(),
            }
        }
        FrameKind::SyncResponse => {
            let (id, text) = decode_payload::<(u32, &[u8])>(kind, payload)?;
            Frame::SyncResponse {
                id,
                text: text.to_vec(),
            }
        }
        FrameKind::Async => Frame::Async {
            text: decode_payload::<&[u8]>(kind, payload)?.to_vec(),
        },
        FrameKind::Focus => Frame::Focus {
            arma_focused: decode_payload::<bool>(kind, payload)?,
        },
        FrameKind::Status => Frame::Status {
            needcfg: decode_payload::<bool>(kind, payload)?,
        },
    };
    Ok(frame)
}

/// Decodes `payload` as the fields `T` of `kind`, requiring every byte to be used.
fn decode_payload<'payload, T: Deserialize<'payload>>(
    kind: FrameKind,
    payload: &'payload [u8],
) -> Result<T> {
    let (value, rest) = postcard::take_from_bytes::<T>(payload)
        .map_err(|source| Error::BadPayload { kind, source })?;
    if rest.is_empty() {
        Ok(value)
    } else {
        Err(Error::TrailingPayloadBytes {
            kind,
            count: rest.len(),
        })
    }
}
