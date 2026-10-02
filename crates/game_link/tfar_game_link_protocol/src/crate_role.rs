//! The one-line statement of what `tfar_game_link_protocol` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_game_link_protocol` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_game_link_protocol: the wire format of the game link: u32 \
     little-endian length, a kind byte and a postcard payload, the \
     HELLO handshake, the frame kinds and the ports each plugin listens \
     on.";
