//! Unit tests for the metadata block, against `getConnectionStatusInfo` (`plugin.cpp:95-106`),
//! `Teamspeak::setMyMetaData` (`Teamspeak.cpp:644-668`), `Teamspeak::getMetaData`
//! (`Teamspeak.cpp:625-642`) and `isPluginEnabledForUser` (`plugin.cpp:421-441`), all under
//! `reference/tfar_1_0/ts/src/`.

use super::*;

#[test]
fn the_status_block_is_the_original_text() {
    assert_eq!(
        format_metadata(true, false, "1.0.0.1", "1.0.329"),
        "Connected to Game: [B]Yes[/B]\nPlaying: [B]No[/B]\nPlugin version: [B]1.0.0.1[/B]\nAddon \
         version: [B]1.0.329[/B]\n"
    );
    assert_eq!(
        format_metadata(false, false, "v", "unknown"),
        "Connected to Game: [B]No[/B]\nPlaying: [B]No[/B]\nPlugin version: [B]v[/B]\nAddon \
         version: [B]unknown[/B]\n"
    );
}

#[test]
fn the_merged_metadata_carries_the_tagged_block() {
    assert_eq!(
        merge_metadata("", &format_metadata(true, true, "1.0.0.1", "1.0.329")),
        "<TFAR>Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\nPlugin version: \
         [B]1.0.0.1[/B]\nAddon version: [B]1.0.329[/B]\n</TFAR>"
    );
}

#[test]
fn a_missing_block_is_appended_after_the_other_text() {
    assert_eq!(merge_metadata("other", "x"), "other<TFAR>x</TFAR>");
    // setMyMetaData("") without tags still appends empty tags (Teamspeak.cpp:652-653).
    assert_eq!(merge_metadata("other", ""), "other<TFAR></TFAR>");
}

#[test]
fn an_existing_block_is_replaced_and_the_text_around_it_kept() {
    assert_eq!(
        merge_metadata("before<TFAR>old</TFAR>after", "new"),
        "before<TFAR>new</TFAR>after"
    );
}

#[test]
fn an_empty_block_removes_the_tags() {
    // Teamspeak.cpp:658-659.
    assert_eq!(
        merge_metadata("before<TFAR>old</TFAR>after", ""),
        "beforeafter"
    );
}

#[test]
fn the_disabled_block_is_the_original_text() {
    assert_eq!(DISABLED_METADATA, "[B]Task Force Radio Plugin Disabled[/B]");
    assert_eq!(
        merge_metadata("<TFAR>x</TFAR>", DISABLED_METADATA),
        "<TFAR>[B]Task Force Radio Plugin Disabled[/B]</TFAR>"
    );
}

#[test]
fn the_block_is_the_text_between_the_first_tags() {
    assert_eq!(
        metadata_block("a<TFAR>inner</TFAR>b<TFAR>2</TFAR>"),
        "inner"
    );
    assert_eq!(metadata_block("no tags"), "");
    assert_eq!(metadata_block("<TFAR>open only"), "");
    assert_eq!(metadata_block("</TFAR>x<TFAR>rest"), "rest");
}

#[test]
fn a_playing_client_is_recognised_by_its_block_prefix() {
    let playing = merge_metadata("", &format_metadata(true, true, "1.0.0.1", "x"));
    assert!(metadata_says_playing(&playing));
    let connected_only = merge_metadata("", &format_metadata(true, false, "1.0.0.1", "x"));
    assert!(!metadata_says_playing(&connected_only));
    assert!(!metadata_says_playing(""));
    assert!(!metadata_says_playing(&merge_metadata(
        "",
        DISABLED_METADATA
    )));
    assert_eq!(
        PLAYING_METADATA_PREFIX,
        "Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\n"
    );
}

#[test]
fn the_info_text_is_the_block_cut_to_the_buffer() {
    // snprintf(*data, INFODATA_BUFSIZE, "%s", ...) with INFODATA_BUFSIZE 512, plugin.cpp:406.
    assert_eq!(info_data_text("x<TFAR>status</TFAR>y"), "status");
    let long = format!("<TFAR>{}</TFAR>", "a".repeat(600));
    assert_eq!(info_data_text(&long).len(), 511);
    let multibyte = format!("<TFAR>{}é</TFAR>", "a".repeat(510));
    assert_eq!(info_data_text(&multibyte), "a".repeat(510));
    assert_eq!(INFO_DATA_BUFFER_SIZE, 512);
}
