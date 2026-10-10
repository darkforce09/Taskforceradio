//! Unit tests for the status metadata.

use super::*;

#[test]
fn the_status_replaces_the_block_and_keeps_the_rest() {
    assert_eq!(
        metadata_with_status("before<TFAR>old</TFAR>after", "new").as_deref(),
        Some("before<TFAR>new</TFAR>after")
    );
}

#[test]
fn an_unchanged_status_writes_nothing() {
    assert_eq!(metadata_with_status("<TFAR>same</TFAR>", "same"), None);
}

#[test]
fn the_plugin_version_is_the_originals() {
    assert_eq!(PLUGIN_VERSION, "0.9.12");
}
