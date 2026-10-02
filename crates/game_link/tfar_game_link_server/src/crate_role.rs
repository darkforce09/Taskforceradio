//! The one-line statement of what `tfar_game_link_server` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_game_link_server` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_game_link_server: the plugin side of the game link: listens \
     on 127.0.0.1, accepts the extension's connection, hands each sync \
     request to the plugin and writes its answer, and queues async \
     messages in arrival order.";
