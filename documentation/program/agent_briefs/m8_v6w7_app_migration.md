**Status:** live

# M8 · V6 (0.9) / W7 (1.0) · Move the apps onto the new library APIs

This brief serves two agents. **V6** owns `crates/version_0_9/tfar_plugin_0_9/**`. **W7** owns `crates/version_1_0/tfar_plugin_1_0/**`.

Read first:
- `/documentation/program/agent_briefs/shared_brief.md` (**Build directory rule**: `target-v6` / `target-w7`)
- the L4 and P5 reports' migration notes, which the orchestrator pastes into your launch prompt
- `/documentation/decisions/game_link.md` (D24, D25)
- your app's current code

## Work
1. **Replace every app-private workaround** with the library API, and delete the private copy and its now-duplicate tests. Keep any test that covers app wiring.
   - Positional remote sounds → `render_remote_tangent_sound` (voice pipeline).
   - Own-voice stream → `PlaybackMixer::stream_producer`.
   - Update check and Piwik → `BackgroundRequest` handles, cancelled and joined at shutdown within the 2 s budget.
   - `system_set_3d_listener_attributes(None, None)`.
   - Routing on the audio path through the pipeline's `is_over_radio_into` scratch. Nothing to do in the app if the pipeline owns it; verify.
   - **W7 only:** the local tangent → `SessionState::apply_local_tangent`; a sync POS async apply → `GameLinkServer::queue_async`.
2. **D24.** Build `ServerConfig` with both ports (`port(flavour, false)`, `port(flavour, true)`). Remove the `TFAR_GAME_LINK_DEBUG` path. Keep the test-only `TFAR_GAME_LINK_PORT` override.
3. **D25.** After game-end handling completes (unmute all, the off sound, metadata to "not in game" once), the plugin returns to dormant: no further metadata, nickname, mute, channel or plugin-command side effects until game activity resumes.
   - Mock test: game session → MISSIONEND/timeout → assert the dormant set of calls (none) over the next 10 s of service ticks, with the time injected or the interval shortened by a test hook. Then game activity resumes, and side effects resume.
4. **The HRTF reset** on `on_talk_status_change(talking = true)` for that sender. W7 removes the 200 ms gap heuristic.
5. **Shutdown proof:** a mock test asserts every thread the plugin started has been joined (service, link, PTT release, web requests, logger) and that `shutdown` returns under 2 s.

## Checks (each alone, with your target dir; logs `target/agent-logs/<v6|w7>-*.log`)
fmt, clippy `--all-targets -D warnings` and test for your app; `cargo xtask dist` with your target dir (the `.so` exports via `nm -D`);
`cargo xtask verify all` (report foreign violations only). If the permission classifier refuses a perturbation edit, report it as not run.

## Report additions
Removed private modules; the new tests; the measured shutdown time.

Budget: M (250k tokens). Stop there and report what is done and what is not.
