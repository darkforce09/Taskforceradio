# Piwik tracking

Builds the `TRACK` requests of both mods and sends them without making anyone wait.

## Contents

```text
crates/services/tfar_web_requests/src/piwik_tracking/
├── custom_variables.rs   the 1.0 SQF custom-variable array, read like the original parser
├── mod.rs                the module lines and the public re-exports
├── tests/                hand-derived URLs, the parser cases and the sender against the responder
├── track_request_0_9.rs  `build_track_request_0_9` and `PIWIK_HOST`
├── track_request_1_0.rs  `build_track_request_1_0` and `PIWIK_BETA_HOST`
└── track_sender.rs       `spawn_track`, `spawn_cancellable_track` and `TRACK_TIMEOUT`
```

## How it works

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

## Boundaries

- Depends on: `http_client` (`HttpClient`, `ProxyChoice`), `background_request`; `url`, `tracing`.
- Used by: the crate root re-exports, so the plugin apps' `TRACK` command handlers.
- Rules: the 1.0 query matches the hand derivation from the C++
  (`a_session_tracker_message_builds_the_originals_query`,
  `the_beta_action_goes_to_the_beta_host_and_names_it_as_the_site`); the 0.9 path goes out as the
  game sent it (`the_game_path_goes_to_the_piwik_host_over_plain_http`); a dead host fails quietly
  within the timeout (`a_dead_host_fails_quietly_within_the_timeout`); a request cancelled before
  it starts is never sent (`a_request_cancelled_before_it_starts_is_never_sent`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  `TRACK` command and its async handling.
- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  0.9 `TRACK` command.
