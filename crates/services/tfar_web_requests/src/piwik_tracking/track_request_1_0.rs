//! The 1.0 Piwik request: `TRACK\taction\tuid\tcustomVars` becomes one `GET` URL.
//!
//! **Role:** builds the query string `TFAR::trackPiwik` in
//! `reference/tfar_1_0/ts/src/task_force_radio.cpp` builds, byte for byte before encoding:
//!
//! ```text
//! piwik.php?idsite=2&rec=1&url="<site>"&action_name=<action>&rand=<rand>&uid=<uid>
//!     &cvar={"<id>":["<name>","<value>"],...}
//! ```
//!
//! where `<site>` and the host are `piwik.dedmen.de` for the action `beta` and otherwise
//! `radio.task-force.ru` on the host [`PIWIK_HOST`]. Action and uid go in as the game sent them,
//! unescaped, as in the original.
//!
//! **Position:** inside `piwik_tracking`; uses `custom_variables`; called by the 1.0 plugin's
//! `TRACK` command, which hands the URL to [`spawn_track`](super::spawn_track).
//!
//! **Signals and state:** none; `rand` comes from a fresh `RandomState`.
//!
//! **Invariants:** the URL is plain HTTP on port 80, as the original's
//! `INTERNET_DEFAULT_HTTP_PORT`; the only encoding applied is the one a request line needs, done by
//! `url`'s query setter: space, `"`, `#`, `<`, `>`, `'` and non-ASCII become `%XX`.

#[cfg(test)]
#[path = "tests/track_request_1_0.rs"]
mod tests;

use std::fmt::Write as _;
use std::hash::{BuildHasher, RandomState};

use url::Url;

use super::custom_variables::parse_custom_variables;
use super::track_request_0_9::PIWIK_HOST;
use crate::{Error, Result};

/// The Piwik host and site the 1.0 mod's beta builds report to.
pub const PIWIK_BETA_HOST: &str = "piwik.dedmen.de";

/// The `url` parameter of every non-beta request.
const RELEASE_SITE: &str = "radio.task-force.ru";

/// The action that selects the beta host.
const BETA_ACTION: &str = "beta";

/// One past the largest value MSVC's `rand()` returns (`RAND_MAX` is 32767).
const RAND_RANGE: u64 = 32_768;

/// Builds the 1.0 tracking URL for `TRACK\t<action>\t<uid>\t<custom_vars_text>`, with a random
/// `rand` parameter.
///
/// # Errors
///
/// [`Error::NoCustomVariables`] when the custom-variable text yields no variable, where the
/// original sends nothing; [`Error::InvalidUrl`] when the URL cannot be formed.
pub fn build_track_request_1_0(action: &str, uid: &str, custom_vars_text: &str) -> Result<Url> {
    build_track_request_1_0_with_rand(action, uid, custom_vars_text, random_rand())
}

/// [`build_track_request_1_0`] with a fixed `rand` parameter.
pub(crate) fn build_track_request_1_0_with_rand(
    action: &str,
    uid: &str,
    custom_vars_text: &str,
    rand: u32,
) -> Result<Url> {
    let variables = parse_custom_variables(custom_vars_text);
    if variables.is_empty() {
        return Err(Error::NoCustomVariables {
            text: custom_vars_text.to_owned(),
        });
    }
    let (host, site) = if action == BETA_ACTION {
        (PIWIK_BETA_HOST, PIWIK_BETA_HOST)
    } else {
        (PIWIK_HOST, RELEASE_SITE)
    };
    let mut query =
        format!("idsite=2&rec=1&url=\"{site}\"&action_name={action}&rand={rand}&uid={uid}&cvar={{");
    for (index, variable) in variables.iter().enumerate() {
        if index > 0 {
            query.push(',');
        }
        // Writing into a `String` cannot fail.
        let _ = write!(
            query,
            "\"{}\":[\"{}\",\"{}\"]",
            variable.id, variable.name, variable.value
        );
    }
    query.push('}');
    let base = format!("http://{host}/piwik.php");
    let mut url = Url::parse(&base).map_err(|source| Error::InvalidUrl { text: base, source })?;
    url.set_query(Some(&query));
    Ok(url)
}

/// A value in MSVC's `rand()` range, from a freshly keyed `RandomState`.
fn random_rand() -> u32 {
    let value = RandomState::new().hash_one(std::time::SystemTime::now()) % RAND_RANGE;
    u32::try_from(value).unwrap_or(0)
}
