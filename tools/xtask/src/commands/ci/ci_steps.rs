//! The steps of the `ci-local` lane, in order: the toolchain steps, then every repository gate.
//!
//! **Role:** the one table of what `cargo xtask ci ci-local` runs.
//!
//! **Position:** read by `ci_local`; names each gate through the verify gate table.
//!
//! **Signals and state:** none; the table is built fresh on each call.
//!
//! **Invariants:** step names are unique; the toolchain steps come first in the order fmt,
//! clippy, clippy-windows, test, deny, typos; every gate in `Gate::ALL` follows once.

#[cfg(test)]
#[path = "tests/ci_steps.rs"]
mod tests;

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use crate::commands::verify::gate::Gate;

/// What a step runs.
#[derive(Debug, Clone)]
pub(crate) enum StepProgram {
    /// A cargo subcommand with its arguments.
    Cargo(Vec<String>),
    /// Another program on the `PATH` with its arguments.
    Tool(String, Vec<String>),
    /// `cargo xtask verify <gate>`, run as this binary.
    Gate(Gate),
}

/// One step of a lane.
#[derive(Debug, Clone)]
pub(crate) struct CiStep {
    /// The step's name, used for its log file.
    pub(crate) name: String,
    /// What it runs.
    pub(crate) program: StepProgram,
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(ToOwned::to_owned).collect()
}

/// The windows clippy step's packages: the extension side of the workspace.
const WINDOWS_CLIPPY: &str = "clippy -p tfar_extension_0_9 -p tfar_extension_1_0 -p tfar_arma_extension_abi \
     -p tfar_game_link_client -p tfar_game_link_protocol -p tfar_extension_probe \
     --target x86_64-pc-windows-gnu -- -D warnings";

/// Every step of `ci-local`, in order.
pub(crate) fn ci_local_steps() -> Vec<CiStep> {
    let mut steps = vec![
        CiStep {
            name: "fmt".to_owned(),
            program: StepProgram::Cargo(words("fmt --all --check")),
        },
        CiStep {
            name: "clippy".to_owned(),
            program: StepProgram::Cargo(words("clippy --workspace --all-targets -- -D warnings")),
        },
        CiStep {
            name: "clippy-windows".to_owned(),
            program: StepProgram::Cargo(words(WINDOWS_CLIPPY)),
        },
        CiStep {
            name: "test".to_owned(),
            program: StepProgram::Cargo(words("test --workspace")),
        },
        CiStep {
            name: "deny".to_owned(),
            program: StepProgram::Tool("cargo-deny".to_owned(), words("deny check")),
        },
        CiStep {
            name: "typos".to_owned(),
            program: StepProgram::Tool("typos".to_owned(), Vec::new()),
        },
    ];
    steps.extend(Gate::ALL.into_iter().map(|gate| CiStep {
        name: gate.name().to_owned(),
        program: StepProgram::Gate(gate),
    }));
    steps
}

impl CiStep {
    /// The command line in words, for the log header.
    pub(crate) fn command_line(&self) -> String {
        match &self.program {
            StepProgram::Cargo(arguments) => format!("cargo {}", arguments.join(" ")),
            StepProgram::Tool(program, arguments) if program == "cargo-deny" => {
                format!("cargo {}", arguments.join(" "))
            }
            StepProgram::Tool(program, arguments) => format!("{program} {}", arguments.join(" "))
                .trim_end()
                .to_owned(),
            StepProgram::Gate(gate) => format!("cargo xtask verify {}", gate.name()),
        }
    }

    /// The process that runs the step from `root`.
    pub(crate) fn command(&self, root: &Path, xtask: &Path) -> Command {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
        let mut command = match &self.program {
            StepProgram::Cargo(arguments) => {
                let mut command = Command::new(cargo);
                command.args(arguments);
                command
            }
            StepProgram::Tool(program, arguments) => {
                let mut command = Command::new(program);
                command.args(arguments);
                command
            }
            StepProgram::Gate(gate) => {
                let mut command = Command::new(xtask);
                command.args(["verify", gate.name()]);
                command
            }
        };
        command.current_dir(root);
        command
    }

    /// Whether the step is a repository gate, whose exit code 2 means "could not run".
    pub(crate) fn is_gate(&self) -> bool {
        matches!(self.program, StepProgram::Gate(_))
    }
}
