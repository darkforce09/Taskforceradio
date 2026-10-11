//! Which game messages count as game activity, the signal that the player is in a mission.

#[cfg(test)]
#[path = "tests/game_activity.rs"]
mod tests;

/// The prefix of the one message that does not count as game activity: the `VERSION` handshake
/// the game sends before and between missions.
pub const IN_GAME_TRIGGER_EXEMPT_PREFIX: &str = "VERSION";

/// Whether the raw message `message` counts as game activity: true unless it is longer than
/// [`IN_GAME_TRIGGER_EXEMPT_PREFIX`] and starts with it. The test runs on the text, before and
/// whatever the parse, as the pipe thread's `startWith("VERSION", command)` in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp` runs it; its strict length test makes a bare
/// `VERSION` count.
pub fn message_counts_as_game_activity(message: &str) -> bool {
    !(message.len() > IN_GAME_TRIGGER_EXEMPT_PREFIX.len()
        && message.starts_with(IN_GAME_TRIGGER_EXEMPT_PREFIX))
}
