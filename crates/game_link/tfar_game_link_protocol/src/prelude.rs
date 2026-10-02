//! The names a caller of `tfar_game_link_protocol` imports with `use tfar_game_link_protocol::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
