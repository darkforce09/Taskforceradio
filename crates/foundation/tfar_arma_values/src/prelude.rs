//! The names a caller of `tfar_arma_values` imports with `use tfar_arma_values::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
