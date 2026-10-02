//! Arma extension probe: a stand-in for Arma that loads an extension DLL and calls it.
//!
//! **Role:** the stand-in for Arma: a Windows executable that loads an extension DLL with
//! `LoadLibrary`, calls `RVExtension` with a message and prints the reply, run under Proton's wine.
//!
//! **Position:** `tools`, tier 0, depending on no workspace crate; developers run it under Proton's
//! wine against a built extension.
//!
//! **Signals and state:** none; one call per run.
//!
//! **Invariants:** the probe passes the message unchanged and prints the reply unchanged.

mod crate_role;

fn main() {
    println!("{}", crate_role::CRATE_ROLE);
}
