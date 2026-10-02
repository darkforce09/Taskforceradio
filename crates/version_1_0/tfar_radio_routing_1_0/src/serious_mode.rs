//! The serious-mode channel: which TeamSpeak channel the `serious_channelName` setting names.

use tfar_arma_values::parse_int;

#[cfg(test)]
#[path = "tests/serious_mode.rs"]
mod tests;

/// The prefix that names a channel by id instead of by name (`ID:<n>`).
pub const CHANNEL_ID_PREFIX: &str = "ID:";

/// A TeamSpeak channel id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId(pub u64);

/// The channel `configured_name` names among `channels` (id and name pairs, in TeamSpeak's
/// order), as `Teamspeak::isInChannel` and the channel name cache read it
/// (`reference/tfar_1_0/ts/src/Teamspeak.cpp:77-87`, `:518-524`):
///
/// - the empty name names no channel (serious mode is off);
/// - `ID:<n>` names channel `n`, read as an Arma integer, whether listed or not; a malformed or
///   negative `n` names none;
/// - any other text names the channel of exactly that name; of several, the last listed, as the
///   cache keeps the last one it saw.
pub fn serious_mode_channel<'a>(
    configured_name: &str,
    channels: impl IntoIterator<Item = (ChannelId, &'a str)>,
) -> Option<ChannelId> {
    if configured_name.is_empty() {
        return None;
    }
    if let Some(id_text) = configured_name.strip_prefix(CHANNEL_ID_PREFIX) {
        let id = parse_int(id_text).ok()?;
        return u64::try_from(id).ok().map(ChannelId);
    }
    channels
        .into_iter()
        .filter(|(_, name)| *name == configured_name)
        .last()
        .map(|(id, _)| id)
}
