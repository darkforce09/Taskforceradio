//! The names a caller of `tfar_game_link_protocol` imports with `use tfar_game_link_protocol::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::flavour::Flavour;
pub use crate::frame::{Frame, FrameKind, Hello, HelloMismatch};
pub use crate::frame_codec::encode;
pub use crate::frame_decoder::FrameDecoder;
pub use crate::frame_io::{read_frame, write_frame};
pub use crate::game_link_port::port;
pub use crate::protocol_constants::{MAGIC, MAX_FRAME_LENGTH, PROTOCOL_VERSION};
