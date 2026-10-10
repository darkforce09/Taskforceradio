**Status:** live

# R1 · Records and tooling pass after M2–M5 slices

You are agent R1, the records-and-tooling keeper for the TFAR Rust port. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**,
`CARGO_TARGET_DIR=.../target-r1`). Read `git log --oneline` from `2396c91` onward to see what landed.

## Tooling fixes (tools/xtask)
1. **`dist` and the target dir.** `dist` hard-codes `target/` and passes it as `--target-dir`. Make it honour `CARGO_TARGET_DIR`, falling back to `target/`, and write `dist/` under the effective target dir. Update its README and tests. Then run `cargo xtask dist` with your target dir and confirm the artifacts.
2. **`dist` and the HRIR sphere.** `dist` must also copy `assets/hrir/IRC_1002_C.bin` and `assets/hrir/CREDITS.md` into each version folder, under a subfolder both plugins will load from: `tfar_hrir/`, beside `radio-sounds/` in the TS plugins dir. Record the path as a constant the plugins will use: `tfar_hrtf` exposes nothing yet, so put the published relative path in the dist README and in `/documentation/architecture/audio_pipeline.md`. Add a test.

## Records (documentation)
3. **`/documentation/program/bug_fixes.md`:**
   - Add these entries, with reference, original, fixed, crate, milestone and test:
     - **F-027:** 0.9 VERSION with an empty password; `reference/tfar_0_9_12/ts/src/plugin.cpp:1789`; tests `version_with_an_empty_password_parses_although_getline_drops_it`, `version_with_three_fields_has_an_empty_password`.
     - **F-028:** 0.9 TANGENT with an empty subtype; `plugin.cpp:1812`; test `a_tangent_with_an_empty_subtype_parses_as_an_unknown_subtype`.
     - **F-029:** 1.0 SPEAKERS stops at the first short record; `reference/tfar_1_0/ts/src/CommandProcessor.cpp:510`; test `a_short_speaker_record_is_skipped_and_the_rest_kept`.
     - **F-030:** the 1.0 Piwik custom-variable parser loops forever on text ending inside a variable; `reference/tfar_1_0/ts/src/task_force_radio.cpp` (cite the loop line); test `text_that_ends_inside_a_variable_voids_the_array`.
   - Fill the Test column for the done fixes, finding each test name with grep in the crates. Known: F-001 (tfar_hrtf), F-003, F-004, F-005, F-006, F-007, F-009, F-010, F-014, F-018, F-020, F-021, F-026.
   - Reword F-012: wrapping doesn't change `cos`, so the fix is code hygiene with no audible effect. Say so.
4. **`/documentation/decisions/workspace.md`:** add the library-map lines for `url`, `winit`, `fastrand` (and `libc` if missing), each with its single allowed crate. Make sure `tools/xtask/src/layout.rs` firewalls match. Edit `layout.rs` only for the firewall table, and run `cargo xtask verify dependency-firewalls`.
5. **`/documentation/program/crate_catalogue.md` and `/documentation/program/agent_briefs/shared_brief.md`:** the edge `tfar_ts3_mock` → `tfar_ts3_host`, and any other edge that differs from today's manifests. Generate the truth from `cargo metadata` and update both tables.
6. **`/documentation/architecture/game_link_protocol.md`:**
   - document the test-only `TFAR_GAME_LINK_PORT` override and the plugin's `TFAR_GAME_LINK_DEBUG=1` debug-port choice, which closes that roadmap open question as the default, pending the owner;
   - document the 1.0 rule that a session closed mid-call → `Not connected to TeamSpeak`.
7. **`/documentation/architecture/audio_pipeline.md`:** `hrtf` processes in f32, everything else in f64. The HRTF block is 120 frames × 4 steps, adding 10 ms. The sphere's diffuse-field EQ makes sources quieter (~−4 dB front, −9 dB rear); record an open question for a makeup gain. `HrtfSource` memory is about 2 MB each. The ILD coefficients are 0.375 (0.9) and 21.5° in radians = 0.37525 (1.0). The 1.0 X3DAudio listener/emitter swap and the front x/y flip must be honoured by `tfar_voice_pipeline_1_0` when it calls the cone. `volumeAttenuation` (1.0) and `volumeFromDistance` (0.9) are owned by the voice-pipeline crates.
8. **`/documentation/program/progress.md` and `/documentation/roadmap.md`:**
   - Mark these done with their commits:
     - H2/H3 `fb6621d`;
     - L3 `52bda9e` (the plugin stubs' metadata exports count as M2 H4);
     - P1 `7901de3`, P3 `2a74b9b`, P4 `2049b4a`;
     - S1 `420d37e`;
     - V1 `f534c75` + `61f66f6`, W1 `654db3b`;
     - the deny/typos config `8c87196`.
   - Record G1 and E1 as running, and V3/W3 as next.
   - Tick the roadmap items that are now code-complete. The owner said **no live tests for now**, so the live acceptance lines stay open with the note "live check deferred by the owner".
   - Add every finding from the reports, with the triage FIX, NOTE or CLOSE:
     - the brief-vs-C++ disagreements V1/W1 found;
     - `split_keeping_every_field` being crate-private (1.0 needs it; W1 used `str::split`);
     - the IRCAM licence question;
     - the makeup-gain question;
     - the HRTF memory;
     - the mock's re-entrant fake-state lock.
9. **`agent_briefs/README.md`:** Contents lists every brief, including this one.

## Do not touch
`crates/version_*/tfar_radio_effects_*` (E1), `tools/tfar_golden_vectors` (G1). Don't change other code beyond `tools/xtask`.

## Checks (each alone, with your target dir; logs `target/agent-logs/r1-*.log`)
xtask fmt, clippy, test; `cargo xtask verify all`, where only E1/G1 files may still fail; `cargo xtask dist`.

## Report additions
Dist artifact list with paths; the gate table.

Budget: M (250k tokens). Stop there and report what is done and what is not.
