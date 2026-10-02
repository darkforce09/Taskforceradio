//! Tests for `reference-manifest`: it writes once and overwrites only with a reason.

use super::*;
use crate::fixture_tree::FixtureTree;

#[test]
fn the_manifest_is_written_once_and_overwritten_only_with_a_reason() {
    let fixture = FixtureTree::empty();
    fixture.write("reference/tfar_sample/a.txt", "a");
    let repository = fixture.repository();
    assert_eq!(run(&repository, None).unwrap(), 1);
    let first = fixture.read("reference/MANIFEST.sha256");
    assert!(first.ends_with("  tfar_sample/a.txt\n"), "{first}");
    assert!(run(&repository, None).is_err());
    assert!(run(&repository, Some("  ")).is_err());
    fixture.write("reference/tfar_sample/b.txt", "b");
    assert_eq!(
        run(&repository, Some("the 1.0 tree was updated")).unwrap(),
        2
    );
}
