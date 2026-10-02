//! Tests for `dist`: the final names, the target folder, the HRIR copy and the target-only writes.

use super::*;
use crate::fixture_tree::FixtureTree;

#[test]
fn the_artifacts_carry_the_names_their_hosts_expect() {
    let names: Vec<(&str, &str)> = ARTIFACTS
        .iter()
        .map(|artifact| (artifact.version_folder, artifact.final_name))
        .collect();
    assert_eq!(
        names,
        [
            ("tfar_0_9", "task_force_radio_linux_amd64.so"),
            ("tfar_1_0", "TFAR_linux_amd64.so"),
            ("tfar_0_9", "task_force_radio_pipe_x64.dll"),
            ("tfar_1_0", "task_force_radio_pipe_x64.dll"),
        ]
    );
    assert!(
        ARTIFACTS
            .iter()
            .all(|artifact| artifact.built.contains(artifact.package))
    );
}

#[test]
fn a_folder_copy_reproduces_the_tree_under_target_only() {
    let fixture = FixtureTree::empty();
    fixture.write("reference/sounds/a.wav", "a");
    fixture.write("reference/sounds/sub/b.wav", "b");
    let target = fixture.root().join("target");
    let copied = copy_folder(
        &target,
        &fixture.root().join("reference/sounds"),
        &target.join("dist/v/radio-sounds"),
    )
    .unwrap();
    assert_eq!(copied, 2);
    assert_eq!(fixture.read("target/dist/v/radio-sounds/sub/b.wav"), "b");
    assert!(
        copy_folder(
            &target,
            &fixture.root().join("reference/sounds"),
            &fixture.root().join("elsewhere")
        )
        .is_err()
    );
}

#[test]
fn the_target_folder_follows_cargo_target_dir_and_falls_back_to_target() {
    let root = Path::new("/repository");
    assert_eq!(
        effective_target_folder(root, None),
        PathBuf::from("/repository/target")
    );
    assert_eq!(
        effective_target_folder(root, Some(OsString::new())),
        PathBuf::from("/repository/target")
    );
    assert_eq!(
        effective_target_folder(root, Some(OsString::from("/elsewhere/target-r1"))),
        PathBuf::from("/elsewhere/target-r1")
    );
    assert_eq!(
        effective_target_folder(root, Some(OsString::from("target-r1"))),
        PathBuf::from("/repository/target-r1")
    );
}

#[test]
fn every_version_folder_gets_the_hrir_sphere_and_its_credits_in_tfar_hrir() {
    let version_folders: std::collections::BTreeSet<&str> = ARTIFACTS
        .iter()
        .map(|artifact| artifact.version_folder)
        .chain(RADIO_SOUNDS.iter().map(|(_, folder)| *folder))
        .collect();
    assert_eq!(
        version_folders.into_iter().collect::<Vec<_>>(),
        VERSION_FOLDERS
    );
    assert_eq!(HRIR_FOLDER, "tfar_hrir");

    let fixture = FixtureTree::empty();
    fixture.write("assets/hrir/IRC_1002_C.bin", "sphere");
    fixture.write("assets/hrir/CREDITS.md", "credits");
    let target = fixture.root().join("target-r1");
    let written = copy_hrir(&target, fixture.root(), &target.join("dist")).unwrap();
    assert_eq!(written.len(), 4);
    for version_folder in VERSION_FOLDERS {
        let folder = format!("target-r1/dist/{version_folder}/tfar_hrir");
        assert_eq!(fixture.read(&format!("{folder}/IRC_1002_C.bin")), "sphere");
        assert_eq!(fixture.read(&format!("{folder}/CREDITS.md")), "credits");
    }
    assert!(copy_hrir(&target, fixture.root(), &fixture.root().join("dist")).is_err());
}

#[test]
fn the_hrir_sources_exist_in_the_repository() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for file in HRIR_FILES {
        assert!(root.join(file).is_file(), "{file} is missing");
    }
}
