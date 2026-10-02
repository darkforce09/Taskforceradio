//! The one-line statement of what `tfar_client_state_0_9` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_client_state_0_9` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_client_state_0_9: the state the 0.9.12 plugin keeps per \
     TeamSpeak client, keyed by real name: position, radios, tangent \
     state, voice volume and the effect and HRTF state of each sender.";
