# Web requests

The `tfar_web_requests` crate: the update check and the Piwik tracking calls, over blocking HTTP
and HTTPS. Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/services/tfar_web_requests/
├── Cargo.toml  the `tfar_web_requests` package (lib): tier 0, workspace lints and dependencies
└── src/        the HTTP client, the update check, the Piwik requests and their sender
```

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
- `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `ureq` (with rustls) for HTTP, `url` for the tracking URLs,
  `version-compare` for the comparison, `tracing` for log lines, `thiserror` for the error type.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no request runs on the caller's thread for `spawn_*`, and none on a TeamSpeak audio
  callback. Tests never reach the real network. Tier 0 in `crates/services`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Bug fixes](/documentation/program/bug_fixes.md) — the version comparison this crate corrects.
