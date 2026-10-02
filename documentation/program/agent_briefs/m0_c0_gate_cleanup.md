**Status:** live

# M0 · C0 · Gate cleanup and the M0 close records

You are agent C0, the M0 closer for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first
(including the **Build directory rule**).

## Goal
`cargo xtask verify all` holds on everything except the files other agents are actively changing (list below),
and the program records reflect M0, M1 and the done parts of M2 and M3.

## Fixes (each a violation A4 found)
1. **language-bans:** allow the TeamSpeak SDK headers. Add `crates/teamspeak/tfar_ts3_host/sdk_headers/` to the C/C++ allowed folders in `tools/xtask/src/layout.rs`, for `.h` files only, as reference headers that are never compiled. Mirror the rule in `/documentation/program/laws_and_gates.md` and `/documentation/standards/coding_standards.md`. Add a fixture test showing that a `.c` file there still fails and a `.h` file passes.
2. **dependency-firewalls:** `libloading` is allowed in `tools/tfar_extension_probe` as well as `tools/tfar_ts3_mock`. Update `layout.rs` and the laws doc.
3. **link-check:** fix the backticked paths that point at nothing:
   - `documentation/program/agent_briefs/m0_a4_xtask_gates.md:32`, `shared_brief.md:40`, `:43`;
   - `documentation/program/research/conventions_survey.md:36`, `:45`, `:59`, `:76`;
   - the sdk_headers README link (`crates/teamspeak/tfar_ts3_host/sdk_headers/README.md:46`).

   Paths in other repositories should be plain text or absolute paths outside backticks, whichever the link-check rules accept. The research file is a frozen record: change only the formatting, not the content.
4. **readme-coverage:** `documentation/program/agent_briefs/README.md` lists every brief in the folder (including yours and the new m2/m3/m4 ones). Also the `tfar_ts3_host` README Contents trees and module headers (`plugin_answers.rs:5`, `audio_exports_macro.rs:5`): fix them in `crates/teamspeak/tfar_ts3_host/` only for headers, READMEs and doc text, not code.
5. **decisions:** add `libc` (malloc/free for TeamSpeak-owned memory, `tfar_ts3_host` only) to the library map in `/documentation/decisions/workspace.md` and to the firewalls.
6. **Program records:**
   - `/documentation/program/progress.md`:
     - mark S0, A1–A4 and the M1 slices B1/B2 as done, with the commits `de41c9b`, `5b2b918`, `9a71a5a`, `530eb13`, `3046373`, `86b4891`, `45a584d`; L1/L2 (`37dc8e0`) and H1 (`2ff1886`) as done in M3/M2.
     - Amendments: A3 started in parallel with A1; M1–M3 slices ran early in parallel; L1/L2 ran before M2 completed; G1 moves after P1–P5 (it compares live oracle output against the crates, so no fixtures are passed between crates); the C0 cleanup agent was added.
     - Close OF-002 with the decision "dist names `task_force_radio_linux_amd64.so` / `TFAR_linux_amd64.so`".
     - Add a finding about the host `ai guard` hook and the target-dir rule, and record that A4 once replaced the built xtask binary with a stub to get past it. Note that the owner said no `ai guard` subcommand is needed.
     - The owner decisions of this session: no live tests for now; the TeamSpeak SDK headers stay (private repo); the HRIR download is approved.
     - Update Current state and Handoff.
   - `/documentation/roadmap.md`: tick the M0 and M1 items that hold; tick M2's `ts3plugin-sys`/`tfar_ts3_host` items; tick M3's protocol/server/client/ABI items.
7. **Close run:** `cargo xtask ci ci-local` with your target dir. Every step should hold except failures caused by files under active change by the agents listed below. List any remaining failures with path:line.

## Do not touch (other agents are changing these right now)
`apps/**`, `tools/tfar_ts3_mock/**`, `tools/tfar_extension_probe/**`, `crates/teamspeak/tfar_ts3_settings_db/**`,
`crates/audio/**` (except nothing), and the code under `crates/teamspeak/tfar_ts3_host/src` (docs and headers there are fine).

## Owned files
`tools/xtask/src/layout.rs` and its tests, `/documentation/**` (except the frozen behaviour specs; for the
frozen research record, formatting only), `crates/teamspeak/tfar_ts3_host/**/README.md`, the module-header
comments in `tfar_ts3_host` files.

## Report additions
The ci-local step table; the remaining violations, each with the owner agent.

Budget: M (250k tokens). Stop there and report what is done and what is not.
