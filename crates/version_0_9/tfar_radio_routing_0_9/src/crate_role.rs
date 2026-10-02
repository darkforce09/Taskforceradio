//! The one-line statement of what `tfar_radio_routing_0_9` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_radio_routing_0_9` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_radio_routing_0_9: the 0.9.12 reception rules: \
     isOverLocalRadio and isOverRadio decide whether a listener hears a \
     speaker directly, over SW, LR or DD radio or through a speaker \
     radio, and with which effect.";
