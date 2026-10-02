//! The `dependency-firewalls` gate: each firewalled external crate only in its allowed crates.
//!
//! **Role:** checks every direct external dependency of every member against the firewall table
//! in `layout`, and that `tracy-client` stays optional behind a feature.
//!
//! **Position:** a repository gate; reads `cargo metadata` through `cargo_workspace`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** could not run exactly when `cargo metadata` fails; a crate not in the table is
//! free; every dependency table counts (normal, dev and build).

#[cfg(test)]
#[path = "tests/dependency_firewalls.rs"]
mod tests;

use anyhow::Result;

use crate::layout::{FIREWALLS, OPTIONAL_ONLY_EXTERNAL};
use crate::support::cargo_workspace::load_members;
use crate::support::repository::Repository;

use super::gate::Violation;
use super::source_files::dependency_line;

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for member in load_members(repository)? {
        let manifest_text = repository.read_text(&member.manifest).unwrap_or_default();
        for dependency in member
            .dependencies
            .iter()
            .filter(|dependency| !dependency.is_workspace)
        {
            let Some((_, scope)) = FIREWALLS.iter().find(|(name, _)| *name == dependency.name)
            else {
                continue;
            };
            let line = dependency_line(&manifest_text, &dependency.name);
            if !scope.admits(&member.name, member.category()) {
                violations.push(Violation::new(
                    &member.manifest,
                    line,
                    format!(
                        "`{}` is firewalled: only {} may depend on it",
                        dependency.name,
                        scope.describe()
                    ),
                ));
            } else if dependency.name == OPTIONAL_ONLY_EXTERNAL && !dependency.is_optional {
                violations.push(Violation::new(
                    &member.manifest,
                    line,
                    format!(
                        "`{OPTIONAL_ONLY_EXTERNAL}` must be optional, behind the `tracy` feature"
                    ),
                ));
            }
        }
    }
    Ok(violations)
}
