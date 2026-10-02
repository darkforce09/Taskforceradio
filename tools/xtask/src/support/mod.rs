//! What the commands share: the repository listing, the workspace members, and the Rust,
//! Markdown, glob, README and reference-manifest readers.

pub(crate) mod cargo_workspace;
pub(crate) mod glob_pattern;
pub(crate) mod markdown;
pub(crate) mod readme_span;
pub(crate) mod reference_hashes;
pub(crate) mod repository;
pub(crate) mod rust_source;
