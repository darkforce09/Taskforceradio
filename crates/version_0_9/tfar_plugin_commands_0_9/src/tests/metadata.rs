//! Unit tests for the metadata block, against `getMetaData` (1240-1262), `setMetaData`
//! (1264-1289), `getConnectionStatusInfo` (1293-1305), `updateUserStatusInfo` (1308-1316),
//! `isPluginEnabledForUser` (2526-2533) and `ts3plugin_infoData` (2363-2375) in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp`.

use super::*;

/// `getConnectionStatusInfo(true, true, true)` with addon `0.9.12`, concatenated by hand from
/// plugin.cpp:1300-1304: `"[I]Connected[/I] [B]" + "Y" + "[/B] [I]Play[/I] [B]" + "Y" +
/// "[/B] [I]P:[/I][B]" + "0.9.12" + "[/B]" + "[I] A: [/I][B]" + addon + "[/B]"`.
const PLAYING_STATUS: &str =
    "[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]";

#[test]
fn the_status_text_is_the_originals_byte_for_byte() {
    assert_eq!(
        format_metadata(true, true, "0.9.12", "0.9.12"),
        PLAYING_STATUS
    );
    assert_eq!(
        format_metadata(false, false, "0.9.12", ""),
        "[I]Connected[/I] [B]N[/B] [I]Play[/I] [B]N[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B][/B]"
    );
    assert_eq!(
        format_metadata(true, false, "0.9.12", "1.0"),
        "[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]N[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]1.0[/B]"
    );
}

#[test]
fn the_constants_are_the_originals() {
    assert_eq!(METADATA_START_TAG, "<TFAR>"); // plugin.cpp:1237
    assert_eq!(METADATA_END_TAG, "</TFAR>"); // plugin.cpp:1238
    assert_eq!(DISABLED_METADATA, "[B]Task Force Radio Plugin Disabled[/B]"); // plugin.cpp:1314
    // getConnectionStatusInfo(true, true, false): no closing `[/B]` after the play flag.
    assert_eq!(
        PLAYING_STATUS_PREFIX,
        "[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y"
    );
    assert_eq!(INFO_DATA_BUFFER_SIZE, 512); // plugin.cpp:2121
}

#[test]
fn merging_replaces_the_block_and_keeps_the_surrounding_text() {
    let existing = "other plugin text <TFAR>old status</TFAR> trailing text";
    assert_eq!(
        merge_metadata(existing, DISABLED_METADATA),
        "other plugin text <TFAR>[B]Task Force Radio Plugin Disabled[/B]</TFAR> trailing text"
    );
}

#[test]
fn merging_into_metadata_without_a_block_gives_the_block_alone() {
    // plugin.cpp:1276-1278: `to_set` starts empty, so the old text is not kept.
    assert_eq!(
        merge_metadata("other plugin text", PLAYING_STATUS),
        format!("<TFAR>{PLAYING_STATUS}</TFAR>")
    );
    assert_eq!(merge_metadata("", "x"), "<TFAR>x</TFAR>");
    assert_eq!(merge_metadata("<TFAR>unclosed", "x"), "<TFAR>x</TFAR>");
}

#[test]
fn merging_twice_gives_the_same_text() {
    let once = merge_metadata("a<TFAR>b</TFAR>c", PLAYING_STATUS);
    assert_eq!(merge_metadata(&once, PLAYING_STATUS), once);
}

#[test]
fn the_block_is_read_from_the_first_start_tag_to_the_next_end_tag() {
    assert_eq!(metadata_block("x<TFAR>status</TFAR>y"), Some("status"));
    assert_eq!(metadata_block("<TFAR></TFAR>"), Some(""));
    assert_eq!(metadata_block("<TFAR>a</TFAR><TFAR>b</TFAR>"), Some("a"));
    assert_eq!(metadata_block("no block"), None);
    assert_eq!(metadata_block("</TFAR><TFAR>"), None);
}

#[test]
fn a_playing_client_is_recognised_by_its_status_prefix() {
    let metadata = merge_metadata("", PLAYING_STATUS);
    assert!(metadata_says_playing(&metadata));
    assert!(metadata_says_playing(&format!(
        "before <TFAR>{PLAYING_STATUS}</TFAR> after"
    )));
}

#[test]
fn a_client_not_playing_or_without_tfar_is_not_recognised() {
    let not_playing = format_metadata(true, false, "0.9.12", "0.9.12");
    let not_connected = format_metadata(false, true, "0.9.12", "0.9.12");
    assert!(!metadata_says_playing(&merge_metadata("", &not_playing)));
    assert!(!metadata_says_playing(&merge_metadata("", &not_connected)));
    assert!(!metadata_says_playing(&merge_metadata(
        "",
        DISABLED_METADATA
    )));
    assert!(!metadata_says_playing(PLAYING_STATUS));
    assert!(!metadata_says_playing(""));
}

#[test]
fn the_prefix_alone_is_not_enough_because_start_with_is_strict() {
    // `startWith` needs the text to be longer than the prefix, plugin.cpp:496.
    let exact = format!("<TFAR>{PLAYING_STATUS_PREFIX}</TFAR>");
    assert!(!metadata_says_playing(&exact));
    let one_more = format!("<TFAR>{PLAYING_STATUS_PREFIX}!</TFAR>");
    assert!(metadata_says_playing(&one_more));
}

#[test]
fn info_data_shows_the_block_text() {
    let metadata = format!("x<TFAR>{PLAYING_STATUS}</TFAR>y");
    assert_eq!(info_data_text(&metadata), Some(PLAYING_STATUS));
    assert_eq!(info_data_text("no block"), None);
}

#[test]
fn info_data_keeps_at_most_511_bytes_at_a_character_boundary() {
    // `snprintf(*data, INFODATA_BUFSIZE, "%s", …)` keeps 511 bytes and the NUL, plugin.cpp:2369.
    let long = "a".repeat(600);
    assert_eq!(
        info_data_text(&format!("<TFAR>{long}</TFAR>")).map(str::len),
        Some(511)
    );
    // 510 ASCII bytes then a two-byte character: the cut falls before it.
    let text = format!("{}é tail", "a".repeat(510));
    assert_eq!(
        info_data_text(&format!("<TFAR>{text}</TFAR>")),
        Some("a".repeat(510).as_str())
    );
}
