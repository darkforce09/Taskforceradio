//! The reception rules: which radios, own or on loudspeaker, carry a sender to the listener.

mod local_radio;
mod radio_receptions;

pub use local_radio::is_over_local_radio;
pub use radio_receptions::{is_over_radio, is_over_radio_into, receptions};
