//! The command line of `cargo xtask`: the subcommands and their arguments.
//!
//! **Role:** declares the `clap` parser for `verify`, `ci`, `dist` and `reference-manifest`.
//!
//! **Position:** parsed by `main`, which dispatches to the command modules.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** the subcommand and gate names are stable: CLAUDE.md and the standards cite
//! them.

use clap::{Parser, Subcommand};

use crate::commands::verify::gate::{GateSelection, parse_gate_selection};

/// The repository task runner: verify gates, the local CI lane, dist and the reference manifest.
#[derive(Debug, Parser)]
#[command(name = "cargo xtask", bin_name = "cargo xtask")]
pub(crate) struct Cli {
    /// The task to run.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Every xtask command.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Runs one repository gate, or `all`; exit 0 held, 1 violation, 2 could not run.
    Verify {
        /// `all` or a gate name such as `crate-tiers`.
        #[arg(value_parser = parse_gate_selection)]
        gate: GateSelection,
    },
    /// Runs a CI lane.
    Ci {
        /// The lane.
        #[command(subcommand)]
        lane: CiLane,
    },
    /// Builds the plugins and extensions in release and copies them, renamed, into target/dist/.
    Dist,
    /// Writes reference/MANIFEST.sha256; refuses to overwrite it without `--force --reason`.
    ReferenceManifest {
        /// Overwrite an existing manifest.
        #[arg(long, requires = "reason")]
        force: bool,
        /// Why the manifest is rewritten (required with `--force`).
        #[arg(long, requires = "force")]
        reason: Option<String>,
    },
}

/// The CI lanes.
#[derive(Debug, Subcommand)]
pub(crate) enum CiLane {
    /// Every toolchain step and every repository gate, in order, stopping at the first failure.
    CiLocal,
}
