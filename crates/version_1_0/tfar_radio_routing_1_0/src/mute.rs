//! The automatic mute rules of serious mode and the record of who this plugin muted.

mod killed_mute_matrix;
mod mute_decision;
mod mute_tracker;

pub use killed_mute_matrix::{MuteChanges, killed_mute_matrix};
pub use mute_decision::{MuteDecision, VOICE_RANGE_MARGIN, mute_decision};
pub use mute_tracker::MuteTracker;
