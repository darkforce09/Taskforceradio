**Status:** live

# M0 · A1 · Standards, templates, CLAUDE.md, decisions

You are agent A1, the standards author for the TFAR Rust port. Read `/documentation/program/agent_briefs/shared_brief.md` first.

## Goal
Write the repository's rulebook. Adapt the TBD-Reforger and TBD-subtitles standards to this project, using
`/documentation/program/research/conventions_survey.md` as the map. Every rule must fit TFAR: two TeamSpeak
plugins (.so), two Arma extensions (Windows DLLs), shared libraries. Remove anything TBD-specific: Enfusion,
Leptos, Postgres, tickets, map engine, ML, AppImage and so on.

## Steps
1. **`CLAUDE.md` at the root, plus `AGENTS.md` as a relative symlink to it.**
   - Sections: a "Current state" paragraph (M0 in progress, resume from `/documentation/program/progress.md`); `## 1. Project laws`; `## 2. Directory atlas`; `## 3. Environment`; `## 4. Commands`; `## 5. Where to look` (a Question | Document table).
   - `## 1. Project laws` are numbered and short, each naming the gate that enforces it:
     - no silent deferrals;
     - Rust only for product code, with C/C++ only in the golden-vector oracle;
     - `reference/` is read-only;
     - use libraries and don't reinvent (D18);
     - strict category layering, and version_0_9 and version_1_0 never depend on each other;
     - self-describing names with the `tfar_` prefix;
     - variants go in subfolders;
     - file size ceilings of 500 and 1000 lines;
     - sibling test files;
     - in-code documentation: the four-point `//!` header, `///` on public items, present tense, no history or milestone ids;
     - no panics in production code (`panic = "abort"`, with the deny list);
     - `unsafe` only in allow-listed FFI crates, each `unsafe` block with a `// SAFETY:` comment;
     - docs move with code;
     - git: Conventional Commits straight to `main`, explicit paths, a Co-Authored-By trailer.
   - `## 2. Directory atlas` is an annotated `text` tree of the target layout (approved plan §3).
   - `## 3. Environment`: Proton Arma, the Flatpak TS paths, toolchain 1.99.0, the mingw linker, and BattlEye off for testing.
   - `## 4. Commands`: the gate commands from plan §9.
2. **`/README.md`:** the project name, a purpose paragraph, a layout table, "Build and check", and "Status" (links to the roadmap).
3. **`/documentation/README.md`:** the map of the documentation tree and the authority ladder (running code > CLAUDE.md > decisions > standards > live docs > frozen records).
4. **`/documentation/standards/`:**
   - `README.md`
   - `coding_standards.md`: errors with `thiserror` in libraries and `anyhow` in tools only; the panic-free rules; the unsafe and FFI rules (C strings, ownership, never unwind across FFI); threading rules for TS audio callbacks (no blocking locks held across TS calls, no allocation-heavy work where avoidable, `rtrb`/`arc-swap` patterns); a dependency policy (workspace dependencies, a why comment, `default-features = false`, firewalls); naming; module layout; tests; file sizes.
   - `documentation_standards.md`
   - `readme_standard.md`: the README kinds crate root, app, area root, domain, leaf, command line, data, and documentation folder, with the exact section order from the survey.
   - `commit_conventions.md`: scopes are crate names in kebab-case, plus `workspace`, `program` and `docs`.
5. **`/documentation/standards/templates/`:**
   - `README.md`
   - `readme_crate_root.md`, `readme_app.md`, `readme_area_root.md`, `readme_domain.md`, `readme_leaf.md`, `readme_command_line.md`, `readme_data.md`, `readme_documentation_folder.md`: each with a skeleton and a short worked sample using a TFAR crate.
   - `decisions_entry.md`, `runbook.md`, `research_snapshot.md`
   - `agent_brief.md`: the per-agent prompt skeleton (Goal, Steps, Owned files, Report additions, Budget line).
   - `module_header.md`: the four-point `//!` header with a sample.
6. **`/documentation/decisions/`:**
   - `README.md`
   - `workspace.md`: D5–D8, D11–D18.
   - `game_link.md`: D2, D22, plus the TCP link decision, the ports and the focus frame.
   - `audio.md`: D19–D21.
   - `bug_policy.md`: D3, D4, D9, D10.
   - Each entry is `### 2026-10-02 — <decision>` with Context, Decision, Consequences, Supersedes. Take the wording from the approved plan §2.
7. **`/documentation/glossary.md`:** SW, LR, DD, tangent, PTT, VAD, serious mode, data frame, speaker radio, intercom, antenna/radio tower, terrain interception, object interception, vehicle isolation, stereo mode, subtype, game link, sync and async message, NEEDCFG, plugin command, metadata block, dormant plugin, HRIR, RBJ, DF2T, golden vector. Each term gets 1–2 sentences.
8. **`/reference/README.md`:**
   - What the two trees are: `tfar_1_0` = upstream master @ `fce977fa` with its own `.git`, gitignored here; `tfar_0_9_12` = the 0.9.12 release.
   - The read-only law, and that `MANIFEST.sha256` (written by `cargo xtask reference-manifest`) guards them.
   - How to restore `tfar_1_0`: `git clone https://github.com/michail-nikolaev/task-force-arma-3-radio reference/tfar_1_0`, then `git -C reference/tfar_1_0 checkout fce977fa`.
   - This README file is the only thing you write under `reference/`.

## Owned files
`/CLAUDE.md`, `/AGENTS.md` (symlink), `/README.md`, `/documentation/README.md`, `/documentation/glossary.md`,
`/documentation/standards/**`, `/documentation/decisions/**`, `/reference/README.md`.

## Checks
- Every document starts with the status line, stays ≤ 500 lines, links from the repository root, and puts diagrams in `text` blocks.
- Every backticked repository path you mention either exists or is clearly marked as planned. Write planned paths as plain text, not code, so the link gate stays green.
- Run `typos` on your files.

## Report additions
List every TBD rule you dropped or changed, and why, in one line each.

Budget: M (250k tokens). Stop there and report what is done and what is not.
