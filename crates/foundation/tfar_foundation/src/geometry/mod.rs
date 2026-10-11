//! Positions, directions and distances in Arma's frame, built on glam.
//!
//! **Role:** the vector and angle helpers every positional crate shares: Arma world positions, view
//! directions, distances (including the underwater rule) and correct normalisation.
//!
//! **Position:** a module of `tfar_foundation`, depending on no workspace crate; used by
//! `tfar_plugin_logic_1_0::antennas`, `tfar_foundation::arma_values`, `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`,
//! `tfar_plugin_logic_0_9::game_commands`, `tfar_plugin_logic_1_0::game_commands`, `tfar_audio::hrtf`, `tfar_plugin_0_9`,
//! `tfar_plugin_1_0`, `tfar_plugin_logic_0_9::radio_routing`, `tfar_plugin_logic_1_0::radio_routing`, `tfar_audio::spatial_audio`.
//!
//! **Signals and state:** none; pure functions and value types, callable from any thread.
//!
//! **Invariants:** every direction it returns has unit length or is the zero vector, never NaN;
//! every constructor rejects non-finite coordinates with an `Error`.

mod angle;
mod direction;
mod error;
mod position;
pub mod prelude;
mod rotation;
mod underwater_distance;

pub use angle::Angle;
pub use direction::Direction;
pub use error::{Error, Result};
pub use position::Position;
pub use rotation::rotate_about_z;
pub use underwater_distance::distance_underwater;
