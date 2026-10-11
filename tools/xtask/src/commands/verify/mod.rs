//! The `cargo xtask verify` gates, one module per gate, with the gate table and its runner.

mod crate_tiers;
mod dependency_firewalls;
mod editorconfig;
mod file_length;
pub(crate) mod gate;
pub(crate) mod gate_runner;
mod history_residue;
mod language_bans;
mod link_check;
mod reference_untouched;
mod source_files;
mod test_placement;
mod unsafe_allowlist;
