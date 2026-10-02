//! Tests for the README span and the Contents grammar.

use super::*;

#[test]
fn the_span_skips_tests_target_and_dot_folders() {
    assert!(is_in_span("crates"));
    assert!(is_in_span("crates/audio/tfar_dsp_filters/src"));
    assert!(is_in_span("documentation/program"));
    assert!(!is_in_span("crates/audio/x/src/tests"));
    assert!(!is_in_span("tools/x/target/debug"));
    assert!(!is_in_span("tools/.cargo"));
    assert!(!is_in_span("reference/tfar_1_0"));
    assert!(!is_in_span(""));
}

#[test]
fn a_contents_block_parses_into_root_entries_and_problems() {
    let text = "# T\n\n## Contents\n\n```text\nx/\n├── a.rs   the a\n├── b/     the b\n│   └── c  nested\n└── d\n```\n\n## Boundaries\n";
    let block = contents_block(text).unwrap();
    assert_eq!(block.root_line, Some((6, "x/".to_owned())));
    let names: Vec<(&str, bool)> = block
        .entries
        .iter()
        .map(|entry| (entry.name.as_str(), entry.is_folder))
        .collect();
    assert_eq!(names, [("a.rs", false), ("b", true)]);
    assert_eq!(block.problems.len(), 2);
}

#[test]
fn a_text_block_outside_contents_is_not_the_contents_block() {
    assert!(contents_block("# T\n\n## How it works\n\n```text\nx/\n```\n").is_none());
}
