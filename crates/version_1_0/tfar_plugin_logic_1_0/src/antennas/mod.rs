//! The 1.0 antenna and radio tower model used for terrain-aware reception.
//!
//! **Role:** the 1.0 antenna model: radio towers and their positions and ranges, and the antenna
//! connection that lets a signal hop past terrain.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on `tfar_foundation::geometry`, `tfar_foundation::arma_values`;
//! used by `tfar_plugin_1_0`, `tfar_plugin_logic_1_0::radio_routing`.
//!
//! **Signals and state:** the registry of antennas the game reports, owned by the caller and kept
//! behind the caller's lock; no thread of its own.
//!
//! **Invariants:** a connection is only found through antennas within their range, and its loss
//! is below 1; "no connection" is `None`, read as [`NO_CONNECTION_LOSS`] where a minimum is taken.

mod antenna;
mod antenna_connection;
mod antenna_manager;
mod error;
mod net_id;
pub mod prelude;

pub use antenna::Antenna;
pub use antenna_connection::{AntennaConnection, NO_CONNECTION_LOSS, loss_or_no_connection};
pub use antenna_manager::AntennaManager;
pub use error::{Error, Result};
pub use net_id::NetId;
