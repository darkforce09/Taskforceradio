//! `cargo xtask`: the repository's single entry point for gates, the local CI lane, dist and
//! the reference manifest.
//!
//! **Role:** parses the command line and runs one command, turning its result into the process
//! exit code.
//!
//! **Position:** `tools`, tier 0, depending on no workspace crate; run through the `cargo xtask`
//! alias in `.cargo/config.toml` by developers and agents.
//!
//! **Signals and state:** none; single-threaded, one command per run.
//!
//! **Invariants:** `verify` exits 0 held, 1 violation, 2 could not run; every other command exits
//! 0 on success and 1 with an `error:` line on failure.

mod cli;
mod commands;
#[cfg(test)]
#[path = "tests/fixture_tree.rs"]
mod fixture_tree;
mod layout;
mod support;

use std::process::ExitCode;

use clap::Parser as _;

use crate::cli::{CiLane, Cli, Command};
use crate::commands::verify::gate_runner;
use crate::commands::{ci, dist, reference_manifest};
use crate::support::repository::Repository;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repository = match Repository::of_this_workspace() {
        Ok(repository) => repository,
        Err(error) => {
            eprintln!("error: {error:#}");
            return ExitCode::from(2);
        }
    };
    let result = match cli.command {
        Command::Verify { gate } => {
            return ExitCode::from(gate_runner::run_selection(gate, &repository).exit_code());
        }
        Command::Ci {
            lane: CiLane::CiLocal,
        } => ci::ci_local::run(&repository).map(ExitCode::from),
        Command::Dist => dist::run(&repository).map(|()| ExitCode::SUCCESS),
        Command::ReferenceManifest { reason, .. } => {
            reference_manifest::run(&repository, reason.as_deref()).map(|_| ExitCode::SUCCESS)
        }
    };
    result.unwrap_or_else(|error| {
        eprintln!("error: {error:#}");
        ExitCode::FAILURE
    })
}
