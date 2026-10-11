//! Arma extension probe: a stand-in for Arma that loads an extension library and calls it.
//!
//! **Role:** the stand-in for Arma: `tfar_extension_probe <library> [call ...]` loads an extension
//! with `libloading`, prints what `RVExtensionVersion` writes, then makes each call through
//! `RVExtension` with Arma's 10240-byte output buffer and prints the reply. Built for windows-gnu
//! it runs under Proton's wine against the DLL; built for Linux it loads a `.so` build of an
//! extension for host testing.
//!
//! **Position:** `tools`, tier 0, depending on no workspace crate (`libloading`, `anyhow`);
//! developers run it by hand.
//!
//! **Signals and state:** none beyond the loaded library; the calls run in order on the main
//! thread.
//!
//! **Invariants:** each call text reaches the extension unchanged except that `\t` stands for a
//! TAB; each reply is printed as the extension wrote it, up to its NUL, with escapes for control
//! characters. Exit code 0 when every call was made, 1 when the library failed or a call could not
//! be made, 2 for a command line the probe cannot read.

// Loading the extension and calling its C exports needs raw pointers and foreign calls.
#![allow(unsafe_code)]

mod extension_library;
mod probe_arguments;

use std::process::ExitCode;

use extension_library::ExtensionLibrary;
use probe_arguments::{ProbeArguments, parse_arguments};

fn main() -> ExitCode {
    let arguments = match parse_arguments(std::env::args().skip(1)) {
        Ok(arguments) => arguments,
        Err(usage) => {
            eprintln!(
                "tfar_extension_probe: loads an Arma extension and calls it the way Arma does.\n{usage}"
            );
            return ExitCode::from(2);
        }
    };
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tfar_extension_probe: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Loads the library, prints its version, then makes and prints every call.
fn run(arguments: &ProbeArguments) -> anyhow::Result<()> {
    let library = ExtensionLibrary::load(&arguments.library_path)?;
    println!("library: {}", library.path().display());
    match library.version() {
        Some(version) => println!("version: {}", version.escape_debug()),
        None => println!("version: (RVExtensionVersion is not exported)"),
    }
    for call in &arguments.calls {
        let reply = library.call(call)?;
        println!("call:  {}", call.escape_debug());
        println!("reply: {}", reply.escape_debug());
    }
    Ok(())
}
