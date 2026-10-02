//! The 0.9 Piwik request: the game sends the whole path and query (`TRACK\tpiwik.php?…`), and the
//! plugin sends it to the Piwik host as it is
//! (`trackPiwik` in `reference/tfar_0_9_12/ts/src/plugin.cpp`).
//!
//! The URL is plain HTTP on port 80, as the original's `INTERNET_DEFAULT_HTTP_PORT`; `url` encodes
//! only what a request line cannot carry (space, `"`, `#`, `<`, `>`, `'` and non-ASCII).

#[cfg(test)]
#[path = "tests/track_request_0_9.rs"]
mod tests;

use url::Url;

use crate::{Error, Result};

/// The Piwik host both originals report to (their `PIWIK_URL`). It may no longer answer; a
/// request to it then fails quietly.
pub const PIWIK_HOST: &str = "nkey.piwik.pro";

/// Builds the 0.9 tracking URL `http://nkey.piwik.pro/<path>` for `TRACK\t<path>`; a leading `/`
/// on `path` is optional.
///
/// # Errors
///
/// [`Error::InvalidUrl`] when the URL cannot be formed.
pub fn build_track_request_0_9(path: &str) -> Result<Url> {
    let base = format!("http://{PIWIK_HOST}/");
    let mut url = Url::parse(&base).map_err(|source| Error::InvalidUrl { text: base, source })?;
    let (path_part, query) = match path.split_once('?') {
        Some((path_part, query)) => (path_part, Some(query)),
        None => (path, None),
    };
    url.set_path(path_part);
    url.set_query(query);
    Ok(url)
}
