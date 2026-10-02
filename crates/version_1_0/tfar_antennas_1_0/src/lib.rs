//! The 1.0 antenna and radio tower model used for terrain-aware reception.
//!
//! **Role:** the 1.0 antenna model: radio towers and their positions and ranges, and the antenna
//! connection that lets a signal hop past terrain.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_geometry`, `tfar_arma_values`;
//! used by `tfar_plugin_1_0`, `tfar_radio_routing_1_0`.
//!
//! **Signals and state:** the registry of antennas the game reports, owned by the caller and kept
//! behind the caller's lock; no thread of its own.
//!
//! **Invariants:** a connection is only found through antennas within their range, and its loss
//! is below 1; "no connection" is `None`, read as [`NO_CONNECTION_LOSS`] where a minimum is taken.

mod antenna;
mod antenna_connection;
mod antenna_manager;
mod crate_role;
mod error;
mod net_id;
pub mod prelude;

pub use antenna::Antenna;
pub use antenna_connection::{AntennaConnection, NO_CONNECTION_LOSS, loss_or_no_connection};
pub use antenna_manager::AntennaManager;
pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use net_id::NetId;
