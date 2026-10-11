//! The error type of `tfar_game_link::protocol`.

use crate::protocol::frame::FrameKind;

/// Every failure `tfar_game_link::protocol` reports.
///
/// After any error but [`Error::ConnectionClosed`] the byte stream is out of step with the frame
/// boundaries, so the receiver closes the connection.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A length prefix of zero: every frame carries at least its kind byte.
    #[error("a game link frame has length 0; every frame carries at least its kind byte")]
    ZeroLength,
    /// A length prefix above [`MAX_FRAME_LENGTH`](crate::protocol::MAX_FRAME_LENGTH), refused before any
    /// allocation; on encode, a frame whose body would be that long.
    #[error("a game link frame has length {length}, above the limit of {limit} bytes")]
    FrameTooLong {
        /// The length the prefix announced, or the body length the encoder would have written.
        length: usize,
        /// The largest legal length.
        limit: u32,
    },
    /// A kind byte outside the frame kind table.
    #[error("a game link frame has the unknown kind byte {kind:#04x}")]
    UnknownKind {
        /// The kind byte that was read.
        kind: u8,
    },
    /// A payload whose fields do not decode as the kind's fields.
    #[error("the {kind} frame payload does not decode: {source}")]
    BadPayload {
        /// The kind the payload belongs to.
        kind: FrameKind,
        /// What `postcard` found wrong.
        #[source]
        source: postcard::Error,
    },
    /// A payload with bytes left over after the kind's fields.
    #[error("the {kind} frame payload has {count} bytes left over after its fields")]
    TrailingPayloadBytes {
        /// The kind the payload belongs to.
        kind: FrameKind,
        /// How many bytes were left over.
        count: usize,
    },
    /// The payload fields of a frame could not be encoded.
    #[error("the {kind} frame payload does not encode: {source}")]
    EncodeFailed {
        /// The kind of the frame being encoded.
        kind: FrameKind,
        /// What `postcard` reported.
        #[source]
        source: postcard::Error,
    },
    /// The peer closed the stream cleanly, before the first byte of a frame.
    #[error("the game link peer closed the connection")]
    ConnectionClosed,
    /// Reading or writing the stream failed, including an end of stream inside a frame.
    #[error("game link stream input or output failed: {source}")]
    Io {
        /// The error the stream reported.
        #[from]
        source: std::io::Error,
    },
}

/// The result of a fallible `tfar_game_link::protocol` operation.
pub type Result<T> = core::result::Result<T, Error>;
