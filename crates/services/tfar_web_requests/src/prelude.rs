//! The names a caller of `tfar_web_requests` imports with `use tfar_web_requests::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
