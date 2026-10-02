//! Tests for the reference manifest format.

use super::*;
use crate::fixture_tree::FixtureTree;

#[test]
fn a_rendered_list_reads_back_unchanged() {
    let fixture = FixtureTree::empty();
    fixture.write("reference/b/two.txt", "two");
    fixture.write("reference/a one.txt", "one");
    fixture.write("reference/README.md", "skipped");
    fixture.write("reference/tfar_1_0/.git/HEAD", "skipped");
    let hashes = hash_tree(&fixture.root().join("reference")).unwrap();
    let paths: Vec<&str> = hashes.keys().map(String::as_str).collect();
    assert_eq!(paths, ["a one.txt", "b/two.txt"]);
    assert_eq!(parse(&render(&hashes)).unwrap(), hashes);
}

#[test]
fn the_sha256_matches_the_known_value() {
    let fixture = FixtureTree::empty();
    fixture.write("abc.txt", "abc");
    assert_eq!(
        hash_file(&fixture.root().join("abc.txt")).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn a_malformed_line_is_an_error() {
    assert!(parse("nothex  path\n").is_err());
    assert!(parse("no separator\n").is_err());
}
