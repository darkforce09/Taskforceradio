# Window control

The handle the plugin keeps and the UI thread it drives: one thread per process, windows opened
and closed on request.

## Contents

```text
crates/interface/tfar_debug_window/src/window_control/
├── debug_window_handle.rs  `DebugWindow::spawn`, `DebugWindowHandle` and the UI thread's loop
├── mod.rs                  the module lines and the re-exports
├── tests/                  the state machine through a fake backend, headless
├── window_backend.rs       the `WindowBackend` trait a real or fake window implements
└── window_state.rs         the request, phase, snapshot and waker the threads share
```

## How it works

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

## Boundaries

- Depends on: `debug_snapshot` (`DebugSnapshot`), `egui_window` (`EframeBackend`); `arc-swap`,
  `parking_lot`, `tracing`.
- Used by: the crate root re-exports, so `tfar_plugin_1_0`; `egui_window` implements
  `WindowBackend` over `WindowSession`.
- Rules: one UI thread per process (`a_second_claim_of_the_process_slot_is_refused`); a hidden
  window opens only on `show` and a closed one stays closed
  (`a_window_the_user_closes_stays_closed_until_the_next_show`,
  `a_window_that_fails_to_open_reports_why_once`); a publish reaches an open window through the
  waker (`a_snapshot_published_while_open_reaches_the_window_at_once`); shutdown joins within one
  second or reports `ShutdownTimedOut`
  (`a_window_that_never_closes_makes_shutdown_time_out_after_one_second`).
