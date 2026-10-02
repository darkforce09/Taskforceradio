//! A fake TeamSpeak host that loads a plugin .so and drives it through scripted scenarios.
//!
//! **Role:** the test host for the plugins: loads a plugin `.so` with `libloading`, hands it a fake
//! `Ts3Functions` table, plays the game side over the game link and runs scripted scenarios.
//!
//! **Position:** `tools`, tier 2, depending on `tfar_game_link_protocol`, `tfar_game_link_client`;
//! the plugin apps' test suites and developers run it.
//!
//! **Signals and state:** the loaded plugin and the fake host state it calls into.
//!
//! **Invariants:** a scenario's outcome depends only on the scenario and the plugin.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
