//! `sanitize_nickname`: the form of a TeamSpeak nickname the game and the plugin compare.

#[cfg(test)]
#[path = "tests/nickname.rs"]
mod tests;

/// `nickname` with every leading and every trailing space replaced by `_`, one for one; spaces
/// inside the name stay. A name of spaces only becomes underscores only.
///
/// The game commands name players in this form, so every nickname read from them is converted
/// before it is compared (`convertNickname` in `reference/tfar_1_0/ts/src/CommandProcessor.cpp`).
pub fn sanitize_nickname(nickname: &str) -> String {
    let core = nickname.trim_matches(' ');
    if core.is_empty() {
        return "_".repeat(nickname.len());
    }
    let leading = nickname.len() - nickname.trim_start_matches(' ').len();
    let trailing = nickname.len() - nickname.trim_end_matches(' ').len();
    let mut sanitized = String::with_capacity(nickname.len());
    sanitized.push_str(&"_".repeat(leading));
    sanitized.push_str(core);
    sanitized.push_str(&"_".repeat(trailing));
    sanitized
}
