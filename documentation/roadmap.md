**Status:** live

# Roadmap

The milestones that take the TFAR Rust port from an empty workspace to two installable mods (0.9.12
and 1.0), each with a checklist and an acceptance test. Rules:
- Tick items in the same commit as the work.
- A milestone is done when its acceptance test passes. Nothing moves to a later milestone without the
  owner saying so.
- Only the owner moves items between milestones.
- Every milestone close runs the full `cargo xtask ci ci-local`.

The program records that drive each milestone (agent rosters, budgets, gates, progress) live in the
[program folder](/documentation/program/README.md).

## M0 — Workspace skeleton, reference archive, gates

The repository, the read-only reference trees, the conventions, every crate as a compiling skeleton
and the gate tooling, so every later milestone starts from a green workspace.

- [x] git init.
- [x] Move the trees to `reference/`, write the reference manifest (MANIFEST.sha256), add the
      reference README.
- [x] Root config, CLAUDE.md and AGENTS.md, the whole documentation tree.
- [x] All 38 crates as compiling skeletons.
- [x] `xtask` with `verify`, `ci`, `new-crate`, `reference-manifest`, and `dist` (skeleton).

**Acceptance:** `cargo xtask ci ci-local` passes; both extension stubs cross-compile to Windows
DLLs; both plugin stubs build as `.so` files; `cargo xtask verify reference-untouched` holds.

## M1 — Foundations

The tier-0 and tier-1 building blocks every other crate stands on.

- [x] `tfar_geometry`: glam-based vectors and Arma angles, with correct normalisation (F-009).
- [x] `tfar_arma_values`: getline-exact field split, Arma arrays, frequency lists, radio subtypes
      matched by name (F-010).
- [x] `tfar_sample_buffer`: interleaved i16 buffers with saturating gain (F-004) and a bounds-safe
      stereo view (F-006).
- [x] The tracing setup shared by all crates.

**Acceptance:** the unit tests pass.

## M2 — TeamSpeak host layer

A safe Rust layer over the TeamSpeak 3 plugin SDK, and a mock host that can load our plugins.

- [x] `ts3plugin-sys` 0.5 adopted. Its layout is verified against the official
      TeamSpeak plugin SDK (GitHub teamspeak/ts3client-pluginsdk) API 26 headers with `size_of`/`offset_of` tests.
- [x] `tfar_ts3_host`: safe wrapper, export macro, logging, config paths from TS (F-018).
- [x] `tfar_ts3_settings_db`.
- [x] `tfar_ts3_mock`.
- [x] The plugin stubs export their metadata.

**Acceptance:** the mock loads both stubs, and the owner sees both plugins load in the Flatpak
TeamSpeak. Status: the mock part holds (f2df927, 30678a8); live check deferred by the owner.

## M3 — Game link

The localhost TCP link that replaces the named pipe (0.9) and the shared memory (1.0), as specified
in the [game link protocol](/documentation/architecture/game_link_protocol.md).

- [x] `tfar_game_link_protocol`, `tfar_game_link_server` (FIFO async, F-007),
      `tfar_game_link_client`.
- [x] `tfar_arma_extension_abi`: the `RVExtension` export and the focus query.
- [x] Both extension apps with real transport.
- [x] `tfar_extension_probe`.

**Acceptance:** the probe under Proton's wine gets `PONG` from the stub plugins; in Arma under
Proton (BattlEye off), `"task_force_radio_pipe" callExtension "TS_INFO\tPING"` returns `PONG` for
both mods. Status: the probe and the extensions pass their tests against the stub plugins on
Linux (30678a8); live check deferred by the owner.

## M4 — Audio primitives

The DSP building blocks, proven against the original C/C++ libraries as golden oracles.

- [x] `tfar_dsp_filters` (sci-rs + biquad).
- [x] `tfar_dynamics` (feed-forward compressor).
- [x] `tfar_hrtf` (the `hrtf` crate, IRCAM sphere, block adapter; F-001).
- [x] `tfar_spatial_audio` (distance curve and cone, simple ILD for the 0.9 fallback; F-012).
- [x] `tfar_playback_mixer` (hound; F-005).
- [x] `tfar_golden_vectors` (oracles: DSPFilters, SimpleComp, F3DAudio curve and cone).

**Acceptance:** the golden tests hold within tolerance. Status: holds at e5eecb5 (the results are in
[progress](/documentation/program/progress.md#m4-audio-primitives)).

## M5 — Services and diagnostics

The features that talk to the outside world or show state: update check, Piwik and the debug window.

- [x] `tfar_web_requests` (update check with real version comparison (F-026), Piwik `TRACK`).
- [x] `tfar_debug_window`.

**Acceptance:** each one is exercised against a local test server or window. Status: the update
check and the Piwik requests pass against a local test server in their tests (243b5f1); the debug
window check is live, deferred by the owner.

## M6 — 0.9 track

The full 0.9.12 plugin and extension, built from the M1–M5 crates.

- [ ] The 6 `version_0_9` crates.
- [ ] The full `tfar_plugin_0_9` and `tfar_extension_0_9`.
- [ ] Mock scenario suites.

**Acceptance:** the mock scenarios pass, and the owner runs a live two-client session with the
0.9.12 mod: SW, LR, DD, speakers, serious mode.

## M7 — 1.0 track

The full 1.0 plugin and extension, built from the M1–M5 crates and never from the 0.9 ones.

- [ ] The 8 `version_1_0` crates.
- [ ] The full `tfar_plugin_1_0` and `tfar_extension_1_0`.
- [ ] Mock suites.

**Acceptance:** the same as M6, plus antennas, intercom, SETCFG/NEEDCFG and the focus-aware channel
move.

## M8 — Release readiness

The artifacts the owner installs by hand, the finished runbook, and every bug fix closed with a test.

- [ ] `cargo xtask dist` produces the final artifacts plus the `radio-sounds` and `tfar_hrir`
      folders.
- [ ] The [manual install runbook](/documentation/runbooks/manual_install_and_test.md) is complete.
- [ ] Every [bug fix](/documentation/program/bug_fixes.md) entry is closed with a test.

**Acceptance:** the owner installs from the dist folder under target by hand and plays both mods.

## Open questions

| Question | Settle by |
|---|---|
| Licence of the Rust port (D16: `UNLICENSED` until decided; upstream is APL-SA) | Before release (M8) |
| Are the `hrtf` crate's click artefacts on fast-moving sources acceptable, or does `tfar_hrtf` need its own crossfade? | M4 |
| Does 32-bit Arma matter? | Settled as no, unless the owner says otherwise |
| How the plugin picks the `-tfdebug` port | Settled on 2026-10-03: it listens on both ports ([D24](/documentation/decisions/game_link.md#2026-10-03--each-plugin-listens-on-both-its-normal-and-its--tfdebug-port-d24)) |
| Does direct speech get a makeup gain after the HRTF, whose diffuse-field EQ costs about 4 dB in front and 9 dB behind? | Settled on 2026-10-03: +4 dB on direct speech ([D23](/documentation/decisions/audio.md#2026-10-03--direct-speech-keeps-a-4-db-makeup-gain-after-the-hrtf-d23)) |
| Does about 2 MB of memory per HRTF source need a cap or a shared processor? | Settled on 2026-10-03: no ([D26](/documentation/decisions/audio.md#2026-10-03--hrtf-memory-of-about-2-mb-per-source-is-accepted-with-no-pooling-d26)) |
| May the IRCAM LISTEN sphere, published with no licence text, ship in the dist folders with its credit? | Before release (M8) |
