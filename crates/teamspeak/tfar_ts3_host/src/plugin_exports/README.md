# Plugin exports

The `ts3plugin_*` export layer: the `Ts3Plugin` trait a plugin app implements, the `ts3_plugin!`
macro that turns it into the C symbols TeamSpeak looks up, and the glue between the two.

## Contents

```text
crates/teamspeak/tfar_ts3_host/src/plugin_exports/
├── audio_exports_macro.rs  the voice-data and rolloff exports of the macro
├── audio_glue.rs           sample pointers and counts into bounded slices, then the trait call
├── event_arguments.rs      the argument bundles: invoker, client move, errors, command, voice data
├── event_exports_macro.rs  the connection, channel, client, error and plugin-command exports
├── event_glue.rs           raw event arguments into newtypes and `&str`, then the trait call
├── export_glue.rs          the hidden `$crate::export_glue` path the macro expansion calls
├── export_macro.rs         `ts3_plugin!`: metadata, lifecycle, command and info-panel exports
├── metadata_glue.rs        the bodies of the metadata, lifecycle, command and `infoData` exports
├── mod.rs                  the module lines and the public re-exports
├── plugin_answers.rs       init, command and error answers, and the info-panel item
├── plugin_memory.rs        `malloc` for the `infoData` text and `free` for `ts3plugin_freeMemory`
├── plugin_trait.rs         `Ts3Plugin`: metadata constants and one no-op default per callback
└── tests/                  the macro on a dummy plugin, and the `infoData` memory
```

## How it works

A plugin app writes `static PLUGIN: MyPlugin = …;` and `tfar_ts3_host::ts3_plugin!(PLUGIN);` at its
crate root. The macro (split over `export_macro.rs`, `event_exports_macro.rs` and
`audio_exports_macro.rs`) expands to one `#[unsafe(no_mangle)] extern "C"` shell per export, with the
argument list of the SDK export prototypes at API 26 (the original keeps them in
`reference/tfar_1_0/ts/src/plugin.h`). Each shell bundles its raw arguments and calls a function
under `$crate::export_glue`, which `export_glue.rs` gathers from the three glue files:

```text
TeamSpeak ──dlsym──▶ ts3plugin_onEditPostProcessVoiceDataEvent (macro shell, app crate)
                       │ raw pointers and counts
                       ▼
                     audio_glue::post_process_voice_data
                       │ null and size checks → &mut [i16], &[u32], &mut u32
                       ▼
                     Ts3Plugin::on_edit_post_process_voice_data (the app's code)
```

The metadata strings are the trait's `&'static CStr` constants, so the pointers never dangle.
`ts3plugin_setFunctionPointers` installs the `Ts3` handle, `ts3plugin_registerPluginID` stores the
ID in it, and `ts3plugin_init` fails with `1` when no table is installed. `ts3plugin_infoData`
copies the plugin's text into a `malloc` block that TeamSpeak hands back to `ts3plugin_freeMemory`,
which calls `free`. Voice callbacks run on TeamSpeak's audio threads: their glue allocates nothing,
locks nothing and calls no TeamSpeak function; a null or inconsistent buffer becomes an empty slice
with `frames == 0`, and a null mask, `edited` or `volume` pointer a local value.

## Boundaries

- Depends on: `ts3_functions` (`Ts3`, `owned_memory::borrowed_text`), `host_ids`, `host_values`;
  `ts3plugin-sys` (`Ts3Functions` by value in `ts3plugin_setFunctionPointers`); `libc` (`malloc`,
  `free`).
- Used by: the plugin apps `tfar_plugin_0_9` and `tfar_plugin_1_0`, which implement `Ts3Plugin`
  and expand `ts3_plugin!`.
- Rules: every export signature matches the SDK prototypes at API 26 in
  `reference/tfar_1_0/ts/src/plugin.h` (`tests/export_macro.rs` calls each one through a function
  pointer); a slice never exceeds `channels * frames` and null becomes empty
  (`post_process_voice_gets_a_slice_bounded_by_the_counts`,
  `null_or_negative_voice_arguments_become_empty_slices`); `infoData` memory is `malloc`ed and
  freed by `ts3plugin_freeMemory` (`tests/plugin_memory.rs`).

## Related documentation

- [Coding standards](/documentation/standards/coding_standards.md) — the FFI and audio-thread rules
  the glue follows.
