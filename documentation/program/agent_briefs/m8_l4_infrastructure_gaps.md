**Status:** live

# M8 · L4 · Infrastructure library gaps

You are agent L4 for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first
(**Build directory rule**: `CARGO_TARGET_DIR=.../target-l4`), then `/documentation/standards/coding_standards.md`. The plugin
apps (`crates/version_0_9/tfar_plugin_0_9`, `crates/version_1_0/tfar_plugin_1_0`) worked around these gaps privately. Read their code to see the
need, but don't edit the apps; wave 2 switches them over.

**Keep every existing public API working.** Add new APIs beside the old ones, or extend them compatibly, so the apps
still build.

## Work
1. **`tfar_game_link_server`:**
   - `ServerConfig` accepts several ports (`ports: Vec<u16>`, keeping a convenience constructor for one port). A plugin binds its normal and its `-tfdebug` port (D24). Each port has its own accept loop and independent bind retry; all of them feed **one** session slot, and the newest valid HELLO wins.
   - `local_addrs()` exposes the bound addresses.
   - Add `GameLinkServer::queue_async(text: &[u8])`, which enqueues into the same FIFO worker as `ASYNC` frames, so a sync POS can schedule its async apply. MISSIONEND generation semantics stay as they are.
   - Tests: HELLO on each port; switching ports replaces the session; a busy debug port leaves the normal one working; `queue_async` ordering against `ASYNC` frames.
2. **`tfar_ts3_host`:**
   - Export `ts3plugin_onTalkStatusChangeEvent(serverConnectionHandlerID, status, isReceivedWhisper, clientID)`, using the API 26 signature from `sdk_headers/`, and add the trait method `on_talk_status_change` with a no-op default.
   - `system_set_3d_listener_attributes` takes `Option` forward/up vectors and passes NULL for `None`, as the original does.
   - Tests through the fake table.
3. **`tfar_ts3_mock`:** a driver `talk_status_change(client, talking)`; the fake for `systemset3DListenerAttributes` accepts NULL vectors; tests.
4. **`tfar_web_requests`:** `spawn_update_check` and `spawn_track` return a `BackgroundRequest` handle.
   - The handle offers `cancel()` and `join_timeout(Duration) -> bool`, and dropping it cancels without blocking.
   - Use a shared cancel flag checked before and after the request, plus ureq's per-request timeout, so a plugin shutdown can join within 2 s.
   - Keep the old function names, or deprecate them with `#[deprecated]` and migrate the docs.
   - Tests with the in-test responder: cancel before start; cancel during a slow response; the join timeout.
5. **`tfar_playback_mixer`:** a non-blocking stream append for audio threads.
   - `PlaybackMixer::stream_producer(name) -> StreamProducer` (`rtrb`-backed, bounded), whose `push(&[i16]) -> usize` never blocks or allocates. The mixer drains it in `mix_into`.
   - The old `append_stream` stays for non-audio callers, documented as such.
   - Tests: no blocking (a contended-lock test proves push succeeds while another thread holds the mixer); bounded drop of the oldest; drain order.

## Owned files
`crates/game_link/tfar_game_link_server/**`, `crates/teamspeak/tfar_ts3_host/**`, `tools/tfar_ts3_mock/**`,
`crates/services/tfar_web_requests/**`, `crates/audio/tfar_playback_mixer/**`. One root-manifest Edit at most (re-read first), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/l4-*.log`)
fmt, clippy `--all-targets -D warnings` and test for each owned crate; `cargo check -p tfar_plugin_0_9 -p tfar_plugin_1_0`
(the apps must still build); `cargo xtask verify all` (report foreign violations only). If the permission classifier refuses a
perturbation edit, report it as not run.

## Report additions
New public API per crate, with the migration notes the apps need (old call → new call).

Budget: L (350k tokens). Stop there and report what is done and what is not.
