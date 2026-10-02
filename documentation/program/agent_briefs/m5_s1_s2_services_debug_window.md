**Status:** live

# M5 · S1 + S2 · `tfar_web_requests` and `tfar_debug_window`

You are agent S1, the services and diagnostics author for the TFAR Rust port. You own both crates in this prompt.
Read `/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md`. Decision D4: every original feature stays, including the update
check, Piwik tracking and the debug window.

## `tfar_web_requests` (crates/services/tfar_web_requests), `ureq` + rustls
- **Update check**, ported from `reference/tfar_1_0/ts/src/task_force_radio.cpp:180-209` and `reference/tfar_0_9_12/ts/src/plugin.cpp:155`:
  - `fetch_latest_version(url, timeout) -> Result<String>` fetches `current_version.txt`. The original host `raw.github.com/michail-nikolaev/task-force-arma-3-radio/master/current_version.txt` now redirects, so use `https://raw.githubusercontent.com/...` and follow redirects.
  - `is_update_available(current: &str, latest: &str) -> bool` uses `version-compare` (0.9: `0.9.12`; 1.0: `1.0.0.1`) and fixes F-026 (the 0.9 `versionNumber` digit-sum bug). Both originals ignore bodies of length ≤ 3 (1.0) or ≥ 10 (0.9), so take a body that trims to a version string, and anything else gives "no update".
  - `spawn_update_check(current, on_update: impl FnOnce() + Send)` runs on a thread with a 10 s timeout and calls `on_update` only on success with a newer version. The plugin prints the original message ("New version of Task Force Arrowhead Radio is available. Check radio.task-force.ru/en") to the TS tab.
- **Piwik tracking**:
  - **0.9** (`reference/tfar_0_9_12/ts/src/plugin.cpp:124`): `TRACK\t<url path>` → `GET http://nkey.piwik.pro<path>`.
  - **1.0** (`reference/tfar_1_0/ts/src/task_force_radio.cpp:212-357`): `TRACK\taction\tuid\tcustomVars`, where `customVars` is an SQF array `[[1,"name","value"],...]`. It builds the exact query string the original builds; read the code fully, including the `_cvar` JSON-ish format and URL encoding. Host: `piwik.dedmen.de` when action == `beta`, else the original `PIWIK_URL` constant (find its value in the 1.0 tree).
  - API:
    - `build_track_request_0_9(path) -> Url`
    - `build_track_request_1_0(action, uid, custom_vars_text) -> Result<Url>`
    - `spawn_track(url)`: fire-and-forget, 5 s timeout, response discarded, errors logged with `tracing`.
  - Keep plain HTTP where the original used it. Document that these hosts may be dead and that the requests fail quietly.
- **Tests:** an in-test `std::net::TcpListener` HTTP responder (no external network):
  - newer, same, older, garbage and empty version bodies;
  - a redirect followed;
  - a timeout;
  - the 1.0 Piwik URL for a sample customVars string, matching an expected string you derive by hand from the C++ code (show the derivation in a comment);
  - the 0.9 URL.

## `tfar_debug_window` (crates/interface/tfar_debug_window), `eframe`/`egui`
The 1.0 `/tfar debug` window (`reference/tfar_1_0/ts/src/DebugUI.cpp`, `TextRenderer.cpp`), redone in egui.
- **`DebugWindow::spawn(title) -> DebugWindowHandle`** starts **one** long-lived UI thread per process.
  - Use the eframe event loop with winit's X11 `with_any_thread(true)`. The Flatpak TS has an X11 socket only, so force X11 and the `glow` renderer.
  - `handle.show()` / `handle.hide()` reopen and close the window without a new event loop, since winit allows only one per process.
  - `handle.publish(DebugSnapshot)` uses `arc-swap`.
  - `handle.shutdown()` joins within 1 s and is called from plugin shutdown.
- **`DebugSnapshot`**: titled sections, each with key/value rows or a table (columns + rows). The plugin fills it, for example the client list with positions and radios, as the original `debugDisplayThing` did. Render each section with egui `CollapsingHeader` + `Grid`.
- If no display is available (`DISPLAY` unset), `spawn` returns an `Error`, and the plugin prints a TS message instead.
- **Tests:** the snapshot model and the handle state machine (show/hide/publish/shutdown) through an internal trait so they run headless. Mark the one real-window test `#[ignore = "needs an X11 display"]`.

## Owned files
`crates/services/tfar_web_requests/**`, `crates/interface/tfar_debug_window/**`. For root manifest additions (e.g.
`url`, `percent-encoding`, an eframe feature), make one Edit, re-reading first, and report it.

## Checks (each alone, with your target dir; logs `target/agent-logs/s1-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API list; the PIWIK_URL value found and the 1.0 query format.

Budget: L (350k tokens). Stop there and report what is done and what is not.
