//! Unit tests for the plugin-enabled cache.

use std::time::Duration;

use super::*;
use crate::client_state::test_support::{ALPHA, now};

const PLAYING: &str =
    "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin version: [B]1[/B]\n</TFAR>";
const NOT_PLAYING: &str = "<TFAR>Connected to Game: [B]No[/B]\nPlaying: [B]No[/B]\n</TFAR>";

/// The original refreshes the cache time on every call, so a positive answer never
/// expires; here it expires ten seconds after the metadata was read.
#[test]
fn a_positive_plugin_check_expires_ten_seconds_after_the_metadata_was_read() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    assert!(client.plugin_enabled_for(Some(PLAYING), start));
    assert!(client.plugin_enabled_for(None, start + Duration::from_secs(5)));
    assert!(client.plugin_enabled_for(None, start + Duration::from_millis(9_999)));
    assert_eq!(
        client.cached_plugin_enabled(start + PLUGIN_ENABLED_CACHE),
        None
    );
    assert!(!client.plugin_enabled_for(Some(NOT_PLAYING), start + PLUGIN_ENABLED_CACHE));
}

#[test]
fn a_negative_answer_is_cached_and_expires_too() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    assert!(!client.plugin_enabled_for(Some(NOT_PLAYING), start));
    assert!(!client.plugin_enabled_for(Some(PLAYING), start + Duration::from_secs(9)));
    assert!(client.plugin_enabled_for(Some(PLAYING), start + Duration::from_secs(10)));
}

#[test]
fn missing_or_empty_metadata_is_no_and_is_not_cached() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    assert!(!client.plugin_enabled_for(None, start));
    assert!(!client.plugin_enabled_for(Some(""), start));
    assert_eq!(client.cached_plugin_enabled(start), None);
    assert!(client.plugin_enabled_for(Some(PLAYING), start));
}

#[test]
fn a_plugin_command_marks_the_client_enabled_from_now() {
    let start = now();
    let mut client = ClientData::new(ALPHA, "alpha");
    client.mark_plugin_enabled(start);
    assert_eq!(client.cached_plugin_enabled(start), Some(true));
    assert_eq!(
        client.cached_plugin_enabled(start + Duration::from_secs(10)),
        None
    );
}
