**Status:** live

# Agent briefs

The prompts the orchestrator gives its Opus sub-agents: one shared brief every agent reads first, and
one brief per agent slice.

## Contents

```text
documentation/program/agent_briefs/
├── m0_a1_standards.md                    M0 A1: standards, templates, CLAUDE.md, decisions, glossary
├── m0_a2_program_records.md              M0 A2: program records, roadmap, architecture, runbooks
├── m0_a3_workspace_skeletons.md          M0 A3: root configuration and all 38 crate skeletons
├── m0_a4_xtask_gates.md                  M0 A4: the xtask tool, every gate, ci, dist, scaffolding
├── m0_c0_gate_cleanup.md                 M0 C0: gate cleanup and the M0 close records
├── m1_b1_geometry_arma_values.md         M1 B1: tfar_geometry and tfar_arma_values
├── m1_b2_sample_buffer.md                M1 B2: tfar_sample_buffer
├── m2_h1_ts3_host.md                     M2 H1: tfar_ts3_host, the TeamSpeak plugin layer
├── m2_h2_mock_settings_db.md             M2 H2 + H3: tfar_ts3_mock and tfar_ts3_settings_db
├── m3_l1_protocol_server.md              M3 L1: tfar_game_link_protocol and tfar_game_link_server
├── m3_l2_extension_abi_client.md         M3 L2: tfar_arma_extension_abi and tfar_game_link_client
├── m3_l3_apps_probe.md                   M3 L3: both extension apps, plugin stubs with the link, the probe
├── m4_p1_p2_filters_dynamics.md          M4 P1 + P2: tfar_dsp_filters and tfar_dynamics
├── m4_p3_hrtf.md                         M4 P3: tfar_hrtf with the IRCAM sphere
├── m4_g1_golden_vectors.md               M4 G1: tfar_golden_vectors, the live C++ and F3DAudio oracles
├── m4_p4_p5_spatial_playback.md          M4 P4 + P5: tfar_spatial_audio and tfar_playback_mixer
├── m4_r1_records_tooling.md              M4 R1: the records and tooling pass after the M2–M5 slices
├── m5_s1_s2_services_debug_window.md     M5 S1 + S2: tfar_web_requests and tfar_debug_window
├── m6_v1_commands_0_9.md                 M6 V1: tfar_game_commands_0_9 and tfar_plugin_commands_0_9
├── m6_v3_state_routing_0_9.md            M6 V3: tfar_client_state_0_9 and tfar_radio_routing_0_9
├── m6m7_e1_radio_effects.md              M6 + M7 E1: tfar_radio_effects_0_9 and tfar_radio_effects_1_0
├── m6m7_v4w4_voice_pipelines.md          M6 + M7 V4 + W4: tfar_voice_pipeline_0_9 and tfar_voice_pipeline_1_0
├── m6m7_v5w6_plugin_apps.md              M6 + M7 V5 + W6: the full TeamSpeak plugin apps
├── m7_w1_commands_settings_1_0.md        M7 W1: tfar_plugin_settings_1_0, the 1.0 game and plugin commands
├── m7_w3_state_routing_antennas_1_0.md   M7 W3: tfar_client_state_1_0, tfar_radio_routing_1_0, tfar_antennas_1_0
├── m8_c1_milestone_close.md              M8 C1: the full gate run, dist check, runbook and milestone close
├── m8_l4_infrastructure_gaps.md          M8 L4: game link ports and queue, talk status, mock, web requests, mixer
├── m8_p5_audio_path_gaps.md              M8 P5: allocation-free routing, F-091, local tangent, remote sounds
├── m8_r3_records.md                      M8 R3: the records pass after the plugin apps, D23 to D26
├── m8_v6w7_app_migration.md              M8 V6 + W7: both plugin apps onto the new library APIs, D24, D25
└── shared_brief.md                       rules, environment facts, crate edges and report format for every agent
```

## How it works

A slice brief is named `m<N>_<agent>_<role>.md`: the milestone number (`m6m7` for a slice that
serves two milestones), the agent id in lowercase (`a1`, `b2`, `u7`) and a snake_case role, so
`m0_a2_program_records.md` is milestone M0, agent A2, the program records. Each brief opens with "You are agent <ID>, …. Read the shared brief first",
then holds Goal, Steps, Owned files, Checks, Report additions and a closing budget line (S = 150k,
M = 250k, L = 350k tokens). The orchestrator writes a milestone's briefs at its start from the
proposed roster in the [program plan](/documentation/program/program_plan.md#5-milestones-and-agents),
and keeps the [shared brief](/documentation/program/agent_briefs/shared_brief.md) current before
each launch. Briefs of a finished milestone stay as the record of what each agent was asked.

## Code

- Each brief names the code paths its agent owns; this folder holds no code.

## Boundaries

- Depends on: the program plan, the crate catalogue, and laws and gates.
- Used by: the orchestrator and every agent it launches.
- Rules: an agent edits only its owned paths; amendments at launch are recorded in
  [progress](/documentation/program/progress.md), not by rewriting a brief already run.

## Related documentation

- [Progress](/documentation/program/progress.md) — which agents ran, their reports and amendments.
- [Resume runbook](/documentation/runbooks/continue_in_claude_code.md) — starting the next session.
