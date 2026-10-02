//! The `crate-tiers` gate: membership, layout metadata, tiers and the category edge matrix.
//!
//! **Role:** checks that every manifest under apps, crates and tools is a member; that each
//! member declares `category`, `tier` and `targets`; that its path is category plus name and its
//! name its folder name; that its tier is 1 plus its highest workspace dependency's (0 with none);
//! and that every workspace edge keeps the edge matrix, the version ban and the app rules.
//!
//! **Position:** a repository gate; reads `cargo metadata` through `cargo_workspace` and the laws
//! from `layout`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** could not run (an error) exactly when `cargo metadata` fails; dev-dependencies
//! count for the version ban and the app ban but not for tiers or the matrix.

#[cfg(test)]
#[path = "tests/crate_tiers.rs"]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use crate::layout::{CATEGORIES, EdgeTable, MEMBER_ROOTS, edge_violation};
use crate::support::cargo_workspace::{EdgeKind, Member, load_members};
use crate::support::repository::{self, Repository};

use super::gate::Violation;
use super::source_files::{dependency_line, line_of};

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let members = load_members(repository)?;
    let mut violations = Vec::new();
    check_membership(repository, &members, &mut violations)?;
    let by_name: BTreeMap<&str, &Member> = members
        .iter()
        .map(|member| (member.name.as_str(), member))
        .collect();
    for member in &members {
        let manifest_text = repository.read_text(&member.manifest).unwrap_or_default();
        check_layout(member, &manifest_text, &mut violations);
        check_edges(member, &by_name, &manifest_text, &mut violations);
        check_tier(member, &by_name, &manifest_text, &mut violations);
    }
    Ok(violations)
}

fn check_membership(
    repository: &Repository,
    members: &[Member],
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let member_manifests: BTreeSet<&str> = members
        .iter()
        .map(|member| member.manifest.as_str())
        .collect();
    for file in repository.files()? {
        let under_roots = MEMBER_ROOTS
            .iter()
            .any(|root| file.starts_with(&format!("{root}/")));
        if under_roots
            && repository::file_name(file) == "Cargo.toml"
            && !repository::has_folder_named(file, "tests")
            && !member_manifests.contains(file.as_str())
        {
            violations.push(Violation::new(
                file,
                1,
                "this manifest is not a workspace member",
            ));
        }
    }
    Ok(())
}

fn check_layout(member: &Member, manifest_text: &str, violations: &mut Vec<Violation>) {
    let layout_line = line_of(manifest_text, "[package.metadata.layout]");
    let Some(layout) = &member.layout else {
        violations.push(Violation::new(
            &member.manifest,
            1,
            "no [package.metadata.layout] with category, tier and targets",
        ));
        return;
    };
    match layout.category.as_deref() {
        None => violations.push(Violation::new(
            &member.manifest,
            layout_line,
            "layout has no `category`",
        )),
        Some(category) if !CATEGORIES.contains(&category) => violations.push(Violation::new(
            &member.manifest,
            line_of(manifest_text, "category ="),
            format!(
                "`{category}` is not a category: expected one of {}",
                CATEGORIES.join(", ")
            ),
        )),
        Some(category) => {
            let expected_folder = format!("{category}/{}", member.name);
            if member.folder != expected_folder {
                violations.push(Violation::new(
                    &member.manifest,
                    line_of(manifest_text, "category ="),
                    format!(
                        "the crate lives at `{}`, but category plus name is `{expected_folder}`",
                        member.folder
                    ),
                ));
            }
        }
    }
    if repository::file_name(&member.folder) != member.name {
        violations.push(Violation::new(
            &member.manifest,
            line_of(manifest_text, "name ="),
            format!(
                "the package `{}` must be named like its folder `{}`",
                member.name,
                repository::file_name(&member.folder)
            ),
        ));
    }
    if layout.tier.is_none() {
        violations.push(Violation::new(
            &member.manifest,
            layout_line,
            "layout has no integer `tier`",
        ));
    }
    match layout.targets.as_deref() {
        Some("any" | "windows") => {}
        Some(other) => violations.push(Violation::new(
            &member.manifest,
            line_of(manifest_text, "targets ="),
            format!("`targets = \"{other}\"`: expected \"any\" or \"windows\""),
        )),
        None => violations.push(Violation::new(
            &member.manifest,
            layout_line,
            "layout has no `targets`",
        )),
    }
}

fn check_edges(
    member: &Member,
    by_name: &BTreeMap<&str, &Member>,
    manifest_text: &str,
    violations: &mut Vec<Violation>,
) {
    let mut reported = BTreeSet::new();
    for dependency in member
        .dependencies
        .iter()
        .filter(|dependency| dependency.is_workspace)
    {
        let Some(target) = by_name.get(dependency.name.as_str()) else {
            continue;
        };
        let table = if dependency.kind == EdgeKind::Development {
            EdgeTable::Development
        } else {
            EdgeTable::NormalOrBuild
        };
        if let Some(problem) = edge_violation(
            &member.name,
            member.category(),
            &target.name,
            target.category(),
            table,
        ) && reported.insert(problem.clone())
        {
            violations.push(Violation::new(
                &member.manifest,
                dependency_line(manifest_text, &dependency.name),
                problem,
            ));
        }
    }
}

fn check_tier(
    member: &Member,
    by_name: &BTreeMap<&str, &Member>,
    manifest_text: &str,
    violations: &mut Vec<Violation>,
) {
    let Some(declared) = member.layout.as_ref().and_then(|layout| layout.tier) else {
        return;
    };
    let mut highest: Option<(u64, &str)> = None;
    for dependency in &member.dependencies {
        if !dependency.is_workspace || dependency.kind == EdgeKind::Development {
            continue;
        }
        let Some(tier) = by_name
            .get(dependency.name.as_str())
            .and_then(|target| target.layout.as_ref())
            .and_then(|layout| layout.tier)
        else {
            continue;
        };
        if highest.is_none_or(|(best, _)| tier > best) {
            highest = Some((tier, dependency.name.as_str()));
        }
    }
    let (computed, reason) = match highest {
        None => (0, "it has no workspace dependency".to_owned()),
        Some((tier, name)) => (
            tier + 1,
            format!("its highest dependency `{name}` is tier {tier}"),
        ),
    };
    if declared != computed {
        violations.push(Violation::new(
            &member.manifest,
            line_of(manifest_text, "tier ="),
            format!("declared tier {declared}, computed tier {computed}: {reason}"),
        ));
    }
}
