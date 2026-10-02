**Status:** live

# Runbooks

Procedures run step by step: installing and testing the Rust plugins and extensions by hand, and
starting the next Claude Code session of the program.

## Contents

```text
documentation/runbooks/
├── continue_in_claude_code.md   start the next session with a pasteable prompt; fresh-clone fixes
└── manual_install_and_test.md   build with dist, install into Flatpak TeamSpeak and a mod, test the link
```

## How it works

Each runbook has Prerequisites, numbered Steps with one command block and a **Expected:** line each,
Verify, Troubleshooting and Related documentation. Commands run from the repository root unless a
step says otherwise. A runbook changes in the same commit as the command or file name it cites.

## Code

- The xtask commands the runbooks call (`cargo xtask dist`, `cargo xtask verify`; planned:
  tools/xtask/) and the probe they run (planned: tools/tfar_extension_probe/).

## Boundaries

- Depends on: the xtask command tree, the dist folder layout, and the game link protocol.
- Used by: the owner at each milestone checkpoint, and the orchestrator when resuming.
- Rules: no runbook step copies files on the owner's behalf through a script (D13); every step has
  an expected result.

## Related documentation

- [Roadmap](/documentation/roadmap.md) — the checkpoints these runbooks serve.
- [Game link protocol](/documentation/architecture/game_link_protocol.md) — the replies the install
  checks expect.
