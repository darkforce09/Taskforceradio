# Debug window source

The source of `tfar_debug_window`: the snapshot model, the handle's state machine and the egui
window that draws it.

## Contents

```text
crates/interface/tfar_debug_window/src/
├── crate_role.rs      `CRATE_ROLE`: what the crate is for, in one sentence
├── debug_snapshot.rs  `DebugSnapshot` and its sections, rows and tables
├── egui_window/       the eframe window backend and the snapshot view
├── error.rs           `Error`, the `thiserror` enum, and `Result`
├── lib.rs             the crate root: module header, `mod` lines and re-exports
├── prelude.rs         the spawn, handle and snapshot types for glob import
├── tests/             unit tests of `CRATE_ROLE` and the snapshot model
└── window_control/    `DebugWindow::spawn`, the handle, the UI thread and the backend seam
```

## How it works

`window_control` owns the threads: `DebugWindow::spawn` checks `DISPLAY`, claims the process's one
UI thread and starts it over the `EframeBackend` from `egui_window`; the handle turns show, hide
and shutdown into requests on shared state, and publish into an `ArcSwap` store. `egui_window`
only draws: on each frame it reads the latest snapshot and lays it out with `snapshot_view`.
`debug_snapshot` is plain data shared by both. The `WindowBackend` trait between them lets the
tests drive the state machine with a fake window and no display.

## Public surface

- `DebugWindow`, `DebugWindowHandle` and `SHUTDOWN_TIMEOUT` from `window_control`.
- `DebugSnapshot`, `DebugSection`, `SectionContent`, `KeyValueRow`, `DebugTable` and `TableRow`
  from `debug_snapshot`.
- `CRATE_ROLE`, `Error`, `Result` and `prelude`.

## Boundaries

- Depends on: `eframe`, `winit`, `arc-swap`, `parking_lot`, `tracing`, `thiserror`.
- Used by: `tfar_plugin_1_0`.
- Rules: only `egui_window` names `eframe` or `winit`; production files hold no `unwrap`,
  `expect`, `panic` or indexing (clippy deny list); tests live in `tests/`
  (`cargo xtask verify test-placement`).
