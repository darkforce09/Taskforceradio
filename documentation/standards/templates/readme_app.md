**Status:** live

# README template: app

**When to use:** a deliverable, in its version category under crates/: the two TeamSpeak plugins
(tfar_plugin_0_9, tfar_plugin_1_0) and the two Arma extensions (tfar_extension_0_9,
tfar_extension_1_0). An app is a `cdylib` (plus an `rlib` for its tests) that a host program loads.
The [README standard](/documentation/standards/readme_standard.md) defines every rule this template
follows; the app kind adds Getting started, Configuration and Exports.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. A code README carries
no status line.

````markdown
# <What the deliverable is, in plain words: no path, no backticks>

<One to three sentences: what the app is, which mod it serves, which host loads it, and how it talks
to its counterpart.>

## Contents

```text
<repository path of the folder>/
├── Cargo.toml  <the package: cdylib + rlib, its category and tier, what each dependency is for>
└── src/        <what the source tree holds: the exports, the threads, the state, the wiring>
```

## How it works

<From the host's first call to shutdown: what each export does, which threads start and stop, where
state lives, how the crates below are wired together. An ASCII diagram in a text block helps here.>

## Getting started

<The commands, run from the repository root, that build the app for its target, test it, and produce
its artifact, each with what to expect. Link the manual install runbook for the rest.>

## Configuration

<Every setting the app reads: Cargo features (tracy, api_profiler), host settings, command-line
switches, files, each with its default and the file that reads it.>

## Exports

- Symbols: <the C ABI symbols the host resolves, each with what it does>
- Artifact: <the file `cargo xtask dist` writes, and the folder under target/dist/ it lands in>
- Installed by: <where the owner copies it by hand, and what the host shows once it loads>

## Boundaries

- Depends on: <the workspace crates the app wires together, from its Cargo.toml>
- Used by: <the host that loads it, the tools that test it>
- Rules: <the invariants a change here must keep, and the test or gate that checks each>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

## Worked sample

Written for `tfar_extension_1_0`, as its README reads once its code is written.

````markdown
# TFAR 1.0 Arma extension

The `tfar_extension_1_0` app: the Windows x64 DLL Arma loads as `task_force_radio_pipe` for the TFAR 1.0
mod. It answers every `callExtension` call from the mod's SQF by relaying it over the
[game link](/documentation/glossary.md#game-link) to the 1.0 TeamSpeak plugin.

## Contents

```text
crates/version_1_0/tfar_extension_1_0/
├── Cargo.toml  cdylib + rlib, tier 2; uses tfar_arma_extension_abi and tfar_game_link_client
└── src/        the RVExtension handler, the async and sync paths, the focus reporter
```

## How it works

`RVExtension` receives the plain-string call. A call ending in `~` is an
[async message](/documentation/glossary.md#sync-and-async-message): it is queued to the link in order and
answered at once, `NEEDCFG` or `OK` for `DFRAME~`, an empty answer for `MISSIONEND~`, `OK` for the rest.
Any other call is a sync round trip that waits at most 1000 ms for the plugin's answer. With no plugin
listening the answer is `Not connected to TeamSpeak`, exactly as the original. A `FOCUS` frame reports
whether the Arma window has focus.

## Getting started

```bash
cargo build -p tfar_extension_1_0 --target x86_64-pc-windows-gnu   # the DLL under target/
cargo test -p tfar_extension_1_0                                     # host tests of the call handling
```

## Configuration

- `-tfdebug` on the Arma command line: connect to the plugin's debug port instead of its normal port.

## Exports

- Symbols: `RVExtension` (every call), `RVExtensionVersion` (the version string Arma logs at load).
- Artifact: `task_force_radio_pipe_x64.dll` in target/dist/tfar_1_0/.
- Installed by: the owner, into the TFAR 1.0 mod root; BattlEye must be off.

## Boundaries

- Depends on: `tfar_arma_extension_abi` (the export glue and output writes), `tfar_game_link_client`
  (the connection to the plugin).
- Used by: Arma 3 under Proton; `tfar_extension_probe` under wine.
- Rules:
  - every game-facing reply string is byte-identical to the original's;
  - async messages reach the plugin in the order the game sent them (`async_messages_keep_their_order`).

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — frames, ports, timeouts.
- [Manual install and test](/documentation/runbooks/manual_install_and_test.md) — installing the DLL.
````
