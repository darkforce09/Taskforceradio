//! The `<TFAR>…</TFAR>` block a 0.9.12 plugin keeps in its TeamSpeak client metadata.
//!
//! **Role:** format the status text other players read to tell whether this plugin runs and is
//! in a game, merge it into the client's metadata, and read it back from another client's.
//!
//! **Position:** a module of `tfar_plugin_commands_0_9`; the plugin app writes the merged text
//! with `setClientSelfVariableAsString(CLIENT_META_DATA)` and reads others' with
//! `getClientVariableAsString(CLIENT_META_DATA)`, and serves `info_data_text` from
//! `ts3plugin_infoData`.
//!
//! **Signals and state:** none; pure functions over strings.
//!
//! **Invariants:** the status text is `getConnectionStatusInfo(…, true)` of
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1293-1305` byte for byte, so a Windows player's
//! `isPluginEnabledForUser` (plugin.cpp:2526) accepts it; the block is the text from the first
//! `<TFAR>` to the first `</TFAR>` after it; merging keeps the text around an existing block.

#[cfg(test)]
#[path = "tests/metadata.rs"]
mod tests;

/// The tag that opens the block (`START_DATA`, plugin.cpp:1237).
pub const METADATA_START_TAG: &str = "<TFAR>";

/// The tag that closes the block (`END_DATA`, plugin.cpp:1238).
pub const METADATA_END_TAG: &str = "</TFAR>";

/// The status text a plugin sets at shutdown (plugin.cpp:1314).
pub const DISABLED_METADATA: &str = "[B]Task Force Radio Plugin Disabled[/B]";

/// The start a block must have, and outgrow, for its client to count as running TFAR and in a
/// game: `getConnectionStatusInfo(true, true, false)`, which stops before the closing `[/B]`
/// (plugin.cpp:2531-2533).
pub const PLAYING_STATUS_PREFIX: &str = "[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y";

/// The size of the buffer `ts3plugin_infoData` fills (`INFODATA_BUFSIZE`, plugin.cpp:2121): the
/// text plus its terminating NUL.
pub const INFO_DATA_BUFFER_SIZE: usize = 512;

/// The status text: whether the game link is connected, whether a game runs, the plugin version
/// and the addon version the game reported, as
/// `[I]Connected[/I] [B]Y[/B] [I]Play[/I] [B]Y[/B] [I]P:[/I][B]0.9.12[/B][I] A: [/I][B]0.9.12[/B]`.
pub fn format_metadata(
    connected: bool,
    playing: bool,
    plugin_version: &str,
    addon_version: &str,
) -> String {
    let connected = yes_or_no(connected);
    let playing = yes_or_no(playing);
    format!(
        "[I]Connected[/I] [B]{connected}[/B] [I]Play[/I] [B]{playing}[/B] [I]P:[/I][B]{plugin_version}[/B][I] A: [/I][B]{addon_version}[/B]"
    )
}

/// `Y` or `N`.
const fn yes_or_no(flag: bool) -> &'static str {
    if flag { "Y" } else { "N" }
}

/// The byte range of the block's text in `metadata`: after the first [`METADATA_START_TAG`] up
/// to the first [`METADATA_END_TAG`] after it.
fn block_range(metadata: &str) -> Option<(usize, usize)> {
    let start = metadata.find(METADATA_START_TAG)? + METADATA_START_TAG.len();
    let length = metadata.get(start..)?.find(METADATA_END_TAG)?;
    Some((start, start + length))
}

/// The text of the block in `metadata`, or `None` without a complete block (`getMetaData`,
/// plugin.cpp:1240-1262, reads that as the empty text).
pub fn metadata_block(metadata: &str) -> Option<&str> {
    let (start, end) = block_range(metadata)?;
    metadata.get(start..end)
}

/// `existing` metadata with its block's text replaced by `status`, the text before and after the
/// block kept (`setMetaData`, plugin.cpp:1264-1289). Without a complete block the result is the
/// block alone: the original drops the rest of the metadata then.
pub fn merge_metadata(existing: &str, status: &str) -> String {
    let (before, after) = match block_range(existing) {
        Some((start, end)) => (
            existing
                .get(..start.saturating_sub(METADATA_START_TAG.len()))
                .unwrap_or_default(),
            existing
                .get(end + METADATA_END_TAG.len()..)
                .unwrap_or_default(),
        ),
        None => ("", ""),
    };
    format!("{before}{METADATA_START_TAG}{status}{METADATA_END_TAG}{after}")
}

/// Whether `client_metadata`, another client's whole metadata, says its plugin is connected to a
/// running game: the block's text is longer than [`PLAYING_STATUS_PREFIX`] and starts with it
/// (`isPluginEnabledForUser` with the strict `startWith`, plugin.cpp:494 and 2526-2533).
pub fn metadata_says_playing(client_metadata: &str) -> bool {
    metadata_block(client_metadata).is_some_and(|block| {
        block.len() > PLAYING_STATUS_PREFIX.len() && block.starts_with(PLAYING_STATUS_PREFIX)
    })
}

/// The text `ts3plugin_infoData` shows for a client whose metadata is `metadata`: the block's
/// text, cut to the [`INFO_DATA_BUFFER_SIZE`] − 1 bytes the original's `snprintf` keeps, at a
/// character boundary; `None` without a block, where the original shows the empty text.
pub fn info_data_text(metadata: &str) -> Option<&str> {
    let block = metadata_block(metadata)?;
    let mut end = block.len().min(INFO_DATA_BUFFER_SIZE - 1);
    while !block.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    block.get(..end)
}
