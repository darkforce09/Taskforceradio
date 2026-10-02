**Status:** live

# Shared brief for every program agent

Read this first. Then read your own prompt file in this folder.

## Rules
1. **No git writes.** Never run `git add`, `git commit`, `git stash`, `git reset` or `git checkout`. The
   orchestrator commits.
2. **Owned files only.** Create or edit only the paths your prompt lists as owned. If you need a change
   elsewhere, put it in your report as a finding.
3. **`reference/` is read-only.** Never write, format, move or touch anything under `reference/`. It holds
   the two original mods byte for byte. Read it freely.
4. **Compile at every save point.** After each crate or module, run the fast check for what you touched.
5. **Tests are never weakened.** Don't delete, ignore or loosen a test to get green.
6. **Every new check gets a perturbation proof.** Break the thing the check guards, show it fails, restore.
7. **One command per shell call.** Each build or test command runs alone, with output to
   `target/agent-logs/<agent id>-<step>.log`. No chained gate scripts. No command over 60 s without a reason.
8. **Formatting:** run `cargo fmt` on your own crates; the gate uses `cargo fmt --all --check`.
9. **No silent deferrals.** Do the whole slice. If something truly can't be done, say exactly what and why.
10. **Names need zero context.** Folder, file, module and symbol names are self-describing, snake_case.

## Build directory rule
- **Never build into the default `target/`.** Prefix every cargo command, including `cargo xtask`, with
  `CARGO_TARGET_DIR=/run/media/system/Disk_2/Projects/Taskforceradio/target-<your agent id>` and
  `CARGO_BUILD_JOBS=4`.
- The reason: a host hook runs `target/debug/xtask ai guard` before every tool call, and our xtask has no such
  command. If the binary exists there, every Bash and Read call is blocked.
- `target-*` folders are gitignored. Logs still go to `target/agent-logs/`.
- Never stub, rename or delete files to get past a hook. Report a block as a finding.
- **No live tests.** There are no Arma, Proton or TeamSpeak runs, and nothing is copied into those folders.

## Environment facts (verified 2026-10-02)
- Repo root: `/run/media/system/Disk_2/Projects/Taskforceradio` (git, branch `main`, no commits yet).
- Toolchain 1.99.0 is pinned in `rust-toolchain.toml` with targets `x86_64-unknown-linux-gnu` and
  `x86_64-pc-windows-gnu`. `x86_64-w64-mingw32-gcc` is installed. `cargo deny` 0.20.2 and `typos` 1.50.3 are
  installed.
- `.cargo/config.toml` already sets the `xtask` alias and the windows-gnu linker plus `+crt-static`.
- Arma 3 runs under Proton; TeamSpeak 3.6.2 is the Flatpak `com.teamspeak.TeamSpeak3` (plugins dir
  `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/`). Nothing in M0 touches either.

## Authoritative sources
- Approved plan: `/home/Samuel/.claude/plans/i-want-to-edit-wobbly-squirrel.md` (decisions D1–D22, tree,
  conventions, roadmap, bug catalogue, game link spec).
- Behaviour specs: `/documentation/program/behaviour_specs/` (0.9 plugin, 1.0 plugin, extensions, Windows
  dependencies).
- Research: `/documentation/program/research/conventions_survey.md`, `library_survey.md`.
- Convention originals to adapt (read, never copy blindly; replace every TBD-specific term):
  - `/run/media/system/Disk_2/Projects/TBD-Reforger/CLAUDE.md`, its `documentation/standards/` and
    documentation/restructure/ and documentation/runbooks/sub_agent_orchestration.md
  - `/run/media/system/Disk_2/Projects/TBD-subtitles/CLAUDE.md`, its `documentation/standards/` (incl.
    `templates/`), `documentation/roadmap.md`, `documentation/runbooks/continue_in_claude_code.md`,
    tools/repo_gates/
  - Zed and rerun raw files in
    `/tmp/claude-1000/-run-media-system-Disk-2-Projects-Taskforceradio/19487e80-757a-431f-981a-ae1156f78b05/scratchpad/{zed,rerun}/`

## Crate list and dependency edges (authoritative for M0)
Tier = 0 with no workspace dependency, else 1 + highest dependency tier.

| Path | Kind | Workspace deps | Tier |
|---|---|---|---|
| crates/foundation/tfar_geometry | lib | — | 0 |
| crates/foundation/tfar_arma_values | lib | tfar_geometry | 1 |
| crates/audio/tfar_sample_buffer | lib | — | 0 |
| crates/audio/tfar_dsp_filters | lib | tfar_sample_buffer | 1 |
| crates/audio/tfar_dynamics | lib | tfar_sample_buffer | 1 |
| crates/audio/tfar_hrtf | lib | tfar_geometry, tfar_sample_buffer | 1 |
| crates/audio/tfar_spatial_audio | lib | tfar_geometry, tfar_sample_buffer | 1 |
| crates/audio/tfar_playback_mixer | lib | tfar_sample_buffer | 1 |
| crates/teamspeak/tfar_ts3_host | lib | — | 0 |
| crates/teamspeak/tfar_ts3_settings_db | lib | — | 0 |
| crates/game_link/tfar_game_link_protocol | lib | — | 0 |
| crates/game_link/tfar_game_link_server | lib | tfar_game_link_protocol | 1 |
| crates/game_link/tfar_game_link_client | lib | tfar_game_link_protocol | 1 |
| crates/game_link/tfar_arma_extension_abi | lib | — | 0 |
| crates/services/tfar_web_requests | lib | — | 0 |
| crates/interface/tfar_debug_window | lib | — | 0 |
| crates/version_0_9/tfar_game_commands_0_9 | lib | tfar_arma_values, tfar_geometry | 2 |
| crates/version_0_9/tfar_plugin_commands_0_9 | lib | tfar_arma_values | 2 |
| crates/version_0_9/tfar_radio_effects_0_9 | lib | tfar_arma_values, tfar_dsp_filters, tfar_sample_buffer | 2 |
| crates/version_0_9/tfar_client_state_0_9 | lib | tfar_geometry, tfar_game_commands_0_9, tfar_radio_effects_0_9, tfar_dsp_filters, tfar_dynamics, tfar_hrtf | 3 |
| crates/version_0_9/tfar_radio_routing_0_9 | lib | tfar_client_state_0_9, tfar_geometry | 4 |
| crates/version_0_9/tfar_voice_pipeline_0_9 | lib | tfar_client_state_0_9, tfar_radio_routing_0_9, tfar_radio_effects_0_9, tfar_dsp_filters, tfar_dynamics, tfar_hrtf, tfar_spatial_audio, tfar_sample_buffer, tfar_playback_mixer | 5 |
| crates/version_1_0/tfar_plugin_settings_1_0 | lib | — | 0 |
| crates/version_1_0/tfar_antennas_1_0 | lib | tfar_geometry, tfar_arma_values | 2 |
| crates/version_1_0/tfar_game_commands_1_0 | lib | tfar_arma_values, tfar_geometry, tfar_plugin_settings_1_0 | 2 |
| crates/version_1_0/tfar_plugin_commands_1_0 | lib | tfar_arma_values | 2 |
| crates/version_1_0/tfar_radio_effects_1_0 | lib | tfar_arma_values, tfar_dsp_filters, tfar_sample_buffer | 2 |
| crates/version_1_0/tfar_client_state_1_0 | lib | tfar_geometry, tfar_game_commands_1_0, tfar_radio_effects_1_0, tfar_plugin_settings_1_0, tfar_dsp_filters, tfar_dynamics, tfar_hrtf | 3 |
| crates/version_1_0/tfar_radio_routing_1_0 | lib | tfar_client_state_1_0, tfar_antennas_1_0, tfar_plugin_settings_1_0, tfar_geometry | 4 |
| crates/version_1_0/tfar_voice_pipeline_1_0 | lib | tfar_client_state_1_0, tfar_radio_routing_1_0, tfar_radio_effects_1_0, tfar_plugin_settings_1_0, tfar_dsp_filters, tfar_dynamics, tfar_hrtf, tfar_spatial_audio, tfar_sample_buffer, tfar_playback_mixer | 5 |
| apps/tfar_plugin_0_9 | cdylib + rlib | tfar_ts3_host, tfar_ts3_settings_db, tfar_game_link_server, tfar_web_requests, tfar_voice_pipeline_0_9, tfar_radio_routing_0_9, tfar_client_state_0_9, tfar_game_commands_0_9, tfar_plugin_commands_0_9, tfar_playback_mixer, tfar_geometry, tfar_arma_values | 6 |
| apps/tfar_plugin_1_0 | cdylib + rlib | same as 0.9 with the `_1_0` crates, plus tfar_plugin_settings_1_0, tfar_antennas_1_0, tfar_debug_window | 6 |
| apps/tfar_extension_0_9 | cdylib + rlib | tfar_arma_extension_abi, tfar_game_link_client | 2 |
| apps/tfar_extension_1_0 | cdylib + rlib | tfar_arma_extension_abi, tfar_game_link_client | 2 |
| tools/xtask | bin | — | 0 |
| tools/tfar_ts3_mock | lib + bin | tfar_ts3_host, tfar_game_link_protocol, tfar_game_link_client | 2 |
| tools/tfar_extension_probe | bin | — | 0 |
| tools/tfar_golden_vectors | lib | — | 0 |

The table matches `cargo metadata` of 2026-10-03 (normal and build dependencies). The workspace
dev-dependencies, for tests only: `tfar_plugin_0_9` and `tfar_plugin_1_0` → `tfar_ts3_mock`,
`tfar_game_link_client`; `tfar_extension_0_9` → `tfar_game_link_server`, `tfar_game_link_protocol`;
`tfar_extension_1_0` → `tfar_game_link_server`; `tfar_golden_vectors` → `tfar_dsp_filters`,
`tfar_dynamics`, `tfar_spatial_audio`, `tfar_sample_buffer`, `tfar_geometry`.

The `[package.metadata.layout] category` is the parent path (`crates/audio`, `apps`, `tools`). `targets` is
`"any"`, except `"windows"` for `tfar_extension_probe` (built for windows-gnu only).

## Category edge matrix and firewalls
See the approved plan §4 ("Category edge matrix", "External-crate firewalls", "Unsafe"). Summary:
- Foundation depends on nothing.
- Audio, teamspeak and game_link depend on foundation and their own category.
- Services and interface depend on foundation.
- version_0_9 and version_1_0 may use foundation, audio, teamspeak, `tfar_game_link_protocol`, services and
  interface, and **never each other**.
- Plugin apps may use anything except the other version's crates; extension apps only foundation and
  game_link.
- Tools may use crates/*, never apps.
- Dev-dependencies are exempt from tier order but never point at apps.

## Report format (≤ 250 words, these numbered sections)
1. Files created or changed.
2. Commands run, exit codes, log paths.
3. Tests and gates and their results.
4. Perturbation proofs.
5. Findings (anything outside your owned files, contradictions, risks).
6. Not done or deviations, with reasons.
