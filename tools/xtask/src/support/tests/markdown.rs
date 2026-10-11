//! Tests for the Markdown reader.

use super::*;

const SAMPLE: &str = "# Title\n\n## Contents\n\n````markdown\n## Inside\n```text\nnested\n```\n````\n\n```text\nroot/\n```\n\n## How it works\n";

#[test]
fn headings_inside_fences_are_skipped() {
    let names: Vec<&str> = headings(SAMPLE)
        .iter()
        .map(|heading| heading.text)
        .collect();
    assert_eq!(names, ["Title", "Contents", "How it works"]);
}

#[test]
fn inline_code_spans_and_links_are_found() {
    let line = "See `a/b.rs` and ``c`d`` then [x](/y.md#z \"title\") but not `[q](/r)`.";
    assert_eq!(inline_code_spans(line), ["a/b.rs", "c`d", "[q](/r)"]);
    assert_eq!(link_targets(line), ["/y.md#z"]);
}

#[test]
fn anchors_follow_github() {
    assert_eq!(
        heading_anchor("5. Milestones and agents"),
        "5-milestones-and-agents"
    );
    assert_eq!(
        heading_anchor("`cargo xtask` — the runner"),
        "cargo-xtask--the-runner"
    );
    assert_eq!(heading_anchors("# A\n## A\n## A\n"), ["a", "a-1", "a-2"]);
}
