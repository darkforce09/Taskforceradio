//! The fixed numbers of the game link wire format.

/// The four bytes every `HELLO` opens with.
pub const MAGIC: [u8; 4] = *b"TFAR";

/// The protocol number both sides send in `HELLO`; a peer with another number is refused.
pub const PROTOCOL_VERSION: u16 = 1;

/// The largest legal value of a frame's length prefix: the kind byte plus the payload, 1 MiB.
pub const MAX_FRAME_LENGTH: u32 = 1_048_576;

/// The size in bytes of the `u32` little-endian length prefix that opens every frame.
pub const FRAME_LENGTH_PREFIX_SIZE: usize = 4;
