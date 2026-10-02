//! Tests for `dist`: the final names and the target-only writes.

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
