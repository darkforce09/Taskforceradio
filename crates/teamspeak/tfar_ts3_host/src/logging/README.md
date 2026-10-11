# TeamSpeak logging

Forwards `tracing` events to TeamSpeak's client log without ever calling TeamSpeak from the thread
that logs, and prints lines into the chat tab in front.

## Contents

```text
crates/teamspeak/tfar_ts3_host/src/logging/
├── install.rs        `install_logging`, `install_logging_at` and `print_to_current_tab`
├── log_forwarder.rs  the logger thread and `LoggingGuard`, which stops and joins it
├── log_layer.rs      `TeamSpeakLogLayer` and the bounded queue it fills
├── mod.rs            the module lines and the public re-exports
└── tests/            the queue, the level filter and the thread separation, with a fake sink
```

## How it works

```text
any thread (audio included)          logger thread "tfar-teamspeak-log"
tracing::info!(…) ─▶ TeamSpeakLogLayer ─try_send─▶ bounded queue ─recv─▶ sink ─▶ Ts3::log_message
                     (format, never blocks;                              (channel = plugin name)
                      full → drop and count)
```

`install_logging(plugin_name)` (from `ts3plugin_init`, after the table is installed) creates one
process-wide queue on first use and makes a registry with `TeamSpeakLogLayer` the global `tracing`
subscriber; later installs reuse both, since `tracing` takes a global default only once. It then
starts the logger thread, whose sink calls `logMessage` with the plugin name as channel and `0` as
log ID. The layer maps `ERROR`, `WARN`, `INFO`, `DEBUG` and `TRACE` to `LogLevel_ERROR`, `WARNING`,
`INFO`, `DEBUG` and `DEVEL`, filters by the level given (`DEBUG` by default), and drops every event
while no logger thread runs. Dropping the `LoggingGuard` (in `ts3plugin_shutdown`) stops
forwarding, queues the stop message, lets the thread finish what was queued and joins it.

## Boundaries

- Depends on: `tracing`, `tracing-subscriber` (`Layer`, `registry`), `crossbeam-channel` (the
  bounded queue); `ts3_functions` (`Ts3::log_message`, `Ts3::print_message_to_current_tab`).
- Used by: the plugin apps, which hold the `LoggingGuard` from `init` to `shutdown` and call
  `print_to_current_tab` in place of the originals' message boxes.
- Rules: only the logger thread calls `logMessage`
  (`lines_reach_the_sink_on_the_logger_thread_never_on_the_emitting_one`,
  `an_audio_thread_event_is_queued_and_delivered_elsewhere`); an emitter never waits on a full
  queue (`a_full_queue_drops_and_counts_instead_of_blocking`); one logger thread per queue
  (`a_second_forwarder_on_a_running_queue_is_refused`).

## Related documentation

- [Coding standards](/documentation/standards/code_and_documentation.md) — the audio-thread logging rule
  and the thread ownership the logger thread follows.
