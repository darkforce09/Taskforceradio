//! The gate table: every repository gate by name, its check, and the violation it reports.
//!
//! **Role:** names each gate, maps a name to its check function, and defines [`Violation`], the
//! one `path:line: gate: message` line a gate prints per problem.
//!
//! **Position:** under `verify <gate>` and `verify all` in `gate_runner`, and the `ci-local` lane,
//! which runs each gate by name.
//!
//! **Signals and state:** none; the table is constant.
//!
//! **Invariants:** gate names are stable (CLAUDE.md and the standards cite them);
//! [`Gate::ALL`] lists every gate once, in the order of the laws.

use anyhow::Result;

use crate::support::repository::Repository;

use super::{
    crate_tiers, dependency_firewalls, editorconfig, file_length, history_residue, language_bans,
    link_check, reference_untouched, test_placement, unsafe_allowlist,
};

/// One problem a gate found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Violation {
    /// The repository-relative path of the offending file or folder.
    pub(crate) path: String,
    /// The 1-based line the problem is on (1 when the whole file is meant).
    pub(crate) line: usize,
    /// What is wrong, in a few words.
    pub(crate) message: String,
}

impl Violation {
    /// A violation at `path:line`.
    pub(crate) fn new(path: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line,
            message: message.into(),
        }
    }
}

/// Every repository gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gate {
    /// Members, layout metadata, tiers and the category edge matrix.
    CrateTiers,
    /// External crates only in their allowed crates.
    DependencyFirewalls,
    /// Where `unsafe_code` may be lifted, and `// SAFETY:` above every unsafe block.
    UnsafeAllowlist,
    /// The 500 and 1000 line ceilings.
    FileLength,
    /// Sibling test files only.
    TestPlacement,
    /// No program ids (bug, decision, milestone) in code or code READMEs.
    HistoryResidue,
    /// Repository-root links, anchors and backticked paths exist.
    LinkCheck,
    /// Final newline, LF, trailing whitespace and indentation.
    Editorconfig,
    /// No scripts, and C/C++ only in the oracle and shim folders.
    LanguageBans,
    /// Every reference file against the reference manifest.
    ReferenceUntouched,
}

impl Gate {
    /// Every gate, in the order of the laws.
    pub(crate) const ALL: [Self; 10] = [
        Self::CrateTiers,
        Self::DependencyFirewalls,
        Self::UnsafeAllowlist,
        Self::FileLength,
        Self::TestPlacement,
        Self::HistoryResidue,
        Self::LinkCheck,
        Self::Editorconfig,
        Self::LanguageBans,
        Self::ReferenceUntouched,
    ];

    /// The gate's command-line name.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::CrateTiers => "crate-tiers",
            Self::DependencyFirewalls => "dependency-firewalls",
            Self::UnsafeAllowlist => "unsafe-allowlist",
            Self::FileLength => "file-length",
            Self::TestPlacement => "test-placement",
            Self::HistoryResidue => "history-residue",
            Self::LinkCheck => "link-check",
            Self::Editorconfig => "editorconfig",
            Self::LanguageBans => "language-bans",
            Self::ReferenceUntouched => "reference-untouched",
        }
    }

    /// The gate with this command-line name.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|gate| gate.name() == name)
    }

    /// Runs the gate: its violations, or an error when it could not run.
    pub(crate) fn check(self, repository: &Repository) -> Result<Vec<Violation>> {
        match self {
            Self::CrateTiers => crate_tiers::check(repository),
            Self::DependencyFirewalls => dependency_firewalls::check(repository),
            Self::UnsafeAllowlist => unsafe_allowlist::check(repository),
            Self::FileLength => file_length::check(repository),
            Self::TestPlacement => test_placement::check(repository),
            Self::HistoryResidue => history_residue::check(repository),
            Self::LinkCheck => link_check::check(repository),
            Self::Editorconfig => editorconfig::check(repository),
            Self::LanguageBans => language_bans::check(repository),
            Self::ReferenceUntouched => reference_untouched::check(repository),
        }
    }
}

/// What `verify` was asked to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GateSelection {
    /// `verify all`.
    All,
    /// `verify <gate>`.
    One(Gate),
}

/// Parses `all` or a gate name, for the command line.
pub(crate) fn parse_gate_selection(text: &str) -> std::result::Result<GateSelection, String> {
    if text == "all" {
        return Ok(GateSelection::All);
    }
    Gate::from_name(text)
        .map(GateSelection::One)
        .ok_or_else(|| {
            let names: Vec<&str> = Gate::ALL.iter().map(|gate| gate.name()).collect();
            format!(
                "unknown gate `{text}`; expected `all` or one of: {}",
                names.join(", ")
            )
        })
}
