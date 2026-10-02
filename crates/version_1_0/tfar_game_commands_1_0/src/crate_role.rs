//! The one-line statement of what `tfar_game_commands_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_game_commands_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_game_commands_1_0: the typed form of every 1.0 game message \
     (POS, TANGENT, FREQ, SETCFG, MISSIONEND, data frames and the \
     rest), parsed with checked token counts, and the answers the plugin gives sync calls.";
