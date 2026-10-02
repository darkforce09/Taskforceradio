# Egui window

The real window: eframe's event loop on X11 with the `glow` renderer, and the egui layout of a
snapshot.

## Contents

```text
crates/interface/tfar_debug_window/src/egui_window/
├── eframe_backend.rs  `EframeBackend`: the native options and the eframe app for one window
├── mod.rs             the module lines and the re-export
├── snapshot_view.rs   `show_snapshot`: collapsing headers over striped grids in a scroll area
└── tests/             the options, a headless draw of a snapshot and the ignored real-window test
```

## How it works

`EframeBackend::run_window` calls `eframe::run_native` with `run_and_return`, so it returns when the
window closes and eframe keeps its event loop in a thread-local for the next window. The first
call builds that loop through the `event_loop_builder` hook with `with_x11()` and
`with_any_thread(true)`. The app's `logic` sends `ViewportCommand::Close` as soon as the session
says to close; its `ui` draws the latest snapshot in a central panel and asks for a repaint after
500 ms. `show_snapshot` draws one `CollapsingHeader` per section (open by default, keyed by its
title) over a striped `Grid`: two columns for key/value rows, a header row and the cells for a
table, highlighted rows in the original's talking-client red.

## Boundaries

- Depends on: `window_control` (`WindowBackend`, `WindowSession`, `Waker`), `debug_snapshot`;
  `eframe`, `winit` (`EventLoopBuilderExtX11`).
- Used by: `window_control::debug_window_handle`, which starts the UI thread over
  `EframeBackend`.
- Rules: only this module names `eframe` and `winit`; every section, key, value and cell of a
  snapshot is drawn (`every_section_title_key_value_and_cell_is_drawn`); the options keep `glow`,
  `run_and_return` and the event-loop hook
  (`the_window_uses_glow_returns_on_close_and_configures_the_event_loop`).
