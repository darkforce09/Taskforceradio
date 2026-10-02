//! The names a caller of `tfar_arma_extension_abi` imports with `use tfar_arma_extension_abi::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::arma_extension;
pub use crate::command_line::arma_command_line_contains;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::foreground_window::foreground_window_is_arma;
pub use crate::reply_buffer::write_reply;
