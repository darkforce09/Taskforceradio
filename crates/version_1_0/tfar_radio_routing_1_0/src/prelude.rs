//! The names a caller of `tfar_radio_routing_1_0` imports with `use tfar_radio_routing_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
