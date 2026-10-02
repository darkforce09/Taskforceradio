**Status:** live

# Template: runbook

**When to use:** a procedure someone runs step by step: installing and testing the built artifacts by
hand, probing an extension under Proton's wine, resuming the program in a new session. Runbooks live in
`documentation/runbooks/`, one file each, named in snake_case after the procedure; a runbook longer than
500 lines becomes a folder with a README index. The
[documentation standards](/documentation/standards/documentation_standards.md) fix the sections.

## Skeleton

Copy the block and replace every `<…>` placeholder; each one says what goes there. Steps are numbered;
each holds one command in its own block, or one action in the game or TeamSpeak, and an **Expected:**
line. A runbook may add a section of reference facts (paths, ports, file names) between Prerequisites and
Steps.

````markdown
**Status:** live

# <The procedure, in plain words: what it achieves>

<One to three sentences: what the procedure does, when to run it, and how long it takes.>

## Prerequisites

- <a tool, file, setting or running program the procedure needs, and how to tell it is there>

## Steps

1. <What the step does, and where it runs: the repository root, TeamSpeak, Arma.>

   ```bash
   <one command>
   ```

   **Expected:** <the output or state that shows the step worked, quoted from a safe run or from the code
   that prints it>

## Verify

<The check that proves the whole procedure worked, as one command block or action and its **Expected:**
line.>

## Troubleshooting

- **<what the operator sees, quoted>:** <why it happens; what to do>

## Related documentation

- [<document title>](/documentation/<path to the document>) — <what it covers>
````

Every command is checked before it is written: a `cargo xtask` command against its own `--help`, a host
action against the host. An **Expected:** line comes from a safe run or from the code that prints the
output. No step copies files into the game, the mod folders or TeamSpeak on the owner's behalf: those
steps are written for the owner to do by hand.

## Worked sample

A shortened procedure for probing a built extension under Proton's wine, as it reads once the probe is
written. The sample sits in a fenced block, so no gate reads its links or paths.

````markdown
**Status:** live

# Probe an extension under wine

Loads a built extension DLL in Proton's wine with the extension probe and sends it one call, to check the
game link without starting Arma. Takes a minute.

## Prerequisites

- TeamSpeak is running with the matching plugin loaded; its plugin list shows it.
- The probe and the extension are built for `x86_64-pc-windows-gnu`.

## Steps

1. Build the probe and the 1.0 extension, from the repository root.

   ```bash
   cargo build -p tfar_extension_probe -p tfar_extension_1_0 --target x86_64-pc-windows-gnu
   ```

   **Expected:** `Finished` and no error; both files exist under target/x86_64-pc-windows-gnu/debug/.

2. Send the call `TS_INFO` with the field `PING` through the DLL with Proton's wine; the probe joins
   its arguments with a tab.

   ```bash
   <proton wine> target/x86_64-pc-windows-gnu/debug/tfar_extension_probe.exe tfar_extension_1_0.dll TS_INFO PING
   ```

   **Expected:** `PONG`.

## Verify

The probe prints `PONG` for both extensions, each with its own plugin running.

## Troubleshooting

- **`Not connected to TeamSpeak`:** the plugin is not listening on its port; check TeamSpeak's plugin
  list and its log for the listener line.

## Related documentation

- [Game link protocol](/documentation/architecture/game_link_protocol.md) — ports and replies.
````
