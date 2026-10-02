# Debug window

The `tfar_debug_window` crate: the 1.0 plugin's debug window, an egui window on X11. Used by
`tfar_plugin_1_0`.

## Contents

```text
crates/interface/tfar_debug_window/
├── Cargo.toml  the `tfar_debug_window` package (lib): tier 0, workspace lints and dependencies
└── src/        the snapshot model, the handle and its UI thread, and the eframe window
```

## How it works

The `/tfar debug` window, redone as panels: the plugin builds a `DebugSnapshot` (titled sections of
key/value rows or tables, such as the client list with positions, frequencies and radios) and
publishes it; the window draws the latest one, each section a collapsible header over a grid.

```text
plugin threads                          UI thread "tfar-debug-window"
DebugWindow::spawn(title) ───────────▶  waits while hidden
handle.show()  ── request Shown ──────▶ eframe::run_native (X11, glow, run_and_return)
handle.publish(snapshot) ── ArcSwap ──▶   draws the latest snapshot, at least every 500 ms
handle.hide()  ── request Hidden ─────▶   closes the window, back to waiting
handle.shutdown() ── ShutDown ────────▶ closes any window, ends; joined within 1 s
```

winit allows one event loop per process, so the crate runs one UI thread for the life of the
process and eframe reuses its thread-local event loop for every window; `show` and `hide` open and
close windows on it. The event loop is forced onto X11 (the Flatpak TeamSpeak has no Wayland
socket) and allowed off the main thread. Without `DISPLAY`, `spawn` returns `Error::NoDisplay` and
the plugin prints a message instead.

## Getting started

Run from the repository root:

```bash
cargo test -p tfar_debug_window              # the snapshot model, the view and the handle, headless
cargo test -p tfar_debug_window -- --ignored  # adds the real-window test; needs an X11 display
```

## Configuration

None: the crate reads no setting. `DISPLAY` must name an X11 display.

## Public surface

- `DebugWindow::spawn`, and `DebugWindowHandle`: `show`, `hide`, `publish`, `is_open`,
  `take_window_error`, `shutdown` (dropping the handle shuts down too); `SHUTDOWN_TIMEOUT`.
- `DebugSnapshot`, `DebugSection`, `SectionContent`, `KeyValueRow`, `DebugTable`, `TableRow`.
- `Error`, `Result`, `CRATE_ROLE` and `prelude`.

## Boundaries

- Depends on: no workspace crate; `eframe` (with `glow`, `x11`, `default_fonts`) for the window,
  `winit` for the X11 event-loop options, `arc-swap` and `parking_lot` for the shared state,
  `tracing` for log lines, `thiserror` for the error type.
- Used by: `tfar_plugin_1_0`.
- Rules: one UI thread per process, ever (`a_second_claim_of_the_process_slot_is_refused`);
  windows reopen on the same thread and event loop
  (`hide_closes_the_window_and_show_reopens_it_on_the_same_thread`,
  `a_real_window_opens_closes_reopens_and_shuts_down_on_one_event_loop`); shutdown joins within
  one second (`shutdown_while_open_closes_the_window_and_joins_within_a_second`). Tier 0 in
  `crates/interface` (`cargo xtask verify crate-tiers`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
- [Library survey](/documentation/program/research/library_survey.md) — the external crates chosen
  for each need and why.
