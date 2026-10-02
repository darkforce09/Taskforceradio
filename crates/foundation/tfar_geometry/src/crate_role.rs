//! The one-line statement of what `tfar_geometry` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_geometry` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_geometry: the vector and angle helpers every positional crate \
     shares: Arma world positions, view directions, distances \
     (including the underwater rule) and correct normalisation.";
