# Xtask commands

The command groups of `cargo xtask`: the verify gates, the CI lane, crate scaffolding, the release
copy and the reference manifest. People, agents and the orchestrator run them from the repository
root.

## Contents

```text
tools/xtask/src/commands/
├── ci/                    `ci ci-local`: the step table and its runner
├── dist.rs                `dist`: release builds, sounds and the HRIR sphere copied into <target>/dist/
├── mod.rs                 the module list
├── new_crate/             `new-crate`: the crate templates, the workspace entry, the README line
├── reference_manifest.rs  `reference-manifest`: writes the checksum list of `reference/`
├── tests/                 tests of `dist` and `reference-manifest`
└── verify/                `verify <gate>` and `verify all`: one module per gate
```

## How it works

`main.rs` calls one entry point per command: `verify::gate_runner::run_selection`,
`ci::ci_local::run`, `dist::run`, `new_crate::scaffold::run` or `reference_manifest::run`. Each
takes the `Repository` of the workspace and returns its outcome; `main` turns it into the exit
code. The verify, ci and new-crate groups are folders with their own READMEs; `dist` and
`reference-manifest` are one file each and are described here.

## Commands

All run from the repository root as `cargo xtask <command>`. Exit code 0 means success, 1 a
failure with an `error:` line; the verify and ci groups have their own codes, in their READMEs.

### dist

- Synopsis: `cargo xtask dist`
- Does: builds `tfar_plugin_0_9` and `tfar_plugin_1_0` in release for Linux and
  `tfar_extension_0_9` and `tfar_extension_1_0` in release for `x86_64-pc-windows-gnu`, then
  copies into `<target>/dist/tfar_0_9/` and `<target>/dist/tfar_1_0/` the plugin as
  `task_force_radio_linux_amd64.so` (0.9) or `TFAR_linux_amd64.so` (1.0), the extension as
  `task_force_radio_pipe_x64.dll`, the version's `radio-sounds/` folder from
  `reference/tfar_0_9_12/ts/radio-sounds/` or `reference/tfar_1_0/ts/radio-sounds/`, and
  `tfar_hrir/` with `assets/hrir/IRC_1002_C.bin` and `assets/hrir/CREDITS.md`. TeamSpeak names a
  plugin after its file, and the original receivers filter plugin commands by that name, so the
  `.so` names are the original plugins' names.
- Target folder: `<target>` is `CARGO_TARGET_DIR` when it is set and not empty (joined to the
  repository root when relative), else `target/`; cargo builds there too.
- HRIR path: both plugins load the sphere from `tfar_hrir/IRC_1002_C.bin` relative to TeamSpeak's
  plugins folder, where `tfar_hrir/` sits beside `radio-sounds/`.
- Exit codes: 0 every artifact copied; 1 a build or a copy failed.
- Example: `cargo xtask dist`, or
  `CARGO_TARGET_DIR=target-r1 cargo xtask dist` to build into and copy under `target-r1/`.

### reference-manifest

- Synopsis: `cargo xtask reference-manifest [--force --reason <REASON>]`
- Does: hashes every file under `reference/` except its README, the manifest and the 1.0 tree's
  `.git/` folder, and writes the sorted `<sha256>  <path>` list to `reference/MANIFEST.sha256`,
  paths relative to `reference/` (so `sha256sum -c` runs from there). An existing manifest is
  overwritten only with `--force` and a non-empty `--reason`, which is printed.
- Exit codes: 0 written; 1 the manifest exists without `--force`, or a file could not be read.
- Example: `cargo xtask reference-manifest`

## Boundaries

- Depends on: `support/` for the repository, the members and the reference hashes; cargo as a
  child process.
- Used by: `main.rs`; people and agents through `cargo xtask`.
- Rules:
  - `dist` writes only under the target folder (`a_folder_copy_reproduces_the_tree_under_target_only`
    in `commands/tests/dist.rs`), follows `CARGO_TARGET_DIR`
    (`the_target_folder_follows_cargo_target_dir_and_falls_back_to_target`), copies the sphere into
    both version folders (`every_version_folder_gets_the_hrir_sphere_and_its_credits_in_tfar_hrir`)
    and the final names stay as listed
    (`the_artifacts_carry_the_names_their_hosts_expect`);
  - `reference-manifest` never overwrites without a reason
    (`commands/tests/reference_manifest.rs`).

## Related documentation

- [Laws and gates](/documentation/program/laws_and_gates.md) — the gate set the commands run.
