//! `cargo xtask ci ci-local`: every step as its own child process with its own log, stopping at
//! the first failure.
//!
//! **Role:** runs a list of [`CiStep`]s in order, writes each step's output to
//! `.workstation/logs/ci/<nn>-<step>.log`, prints `held`, `failed` or `not run` per step, and forms the
//! lane's exit code.
//!
//! **Position:** called by `main`; runs cargo, cargo-deny, typos and this binary's own
//! `verify <gate>` as children.
//!
//! **Signals and state:** one child process at a time; writes only under `.workstation/logs/ci/`.
//!
//! **Invariants:** a step that fails stops the lane and every later step prints `not run`; a
//! missing tool, or a gate that exits 2, is `not run`, never `held`; exit 0 only when every step
//! held.

#[cfg(test)]
#[path = "tests/ci_local.rs"]
mod tests;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};

use crate::support::repository::Repository;

use super::ci_steps::{CiStep, ci_local_steps};

/// The outcome of one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepOutcome {
    /// The step exited 0.
    Held,
    /// The step exited non-zero.
    Failed,
    /// The step could not run, or the lane stopped before it.
    NotRun,
}

impl StepOutcome {
    fn word(self) -> &'static str {
        match self {
            Self::Held => "held",
            Self::Failed => "failed",
            Self::NotRun => "not run",
        }
    }
}

/// Runs the `ci-local` lane in the repository and returns its exit code.
pub(crate) fn run(repository: &Repository) -> Result<u8> {
    let xtask = std::env::current_exe().context("finding the xtask binary")?;
    let log_folder = repository.absolute(".workstation/logs/ci");
    let outcomes = run_steps(
        &ci_local_steps(),
        repository.root(),
        &xtask,
        &log_folder,
        &mut std::io::stdout().lock(),
    )?;
    Ok(exit_code(&outcomes))
}

/// The lane's exit code: 0 all held, 1 a step failed, 2 a step could not run.
pub(crate) fn exit_code(outcomes: &[StepOutcome]) -> u8 {
    match outcomes
        .iter()
        .find(|outcome| **outcome != StepOutcome::Held)
    {
        None => 0,
        Some(StepOutcome::Failed) => 1,
        Some(_) => 2,
    }
}

/// Runs the steps in order from `root`, logging each to `log_folder` and printing one status line
/// per step to `out`.
pub(crate) fn run_steps(
    steps: &[CiStep],
    root: &Path,
    xtask: &Path,
    log_folder: &Path,
    out: &mut impl Write,
) -> Result<Vec<StepOutcome>> {
    std::fs::create_dir_all(log_folder)
        .with_context(|| format!("creating {}", log_folder.display()))?;
    let mut outcomes = Vec::new();
    let mut stopped = false;
    for (index, step) in steps.iter().enumerate() {
        let log_path = log_folder.join(format!("{:02}-{}.log", index + 1, step.name));
        let outcome = if stopped {
            StepOutcome::NotRun
        } else {
            run_step(step, root, xtask, &log_path)?
        };
        let _ = writeln!(
            out,
            "{:>2}  {:<22} {:<8} {}",
            index + 1,
            step.name,
            outcome.word(),
            if stopped {
                String::new()
            } else {
                log_path.display().to_string()
            }
        );
        if outcome != StepOutcome::Held {
            stopped = true;
        }
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

fn run_step(step: &CiStep, root: &Path, xtask: &Path, log_path: &Path) -> Result<StepOutcome> {
    let mut log =
        File::create(log_path).with_context(|| format!("creating {}", log_path.display()))?;
    writeln!(log, "$ {}", step.command_line())?;
    log.flush()?;
    let mut command = step.command(root, xtask);
    command
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log.try_clone()?);
    let status = match command.status() {
        Ok(status) => status,
        Err(error) => {
            writeln!(log, "could not start `{}`: {error}", step.command_line())?;
            return Ok(StepOutcome::NotRun);
        }
    };
    writeln!(log, "exit status: {status}")?;
    Ok(match status.code() {
        Some(0) => StepOutcome::Held,
        Some(2) if step.is_gate() => StepOutcome::NotRun,
        _ => StepOutcome::Failed,
    })
}
