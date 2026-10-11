# Taskforceradio

A Rust rewrite of the native parts of Task Force Arrowhead Radio (TFAR), the Arma 3 radio mod, for Arma 3
(Windows build) under Proton with TeamSpeak 3 as the native Linux Flatpak. The originals link game and
TeamSpeak through Win32 named objects a native plugin cannot reach, so each mod (1.0 and 0.9.12) gets an
Arma extension (Windows x64 DLL, still named `task_force_radio_pipe`) and a TeamSpeak plugin (Linux
`.so`) that talk over localhost TCP, plus the library crates they share. The original mods sit
byte-identical and read-only in `reference/` as the specification; no SQF changes. Released as `v0.1.1`.

---

## 1. Core Project Laws

The project is maintained by AI agents. Keep it small: every rule below states an outcome, and anything that serves no outcome is deleted rather than documented.

1. **No silent deferrals.** Do the whole ask. Never invent "out of scope", "deferred" or ship an MVP and call the task done. Only Sam's explicit words defer a piece; record it in the open-work checklist in [README.md](/README.md#open-work). Truly blocked: stop and ask.
2. **Git: direct to `main`.** Never create a branch by hand. Commit only when asked. Conventional Commits, staged with explicit paths.
3. **Everything earns its place.** A crate, module, command, check, test, README or document exists only if something uses it: CI, a runbook step, or a reader who needs it. Investigations and probes live in `.workstation/` and are deleted with their task; they never become commands. Extend an existing crate before adding one; a new crate needs a real boundary (a cdylib deliverable, an `unsafe` FFI crate, a heavy-dependency firewall, a tool binary). Delete dead code instead of explaining it. When the clean fix needs a refactor, do the refactor.
4. **Rust only; `reference/` is read-only.** Product code is Rust; no shell, Python, Node or Makefiles for any purpose, a repository chore is a `cargo xtask` command. C/C++ exists only in the golden-vector oracle and as the TeamSpeak SDK headers (`verify language-bans`). Never write, format, move or delete anything under `reference/` except its README (`verify reference-untouched` against `reference/MANIFEST.sha256`).
5. **Boundaries.** Crates sit at `crates/<category>/<crate>` or `tools/<crate>` and declare `category`, `tier` and `targets`; edges point down the tiers and follow the category edge matrix; `version_0_9` and `version_1_0` never depend on each other; the extensions use only foundation and game_link; nothing depends on a deliverable (`verify crate-tiers`). Heavy external crates stay behind their firewall crate (`verify dependency-firewalls`). `unsafe` only in the allow-listed FFI crates, each block under a `// SAFETY:` comment (`verify unsafe-allowlist`). Use a library before writing one (the library map in [workspace decisions](/documentation/decisions/workspace.md)). Details: [crate boundary rules](/documentation/standards/crate_boundary_rules.md).
6. **No panics in production code.** `panic = "abort"`; `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, `dbg_macro` are denied outside tests. A panic would take TeamSpeak or Arma down.
7. **Names and comments say what things are now.** Self-describing snake_case names with the `tfar_` crate prefix and `_0_9`/`_1_0` version tags; related variants in named subfolders. Comments explain what and why (invariants, units, host constraints), never history, milestone, finding or decision ids (`verify history-residue`). Upstream `reference/…:line` cites are the specification and stay. `///` docs on public items. Details: [code and documentation](/documentation/standards/code_and_documentation.md).
8. **Documentation is small and true.** One README per crate, top-level category or documentation area: purpose, how to use, boundaries; no file-tree listings. Open work lives in the README checklist, never in docs. When a change alters a surface, command or boundary, update its README in the same commit.
9. **Tests cover core logic only.** Wire formats and command parsing, audio math against golden vectors, routing and client state, FFI layouts, the fixed original bugs, the xtask gates. Never pin prose, constants, file layout or `Debug`/`Display` output; no property or failpoint suites. Tests live in sibling `tests/` files (`verify test-placement`); production files ≤ 500 lines (`verify file-length`). Details: [testing](/documentation/standards/testing.md).
10. **CI is the law.** `cargo xtask ci ci-local` is the CI: there is no hosted runner because the reference trees exist only on this machine. Every rule with a checker runs in it; a check not in it is deleted. A step that cannot run is a bug to fix, never a skip. Before committing: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the tests of what you touched; run `ci ci-local` before a release.
11. **Everything stays in the repository.** Nothing project-related goes to `~`, `~/.cache` or `/tmp`: machine state in the gitignored `.workstation/` (logs in `.workstation/logs/`), worktrees in `.worktrees/`, build output and test scratch in `target/`. Nothing builds or copies into the game or TeamSpeak folders; Sam installs by hand.

---

## 2. Agent Environment and Workflow

- **Build:** Rust 1.99.0 pinned in `rust-toolchain.toml` with the targets `x86_64-unknown-linux-gnu` (plugins, tools) and `x86_64-pc-windows-gnu` (extensions, probe). The Windows linker is `x86_64-w64-mingw32-gcc` with `+crt-static` (`.cargo/config.toml`), so the DLL needs no runtime files. One `target/`; `cargo deny` and `typos` are installed. Builds run in the `agents` distrobox or on the host (same glibc).
- **Game and voice:** Arma 3 runs under Proton (Steam app 107410); the extension loads as `task_force_radio_pipe_x64.dll` from the mod root, with BattlEye off. TeamSpeak 3.6.2 is the Flatpak `com.teamspeak.TeamSpeak3` (plugin API 26); plugins go in `~/.var/app/com.teamspeak.TeamSpeak3/.ts3client/plugins/` (Sam's live install; never touch it).
- **Reference trees:** `reference/tfar_1_0/` (upstream clone) and `reference/tfar_0_9_12/` (local only, cannot be re-cloned) are gitignored. Never run `git clean -x`/`-X` or any bulk delete that could reach them; restore steps are in the [reference README](/reference/README.md).
- **Machine-local state** (`.workstation/`, gitignored): `logs/` (`ci ci-local` writes one log per step to `logs/ci/`), session notes and probes.
- **Tracker:** the open-work checklist in [README.md](/README.md#open-work).
- **Remote:** `origin` is `https://github.com/darkforce09/Taskforceradio` (public). Push only on Sam's go.
- **Path:** `/run/media/system/Disk_2/Projects/Taskforceradio`.
- **Orchestration is opt-in.** Work in the chat by default; sub-agents only for broad searches or genuinely parallel pieces; multi-agent programs only when Sam asks.
- **Bash discipline.** One build, test or gate per Bash call, each with its own log under `.workstation/logs/`; read logs and large files in ranges.

---

## 3. Repository Map

| Folder | Holds |
|---|---|
| `crates/foundation/` | Pure shared types: geometry on `glam`, Arma value parsing |
| `crates/audio/` | The voice DSP pipeline shared by both plugins: sample buffers, filters, dynamics, HRTF, spatial curves, playback mixer |
| `crates/teamspeak/` | `tfar_ts3_host` (the TeamSpeak C ABI, `unsafe`) and `tfar_ts3_settings_db` (rusqlite) |
| `crates/game_link/` | The extension ↔ plugin TCP protocol, its server and client, and `tfar_arma_extension_abi` (Arma C ABI, `unsafe`, Windows) |
| `crates/services/` | `tfar_web_requests`: update check and Piwik over `ureq` + rustls |
| `crates/interface/` | `tfar_debug_window`: the egui debug window |
| `crates/version_0_9/`, `crates/version_1_0/` | Each mod's TeamSpeak plugin and Arma extension (the deliverables) and its voice-logic libraries |
| `tools/` | `xtask` (gates, CI lane, dist, reference manifest), `tfar_ts3_mock` (loads a built plugin with a fake TeamSpeak), `tfar_extension_probe` (Windows exe that calls the DLL under Proton), `tfar_golden_vectors` (C/C++ oracles the audio tests compare against) |
| `assets/hrir/` | The IRCAM HRIR sphere the HRTF loads, with `CREDITS.md` |
| `documentation/` | Architecture, decisions, behaviour specs of the originals, standards, glossary, the manual install runbook; entry [documentation/README.md](/documentation/README.md) |
| `reference/` | The two original mods, read-only, guarded by `MANIFEST.sha256` |
| `.workstation/`, `.worktrees/`, `target/` | Gitignored machine state, linked worktrees, build output |

---

## 4. Canonical Commands

```bash
cargo build --workspace                                   # plugins and tools for Linux
cargo build -p tfar_extension_1_0 -p tfar_extension_0_9 --target x86_64-pc-windows-gnu
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p <crate>                                     # the crates you touched
cargo xtask verify <gate>                                 # one gate, or `all`; exit 0 held, 1 violation, 2 could not run
cargo xtask ci ci-local                                   # the CI: fmt, clippy (Linux + Windows), test, deny, typos, every gate
cargo xtask dist                                          # release artifacts in target/dist/tfar_1_0/ and target/dist/tfar_0_9/
cargo xtask reference-manifest                            # rewrites reference/MANIFEST.sha256 (needs --force --reason)
```

The gates: `crate-tiers`, `dependency-firewalls`, `unsafe-allowlist`, `file-length`, `test-placement`, `history-residue`, `link-check`, `editorconfig`, `language-bans`, `reference-untouched`. Manual install and live test: [runbook](/documentation/runbooks/manual_install_and_test.md).
