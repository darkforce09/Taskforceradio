//! The one-line statement of what `tfar_radio_routing_1_0` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_radio_routing_1_0` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_radio_routing_1_0: the 1.0 reception rules: isOverRadio and \
     its neighbours decide whether a listener hears a speaker over radio \
     (with antennas and underwater range), over intercom or through a \
     speaker radio, and with which error level, with the serious-mode \
     mute rules.";
