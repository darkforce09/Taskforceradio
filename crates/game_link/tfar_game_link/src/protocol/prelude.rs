//! The names a caller of `tfar_game_link::protocol` imports with `use crate::protocol::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::protocol::flavour::Flavour;
pub use crate::protocol::frame::{Frame, FrameKind, Hello, HelloMismatch};
pub use crate::protocol::frame_codec::encode;
pub use crate::protocol::frame_decoder::FrameDecoder;
pub use crate::protocol::frame_io::{read_frame, write_frame};
pub use crate::protocol::game_link_port::port;
pub use crate::protocol::protocol_constants::{MAGIC, MAX_FRAME_LENGTH, PROTOCOL_VERSION};
