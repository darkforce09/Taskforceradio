//! The one-line statement of what `tfar_client_state_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_client_state_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_client_state_1_0: the state the 1.0 plugin keeps per \
     TeamSpeak client, keyed by real name: position, radios, tangent \
     state, vehicle and intercom, and the effect and HRTF state of each \
     sender, with the local player's game data and the apply function of \
     each game and plugin message.";
