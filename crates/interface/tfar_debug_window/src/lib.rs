//! The 1.0 plugin's debug window, an egui window on X11.
//!
//! **Role:** the 1.0 plugin's debug window: one long-lived UI thread running an `eframe` window on
//! X11 with the panels that show client and radio state.
//!
//! **Position:** `crates/interface`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_1_0`.
//!
//! **Signals and state:** the UI thread and the latest published snapshot.
//!
//! **Invariants:** one event loop per process, joined at shutdown.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
