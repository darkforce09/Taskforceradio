# Web requests source

The source of `tfar_web_requests`: one blocking HTTP client, the update check and the Piwik
tracking requests built on it.

## Contents

```text
crates/services/tfar_web_requests/src/
├── background_request.rs  `BackgroundRequest`: the cancel flag, the join with a deadline, the drop
├── crate_role.rs          `CRATE_ROLE`: what the crate is for, in one sentence
├── error.rs               `Error`, the `thiserror` enum, and `Result`
├── http_client.rs         the `ureq` agent: whole-request timeout, redirects, body limit, errors
├── lib.rs                 the crate root: module header, `mod` lines and re-exports
├── piwik_tracking/        the 0.9 and 1.0 `TRACK` URLs and the detached and cancellable senders
├── prelude.rs             the functions a plugin calls, for glob import
├── tests/                 unit tests of `CRATE_ROLE`, the handle and the loopback HTTP responder
└── update_check/          fetching the published version, the comparison and the background checks
```

## How it works

`http_client` is the only module that talks to `ureq`: `HttpClient::new(timeout, proxy)` builds an
agent with one timeout for the whole request, up to five redirects, the `TFAR` user agent and
4xx/5xx as errors, and maps a `ureq` timeout to `Error::TimedOut` and anything else to
`Error::Request`. `update_check` and `piwik_tracking` each start their own named thread with a
client and return at once; their cancellable variants start it through
`BackgroundRequest::spawn`, which shares a cancel flag with the worker and a channel that
disconnects when the worker ends, so `join_timeout` can wait with a deadline. The tests give the client `ProxyChoice::Direct` and point it at
`tests/local_responder.rs`, a `TcpListener` on 127.0.0.1 that plays one scripted reply per
connection (a body, a slow body, a redirect, a 404 or silence) and records each request line.

## Public surface

- The update check and Piwik items listed in the crate README, re-exported at the crate root from
  `update_check` and `piwik_tracking`; `Url` from `url`.
- `CRATE_ROLE`, `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: `ureq`, `url`, `version-compare`, `tracing`, `thiserror`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: only `http_client` names `ureq`; production files hold no `unwrap`, `expect`, `panic` or
  indexing (clippy deny list); tests live in `tests/` (`cargo xtask verify test-placement`) and
  reach only the loopback responder.
