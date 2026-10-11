# Debug window

The `tfar_debug_window` crate: the 1.0 plugin's debug window, an egui window on X11. Used by
`tfar_plugin_1_0`.

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

`window_control` owns the threads: `DebugWindow::spawn` checks `DISPLAY`, claims the process's one
UI thread and starts it over the `EframeBackend` from `egui_window`; the handle turns show, hide
and shutdown into requests on shared state, and publish into an `ArcSwap` store. `egui_window`
only draws: on each frame it reads the latest snapshot and lays it out with `snapshot_view`.
`debug_snapshot` is plain data shared by both. The `WindowBackend` trait between them lets the
tests drive the state machine with a fake window and no display.

## Modules

### `src/egui_window/`

`EframeBackend::run_window` calls `eframe::run_native` with `run_and_return`, so it returns when the
window closes and eframe keeps its event loop in a thread-local for the next window. The first
call builds that loop through the `event_loop_builder` hook with `with_x11()` and
`with_any_thread(true)`. The app's `logic` sends `ViewportCommand::Close` as soon as the session
says to close; its `ui` draws the latest snapshot in a central panel and asks for a repaint after
500 ms. `show_snapshot` draws one `CollapsingHeader` per section (open by default, keyed by its
title) over a striped `Grid`: two columns for key/value rows, a header row and the cells for a
table, highlighted rows in the original's talking-client red.

### `src/window_control/`

```text
request (plugin threads):  Hidden ⇄ Shown ──▶ ShutDown      (ShutDown is final)
phase   (UI thread):       Hidden ──▶ Open ──▶ Hidden ... ──▶ Stopped

UI thread: wait on the condition variable while request == Hidden
           request == Shown    → phase Open, backend.run_window(session) blocks until it closes
           window closed       → phase Hidden; a still-Shown request falls back to Hidden
           request == ShutDown → drop the backend, phase Stopped
```

`show`, `hide` and `shutdown` set the request under the mutex, notify the condition variable and,
with no lock held, call the waker the open window registered (eframe: `request_repaint`), so the
window sees the request on its next frame and closes itself. `publish` stores the snapshot in an
`ArcSwap` and wakes the window the same way. `shutdown` waits on the phase for at most
`SHUTDOWN_TIMEOUT` and only then joins, so it never hangs on a stuck window. A window that fails to
open leaves its reason for `take_window_error` and is not retried until the next `show`.

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
- `Error`, `Result` and `prelude`.

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

- [TFAR 1.0 plugin behaviour](/documentation/behaviour_specs/tfar_1_0_plugin.md) — the
  original 1.0 plugin this crate ports.
