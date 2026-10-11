//! The names a caller of `tfar_extension_0_9` imports with `use tfar_extension_0_9::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::call_handling::{
    GameLinkCalls, NOT_CONNECTED_TO_TEAMSPEAK, PIPE_CLOSED_BY_TEAMSPEAK, PIPE_NOT_OPENED,
    answer_call,
};
pub use crate::extension_exports::{DEBUG_SWITCH, EXTENSION_VERSION, FLAVOUR, client_config};
