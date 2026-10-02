//! The names a caller of `tfar_golden_vectors` imports with `use tfar_golden_vectors::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
