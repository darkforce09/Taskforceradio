**Status:** live

# M8 · R3 · Records pass after the plugin apps

You are agent R3, the records keeper for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md`
first (including the **Build directory rule**, `CARGO_TARGET_DIR=.../target-r3`). This is a **docs-only** slice.
Read `git log --oneline 1998d0f..HEAD` and the crate READMEs of what landed since.

## Work
1. **`/documentation/program/bug_fixes.md`.** Add the entries below, each with reference (file:line in `reference/`), original behaviour, fixed behaviour, owning crate, milestone and test (find each test with grep):
   - F-032 (uninitialised 0.9 fields) and F-033 (stale lookup after a rename): `tfar_client_state_0_9`.
   - F-041 (NetID `strchr`), F-042 (receive counter drift), F-043 (RELEASE_ALL_TANGENTS keeps the reception), F-044 (a tangent press resets the extrapolation clock), F-045 (static object-interception strength), F-046 (cutoff ≤ 0 Hz and the u8 wrap), F-047 (`rstflt` forgets airborne effects): the `tfar_antennas_1_0` / `tfar_client_state_1_0` crates.
   - F-051 (a zero reach divides by zero): `tfar_voice_pipeline_0_9`.
   - F-061 (look-back before fill), F-062 (speaker muffling skipped in the same vehicle, `plugin.cpp:690`), F-063 (the intercom never ducks): `tfar_voice_pipeline_1_0`.
   - F-071 (press-release-press inside the PTT delay sends a release while held): `tfar_plugin_0_9`.
   - F-081 (`tangentPressed` not cleared after RELEASE_ALL_TANGENTS), F-082 (the original nickname re-saved on every rename, `Teamspeak.cpp:251`): `tfar_plugin_1_0`.

   Fill the Test column of every entry; "pending" remains only where a test truly does not exist, and those go in the report.
2. **Edge tables.** Generate the true dependency edges from `cargo metadata --no-deps --format-version 1` (run with your target dir). Rewrite the edge tables in `/documentation/program/crate_catalogue.md` and `/documentation/program/agent_briefs/shared_brief.md` to match, including tiers.
3. **`/documentation/architecture/audio_pipeline.md`:**
   - the 1.0 attenuation is `10^(-1.5·d/maxD)` (the original's `10^((d/(2·maxD))·-60/20)`);
   - the +4 dB `HRTF_MAKEUP_GAIN_DB` is decided (D23);
   - the F-063 ducking;
   - the HRTF source reset when a sender starts talking;
   - speaker radios get HRTF without the makeup gain.
4. **`/documentation/decisions/`.** Entries dated 2026-10-03:
   - **D23:** keep the +4 dB makeup gain.
   - **D24:** each plugin listens on both its normal and its `-tfdebug` port. This replaces the `TFAR_GAME_LINK_DEBUG` env var.
   - **D25:** a plugin returns to dormant after its game-end handling.
   - **D26:** HRTF memory (about 2 MB per source) is accepted, with no pooling.

   Put them in the matching subject files (audio.md, game_link.md).
5. **`/documentation/architecture/game_link_protocol.md`.** Document D24 (both ports bound, one session slot, newest valid HELLO wins) and D25 (dormant again after game end). Remove `TFAR_GAME_LINK_DEBUG`, but keep the test-only `TFAR_GAME_LINK_PORT`.
6. **`/documentation/program/progress.md`:**
   - close OF-011 (D25), OF-022 (D23), OF-023 (D26), OF-026 (done in CLAUDE.md);
   - OF-021 stays a NOTE: check the IRCAM terms before any redistribution;
   - record the commits V3 `4ffce7d`, W3 `5dfd3f5`, V4 `4018568`, W4 `f8e9d5f`, V5 `a742c88`, W6 `fd9593a`, handoff `6bad8f2`;
   - add the completion plan (waves 1–3: R3, L4, P5, then V6 ∥ W7, then C1) as the current milestone section;
   - update the Handoff.
7. **`/CLAUDE.md`:** update the "Current state" paragraph only: all code is written, and the completion waves are under way.
8. **`/documentation/program/agent_briefs/README.md`:** Contents lists every brief, including m8_*.

## Do not touch
Any code; any crate other than READMEs, and none at all of those that L4 or P5 own. Those are `tfar_game_link_server`, `tfar_ts3_host`, `tfar_ts3_mock`, `tfar_web_requests`, `tfar_playback_mixer`, both routing crates, both client-state crates and both voice pipelines.

## Checks (each alone, with your target dir; logs `target/agent-logs/r3-*.log`)
`typos`; `cargo xtask verify status-lines`, `link-check`, `markdown-placement`, `readme-coverage` and `readme-sections`. Report foreign violations only.

## Report additions
The F-id count with tests vs pending; the list of edges that changed.

Budget: M (250k tokens). Stop there and report what is done and what is not.
