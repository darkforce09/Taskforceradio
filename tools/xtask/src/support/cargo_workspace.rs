//! The workspace members as the crate gates see them: name, folder, layout metadata and
//! dependency edges, read through `cargo metadata`.
//!
//! **Role:** runs `cargo metadata --no-deps` once and turns each member into a [`Member`] with
//! repository-relative paths and its `[package.metadata.layout]` table.
//!
//! **Position:** used by `crate-tiers`, `dependency-firewalls` and `new-crate`; calls cargo.
//!
//! **Signals and state:** none; one child process per call.
//!
//! **Invariants:** paths are relative to the workspace root and use `/`; a dependency is a
//! workspace edge exactly when it is a path dependency.

use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind, MetadataCommand};

use crate::support::repository::Repository;

/// The kind of a dependency edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeKind {
    /// `[dependencies]`.
    Normal,
    /// `[dev-dependencies]`.
    Development,
    /// `[build-dependencies]`.
    Build,
}

/// One dependency of a member.
#[derive(Debug, Clone)]
pub(crate) struct MemberDependency {
    /// The package name of the dependency.
    pub(crate) name: String,
    /// Which table declares it.
    pub(crate) kind: EdgeKind,
    /// Whether it is a path dependency on another workspace crate.
    pub(crate) is_workspace: bool,
    /// Whether it is optional (behind a feature).
    pub(crate) is_optional: bool,
}

/// The `[package.metadata.layout]` table of a member; a missing key is `None`.
#[derive(Debug, Clone, Default)]
pub(crate) struct Layout {
    /// The parent path: `crates/<category>`, `apps` or `tools`.
    pub(crate) category: Option<String>,
    /// The declared tier.
    pub(crate) tier: Option<u64>,
    /// `any` or `windows`.
    pub(crate) targets: Option<String>,
}

/// One workspace member.
#[derive(Debug, Clone)]
pub(crate) struct Member {
    /// The package name.
    pub(crate) name: String,
    /// The repository-relative path of its manifest.
    pub(crate) manifest: String,
    /// The repository-relative path of its folder.
    pub(crate) folder: String,
    /// Its layout table, `None` when the table is missing.
    pub(crate) layout: Option<Layout>,
    /// Every dependency it declares.
    pub(crate) dependencies: Vec<MemberDependency>,
}

impl Member {
    /// The declared category, or the empty string.
    pub(crate) fn category(&self) -> &str {
        self.layout
            .as_ref()
            .and_then(|layout| layout.category.as_deref())
            .unwrap_or("")
    }
}

/// Every workspace member of the repository, sorted by folder.
#[allow(
    clippy::redundant_closure_for_method_calls,
    reason = "the method path needs the `serde_json` crate, which xtask does not name"
)]
pub(crate) fn load_members(repository: &Repository) -> Result<Vec<Member>> {
    let metadata = MetadataCommand::new()
        .manifest_path(repository.absolute("Cargo.toml"))
        .no_deps()
        .exec()
        .context("cargo metadata failed")?;
    let workspace_root = metadata.workspace_root.clone();
    let mut members = Vec::new();
    for package in metadata.workspace_packages() {
        let manifest = package
            .manifest_path
            .strip_prefix(&workspace_root)
            .map_or_else(|_| package.manifest_path.to_string(), ToString::to_string)
            .replace('\\', "/");
        let folder = manifest
            .strip_suffix("/Cargo.toml")
            .unwrap_or("")
            .to_owned();
        let layout_value = package
            .metadata
            .get("layout")
            .filter(|value| value.is_object());
        let layout = layout_value.map(|value| Layout {
            category: value
                .get("category")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
            tier: value.get("tier").and_then(|item| item.as_u64()),
            targets: value
                .get("targets")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned),
        });
        let dependencies = package
            .dependencies
            .iter()
            .map(|dependency| MemberDependency {
                name: dependency.name.clone(),
                kind: match dependency.kind {
                    DependencyKind::Development => EdgeKind::Development,
                    DependencyKind::Build => EdgeKind::Build,
                    _ => EdgeKind::Normal,
                },
                is_workspace: dependency.path.is_some(),
                is_optional: dependency.optional,
            })
            .collect();
        members.push(Member {
            name: package.name.to_string(),
            manifest,
            folder,
            layout,
            dependencies,
        });
    }
    members.sort_by(|left, right| left.folder.cmp(&right.folder));
    Ok(members)
}
