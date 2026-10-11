//! The frames the Arma extensions and the TeamSpeak plugins exchange over localhost TCP.
//!
//! **Role:** the wire format of the game link: a `u32` little-endian length, a kind byte and a
//! `postcard` payload; the `HELLO` handshake values, the frame kinds, the streaming decoder, the
//! blocking read and write helpers and the port each plugin listens on.
//!
//! **Position:** `crates/game_link`, tier 0, depending on no workspace crate; used by
//! `tfar_game_link_client`, `tfar_game_link_server`, `tfar_ts3_mock`.
//!
//! **Signals and state:** none but the bytes a [`FrameDecoder`] buffers; every function runs on the
//! thread that owns the socket.
//!
//! **Invariants:** a frame decodes back to the frame that was encoded; a zero or oversized length,
//! an unknown kind and a payload that does not decode exactly are errors, never a guess.

mod error;
mod flavour;
mod frame;
mod frame_codec;
mod frame_decoder;
mod frame_io;
mod game_link_port;
pub mod prelude;
mod protocol_constants;

pub use error::{Error, Result};
pub use flavour::Flavour;
pub use frame::{Frame, FrameKind, Hello, HelloMismatch};
pub use frame_codec::{decode_frame_body, encode, frame_body_length};
pub use frame_decoder::FrameDecoder;
pub use frame_io::{read_frame, write_frame};
pub use game_link_port::{PORT_0_9, PORT_0_9_DEBUG, PORT_1_0, PORT_1_0_DEBUG, port};
pub use protocol_constants::{FRAME_LENGTH_PREFIX_SIZE, MAGIC, MAX_FRAME_LENGTH, PROTOCOL_VERSION};
