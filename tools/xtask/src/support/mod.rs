//! What the commands share: the repository listing, the workspace members, and the Rust,
//! Markdown and reference-manifest readers.

pub(crate) mod cargo_workspace;
pub(crate) mod markdown;
pub(crate) mod reference_hashes;
pub(crate) mod repository;
pub(crate) mod rust_source;
