**Status:** frozen record (2026-10-02)

# Conventions survey: TBD-Reforger, TBD-subtitles, Zed, rerun

This survey is the source the workspace conventions were adopted from (decision D8). Paths are absolute so
agents can open the originals.
- TBD-Reforger: `/run/media/system/Disk_2/Projects/TBD-Reforger`
- TBD-subtitles: `/run/media/system/Disk_2/Projects/TBD-subtitles`

## 1. Adopted from TBD-Reforger
- **Layout:** `apps/` (deliverables), `crates/<category>/<name>/` (libraries), `tools/` (xtask and dev
  tools), `documentation/` (the only place Markdown lives apart from READMEs).
- **Tier metadata** in every manifest:
  ```toml
  [package.metadata.layout]
  category = "crates/foundation"
  tier = 0
  targets = "any"
  ```
  Tier 0 means no workspace dependency; otherwise 1 + the highest dependency tier, so edges point strictly
  down. `verify crate-tiers` also checks: every manifest is a member, path = category + name, package name
  = folder name, a category edge matrix, and external-crate firewalls.
- **Crate anatomy** (`verify crate-anatomy`): `lib.rs` ≤ 80 lines (header, attributes, `mod`, `pub use`);
  `pub mod prelude`; `error.rs` with a `thiserror` `Error` and `Result` when fallible; no `anyhow` in
  libraries; README with a Contents block; edition, rust-version, lints and every dependency from the
  workspace; dev-only features; no primitive-typed public `*_id` (newtype IDs); no `pub use` of another
  workspace crate outside `prelude.rs`.
- **Root manifest:** `resolver = "3"`; members with globs (`"crates/*/*"`); `[workspace.package]` edition
  2024, rust-version, license, publish false; `[workspace.dependencies]` lists internal crates by path and
  shared externals with off default features and a reason comment; `[workspace.lints]` rust `unsafe_code =
  "deny"`, `missing_docs`, `unreachable_pub`; clippy `dbg_macro`, `todo`, `unwrap_used`; CI uses `-D
  warnings`. `clippy.toml`: `allow-unwrap-in-tests = true`.
- **`.cargo/config.toml`:** `[alias] xtask = "run --package xtask --"`.
- **xtask groups:** `verify` (file-length, no-python, no-node, no-shell, crate-tiers, crate-anatomy,
  readme-coverage, markdown-placement, link-check, …), `ci <task>` (static task table in
  tools/xtask/src/commands/ci/task_definitions.rs, composite `ci-local`), `mk`, `refactor relocate`, etc.
- **Module header:** four-point `//!` contract: `**Role:**`, `**Position:**`, `**Signals & state:**`,
  `**Invariants:**`.
- **Tests:** sibling files `#[cfg(test)] #[path = "tests/<file>.rs"] mod tests;`; inline `mod tests {}`
  forbidden.
- **CLAUDE.md laws:** no silent deferrals; commit straight to `main`; clean architecture over hacks;
  self-describing names; variants in subfolders; strict boundary layers; file size ceilings (500 production,
  1000 tests); in-code docs (present tense, no history or ticket ids, `///` on public items);
  docs ship in the same commit. `AGENTS.md` is a symlink to `CLAUDE.md`.
- **Program records** (documentation/restructure/ in Reforger):
  - `program_plan.md`: Context · Verified findings · Binding decisions (`| Id | Decision |`) · Execution model
    · Stages table (`| Stage | Agents and content | Extra gates, checkpoints |`, budgets S = 150k, M =
    250k, L = 350k, ∥ for parallel) · Shared brief essentials · Risks.
  - `crate_catalogue.md`, `laws_and_gates.md` (fast stage gate vs full gate set), `target_file_tree.md`.
  - `progress.md`: `## Current state` (Branch, Current stage, Last green commit, Next action, Blocked on) ·
    `## Stages` (`- [ ] <ID> (<budget>) <role> — status — commit — notes`) · `## Execution log` (`| Date |
    Event | Detail | Logs |`) · `## Amendments` · `## Open findings` (F-NNN, FIX/NOTE/CLOSE) · `## Handoff`.
  - `agent_briefs/shared_brief.md`: Rules (no git writes; owned files only; compile at every save point;
    tests never weakened; a perturbation proof for every new check; one command per shell call, logged) ·
    Efficiency · Spec · Report format (≤ 250 words: files; commands, exit codes, logs; tests; perturbations;
    findings; not run or deviations).
  - Per-agent brief: "You are agent <ID>, <role>… Read the shared brief first"; Goal · Steps · Owned files ·
    Report additions; ends "Budget: M (250k tokens). Stop there and report done and not done."
  - Orchestration method: documentation/runbooks/sub_agent_orchestration.md in Reforger.
- **Documentation rules:** snake_case names; first line `**Status:** live | frozen record | archived` (code
  READMEs carry none); live docs ≤ 500 lines; repository-root links (`/documentation/...`); a backticked path
  must exist; ASCII diagrams in `text` blocks; decisions as `### YYYY-MM-DD — <decision>` with Context,
  Decision, Consequences, Supersedes.
- **README standard:** every folder has one (except `tests`, `generated`, dot-folders), in order: H1 + purpose
  · `## Contents` (`text` tree of direct children with roles, gate-checked) · `## How it works` · kind sections
  (crate root: Getting started, Configuration, Public surface; area root: Getting started; CLI: Commands) ·
  `## Boundaries` (exactly `Depends on:`, `Used by:`, `Rules:`) · `## Related documentation`.

## 2. Adopted from TBD-subtitles
- **Milestones** live in one file, `documentation/roadmap.md`: `## M<n> — <title>`, a `- [ ]` checklist of
  deliverables, and a closing `**Acceptance:**` paragraph. "Tick items in the same commit as the work. A
  milestone is done when its acceptance test passes; nothing moves to a later milestone without the owner
  saying so." Half-steps (`M0.5`) for spikes; `## Open questions` table at the end.
- **Per-dependency "why" comments** in every manifest; `default-features = false` preferred.
- **Runbook** `documentation/runbooks/continue_in_claude_code.md` with a pasteable resume prompt.
- **Gates** are Rust (tools/repo_gates): language-bans, file-length, module-headers, test-placement,
  prose-rules, editorconfig, crate-layering, readme-coverage, readme-sections, markdown-placement,
  status-lines, link-check. Exit codes: 0 held, 1 violation, 2 could not run. Each gate has a test file.
- **Law: Rust only** for repository tasks (no Python, shell, Makefiles, Node).
- **Commits:** `type(scope): lowercase summary` ≤ 72 chars, types feat/fix/refactor/perf/test/docs/build/chore,
  scope = crate name in kebab-case, 2–4 line body, explicit paths (never `git add -A`), Co-Authored-By.
- **Verification before every commit:** fmt check, clippy `-D warnings`, test, gates.
- **Sub-agent prompts:** Opus only; structure = shared rules block, files owned, verified facts with
  file:line, exact API, changes, tests, commands, short report; parallel agents in worktrees with their own
  `CARGO_TARGET_DIR`.

## 3. Adopted from Zed and rerun (raw copies were fetched to the session scratchpad)
- **Zed:** folder name = package name, snake_case; internal crates path-only in `[workspace.dependencies]`;
  every dependency `foo.workspace = true`; `[lints] workspace = true`; a `new-crate` scaffolder; `xtask
  clippy` and `package-conformity` (checks workspace lints and workspace deps) built on `cargo_metadata`;
  `clippy.toml` `disallowed-methods` entries each with reason and replacement; `rustfmt.toml` `style_edition
  = "2024"`; typos config.
- **rerun:** a short project prefix on internal crates (`re_` → `tfar_`); crates grouped into layer folders
  with a README each and the rule "a crate may depend on its own folder or any folder below it, never above;
  dev-dependencies exempt"; one glob per layer in `members`; internal deps grouped under `# crates/<layer>:`
  banners; a curated clippy list (`pedantic` at priority -1 plus restriction picks such as `unwrap_used`,
  `undocumented_unsafe_blocks`, `todo`, `dbg_macro`, each opt-out with a reason); `deny.toml` (graph
  targets limited to our triples, `multiple-versions = "deny"`, every ignore with a reason, licence
  allow-list, `unknown-git = "deny"`); CODE_STYLE highlights: avoid `unwrap`/`expect`/indexing, `thiserror`
  in libraries and `anyhow` in apps, short `lib.rs`, every public item documented, `TODO(name)` only.
- **Not adopted:** Zed's ~130 shell scripts and dylint; rerun's pixi/Python tooling and generated crate graph.
