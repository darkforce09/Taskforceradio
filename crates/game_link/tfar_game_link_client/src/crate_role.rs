//! The one-line statement of what `tfar_game_link_client` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_game_link_client` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_game_link_client: the extension side of the game link: \
     connects lazily with a connect timeout of at most 50 ms and a 1 s \
     backoff, sends sync, async and focus frames, waits at most 1000 ms \
     for a sync response, and reports each outcome without any \
     game-facing string.";
