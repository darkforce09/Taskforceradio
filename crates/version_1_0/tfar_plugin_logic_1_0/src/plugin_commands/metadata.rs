//! The TFAR block of a client's TeamSpeak metadata: its text, how it merges into the metadata,
//! and how other clients read it.
//!
//! **Role:** format the status block the plugin publishes every 4 s, put it between the
//! `<TFAR>` tags of the client's metadata without touching the text around them, and read
//! another client's block for the info panel and the plugin-enabled check.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::plugin_commands`; called by `tfar_plugin_1_0` around
//! `getClientVariableAsString(CLIENT_META_DATA)` and `setClientSelfVariableAsString`.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** the block text is `getConnectionStatusInfo`'s
//! (`reference/tfar_1_0/ts/src/plugin.cpp:95-106`) byte for byte, `\n` after each line; the merge
//! is `Teamspeak::setMyMetaData` (`Teamspeak.cpp:644-668`) and the read `Teamspeak::getMetaData`
//! (`Teamspeak.cpp:625-642`), each using the first `<TFAR>` and the first `</TFAR>`.

#[cfg(test)]
#[path = "tests/metadata.rs"]
mod tests;

/// The tag that opens the TFAR block (`reference/tfar_1_0/ts/src/common.hpp:42`).
pub const METADATA_START_TAG: &str = "<TFAR>";

/// The tag that closes the TFAR block (`common.hpp:43`).
pub const METADATA_END_TAG: &str = "</TFAR>";

/// The block the plugin publishes while it is disabled (`plugin.cpp:112`).
pub const DISABLED_METADATA: &str = "[B]Task Force Radio Plugin Disabled[/B]";

/// The start of the block of a client that is connected to the game and playing: what
/// `isPluginEnabledForUser` looks for (`plugin.cpp:437-438`).
pub const PLAYING_METADATA_PREFIX: &str = "Connected to Game: [B]Yes[/B]\nPlaying: [B]Yes[/B]\n";

/// The size of the buffer `ts3plugin_infoData` fills, the NUL included
/// (`INFODATA_BUFSIZE`, `common.hpp:52`).
pub const INFO_DATA_BUFFER_SIZE: usize = 512;

/// The status block: whether the game is connected, whether the player is in a mission, and the
/// plugin and addon versions, each line ending in `\n`.
pub fn format_metadata(
    connected: bool,
    playing: bool,
    plugin_version: &str,
    addon_version: &str,
) -> String {
    format!(
        "Connected to Game: {}\nPlaying: {}\nPlugin version: [B]{plugin_version}[/B]\nAddon \
         version: [B]{addon_version}[/B]\n",
        yes_no(connected),
        yes_no(playing),
    )
}

/// `[B]Yes[/B]` or `[B]No[/B]`.
const fn yes_no(value: bool) -> &'static str {
    if value { "[B]Yes[/B]" } else { "[B]No[/B]" }
}

/// `existing` metadata with `block` as its TFAR block:
///
/// - without both tags, `existing` followed by `<TFAR>block</TFAR>` (an empty block too);
/// - with both, the text before the first `<TFAR>` and after the first `</TFAR>` kept and the
///   tags and block between them replaced, or removed when `block` is empty.
pub fn merge_metadata(existing: &str, block: &str) -> String {
    let (Some(start), Some(end)) = (
        existing.find(METADATA_START_TAG),
        existing.find(METADATA_END_TAG),
    ) else {
        return format!("{existing}{METADATA_START_TAG}{block}{METADATA_END_TAG}");
    };
    let before = existing.get(..start).unwrap_or_default();
    let after = existing
        .get(end.saturating_add(METADATA_END_TAG.len())..)
        .unwrap_or_default();
    if block.is_empty() {
        format!("{before}{after}")
    } else {
        format!("{before}{METADATA_START_TAG}{block}{METADATA_END_TAG}{after}")
    }
}

/// The TFAR block of a client's `metadata`: the text between the first `<TFAR>` and the first
/// `</TFAR>`, or the empty text when either tag is missing. When the first `</TFAR>` comes
/// before the first `<TFAR>`, the original's length wraps and the block runs to the end.
pub fn metadata_block(metadata: &str) -> &str {
    let (Some(start), Some(end)) = (
        metadata.find(METADATA_START_TAG),
        metadata.find(METADATA_END_TAG),
    ) else {
        return "";
    };
    let block_start = start.saturating_add(METADATA_START_TAG.len());
    if end >= block_start {
        metadata.get(block_start..end).unwrap_or_default()
    } else {
        metadata.get(block_start..).unwrap_or_default()
    }
}

/// Whether a client's `metadata` says it is connected to the game and playing: its block is not
/// empty and starts with [`PLAYING_METADATA_PREFIX`].
pub fn metadata_says_playing(metadata: &str) -> bool {
    let block = metadata_block(metadata);
    !block.is_empty() && block.starts_with(PLAYING_METADATA_PREFIX)
}

/// The text of the info panel for a client whose metadata is `metadata`: its block, cut to fit
/// [`INFO_DATA_BUFFER_SIZE`] with the NUL (at most 511 bytes), at a character boundary.
pub fn info_data_text(metadata: &str) -> &str {
    let block = metadata_block(metadata);
    let mut length = block.len().min(INFO_DATA_BUFFER_SIZE.saturating_sub(1));
    while !block.is_char_boundary(length) {
        length = length.saturating_sub(1);
    }
    block.get(..length).unwrap_or_default()
}
