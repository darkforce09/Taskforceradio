//! The names a caller of `tfar_antennas_1_0` imports with `use tfar_antennas_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::antenna::Antenna;
pub use crate::antenna_connection::{AntennaConnection, NO_CONNECTION_LOSS, loss_or_no_connection};
pub use crate::antenna_manager::AntennaManager;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::net_id::NetId;
