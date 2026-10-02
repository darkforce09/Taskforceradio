//! The one-line statement of what `tfar_golden_vectors` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_golden_vectors` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_golden_vectors: the test oracle for the audio crates: builds \
     the original DSPFilters, SimpleComp and clunk code from reference/ \
     and FAudio's F3DAudio with cc, and turns their output into \
     fixtures the Rust crates must match within tolerance.";
