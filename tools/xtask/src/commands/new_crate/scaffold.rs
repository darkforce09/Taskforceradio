//! `cargo xtask new-crate <category> <name>`: a new crate skeleton, its workspace entry and its
//! parent README line.
//!
//! **Role:** checks the name, the category and every requested dependency edge against the laws,
//! computes the tier, writes the skeleton files, inserts the `[workspace.dependencies]` entry and
//! adds the crate to its category README's Contents.
//!
//! **Position:** called by `main`; reads members through `cargo_workspace`, renders through
//! `crate_templates`, edits through `workspace_entry` and `parent_readme_entry`.
//!
//! **Signals and state:** none; writes files under the repository.
//!
//! **Invariants:** nothing is written unless every check passes; the tier is 0 with no
//! dependency, else 1 plus the highest dependency tier; an existing folder is never overwritten.

#[cfg(test)]
#[path = "tests/scaffold.rs"]
mod tests;

use anyhow::{Context, Result, bail};

use crate::layout::{CATEGORIES, EdgeTable, VERSION_0_9, VERSION_1_0, edge_violation};
use crate::support::cargo_workspace::load_members;
use crate::support::repository::Repository;

use super::crate_templates::{CrateSpec, crate_files};
use super::parent_readme_entry::with_contents_entry;
use super::workspace_entry::with_workspace_entry;

/// What the command line asked for.
#[derive(Debug, Clone)]
pub(crate) struct NewCrateRequest {
    /// `crates/<category>`, a bare `<category>` or `tools`.
    pub(crate) category: String,
    /// The package and folder name.
    pub(crate) name: String,
    /// One sentence on what the crate is for.
    pub(crate) summary: Option<String>,
    /// The workspace crates it depends on.
    pub(crate) depends_on: Vec<String>,
}

/// The category path a request names: `audio` becomes `crates/audio`.
pub(crate) fn normalised_category(category: &str) -> Result<String> {
    let trimmed = category.trim_end_matches('/');
    let candidate = if CATEGORIES.contains(&trimmed) {
        trimmed.to_owned()
    } else {
        format!("crates/{trimmed}")
    };
    if !CATEGORIES.contains(&candidate.as_str()) {
        bail!(
            "`{category}` is not a category: expected one of {}",
            CATEGORIES.join(", ")
        );
    }
    Ok(candidate)
}

fn check_name(name: &str, category: &str) -> Result<()> {
    let is_snake_case = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if !is_snake_case || !name.starts_with("tfar_") {
        bail!("`{name}` must be snake_case with the `tfar_` prefix");
    }
    let suffix = match category {
        VERSION_0_9 => Some("_0_9"),
        VERSION_1_0 => Some("_1_0"),
        _ => None,
    };
    if let Some(suffix) = suffix.filter(|suffix| !name.ends_with(suffix)) {
        bail!("a crate in {category} ends with `{suffix}`");
    }
    Ok(())
}

/// Plans the crate: the checked spec, without writing anything.
pub(crate) fn plan(repository: &Repository, request: &NewCrateRequest) -> Result<CrateSpec> {
    let category = normalised_category(&request.category)?;
    check_name(&request.name, &category)?;
    let members = load_members(repository)?;
    if members.iter().any(|member| member.name == request.name) {
        bail!("a workspace member is already named `{}`", request.name);
    }
    let mut highest: Option<u64> = None;
    for dependency in &request.depends_on {
        let member = members
            .iter()
            .find(|member| member.name == *dependency)
            .with_context(|| format!("`{dependency}` is not a workspace member"))?;
        if let Some(problem) = edge_violation(
            &request.name,
            &category,
            &member.name,
            member.category(),
            EdgeTable::NormalOrBuild,
        ) {
            bail!("{problem}");
        }
        let tier = member
            .layout
            .as_ref()
            .and_then(|layout| layout.tier)
            .with_context(|| format!("`{dependency}` declares no tier"))?;
        highest = Some(highest.map_or(tier, |best| best.max(tier)));
    }
    let summary = request
        .summary
        .clone()
        .unwrap_or_else(|| "A crate whose code is not written yet.".to_owned());
    let summary = if summary.ends_with('.') {
        summary
    } else {
        format!("{summary}.")
    };
    Ok(CrateSpec {
        category,
        name: request.name.clone(),
        tier: highest.map_or(0, |tier| tier + 1),
        summary,
        dependencies: request.depends_on.clone(),
    })
}

/// Writes the crate, the workspace entry and the parent README line; returns the spec written.
pub(crate) fn run(repository: &Repository, request: &NewCrateRequest) -> Result<CrateSpec> {
    let spec = plan(repository, request)?;
    let folder = spec.folder();
    if repository.absolute(&folder).exists() {
        bail!("{folder} already exists");
    }
    let parent_readme = format!("{}/README.md", spec.category);
    let parent_text = repository.read_text(&parent_readme)?;
    let role = spec.summary.trim_end_matches('.').to_owned();
    let role = {
        let mut characters = role.chars();
        characters
            .next()
            .map(|first| first.to_lowercase().chain(characters).collect::<String>())
            .unwrap_or_default()
    };
    let new_parent_text = with_contents_entry(
        &parent_text,
        &spec.name,
        &format!("{role}, crate `{}`", spec.name),
    )?;
    let manifest_text = repository.read_text("Cargo.toml")?;
    let new_manifest_text = with_workspace_entry(&manifest_text, &spec.category, &spec.name)?;
    for (relative, contents) in crate_files(&spec) {
        let path = repository.absolute(&format!("{folder}/{relative}"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
    }
    std::fs::write(repository.absolute("Cargo.toml"), new_manifest_text)
        .context("writing Cargo.toml")?;
    std::fs::write(repository.absolute(&parent_readme), new_parent_text)
        .with_context(|| format!("writing {parent_readme}"))?;
    Ok(spec)
}
