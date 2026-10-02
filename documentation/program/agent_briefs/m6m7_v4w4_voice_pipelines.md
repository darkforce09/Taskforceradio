**Status:** live

# M6/M7 · V4 + W4 · `tfar_voice_pipeline_0_9` and `tfar_voice_pipeline_1_0`

You are agent V4, the voice-pipeline author for the TFAR Rust port. You own both version crates, which never depend
on each other (law 5). Read `/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory
rule**), then:
- `/documentation/standards/coding_standards.md` (the audio-thread rules)
- `/documentation/architecture/audio_pipeline.md`
- **`/documentation/program/behaviour_specs/tfar_0_9_plugin.md` §3** and **`tfar_1_0_plugin.md` §3**
- `/documentation/program/bug_fixes.md`, F-013 and every F-id marked as a voice-pipeline crate
- the originals: `reference/tfar_0_9_12/ts/src/plugin.cpp:2653-2828` (the Stereo function) and `reference/tfar_1_0/ts/src/plugin.cpp:455-743` (`processVoiceData`)
- the public APIs of every crate you depend on (the manifests list them): client state, routing, radio effects, dsp filters, dynamics, hrtf, spatial audio, sample buffer and playback mixer

## What each crate provides
A **per-sender voice processor** that the plugin calls from the TS audio callbacks with an already-locked
`&mut ServerRadioData` / `&mut GameData` (whatever the version's client-state crate names it). It does exactly the
step list of its version's §3 "processVoiceData / Stereo function", with the library replacements and the fixes.

- **Distance gain:** the version's distance-gain function, owned here:
  - 0.9 `volume_from_distance` (P:698): `d^-0.3 · max(0, maxD − d)/maxD`;
  - 1.0 `volume_attenuation` (helpers.hpp:122): the 10^(−3·d/maxD) curve.
- **Direction:** `tfar_hrtf::HrtfSource`, keyed per sender and per radio id from the client state, replaces both the clunk ITD and the 1.0 X3DAudio direction. For 1.0, the X3DAudio *distance and cone* parts come from `tfar_spatial_audio`:
  - `DistanceCurve::tfar_voice(voice_volume, volume_attenuation)`;
  - `EmitterCone::TFAR_VOICE` when the `voice_cone` setting is on;
  - honour the original's swapped listener/emitter and the negated front x/y (`helpers.cpp:46-110`), so the cone measures the *sender's* facing towards me;
  - the simple ILD is not used for 1.0 direct speech, since HRTF replaces it.
- **Simple ILD on speaker radios:** the original used clunk + simple ILD. Use HRTF for the direction instead, and keep the ILD only where the original used ILD **without** clunk (1.0 playbacks). Document every choice in the crate README "How it works".
- **Makeup gain:** `HRTF_MAKEUP_GAIN_DB`, which compensates the IRCAM diffuse-field EQ (P3 measured about −4 dB ahead). Set it to +4 dB, as a named constant with a comment, so direct speech straight ahead keeps its original loudness. The roadmap open question stays for the owner.
- **Compressors:** 1.0 runs one for direct speech and one for radio, each once per frame (F-013). 0.9 has one compressor with one radio run, as the original.
- **Object interception (1.0):** the low-pass and the one-frame-delay buffer as specified.
- **Vehicles, underwater, speakers, phone, intercom:** every branch of the spec, with the effect-error mapping from the routing crate.
- **Output:** `globalVolume` gain at the end.
- **Captured voice (the my_radio_voice path):** `process_own_voice_for_speakers(...)` returns the stereo block the plugin appends to the playback-mixer stream.
- **No allocation on the audio path.** Scratch buffers are owned by a `VoiceScratch` struct created once per server.

## API shape (both crates)
- `VoiceProcessor::new(...)` builds the scratch.
- `process_sender(&mut self, state: &mut <ClientState>, sender: &str, samples: &mut StereoBuffer, now: Instant) -> VoiceOutcome { Processed, Silenced, PassThrough }`. PassThrough means a non-TFAR user is heard untouched.
- The plugin app does the TS-specific channel views through `tfar_sample_buffer::copy_to_stereo` / `write_back_from_stereo`.

## Tests (per crate)
- **Scenario tests** for every branch, using synthetic state:
  - a non-TFAR sender passes through;
  - a dead listener in serious mode is silenced;
  - in range vs out of range, with the gain matching the distance function;
  - the vehicle low-pass (attenuation at 8 kHz);
  - can't speak (100 Hz low-pass);
  - SW radio within range, with the effect applied (band-limited energy);
  - an LR radio with an antenna;
  - DD underwater;
  - a speaker on the ground at a distance;
  - phone;
  - intercom with ducking (1.0);
  - object interception (1.0);
  - the cone front vs back (1.0);
  - spectator rules (1.0).
- **HRTF direction:** a sender to the right gives right-ear-dominant output.
- **Allocation:** the processor's capacities stay unchanged across 1,000 frames.
- **F-013:** each compressor envelope advances once per frame.

## Owned files
`crates/version_0_9/tfar_voice_pipeline_0_9/**`, `crates/version_1_0/tfar_voice_pipeline_1_0/**`. If a needed edge is missing
(for example game_commands types), add it only if the tier and category rules allow, run `cargo xtask verify crate-tiers`, and
report it. Root manifest: one Edit at most, reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/v4-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API per crate; a branch → test table; every deliberate deviation from the original, with its reason.

Budget: L (350k tokens). Stop there and report what is done and what is not.
