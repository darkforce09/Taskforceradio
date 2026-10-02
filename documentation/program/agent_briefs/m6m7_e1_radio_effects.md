**Status:** live

# M6/M7 · E1 · `tfar_radio_effects_0_9` and `tfar_radio_effects_1_0`

You are agent E1, the radio-effects author for the TFAR Rust port. You own both version crates. They never depend on
each other (law 5), so any duplication between them is deliberate. Read
`/documentation/program/agent_briefs/shared_brief.md` first (including the **Build directory rule**), then
`/documentation/standards/coding_standards.md`, `/documentation/architecture/audio_pipeline.md`, the behaviour specs §3
"Radio effects" / "RadioEffect.h", and `/documentation/program/bug_fixes.md` (F-011 for 1.0, F-019 for 0.9). Read the
originals: `reference/tfar_0_9_12/ts/src/RadioEffect.h` and `reference/tfar_1_0/ts/src/RadioEffect.hpp`. Use
`tfar_dsp_filters` (RBJ, Butterworth band-pass) and `tfar_sample_buffer`.

## Per crate
- **A `RadioEffect` type per effect**, each with `set_error_level(f64)` and `process(&mut [f32])` over a mono block:
  - 0.9: `LongRange`, `Personal`, `UnderWater`. Airborne uses LongRange in 0.9.
  - 1.0: `LongRange`, `Personal`, `Airborne`, `UnderWater`.
- **The processing order of the version, exactly:**
  - 0.9: delay → ring mod → foldback → RBJ HP → RBJ LP → ×30. Personal runs its mic HP 900/0.85 and LP 3000/2.0 before computing the mean; then speaker HP 520/0.97 and LP 4300/2.0.
  - 1.0: foldback → ×30 → delay → ring mod → HP → LP, with the filter table of the 1.0 spec.
- **Shared details:**
  - the 2400-sample delay line with a 2399-sample effective delay;
  - the ring-mod phase stepping by 90/48000 and resetting above 1;
  - foldback with its threshold rule;
  - the mean-abs `x` term;
  - UnderWater: random sample dropouts, then Butterworth BP 2 1000/400, ×30.
- **Random numbers:** use the `fastrand` crate with a per-effect seeded generator (add it to `[workspace.dependencies]` with one Edit, re-reading first), so tests are deterministic. The original `rand()` was global and unseeded. Document it.
- **`error_level_from_distance` (`calcErrorLevel`) with the fixes:**
  - 1.0 (F-011): the slope multiplied by 10 over each 0.1 segment.
  - 0.9 (F-019): the sign fixed to `(to − from)`, the index clamped, and the 0.9 table kept (last entry .999).
- **Per-version `process_radio_effect(effect, buffer: &mut StereoBuffer, gain, stereo_mode)`:**
  - 0.9 (`reference/tfar_0_9_12/ts/src/plugin.cpp:2476`): mono mixdown, `/32767 · gain`, effect, clip, `·32766`; stereo mode 1 = left only, 2 = right only (gain ×1.5), the other channel zeroed.
  - 1.0 (`RadioEffect.hpp:230`): not stereo → gain ×1.5; mono = sum·gain/(channels·32766); effect; `clamp·32766` to all or one channel.
- **Effects are created off the audio thread.** `process` allocates nothing.

## Tests (per crate)
- Silence in gives silence out.
- `error_level_from_distance` hits the table points, is monotone after the fix, is continuous across segments, and is clamped.
- Error 0 vs error 1 gives a measurably different output (spectral flatness, or energy in the ring-mod band).
- The delay is exactly 2399 samples, measured on an impulse with error 0.
- Stereo modes route to the right channel(s) with the right gain.
- UnderWater is deterministic for a seed.
- The filter band of each effect is measured: the attenuation at 100 Hz and at 8 kHz versus the band centre.
- Pin F-011 and F-019 with named tests that show the original's wrong value next to ours, computed inline from the original formula.

## Owned files
`crates/version_0_9/tfar_radio_effects_0_9/**`, `crates/version_1_0/tfar_radio_effects_1_0/**`. One root-manifest Edit (`fastrand`), reported.

## Checks (each alone, with your target dir; logs `target/agent-logs/e1-*.log`)
fmt, clippy `--all-targets -D warnings`, test for both crates; `cargo xtask verify all` (report foreign violations only).

## Report additions
Public API per crate; a table of the two versions' differences as implemented; the F-ids pinned, by test name.

Budget: L (350k tokens). Stop there and report what is done and what is not.
