# TeamSpeak plugin SDK headers

The official C headers of the TeamSpeak 3 client plugin SDK at plugin API 26 (clients 3.5 and 3.6),
kept byte for byte. They are never compiled: the layout tests of `tfar_ts3_host` parse them as text
to check the `ts3plugin-sys` bindings against the order and values TeamSpeak itself uses.

## Contents

```text
crates/teamspeak/tfar_ts3_host/sdk_headers/
├── logtypes.h                 the `LogLevel` and `LogTypes` enums `logMessage` takes
├── plugin_definitions.h       the plugin-side enums: configure offer, info item type, menus, hotkeys
├── public_definitions.h       ids, connect status, client, channel and server property enums
├── public_rare_definitions.h  the rarer enums, `PluginTargetMode` among them
└── ts3_functions.h            the `TS3Functions` table of host function pointers, in ABI order
```

## How it works

`src/tests/sdk_layout.rs` reads `ts3_functions.h` at test time, numbers every `(*name)(` function
pointer inside `struct TS3Functions` in order, and checks that `size_of::<Ts3Functions>()` is the
pointer count times the pointer size and that every function `tfar_ts3_host` calls sits at offset
index × pointer size in the `ts3plugin-sys` struct. The same test parses the enums of the other
headers and checks every property flag, status and target value the crate passes to TeamSpeak. No C
compiler is involved; the headers are read as text only.

## Format

- Encoding: C headers, ASCII, LF line ends, exactly as published upstream; the file names are the
  upstream names with the upstream folders flattened.
- Layout: upstream `include/ts3_functions.h` and `include/plugin_definitions.h`, then
  `include/teamspeak/public_definitions.h` and `include/teamspeak/public_rare_definitions.h`, then
  `include/teamlog/logtypes.h`. The `#include` lines inside still name the upstream folders; nothing
  resolves them.
- Adding a file: copy it unchanged from the same upstream commit, add a line above and to the
  checksum list below, and run `cargo test -p tfar_ts3_host`.

## Producers and consumers

- Producers: TeamSpeak Systems GmbH, repository
  <https://github.com/teamspeak/ts3client-pluginsdk>, tag `26`, commit
  `b2f4b8e574d13c584ba8cae0c5ab6e1a0c3048e6` (2023-07-31, "update readme"); between that tag and
  `master` only the upstream README and CI file differ. Fetched on 2026-10-02. Upstream states
  "Copyright TeamSpeak Systems GmbH. All rights reserved."; the copies serve the private build's tests
  and ship in no artifact.
- Consumers: the layout tests in `crates/teamspeak/tfar_ts3_host/src/tests/sdk_layout.rs`.

SHA-256 of the copies (equal to the upstream files at that commit):

```text
7630586c5f3dfd0ab37f31b3358633d24784f9257c1d1c04defb440a673fc91c  logtypes.h
09839fd1aa076df3e12a2f1d01b0c54fa00d129df6786ded4d3221f3ea7f8326  plugin_definitions.h
b41171b42e0bc525f77cbcc01d37b7e0449642fd177851564473ddd033b7f870  public_definitions.h
a201480d4ce253cca0dd471be9e495fe0a4673b302d2897a2dd9d172dac606c8  public_rare_definitions.h
0cad4df43e0dd2cf713a2db4fcb8ca84bb2fab97e1a54502f5e62f2e49bc9643  ts3_functions.h
```

## Boundaries

- Depends on: the upstream SDK at the pinned commit; nothing in the workspace.
- Used by: the `tfar_ts3_host` unit tests only; no build script or product code reads them.
- Rules: the files stay byte for byte as published (the checksums above); a newer plugin API means
  a new pinned commit and a rerun of the layout test (`cargo test -p tfar_ts3_host sdk_layout`).

## Related documentation

- [TFAR 1.0 plugin behaviour](/documentation/program/behaviour_specs/tfar_1_0_plugin.md) — the
  TeamSpeak functions the 1.0 plugin calls.
- [TFAR 0.9.12 plugin behaviour](/documentation/program/behaviour_specs/tfar_0_9_plugin.md) — the
  TeamSpeak functions the 0.9.12 plugin calls.
