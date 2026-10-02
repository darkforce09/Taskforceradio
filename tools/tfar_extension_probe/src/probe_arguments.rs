//! The probe's command line: the extension library to load and the calls to make.

use std::path::PathBuf;

#[cfg(test)]
#[path = "tests/probe_arguments.rs"]
mod tests;

/// The calls made when the command line names none: the ping and the version query.
pub(crate) const DEFAULT_CALLS: &[&str] = &["TS_INFO\tPING", "TS_INFO\tVERSION"];

/// The usage line printed for a command line the probe cannot read.
pub(crate) const USAGE: &str = "usage: tfar_extension_probe <path-to-extension-library> [call ...]\n\
     A call's `\\t` stands for a TAB, the field separator of every TFAR call; with no call the \
     probe sends `TS_INFO\\tPING` and `TS_INFO\\tVERSION`.";

/// What one probe run does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProbeArguments {
    /// The extension library to load: the DLL under wine, a `.so` build on Linux.
    pub(crate) library_path: PathBuf,
    /// The call texts, in order, with every `\t` already turned into a TAB.
    pub(crate) calls: Vec<String>,
}

/// Reads the arguments after the program name.
///
/// # Errors
///
/// [`USAGE`] when no library path is given or it is `-h` or `--help`.
pub(crate) fn parse_arguments(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProbeArguments, &'static str> {
    let mut arguments = arguments.into_iter();
    let library_path = match arguments.next() {
        Some(path) if path != "-h" && path != "--help" && !path.is_empty() => PathBuf::from(path),
        _ => return Err(USAGE),
    };
    let mut calls: Vec<String> = arguments.map(|call| expand_tab_escapes(&call)).collect();
    if calls.is_empty() {
        calls = DEFAULT_CALLS
            .iter()
            .map(|call| (*call).to_owned())
            .collect();
    }
    Ok(ProbeArguments {
        library_path,
        calls,
    })
}

/// `text` with every two-character `\t` replaced by a TAB; nothing else changes.
pub(crate) fn expand_tab_escapes(text: &str) -> String {
    text.replace("\\t", "\t")
}
