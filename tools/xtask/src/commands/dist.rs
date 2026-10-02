//! `cargo xtask dist`: release builds of the four deliverables, copied under their final names
//! into `target/dist/tfar_1_0/` and `target/dist/tfar_0_9/`.
//!
//! **Role:** builds the two plugins for Linux and the two extensions for `x86_64-pc-windows-gnu`
//! in release, then copies each artifact under the name its host expects, with each version's
//! `radio-sounds/` folder from `reference/`.
//!
//! **Position:** called by `main`; runs cargo as a child and reads `reference/` only.
//!
//! **Signals and state:** none; one cargo build at a time.
//!
//! **Invariants:** every write lands under `target/`; `reference/` is only read; TeamSpeak takes
//! a plugin's name from its file name, so the `.so` names are those of the original plugins
//! (`task_force_radio_linux_amd64.so` for 0.9, `TFAR_linux_amd64.so` for 1.0), and both DLLs are
//! `task_force_radio_pipe_x64.dll`.

#[cfg(test)]
#[path = "tests/dist.rs"]
mod tests;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::support::repository::Repository;

/// The Windows target the extensions are built for.
pub(crate) const WINDOWS_TARGET: &str = "x86_64-pc-windows-gnu";

/// One file `dist` copies: where cargo builds it and the name it gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DistArtifact {
    /// The package that builds it.
    pub(crate) package: &'static str,
    /// The file cargo writes, relative to the target folder.
    pub(crate) built: &'static str,
    /// The version folder under `target/dist/`.
    pub(crate) version_folder: &'static str,
    /// The final file name.
    pub(crate) final_name: &'static str,
}

/// The four deliverables.
pub(crate) const ARTIFACTS: &[DistArtifact] = &[
    DistArtifact {
        package: "tfar_plugin_0_9",
        built: "release/libtfar_plugin_0_9.so",
        version_folder: "tfar_0_9",
        final_name: "task_force_radio_linux_amd64.so",
    },
    DistArtifact {
        package: "tfar_plugin_1_0",
        built: "release/libtfar_plugin_1_0.so",
        version_folder: "tfar_1_0",
        final_name: "TFAR_linux_amd64.so",
    },
    DistArtifact {
        package: "tfar_extension_0_9",
        built: "x86_64-pc-windows-gnu/release/tfar_extension_0_9.dll",
        version_folder: "tfar_0_9",
        final_name: "task_force_radio_pipe_x64.dll",
    },
    DistArtifact {
        package: "tfar_extension_1_0",
        built: "x86_64-pc-windows-gnu/release/tfar_extension_1_0.dll",
        version_folder: "tfar_1_0",
        final_name: "task_force_radio_pipe_x64.dll",
    },
];

/// The `radio-sounds/` folders: the reference source and the version folder they go into.
pub(crate) const RADIO_SOUNDS: &[(&str, &str)] = &[
    ("reference/tfar_0_9_12/ts/radio-sounds", "tfar_0_9"),
    ("reference/tfar_1_0/ts/radio-sounds", "tfar_1_0"),
];

/// Builds and copies every deliverable.
pub(crate) fn run(repository: &Repository) -> Result<()> {
    let target = repository.absolute("target");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let shell = xshell::Shell::new()?;
    shell.change_dir(repository.root());
    let target_flag = target.display().to_string();
    xshell::cmd!(
        shell,
        "{cargo} build --release --target-dir {target_flag} -p tfar_plugin_0_9 -p tfar_plugin_1_0"
    )
    .run()
    .context("building the plugins for Linux")?;
    xshell::cmd!(
        shell,
        "{cargo} build --release --target-dir {target_flag} --target {WINDOWS_TARGET} -p tfar_extension_0_9 -p tfar_extension_1_0"
    )
    .run()
    .context("building the extensions for windows-gnu")?;
    let dist = target.join("dist");
    for (_, version_folder) in RADIO_SOUNDS {
        let folder = dist.join(version_folder);
        if folder.exists() {
            std::fs::remove_dir_all(&folder)
                .with_context(|| format!("clearing {}", folder.display()))?;
        }
    }
    for artifact in ARTIFACTS {
        let source = target.join(artifact.built);
        let destination = dist.join(artifact.version_folder).join(artifact.final_name);
        copy_file(&target, &source, &destination)?;
        println!("{} -> {}", source.display(), destination.display());
    }
    for (source, version_folder) in RADIO_SOUNDS {
        let destination = dist.join(version_folder).join("radio-sounds");
        let copied = copy_folder(&target, &repository.absolute(source), &destination)?;
        println!("{source}/ -> {} ({copied} files)", destination.display());
    }
    Ok(())
}

/// Refuses any destination outside the target folder.
fn ensure_under(target: &Path, destination: &Path) -> Result<()> {
    if !destination.starts_with(target) {
        bail!(
            "{} lies outside {}; dist writes only under target/",
            destination.display(),
            target.display()
        );
    }
    Ok(())
}

fn copy_file(target: &Path, source: &Path, destination: &Path) -> Result<()> {
    ensure_under(target, destination)?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::copy(source, destination)
        .with_context(|| format!("copying {} to {}", source.display(), destination.display()))?;
    Ok(())
}

/// Copies every file under `source` to the same relative path under `destination`; returns the
/// number of files copied.
pub(crate) fn copy_folder(target: &Path, source: &Path, destination: &Path) -> Result<usize> {
    ensure_under(target, destination)?;
    let mut copied = 0;
    for entry in walkdir::WalkDir::new(source).sort_by_file_name() {
        let entry = entry.with_context(|| format!("walking {}", source.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative: PathBuf = entry
            .path()
            .strip_prefix(source)
            .context("a walked path left its folder")?
            .to_path_buf();
        copy_file(target, entry.path(), &destination.join(relative))?;
        copied += 1;
    }
    if copied == 0 {
        bail!("{} holds no file to copy", source.display());
    }
    Ok(copied)
}
