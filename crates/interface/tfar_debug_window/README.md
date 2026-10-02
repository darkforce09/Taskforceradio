# Debug window

The `tfar_debug_window` crate: the 1.0 plugin's debug window, an egui window on X11. Used by
`tfar_plugin_1_0`.

## Contents

```text
crates/interface/tfar_debug_window/
├── Cargo.toml  the `tfar_debug_window` package (lib): tier 0, workspace lints and dependencies
└── src/        the crate root, the role constant, the error type and the prelude
```

## How it works

The 1.0 plugin's debug window: one long-lived UI thread running an `eframe` window on X11 with the
panels that show client and radio state. The code is not written yet: `src/lib.rs` carries the
module header and re-exports, `src/crate_role.rs` states the crate's role in one constant, and
`src/error.rs` holds the `thiserror` error type.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_debug_window  # the unit tests
```

## Configuration

None: the crate reads no setting.

## Public surface

Today the crate exports `CRATE_ROLE`, `Error`, `Result` and `prelude`; the surface it is for:

- `DebugWindow::spawn` and `DebugWindow::join`: the one UI thread per process.
- `DebugSnapshot`: the state the plugin publishes for the panels to draw.

## Boundaries

- Depends on: no workspace crate; `thiserror` for the error type. Planned externals: `eframe`,
  `egui`.
- Used by: `tfar_plugin_1_0`.
- Rules: one event loop per process, joined at shutdown. Tier 0 in `crates/interface`: every
  workspace dependency has a lower tier and follows the edge matrix in `crates/README.md`
  (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
