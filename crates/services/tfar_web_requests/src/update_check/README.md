# Update check

Fetches the published TFAR version and tells the plugin when it is newer than its own.

## Contents

```text
crates/services/tfar_web_requests/src/update_check/
├── latest_version.rs      `fetch_latest_version`, the detached and cancellable checks, constants
├── mod.rs                 the module lines and the public re-exports
├── tests/                 the comparison table and the fetch against the loopback responder
└── version_comparison.rs  `is_version_string` and `is_update_available`
```

## How it works

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

## Boundaries

- Depends on: `http_client` (`HttpClient`, `ProxyChoice`), `background_request`; `version-compare`, `tracing`.
- Used by: the crate root re-exports, so the plugin apps' `ts3plugin_init`.
- Rules: the callback runs at most once and only on a strictly newer version
  (`a_newer_body_calls_back_once`, `the_same_body_calls_nothing`, `an_older_body_calls_nothing`,
  `a_garbage_body_calls_nothing`, `an_empty_body_calls_nothing`); `0.9.12` is newer than `0.9.3`
  (`components_compare_as_numbers_so_0_9_12_is_newer_than_0_9_3`); a redirect is followed
  (`a_redirect_is_followed_to_the_version_file`); a silent server gives `Error::TimedOut` within
  the timeout (`a_silent_server_times_out_within_the_timeout`); a cancelled check sends nothing or calls
  nothing (`a_check_cancelled_before_it_starts_sends_nothing_and_calls_nothing`,
  `a_check_cancelled_during_a_slow_answer_drops_the_answer_and_joins`).

## Related documentation

- [Bug fixes](/documentation/program/bug_fixes.md) — the 0.9 version comparison this module
  corrects.
