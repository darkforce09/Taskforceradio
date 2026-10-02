//! Fixture trees for the gate tests: a temporary git folder, and a small sample workspace that
//! every gate holds on.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::commands::new_crate::crate_templates::{CrateSpec, crate_files};
use crate::commands::verify::gate::{Gate, Violation};
use crate::support::reference_hashes;
use crate::support::repository::Repository;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

/// A temporary folder with `git init` run in it; removed on drop.
pub(crate) struct FixtureTree {
    root: PathBuf,
}

impl FixtureTree {
    /// An empty fixture with a git repository.
    pub(crate) fn empty() -> Self {
        let unique = format!(
            "xtask-fixture-{}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::SeqCst),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).unwrap();
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(status.success(), "git init failed in {}", root.display());
        Self { root }
    }

    /// The sample workspace: three crates, the area and documentation READMEs, the README
    /// standard's kind table, and a reference tree with its manifest.
    pub(crate) fn sample_workspace() -> Self {
        let fixture = Self::empty();
        fixture.write(".gitignore", "/target/\n");
        fixture.write("Cargo.toml", SAMPLE_ROOT_MANIFEST);
        fixture.write(
            "README.md",
            "# Sample workspace\n\nA fixture workspace for the gate tests.\n",
        );
        fixture.write_crate("crates/foundation", "tfar_geometry", 0, &[]);
        fixture.write_crate(
            "crates/version_0_9",
            "tfar_game_commands_0_9",
            1,
            &["tfar_geometry"],
        );
        fixture.write_crate(
            "crates/version_1_0",
            "tfar_game_commands_1_0",
            1,
            &["tfar_geometry"],
        );
        fixture.write(
            "crates/README.md",
            &area_readme(
                "crates",
                "Crates",
                &["foundation/", "version_0_9/", "version_1_0/"],
            ),
        );
        fixture.write(
            "crates/foundation/README.md",
            &area_readme("crates/foundation", "Foundation", &["tfar_geometry/"]),
        );
        fixture.write(
            "crates/version_0_9/README.md",
            &area_readme(
                "crates/version_0_9",
                "Version 0.9",
                &["tfar_game_commands_0_9/"],
            ),
        );
        fixture.write(
            "crates/version_1_0/README.md",
            &area_readme(
                "crates/version_1_0",
                "Version 1.0",
                &["tfar_game_commands_1_0/"],
            ),
        );
        fixture.write("documentation/README.md", SAMPLE_DOCUMENTATION_README);
        fixture.write("documentation/standards/README.md", SAMPLE_STANDARDS_README);
        fixture.write(
            "documentation/standards/readme_standard.md",
            SAMPLE_README_STANDARD,
        );
        fixture.write(
            "reference/README.md",
            "# Reference\n\nThe original mods, byte for byte.\n",
        );
        fixture.write("reference/tfar_sample/ts/radio-sounds/clip.wav", "RIFF");
        fixture.write(
            "reference/tfar_sample/ts/src/plugin.cpp",
            "int main() { return 0; }\n",
        );
        fixture.write_reference_manifest();
        fixture
    }

    /// Writes one crate skeleton with the `new-crate` templates.
    pub(crate) fn write_crate(&self, category: &str, name: &str, tier: u64, dependencies: &[&str]) {
        let spec = CrateSpec {
            category: category.to_owned(),
            name: name.to_owned(),
            tier,
            summary: format!("The sample crate {name} for the gate tests."),
            dependencies: dependencies
                .iter()
                .map(|dependency| (*dependency).to_owned())
                .collect(),
        };
        for (relative, contents) in crate_files(&spec) {
            self.write(&format!("{category}/{name}/{relative}"), &contents);
        }
    }

    /// Rewrites `reference/MANIFEST.sha256` from the fixture's reference tree.
    pub(crate) fn write_reference_manifest(&self) {
        let hashes = reference_hashes::hash_tree(&self.root.join("reference")).unwrap();
        self.write(
            "reference/MANIFEST.sha256",
            &reference_hashes::render(&hashes),
        );
    }

    /// The fixture's root folder.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// A fresh repository view of the fixture (nothing cached).
    pub(crate) fn repository(&self) -> Repository {
        Repository::at(&self.root)
    }

    /// Writes a file, creating its folders.
    pub(crate) fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Reads a file.
    pub(crate) fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.root.join(relative)).unwrap()
    }

    /// Replaces the one occurrence of `from` in a file with `to`.
    pub(crate) fn replace(&self, relative: &str, from: &str, to: &str) {
        let text = self.read(relative);
        assert!(text.contains(from), "{relative} does not contain {from:?}");
        self.write(relative, &text.replacen(from, to, 1));
    }

    /// Deletes a file.
    pub(crate) fn remove(&self, relative: &str) {
        std::fs::remove_file(self.root.join(relative)).unwrap();
    }

    /// Runs a gate on the fixture; panics when it could not run.
    pub(crate) fn violations(&self, gate: Gate) -> Vec<Violation> {
        gate.check(&self.repository())
            .unwrap_or_else(|error| panic!("{} could not run: {error:#}", gate.name()))
    }
}

impl Drop for FixtureTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// The messages of a gate's violations, one `path:line: message` string each, for assertions.
pub(crate) fn described(violations: &[Violation]) -> Vec<String> {
    violations
        .iter()
        .map(|violation| {
            format!(
                "{}:{}: {}",
                violation.path, violation.line, violation.message
            )
        })
        .collect()
}

/// An area-root README listing the given child folders.
pub(crate) fn area_readme(folder: &str, title: &str, children: &[&str]) -> String {
    let width = children.iter().map(|child| child.len()).max().unwrap_or(0) + 2;
    let mut entries = String::new();
    for (index, child) in children.iter().enumerate() {
        let branch = if index + 1 == children.len() {
            "└── "
        } else {
            "├── "
        };
        let padding = " ".repeat(width - child.len());
        let _ = writeln!(entries, "{branch}{child}{padding}a sample child folder");
    }
    format!(
        "# {title}\n\nA sample area for the gate tests.\n\n## Contents\n\n```text\n{folder}/\n{entries}```\n\n\
         ## How it works\n\nEach child holds sample code.\n\n## Getting started\n\nNothing to run.\n\n\
         ## Boundaries\n\n- Depends on: nothing.\n- Used by: nothing.\n- Rules: none beyond the laws.\n"
    )
}

const SAMPLE_ROOT_MANIFEST: &str = r#"# The sample workspace of the xtask gate tests.
[workspace]
resolver = "3"
members = ["crates/*/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.99"
license = "UNLICENSED"
publish = false

[workspace.dependencies]
# crates/foundation:
tfar_geometry = { path = "crates/foundation/tfar_geometry" }

# crates/version_0_9:
tfar_game_commands_0_9 = { path = "crates/version_0_9/tfar_game_commands_0_9" }

# crates/version_1_0:
tfar_game_commands_1_0 = { path = "crates/version_1_0/tfar_game_commands_1_0" }

# External crates.
thiserror = { version = "2.0.21", default-features = false }
rusqlite = { version = "0.40.2", default-features = false }
anyhow = { version = "1.0.104", default-features = false }

[workspace.lints.rust]
unsafe_code = "deny"
"#;

const SAMPLE_DOCUMENTATION_README: &str = "**Status:** live

# Documentation

The sample documentation tree.

## Contents

```text
documentation/
└── standards/  the sample standards
```

## How it works

One folder of standards.

## Code

- [Crates](/crates/README.md) — the sample crates.

## Boundaries

- Depends on: nothing.
- Used by: nothing.
- Rules: every document opens with a status line.
";

const SAMPLE_STANDARDS_README: &str = "**Status:** live

# Standards

The sample standards.

## Contents

```text
documentation/standards/
└── readme_standard.md  the sample README standard and its kind table
```

## Code

- [Crates](/crates/README.md#contents) — the sample crates.

## Boundaries

- Depends on: nothing.
- Used by: the readme-sections gate.
- Rules: the kind table stays parseable.
";

const SAMPLE_README_STANDARD: &str = "**Status:** live

# README standard

The sample standard.

## Kinds

| Kind | Kind sections, in order | Template | Examples |
|---|---|---|---|
| crate root | Getting started, Configuration, Public surface | crate root | a library |
| app | Getting started, Configuration, Exports | app | an app |
| area root | Getting started | area root | crates |
| domain or subsystem | Public surface | domain | a src folder |
| leaf | none | leaf | a module folder |
| command-line | Commands | command-line | xtask commands |
| data | Format, Producers and consumers | data | assets |
| documentation folder | Code | documentation folder | documentation |

The legal `##` headings, in their one order, are: Contents, How it works, Getting started, Configuration,
Public surface, Exports, Commands, Format, Producers and consumers, Code, Boundaries, Related
documentation. The readme-sections gate holds that order.
";
