//! Runs `verify <gate>` and `verify all`: prints each violation and forms the exit code.
//!
//! **Role:** runs the selected gates, prints one `path:line: gate: message` line per violation on
//! stdout and one verdict line per gate on stderr, and folds the verdicts into one exit code.
//!
//! **Position:** called by `main` for `cargo xtask verify`; calls [`Gate::check`].
//!
//! **Signals and state:** none; writes to the given streams only.
//!
//! **Invariants:** exit 0 means every selected gate held, 1 that a violation was found, 2 that a
//! gate could not run; a gate that could not run never reports success.

#[cfg(test)]
#[path = "tests/gate_runner.rs"]
mod tests;

use std::io::Write;

use crate::support::repository::Repository;

use super::gate::{Gate, GateSelection};

/// The outcome of one or more gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Verdict {
    /// Every gate held.
    Held,
    /// A gate found a violation.
    Violated,
    /// A gate could not run.
    CouldNotRun,
}

impl Verdict {
    /// The process exit code of the verdict.
    pub(crate) fn exit_code(self) -> u8 {
        match self {
            Self::Held => 0,
            Self::Violated => 1,
            Self::CouldNotRun => 2,
        }
    }
}

/// Runs one gate, printing its violations to `violations` and its verdict to `verdicts`.
pub(crate) fn run_gate(
    gate: Gate,
    repository: &Repository,
    violations: &mut impl Write,
    verdicts: &mut impl Write,
) -> Verdict {
    match gate.check(repository) {
        Ok(found) if found.is_empty() => {
            let _ = writeln!(verdicts, "{}: held", gate.name());
            Verdict::Held
        }
        Ok(found) => {
            for violation in &found {
                let _ = writeln!(
                    violations,
                    "{}:{}: {}: {}",
                    violation.path,
                    violation.line,
                    gate.name(),
                    violation.message
                );
            }
            let _ = writeln!(verdicts, "{}: {} violation(s)", gate.name(), found.len());
            Verdict::Violated
        }
        Err(error) => {
            let _ = writeln!(verdicts, "{}: could not run: {error:#}", gate.name());
            Verdict::CouldNotRun
        }
    }
}

/// Runs the selection on stdout and stderr and returns the worst verdict.
pub(crate) fn run_selection(selection: GateSelection, repository: &Repository) -> Verdict {
    let gates: Vec<Gate> = match selection {
        GateSelection::All => Gate::ALL.to_vec(),
        GateSelection::One(gate) => vec![gate],
    };
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    gates
        .into_iter()
        .map(|gate| run_gate(gate, repository, &mut stdout, &mut stderr))
        .max()
        .unwrap_or(Verdict::Held)
}
