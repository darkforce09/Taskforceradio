//! The one-line statement of what `tfar_extension_probe` is for, kept beside the code it describes.

#[cfg(test)]
#[path = "tests/crate_role.rs"]
mod tests;

/// What `tfar_extension_probe` is for, in one sentence.
pub(crate) const CRATE_ROLE: &str = "tfar_extension_probe: the stand-in for Arma: a Windows executable \
     that loads an extension DLL with LoadLibrary, calls RVExtension \
     with a message and prints the reply, run under Proton's wine.";
