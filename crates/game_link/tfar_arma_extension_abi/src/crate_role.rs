//! The one-line statement of what `tfar_arma_extension_abi` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_arma_extension_abi` is for, in one sentence.
pub const CRATE_ROLE: &str = "tfar_arma_extension_abi: the Arma side of an extension DLL: the \
     RVExtension and RVExtensionVersion exports, output writes \
     truncated to Arma's buffer size, and the focus query through \
     GetForegroundWindow.";
