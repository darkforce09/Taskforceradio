//! The 1.0 antenna and radio tower model used for terrain-aware reception.
//!
//! **Role:** the 1.0 antenna model: radio towers and their positions and ranges, and the antenna
//! connection that lets a signal hop past terrain.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_geometry`, `tfar_arma_values`;
//! used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.
//!
//! **Signals and state:** the registry of antennas the game reports.
//!
//! **Invariants:** a connection is only found through antennas within their range.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
