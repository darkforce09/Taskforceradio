# Service crates

The plugins' outbound network services: the update check and the Piwik tracking calls.

## How it works

`tfar_web_requests` makes blocking HTTPS calls on a worker thread, so no request touches a TeamSpeak
audio callback.

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_web_requests
```

## Boundaries

- Depends on: `crates/foundation` only.
- Used by: the plugin apps.
- Rules: a services crate depends only on foundation crates; `ureq` stays in `tfar_web_requests`
  (`cargo xtask verify crate-tiers` holds the edges, `cargo xtask verify dependency-firewalls` the
  external crates); each crate declares `category = "crates/services"` and its tier: 0 without a
  workspace dependency, else one above the highest.

## Related documentation

