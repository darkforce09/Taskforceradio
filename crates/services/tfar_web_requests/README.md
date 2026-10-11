# Web requests

The `tfar_web_requests` crate: the update check and the Piwik tracking calls, over blocking HTTP
and HTTPS. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## How it works

The plugins' outbound HTTP, each request on a short-lived named thread so no TeamSpeak callback
waits on the network:

```text
ts3plugin_init ──spawn_update_check──▶ "tfar-update-check" ──GET (10 s)──▶ raw.githubusercontent.com
                                           │ newer?  on_update() → the plugin prints the message
TRACK command ──build_track_request_*──▶ Url ──spawn_track──▶ "tfar-piwik-track" ──GET (5 s)──▶ Piwik
```

The update check fetches `current_version.txt` over HTTPS (redirects followed) and calls back only
when the body trims to a dotted version newer than the plugin's, compared with `version-compare`;
an empty, garbage or error body is no update. The tracking requests follow the originals: 0.9
sends the path the game formatted, 1.0 builds `piwik.php?idsite=2&rec=1&url="…"&action_name=…
&rand=…&uid=…&cvar={…}` from the action, uid and SQF custom-variable array. Both go over plain HTTP
to `nkey.piwik.pro` (or `piwik.dedmen.de` for the 1.0 action `beta`). Those hosts may be gone:
every failure is logged at debug level and goes no further.

`spawn_cancellable_update_check` and `spawn_cancellable_track` run the same requests behind a
`BackgroundRequest` handle, so a plugin can stop them at shutdown: `cancel()` sets a flag the
worker checks before its request (a request cancelled in time is never sent) and after it (a
cancelled check never calls back); `join_timeout(budget)` waits for the worker up to a deadline;
dropping the handle cancels and never blocks. A request already in flight is not interrupted; its
`ureq` timeout ends it. `spawn_update_check` and `spawn_track` stay as they were, detached.

`http_client` is the only module that talks to `ureq`: `HttpClient::new(timeout, proxy)` builds an
agent with one timeout for the whole request, up to five redirects, the `TFAR` user agent and
4xx/5xx as errors, and maps a `ureq` timeout to `Error::TimedOut` and anything else to
`Error::Request`. `update_check` and `piwik_tracking` each start their own named thread with a
client and return at once; their cancellable variants start it through
`BackgroundRequest::spawn`, which shares a cancel flag with the worker and a channel that
disconnects when the worker ends, so `join_timeout` can wait with a deadline. The tests give the client `ProxyChoice::Direct` and point it at
`tests/local_responder.rs`, a `TcpListener` on 127.0.0.1 that plays one scripted reply per
connection (a body, a slow body, a redirect, a 404 or silence) and records each request line.

## Modules

### `src/piwik_tracking/`

```text
0.9  TRACK\t<path>                         → http://nkey.piwik.pro/<path>
1.0  TRACK\t<action>\t<uid>\t<customVars>  → http://<host>/piwik.php?idsite=2&rec=1&url="<site>"
                                               &action_name=<action>&rand=<0..32767>&uid=<uid>
                                               &cvar={"<id>":["<name>","<value>"],...}
     host and site: piwik.dedmen.de for action "beta"; else nkey.piwik.pro and radio.task-force.ru
spawn_track(url) → thread "tfar-piwik-track": GET (5 s), body read and dropped, failure logged
spawn_cancellable_track(url) → BackgroundRequest: the same, never sent once cancelled before it
```

`parse_custom_variables` reads `[[1,"name","value"],...]` the way the original's character
stream does: whitespace skipped everywhere, the id to the next comma parsed like `std::stoi`, the
name to the next comma, the value to the next `]`, spaces and double quotes trimmed from both
ends. An id `std::stoi` refuses, or text that ends inside a variable, voids the whole array, and an
empty array means no request (`Error::NoCustomVariables`). Action, uid and variables go into the
query unescaped, as in the original; `url`'s query setter then encodes only what a request line
cannot carry (space, `"`, `#`, `<`, `>`, `'`, non-ASCII). Both hosts may no longer answer, so a
failed request is a debug log line and nothing else.

### `src/update_check/`

```text
spawn_update_check(current, on_update)
  └─ thread "tfar-update-check": GET UPDATE_CHECK_URL (10 s, redirects followed)
       ├─ body trims to a dotted version newer than current → on_update()
       └─ same, older, garbage, empty, 4xx/5xx, timeout       → a debug log line only
spawn_cancellable_update_check(current, on_update) → BackgroundRequest
  └─ the same thread; cancelled before the GET → nothing sent; cancelled after → no on_update()
```

`fetch_latest_version` returns the body trimmed and unchecked; `is_update_available` decides. A
version is one or more groups of ASCII digits joined by single dots; `version-compare` then
compares component by component as numbers, every component counting. The plugin prints
`UPDATE_AVAILABLE_MESSAGE` from its callback where the originals opened a message box.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_web_requests  # unit tests, against a loopback responder only (under 1 s)
```

## Configuration

None: the crate reads no setting. The timeouts are constants (`UPDATE_CHECK_TIMEOUT` 10 s,
`TRACK_TIMEOUT` 5 s); requests honour `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY`.

## Public surface

- Update check: `fetch_latest_version`, `is_update_available`, `is_version_string`,
  `spawn_update_check`, `spawn_cancellable_update_check`, `UPDATE_CHECK_URL`, `UPDATE_CHECK_TIMEOUT`, `UPDATE_AVAILABLE_MESSAGE`.
- Piwik tracking: `build_track_request_0_9`, `build_track_request_1_0`, `spawn_track`,
  `spawn_cancellable_track`,
  `parse_custom_variables`, `CustomVariable`, `PIWIK_HOST`, `PIWIK_BETA_HOST`, `TRACK_TIMEOUT`, and
  `Url` (re-exported from `url`).
- `BackgroundRequest`: `cancel`, `is_cancelled`, `is_finished`, `join_timeout`; dropping it
  cancels without waiting.
- `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `ureq` (with rustls) for HTTP, `url` for the tracking URLs,
  `version-compare` for the comparison, `tracing` for log lines, `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no request runs on the caller's thread for `spawn_*`, and none on a TeamSpeak audio
  callback. Tests never reach the real network.

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Bug fixes](/documentation/architecture/bug_fixes.md) — the version comparison this crate corrects.
