//! The reception rules: how the local player hears a sender over radio.

mod intercom;
mod local_radio;
mod over_radio;
mod speakers;

pub use local_radio::is_over_local_radio;
pub use over_radio::is_over_radio;
