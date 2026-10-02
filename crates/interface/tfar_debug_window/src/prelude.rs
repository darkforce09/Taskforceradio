//! The names a caller of `tfar_debug_window` imports with `use tfar_debug_window::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::debug_snapshot::{DebugSection, DebugSnapshot, DebugTable};
pub use crate::window_control::{DebugWindow, DebugWindowHandle};
