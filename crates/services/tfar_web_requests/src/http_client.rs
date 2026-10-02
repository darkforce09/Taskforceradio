//! The one blocking HTTP client both features use: a `ureq` agent with a whole-request timeout,
//! redirects followed and a small body limit.
//!
//! **Role:** turns a URL and a timeout into a body string or a typed [`Error`]; nothing else in
//! the crate talks to `ureq`.
//!
//! **Position:** inside `tfar_web_requests`; used by `update_check` and `piwik_tracking`.
//!
//! **Signals and state:** none; each call blocks the calling worker thread for at most the
//! timeout, and the agent keeps no connection between calls.
//!
//! **Invariants:** a 4xx or 5xx status is an error; the body read stops at
//! [`MAX_BODY_BYTES`]; the system proxy from the environment is honoured, as the originals'
//! `INTERNET_OPEN_TYPE_PRECONFIG` honoured the Windows one.

use std::time::Duration;

use crate::{Error, Result};

/// The `User-Agent` both originals sent (`InternetOpen(L"TFAR", …)`).
const USER_AGENT: &str = "TFAR";

/// The most redirects followed; `raw.github.com` needs one to reach `raw.githubusercontent.com`.
const MAX_REDIRECTS: u32 = 5;

/// The most body bytes read: a version file is a few bytes, a Piwik answer a 1x1 GIF.
pub(crate) const MAX_BODY_BYTES: u64 = 4096;

/// Whether requests go through the proxy the environment names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProxyChoice {
    /// Use `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY`, as the plugin does.
    FromEnvironment,
    /// Connect directly, as the tests do towards their loopback responder.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "only the tests connect directly")
    )]
    Direct,
}

/// A blocking HTTP client with one timeout for the whole request, redirects included.
#[derive(Debug)]
pub(crate) struct HttpClient {
    agent: ureq::Agent,
    timeout: Duration,
}

impl HttpClient {
    /// A client whose every request, connect to last body byte, ends within `timeout`.
    pub(crate) fn new(timeout: Duration, proxy: ProxyChoice) -> Self {
        let builder = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .max_redirects(MAX_REDIRECTS)
            .user_agent(USER_AGENT)
            .http_status_as_error(true);
        let builder = match proxy {
            ProxyChoice::FromEnvironment => builder,
            ProxyChoice::Direct => builder.proxy(None),
        };
        Self {
            agent: builder.build().new_agent(),
            timeout,
        }
    }

    /// Sends `GET url` and returns the body as text, read up to [`MAX_BODY_BYTES`].
    pub(crate) fn get_text(&self, url: &str) -> Result<String> {
        let mut response = self
            .agent
            .get(url)
            .call()
            .map_err(|source| self.request_error(url, source))?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_string()
            .map_err(|source| self.request_error(url, source))
    }

    /// Sends `GET url`, reads the answer to its end (so the server sees a complete exchange, as
    /// the originals' `InternetReadFile` loop did) and drops it.
    pub(crate) fn get_and_discard(&self, url: &str) -> Result<()> {
        self.get_text(url).map(drop)
    }

    /// Classifies a `ureq` failure: a timeout is [`Error::TimedOut`], anything else
    /// [`Error::Request`].
    fn request_error(&self, url: &str, source: ureq::Error) -> Error {
        match source {
            ureq::Error::Timeout(_) => Error::TimedOut {
                url: url.to_owned(),
                timeout: self.timeout,
            },
            other => Error::Request {
                url: url.to_owned(),
                source: Box::new(other),
            },
        }
    }
}
