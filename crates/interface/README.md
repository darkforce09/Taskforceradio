# Interface crates

The plugins' own user interface: the 1.0 debug window.

## Contents

```text
crates/interface/
└── tfar_debug_window/  tier 0: the debug window
```

## How it works

`tfar_debug_window` runs one long-lived `eframe` thread on X11 and draws the snapshots the 1.0
plugin publishes.

## Getting started

Run from the repository root:

```bash
# the unit tests of every crate in this category
cargo test \
  -p tfar_debug_window
```

## Boundaries

- Depends on: `crates/foundation` only.
- Used by: `tfar_plugin_1_0`.
- Rules: an interface crate depends only on foundation crates; `eframe` and `egui` stay in
  `tfar_debug_window` (`cargo xtask verify crate-tiers` holds the edges,
  `cargo xtask verify dependency-firewalls` the external crates); each crate declares
  `category = "crates/interface"` and its tier: 0 without a workspace dependency, else one above the
  highest.

## Related documentation

- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen for each need.
