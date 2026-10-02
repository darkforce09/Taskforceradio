//! Writing a reply into the output buffer Arma passes to `RVExtension`.

#[cfg(test)]
#[path = "tests/reply_buffer.rs"]
mod tests;

/// Writes `reply` into `output` as a NUL-terminated C string.
///
/// At most `output.len() − 1` bytes of `reply` are written, cut back to a UTF-8 character boundary
/// so Arma never receives half a character, then a NUL. Bytes after the NUL are left as they were.
/// An empty `output` has no room for the NUL, so nothing is written.
pub fn write_reply(output: &mut [u8], reply: &str) {
    let Some(room) = output.len().checked_sub(1) else {
        return;
    };
    let kept = reply.floor_char_boundary(room.min(reply.len()));
    let kept_bytes = reply.as_bytes().get(..kept).unwrap_or_default();
    if let Some(destination) = output.get_mut(..kept) {
        destination.copy_from_slice(kept_bytes);
    }
    if let Some(terminator) = output.get_mut(kept) {
        *terminator = 0;
    }
}
