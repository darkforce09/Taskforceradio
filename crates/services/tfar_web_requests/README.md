# Web requests

The `tfar_web_requests` crate: the update check and the Piwik tracking calls, over blocking HTTPS.
Used by `tfar_plugin_0_9`, `tfar_plugin_1_0`.

## Contents

```text
crates/services/tfar_web_requests/
├── Cargo.toml  the `tfar_web_requests` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The plugins' outbound HTTP: the update check against the published version and the Piwik `TRACK`
calls, over `ureq` with rustls, off the audio thread. The code is not written yet: `src/lib.rs`
carries the module header and re-exports, `src/crate_role.rs` states the crate's role in one
constant, and `src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_web_requests  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `check_for_update`: fetches the latest version and compares it with `version-compare`.
- `track_event`: one Piwik tracking call.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `ureq`,
  `version-compare`.
- Used by: `tfar_plugin_0_9`, `tfar_plugin_1_0`.
- Rules: no request runs on a TeamSpeak audio callback. Tier 0 in `crates/services`: every workspace
  dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  original 0.9.12 plugin this crate ports.
- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
