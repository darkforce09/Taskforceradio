//! Positions, directions and distances in Arma's frame, built on glam.
//!
//! **Role:** the vector and angle helpers every positional crate shares: Arma world positions, view
//! directions, distances (including the underwater rule) and correct normalisation.
//!
//! **Position:** `crates/foundation`, tier 0, depending on no workspace crate; used by
//! `tfar_antennas_1_0`, `tfar_arma_values`, `tfar_client_state_0_9`, `tfar_client_state_1_0`,
//! `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_hrtf`, `tfar_plugin_0_9`,
//! `tfar_plugin_1_0`, `tfar_radio_routing_0_9`, `tfar_radio_routing_1_0`, `tfar_spatial_audio`.
//!
//! **Signals and state:** none; pure functions and value types.
//!
//! **Invariants:** every direction it returns has unit length or is the zero vector, never NaN.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
