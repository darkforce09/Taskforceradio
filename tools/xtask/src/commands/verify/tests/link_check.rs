//! Fixture tests for the `link-check` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const DOC: &str = "documentation/notes.md";

fn found_with(body: &str) -> Vec<String> {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(DOC, &format!("**Status:** live\n\n# Notes\n\n{body}\n"));
    described(&check(&fixture.repository()).unwrap())
}

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn good_links_and_paths_hold() {
    let found = found_with(
        "See [crates](/crates/README.md#contents), [here](#notes), [web](https://example.com),\n\
         `crates/foundation/tfar_geometry/src/lib.rs:3`, `/documentation/README.md`, `crates/foundation/`,\n\
         `reference/tfar_sample/ts/src/plugin.cpp:1`, `roadmap.md`, `src/lib.rs`, `cargo xtask verify all`,\n\
         `crates/<category>/`, `apps/*`, `target/dist/tfar_1_0/TFAR_linux_amd64.so`.",
    );
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn a_broken_repository_root_link_fails() {
    let found = found_with("See [gone](/documentation/gone.md).");
    assert_eq!(
        found,
        vec![format!(
            "{DOC}:5: the link target `/documentation/gone.md` does not exist"
        )]
    );
}

#[test]
fn a_parent_relative_link_fails() {
    let found = found_with("See [up](../crates/README.md).");
    assert!(
        found.iter().any(|line| line.contains("climbs with `../`")),
        "{found:?}"
    );
}

#[test]
fn a_missing_anchor_fails() {
    let found = found_with("See [crates](/crates/README.md#no-such-heading) and [self](#nowhere).");
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found.iter().all(|line| line.contains("is not a heading")),
        "{found:?}"
    );
}

#[test]
fn a_backticked_repository_path_that_does_not_exist_fails() {
    let found = found_with("The planned `crates/audio/tfar_hrtf/src/lib.rs` is not there.");
    assert!(
        found
            .iter()
            .any(|line| line.contains("`crates/audio/tfar_hrtf/src/lib.rs` does not exist")),
        "{found:?}"
    );
}

#[test]
fn text_inside_code_fences_is_not_checked() {
    let found = found_with("```text\n[gone](/documentation/gone.md) `crates/gone/`\n```");
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn backticked_strings_are_paths_only_under_a_top_level_folder() {
    assert_eq!(
        repository_path_in("crates/audio/").as_deref(),
        Some("crates/audio")
    );
    assert_eq!(
        repository_path_in("/documentation/x.md#y").as_deref(),
        Some("documentation/x.md")
    );
    assert_eq!(
        repository_path_in("reference/a.cpp:14-20").as_deref(),
        Some("reference/a.cpp")
    );
    assert_eq!(repository_path_in("roadmap.md"), None);
    assert_eq!(repository_path_in("src/lib.rs"), None);
    assert_eq!(repository_path_in(".git/info/attributes"), None);
    assert_eq!(repository_path_in("crates/<category>"), None);
    assert_eq!(repository_path_in("cargo test -p x"), None);
}
