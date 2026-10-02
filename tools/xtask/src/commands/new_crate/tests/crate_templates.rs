//! Tests for the crate templates.

use super::*;

fn spec(category: &str) -> CrateSpec {
    CrateSpec {
        category: category.to_owned(),
        name: "tfar_sample_thing".to_owned(),
        tier: 1,
        summary: "A sample crate with a \"quoted\" word in its summary.".to_owned(),
        dependencies: vec!["tfar_geometry".to_owned()],
    }
}

#[test]
fn titles_drop_the_prefix_and_capitalise() {
    assert_eq!(title_of("tfar_sample_buffer"), "Sample buffer");
    assert_eq!(title_of("xtask"), "Xtask");
}

#[test]
fn wrapping_keeps_lines_within_the_width() {
    let lines = wrap("one two three four five six seven", 10);
    assert!(
        lines.iter().all(|line| line.chars().count() <= 10),
        "{lines:?}"
    );
    assert_eq!(lines.join(" "), "one two three four five six seven");
}

#[test]
fn a_library_skeleton_has_every_standard_file() {
    let names: Vec<String> = crate_files(&spec("crates/audio"))
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        names,
        [
            "Cargo.toml",
            "README.md",
            "src/README.md",
            "src/lib.rs",
            "src/prelude.rs",
            "src/error.rs",
            "src/crate_role.rs",
            "src/tests/crate_role.rs"
        ]
    );
}

#[test]
fn an_app_skeleton_is_a_cdylib_that_lifts_unsafe_code_with_a_reason() {
    let files = crate_files(&spec("apps"));
    let manifest = &files
        .iter()
        .find(|(name, _)| name == "Cargo.toml")
        .unwrap()
        .1;
    assert!(manifest.contains("crate-type = [\"cdylib\", \"rlib\"]"));
    assert!(manifest.contains("description = \"A sample crate with a \\\"quoted\\\" word"));
    let lib = &files
        .iter()
        .find(|(name, _)| name == "src/lib.rs")
        .unwrap()
        .1;
    let lines: Vec<&str> = lib.lines().collect();
    let lift = lines
        .iter()
        .position(|line| *line == "#![allow(unsafe_code)]")
        .unwrap();
    assert!(lines[lift - 1].starts_with("// "));
    let readme = &files
        .iter()
        .find(|(name, _)| name == "README.md")
        .unwrap()
        .1;
    assert!(readme.contains("## Exports") && !readme.contains("## Public surface"));
}
