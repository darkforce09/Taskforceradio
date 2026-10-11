//! The 1.0 plugin's debug window, an egui window on X11.
//!
//! **Role:** the 1.0 plugin's `/tfar debug` window: one long-lived UI thread running an `eframe`
//! window on X11 that draws the latest [`DebugSnapshot`] the plugin publishes, section by section.
//!
//! **Position:** `crates/interface`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_1_0`.
//!
//! **Signals and state:** the UI thread `tfar-debug-window`, the plugin's show, hide and shutdown
//! requests behind a mutex and condition variable, and the latest snapshot in an `ArcSwap`.
//!
//! **Invariants:** one UI thread and one event loop per process, reused for every window and
//! joined at shutdown within one second.

mod debug_snapshot;
mod egui_window;
mod error;
pub mod prelude;
mod window_control;

pub use debug_snapshot::{
    DebugSection, DebugSnapshot, DebugTable, KeyValueRow, SectionContent, TableRow,
};
pub use error::{Error, Result};
pub use window_control::{DebugWindow, DebugWindowHandle, SHUTDOWN_TIMEOUT};
