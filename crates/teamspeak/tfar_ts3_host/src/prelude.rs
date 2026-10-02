//! The names a caller of `tfar_ts3_host` imports with `use tfar_ts3_host::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
