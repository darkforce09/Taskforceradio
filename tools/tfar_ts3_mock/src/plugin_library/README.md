# Loaded plugin library

A TeamSpeak plugin `.so` opened with `libloading`: its resolved `ts3plugin_*` exports and the
metadata it reports before it is started.

## Contents

```text
tools/tfar_ts3_mock/src/plugin_library/
├── loaded_plugin.rs   `PluginLibrary` and `PluginMetadata`: open, resolve, read the metadata
├── mod.rs             the module lines and the public re-exports
└── plugin_exports.rs  `PluginExports`: one typed pointer per export, required and optional
```

## Boundaries

- Depends on: `libloading`, `ts3plugin-sys` (the `Ts3Functions` argument of
  `ts3plugin_setFunctionPointers`); the plugin `.so`.
- Used by: `mock_host`, the binary, and tests that call an export no driver covers.
- Rules: each export pointer's type is the C signature of `src/plugin.h` at API 26 that
  `tfar_ts3_host::ts3_plugin!` generates; the `Library` is the last field of `PluginLibrary`, so
  it unloads after the pointers taken from it are gone.
