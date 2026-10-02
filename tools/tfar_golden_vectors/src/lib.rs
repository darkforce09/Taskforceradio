//! Reference outputs of the original C++ audio code, used as test fixtures for the audio crates.
//!
//! **Role:** the test oracle for the audio crates: builds the original DSPFilters, SimpleComp and
//! clunk code from `reference/` and FAudio's F3DAudio with `cc`, and turns their output into
//! fixtures the Rust crates must match within tolerance.
//!
//! **Position:** `tools`, tier 0, depending on no workspace crate; the audio crates' tests read its
//! fixtures.
//!
//! **Signals and state:** none; fixtures are generated data.
//!
//! **Invariants:** the oracle code is built only here, never into a deliverable.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
