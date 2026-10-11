//! Fixture trees for the gate tests: a temporary git folder under the repository's `target/tmp/`,
//! and a small sample workspace that every gate holds on.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::commands::verify::gate::Violation;
use crate::support::reference_hashes;
use crate::support::repository::Repository;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

/// The folder fixtures are made in: `target/tmp/` of this checkout, never the system temp folder.
fn fixture_parent() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp")
}

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
        let root = fixture_parent().join(unique);
        std::fs::create_dir_all(&root).unwrap();
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(status.success(), "git init failed in {}", root.display());
        Self { root }
    }

    /// The sample workspace: three crates, a crates README, a documentation README, and a
    /// reference tree with its manifest.
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
            "# Crates\n\n## Contents\n\nThe sample crates.\n",
        );
        fixture.write(
            "documentation/README.md",
            "# Documentation\n\nThe sample documentation.\n",
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

    /// Writes one crate: its manifest with layout metadata, a crate root and one module with a
    /// sibling test file.
    pub(crate) fn write_crate(&self, category: &str, name: &str, tier: u64, dependencies: &[&str]) {
        let mut manifest = format!(
            "[package]\nname = \"{name}\"\nversion.workspace = true\nedition.workspace = true\n\
             rust-version.workspace = true\nlicense.workspace = true\npublish.workspace = true\n\n\
             [package.metadata.layout]\ncategory = \"{category}\"\ntier = {tier}\ntargets = \"any\"\n\n\
             [dependencies]\nthiserror.workspace = true\n"
        );
        for dependency in dependencies {
            let _ = writeln!(manifest, "{dependency}.workspace = true");
        }
        manifest.push_str("\n[lints]\nworkspace = true\n");
        let folder = format!("{category}/{name}");
        self.write(&format!("{folder}/Cargo.toml"), &manifest);
        self.write(
            &format!("{folder}/src/lib.rs"),
            &format!("//! The sample crate `{name}`.\n\nmod sample;\n"),
        );
        self.write(
            &format!("{folder}/src/sample.rs"),
            "//! A sample module.\n\n#[cfg(test)]\n#[path = \"tests/sample.rs\"]\nmod tests;\n\n\
             /// A sample value.\npub const SAMPLE: u8 = 1;\n",
        );
        self.write(
            &format!("{folder}/src/tests/sample.rs"),
            "use super::*;\n\n#[test]\nfn the_sample_is_one() {\n    assert_eq!(SAMPLE, 1);\n}\n",
        );
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
