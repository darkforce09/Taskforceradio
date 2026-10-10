//! The `crate-anatomy` gate: the standard files of a crate and the shape of its manifest.
//!
//! **Role:** checks every manifest under crates and tools for workspace inheritance, lints,
//! `workspace = true` dependencies and allowed features, and every library for a short `lib.rs`
//! of only header, attributes, `mod`, `pub mod` and `pub use`, a `prelude.rs`, a `thiserror`
//! `error.rs`, and no `anyhow`.
//!
//! **Position:** a repository gate; parses manifests with `toml_edit` and reads sources through
//! `rust_source`.
//!
//! **Signals and state:** none.
//!
//! **Invariants:** could not run exactly when a manifest does not parse; a workspace crate is
//! re-exported only from `prelude.rs`, never from `lib.rs`.

#[cfg(test)]
#[path = "tests/crate_anatomy.rs"]
mod tests;

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item};

use crate::layout::{
    FEATURE_ANYWHERE, LIB_RS_MAX_LINES, MEMBER_ROOTS, PLUGIN_APP_FEATURES, PLUGIN_APPS,
};
use crate::support::repository::{self, Repository};
use crate::support::rust_source;

use super::gate::Violation;
use super::source_files::{dependency_line, is_test_file, line_of};

const INHERITED_PACKAGE_FIELDS: &[&str] = &["edition", "rust-version", "license", "publish"];
const DEPENDENCY_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// One crate the gate inspects.
struct CrateUnderCheck<'repository> {
    folder: String,
    manifest: String,
    manifest_text: String,
    document: DocumentMut,
    repository: &'repository Repository,
}

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let manifests: Vec<String> = repository
        .files()?
        .iter()
        .filter(|file| {
            MEMBER_ROOTS
                .iter()
                .any(|root| file.starts_with(&format!("{root}/")))
        })
        .filter(|file| repository::file_name(file) == "Cargo.toml" && !is_test_file(file))
        .cloned()
        .collect();
    let workspace_crates: Vec<&str> = manifests
        .iter()
        .map(|manifest| repository::file_name(repository::parent_folder(manifest)))
        .collect();
    let mut violations = Vec::new();
    for manifest in &manifests {
        let manifest_text = repository.read_text(manifest)?;
        let document: DocumentMut = manifest_text
            .parse()
            .with_context(|| format!("{manifest} does not parse"))?;
        let under_check = CrateUnderCheck {
            folder: repository::parent_folder(manifest).to_owned(),
            manifest: manifest.clone(),
            manifest_text,
            document,
            repository,
        };
        under_check.check_manifest(&mut violations);
        under_check.check_library(&workspace_crates, &mut violations)?;
    }
    Ok(violations)
}

fn is_workspace_inherited(item: Option<&Item>) -> bool {
    item.and_then(Item::as_table_like)
        .and_then(|table| table.get("workspace"))
        .and_then(Item::as_bool)
        == Some(true)
}

impl CrateUnderCheck<'_> {
    fn name(&self) -> &str {
        repository::file_name(&self.folder)
    }

    fn violation(&self, line: usize, message: impl Into<String>) -> Violation {
        Violation::new(&self.manifest, line, message)
    }

    fn check_manifest(&self, violations: &mut Vec<Violation>) {
        if !is_workspace_inherited(self.document.get("lints")) {
            violations.push(self.violation(
                line_of(&self.manifest_text, "[lints]"),
                "needs `[lints] workspace = true`",
            ));
        }
        let package = self.document.get("package").and_then(Item::as_table_like);
        for field in INHERITED_PACKAGE_FIELDS {
            if !is_workspace_inherited(package.and_then(|table| table.get(field))) {
                violations.push(self.violation(
                    line_of(&self.manifest_text, field),
                    format!("`{field}` must come from the workspace (`{field}.workspace = true`)"),
                ));
            }
        }
        let mut tables: Vec<(String, &Item)> = DEPENDENCY_TABLES
            .iter()
            .filter_map(|table| {
                self.document
                    .get(table)
                    .map(|item| ((*table).to_owned(), item))
            })
            .collect();
        if let Some(targets) = self.document.get("target").and_then(Item::as_table_like) {
            for (platform, platform_item) in targets.iter() {
                for table in DEPENDENCY_TABLES {
                    if let Some(item) = platform_item.get(table) {
                        tables.push((format!("target.{platform}.{table}"), item));
                    }
                }
            }
        }
        for (table_name, item) in tables {
            let Some(table) = item.as_table_like() else {
                continue;
            };
            for (dependency, value) in table.iter() {
                if !is_workspace_inherited(Some(value)) {
                    violations.push(self.violation(
                        dependency_line(&self.manifest_text, dependency),
                        format!("`{dependency}` in [{table_name}] must be `{dependency}.workspace = true`"),
                    ));
                }
            }
        }
        if let Some(features) = self.document.get("features").and_then(Item::as_table_like) {
            for (feature, _) in features.iter() {
                let allowed = feature == FEATURE_ANYWHERE
                    || (PLUGIN_APP_FEATURES.contains(&feature)
                        && PLUGIN_APPS.contains(&self.name()));
                if !allowed {
                    violations.push(self.violation(
                        line_of(&self.manifest_text, &format!("{feature} =")),
                        format!(
                            "feature `{feature}` is not allowed: only `{FEATURE_ANYWHERE}`, and `tracy` and `api_profiler` in the plugin apps"
                        ),
                    ));
                }
            }
        }
    }

    fn has_file(&self, relative_to_crate: &str) -> Result<bool> {
        self.repository
            .has_file(&format!("{}/{relative_to_crate}", self.folder))
    }

    fn check_library(
        &self,
        workspace_crates: &[&str],
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        let is_library = self.has_file("src/lib.rs")? || self.document.get("lib").is_some();
        if !is_library {
            return Ok(());
        }
        let has_binary = self.has_file("src/main.rs")? || self.document.get("bin").is_some();
        let lib_rs = format!("{}/src/lib.rs", self.folder);
        if self.has_file("src/lib.rs")? {
            let text = self.repository.read_text(&lib_rs)?;
            violations.extend(lib_rs_violations(&lib_rs, &text, workspace_crates));
        } else {
            violations.push(self.violation(1, "a library needs `src/lib.rs`"));
        }
        if !self.has_file("src/prelude.rs")? {
            violations.push(self.violation(1, "a library needs `src/prelude.rs`"));
        }
        let error_rs = format!("{}/src/error.rs", self.folder);
        if !self.has_file("src/error.rs")? {
            violations
                .push(self.violation(1, "a library needs `src/error.rs` with a `thiserror` error"));
        } else if !self.repository.read_text(&error_rs)?.contains("thiserror") {
            violations.push(Violation::new(
                &error_rs,
                1,
                "the error type must derive `thiserror::Error`",
            ));
        }
        let declares_anyhow = DEPENDENCY_TABLES
            .iter()
            .filter_map(|table| self.document.get(table).and_then(Item::as_table_like))
            .any(|table| table.contains_key("anyhow"));
        if declares_anyhow && !has_binary {
            violations.push(self.violation(
                dependency_line(&self.manifest_text, "anyhow"),
                "a library never uses `anyhow`; use a `thiserror` error",
            ));
        }
        for source in self
            .repository
            .files_under(&format!("{}/src", self.folder))?
        {
            let is_binary_source = source.ends_with("/src/main.rs") || source.contains("/src/bin/");
            if !repository::has_extension(source, "rs") || is_binary_source || is_test_file(source)
            {
                continue;
            }
            let text = self.repository.read_text(source)?;
            if let Some(index) = rust_source::code_lines(&text)
                .iter()
                .position(|line| line.contains("anyhow"))
            {
                violations.push(Violation::new(
                    source,
                    index + 1,
                    "library code never uses `anyhow`",
                ));
            }
        }
        Ok(())
    }
}

/// What is wrong with a `lib.rs`: its length and every statement that is not a header,
/// attribute, `mod`, `pub mod` or `pub use`.
pub(crate) fn lib_rs_violations(
    path: &str,
    text: &str,
    workspace_crates: &[&str],
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let line_count = text.lines().count();
    if line_count > LIB_RS_MAX_LINES {
        violations.push(Violation::new(
            path,
            LIB_RS_MAX_LINES + 1,
            format!("`lib.rs` has {line_count} lines; the limit is {LIB_RS_MAX_LINES}"),
        ));
    }
    let mut statement = String::new();
    let mut statement_line = 0;
    for (index, line) in rust_source::code_lines(text).iter().enumerate() {
        let trimmed = line.trim();
        if statement.is_empty() {
            if trimmed.is_empty() {
                continue;
            }
            statement_line = index + 1;
        }
        statement.push_str(trimmed);
        statement.push(' ');
        if is_complete_statement(&statement) {
            if let Some(problem) = statement_problem(statement.trim(), workspace_crates) {
                violations.push(Violation::new(path, statement_line, problem));
            }
            statement.clear();
        }
    }
    if !statement.trim().is_empty() {
        violations.push(Violation::new(
            path,
            statement_line,
            "an unterminated item at the end of `lib.rs`",
        ));
    }
    violations
}

fn is_complete_statement(statement: &str) -> bool {
    let trimmed = statement.trim();
    if trimmed.starts_with('#') {
        return trimmed.matches('[').count() == trimmed.matches(']').count();
    }
    let braces_balanced = trimmed.matches('{').count() == trimmed.matches('}').count();
    braces_balanced && (trimmed.ends_with(';') || trimmed.ends_with('}'))
}

fn statement_problem(statement: &str, workspace_crates: &[&str]) -> Option<String> {
    if statement.starts_with("#![") || statement.starts_with("#[") {
        return None;
    }
    let is_module = |rest: &str| {
        rest.strip_suffix(';').is_some_and(|name| {
            name.trim()
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
        })
    };
    if let Some(rest) = statement
        .strip_prefix("pub mod ")
        .or_else(|| statement.strip_prefix("mod "))
    {
        if is_module(rest) {
            return None;
        }
        return Some(
            "`lib.rs` declares modules only as `mod name;` or `pub mod name;`, never inline"
                .to_owned(),
        );
    }
    if let Some(path) = statement.strip_prefix("pub use ") {
        let first_segment = path
            .trim_start_matches("::")
            .split("::")
            .next()
            .unwrap_or("")
            .trim();
        if workspace_crates.contains(&first_segment) {
            return Some(format!(
                "`lib.rs` re-exports the workspace crate `{first_segment}`; re-export it from `prelude.rs` only"
            ));
        }
        return None;
    }
    let preview: String = statement.chars().take(60).collect();
    Some(format!(
        "`lib.rs` holds only the header, attributes, `mod`, `pub mod` and `pub use`; found `{preview}`"
    ))
}
