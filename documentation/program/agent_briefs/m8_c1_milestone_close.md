**Status:** live

# M8 · C1 · Full gate run and milestone close

You are agent C1, the closer for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first
(**Build directory rule**: `CARGO_TARGET_DIR=.../target-c1`), then `CLAUDE.md` §4 and `/documentation/program/laws_and_gates.md`.

## Work
1. **Run `cargo xtask ci ci-local` with your target dir.** If a step fails:
   - fix it, when the fix is small and mechanical: formatting, a README Contents entry, a missing module header, a typo, a lint;
   - otherwise report it with path:line and the failing command.

   Repeat until every step holds. Never weaken a test or a gate.
2. **Run `cargo xtask dist`.** Confirm, under `target-c1/dist/`, per version folder:
   - the plugin `.so`: `nm -D --defined-only` shows the `ts3plugin_*` exports;
   - `task_force_radio_pipe_x64.dll`: `x86_64-w64-mingw32-objdump -p` shows `RVExtension` and `RVExtensionVersion` exported, and imports only system DLLs (no libgcc or libwinpthread DLL);
   - `radio-sounds/` with 18 files;
   - `tfar_hrir/` with the sphere and its credits.

   List each with its size.
3. **`/documentation/roadmap.md`:** tick every code-complete item of M0–M8. Live-acceptance lines stay open with "live check deferred by the owner". Close the open questions D23–D26 settled.
4. **`/documentation/runbooks/manual_install_and_test.md`:** complete it against the real dist layout, with numbered steps, each with an Expected line:
   - copy the plugin `.so` + `radio-sounds/` + `tfar_hrir/` into `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`;
   - remove the stale `task_force_radio_win*.dll` / `TFAR_win*.dll` there by hand;
   - copy the DLL to the matching mod root (`reference/` is the archive; the owner's installed mod folder is a placeholder path);
   - BattlEye off;
   - check the TeamSpeak plugin list;
   - the probe under Proton's wine (`~/.local/share/Steam/steamapps/common/Proton 10.0/files/bin/wine` with a scratch `WINEPREFIX`): `tfar_extension_probe.exe <dll>` with TeamSpeak running gives `PONG`;
   - in Arma (BE off), `"task_force_radio_pipe" callExtension "TS_INFO\tPING"` gives `PONG`.
5. **`/documentation/program/bug_fixes.md`:** every entry has a test name. List any that doesn't.
6. **`/documentation/program/progress.md`:** the first fully green `ci-local` commit placeholder (the orchestrator fills the hash); the milestones state; the Handoff (only the owner's live checks remain).

## Owned files
Small mechanical fixes anywhere the gates point (report each), plus `/documentation/roadmap.md`, the runbook, `bug_fixes.md` and `progress.md`. Nothing under `reference/`.

## Report additions
The ci-local step table (held/failed/not run, with logs); the dist artifact table; every fix you made, as path:what.

Budget: M (250k tokens). Stop there and report what is done and what is not.
