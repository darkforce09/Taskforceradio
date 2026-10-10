**Status:** live

# M8 · P5 · Audio-path library gaps

You are agent P5 for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first
(**Build directory rule**: `CARGO_TARGET_DIR=.../target-p5`), then `/documentation/standards/coding_standards.md` (the
audio-thread rules). The plugin apps worked around these gaps privately. Read
crates/version_0_9/tfar_plugin_0_9/src/remote_sounds.rs (removed since) and the 1.0 app's equivalent, plus its local-tangent code, but don't edit the
apps; wave 2 switches them.

**Keep every existing public API working.** Add beside the old APIs, so the apps still build.

## Work
1. **`tfar_radio_routing_0_9` and `tfar_radio_routing_1_0`.** Add `is_over_radio_into(..., out: &mut Vec<HeardVia>)`, which clears and refills `out`, reusing its capacity.
   - Per-frame `String` clones go away: `HeardVia` ids become `Arc<str>` shared with the client state, or borrowed. Choose the least invasive change that removes per-frame heap allocation, and document it.
   - The old `is_over_radio` becomes a wrapper.
   - Tests: identical results to the old function on the existing case tables; a capacity-stability test over 1,000 frames, asserting no reallocation once warm.
2. **`tfar_client_state_0_9`:** a client entry created without any POS starts at the invalid data frame (9999), not frame 0, so it doesn't count as current. This is the V5 finding; propose it as F-091, with a test.
3. **`tfar_client_state_1_0`:** move W6's local-tangent state change into `SessionState::apply_local_tangent(...)`, with the press/release/changed semantics of spec §8 "PTT and VAD" and the F-081 fix (`tangentPressed` cleared by RELEASE_ALL_TANGENTS). It returns the owned decisions the app acts on. Port W6's tests.
4. **`tfar_voice_pipeline_0_9` and `tfar_voice_pipeline_1_0`:**
   - Switch the audio path to `is_over_radio_into` with a scratch `Vec` owned by `VoiceScratch`, and extend the capacity test.
   - Add `render_remote_tangent_sound(wave: &[i16], plan: &RemoteSoundPlan, hrir: Option<&HrirLibrary>) -> Result<Playback>` per version: the positional remote start/end sound (gain, vehicle low-pass, ground speaker band-pass, underwater low-pass, HRTF or ILD placement), moved from the apps' private versions with their tests. It runs off the audio thread.

## Owned files
`crates/version_0_9/tfar_radio_routing_0_9/**`, `crates/version_1_0/tfar_radio_routing_1_0/**`, `crates/version_0_9/tfar_client_state_0_9/**`,
`crates/version_1_0/tfar_client_state_1_0/**`, `crates/version_0_9/tfar_voice_pipeline_0_9/**`, `crates/version_1_0/tfar_voice_pipeline_1_0/**`.
One root-manifest Edit at most (re-read first), reported. New edges only if the tier and category rules allow; run `cargo xtask verify crate-tiers`.

## Checks (each alone, with your target dir; logs `target/agent-logs/p5-*.log`)
fmt, clippy `--all-targets -D warnings` and test for each owned crate; `cargo check -p tfar_plugin_0_9 -p tfar_plugin_1_0`;
`cargo xtask verify all` (report foreign violations only). If the permission classifier refuses a perturbation edit,
report it as not run.

## Report additions
New public API with migration notes for the apps; F-091 and its test name.

Budget: L (350k tokens). Stop there and report what is done and what is not.
