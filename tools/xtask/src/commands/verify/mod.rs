//! The `cargo xtask verify` gates, one module per gate, with the gate table and its runner.

mod crate_anatomy;
mod crate_tiers;
mod dependency_firewalls;
mod editorconfig;
mod file_length;
pub(crate) mod gate;
pub(crate) mod gate_runner;
mod language_bans;
mod link_check;
mod markdown_placement;
mod module_headers;
mod readme_coverage;
mod readme_sections;
mod reference_untouched;
mod source_files;
mod status_lines;
mod test_placement;
mod unsafe_allowlist;
