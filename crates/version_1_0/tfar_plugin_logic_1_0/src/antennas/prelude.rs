//! The names a caller of `tfar_plugin_logic_1_0::antennas` imports with `use crate::antennas::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::antennas::antenna::Antenna;
pub use crate::antennas::antenna_connection::{
    AntennaConnection, NO_CONNECTION_LOSS, loss_or_no_connection,
};
pub use crate::antennas::antenna_manager::AntennaManager;
pub use crate::antennas::net_id::NetId;
