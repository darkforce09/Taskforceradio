//! The names a caller of `tfar_extension_1_0` imports with `use tfar_extension_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::call_handling::{
    ASYNC_ACCEPTED, ASYNC_MARKER, CONFIGURATION_NEEDED, DATA_FRAME_COMMAND, GameLinkCalls,
    MISSION_END_COMMAND, NOT_CONNECTED_TO_TEAMSPEAK, answer_call,
};
pub use crate::extension_exports::{DEBUG_SWITCH, EXTENSION_VERSION, FLAVOUR, client_config};
pub use crate::focus_watch::{FOCUS_READ_INTERVAL, FocusWatch};
