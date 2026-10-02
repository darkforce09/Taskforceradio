# Fake TeamSpeak state

The model of a TeamSpeak client the fakes answer from, the ledger of the memory they hand out,
and the session that lets one mock host at a time own them.

## Contents

```text
tools/tfar_ts3_mock/src/fake_state/
├── allocation_ledger.rs  the `malloc`ed blocks handed out, the frees, and the `LeakReport`
├── fake_teamspeak.rs     `FakeTeamSpeak`, `FakeServer`, `FakeChannel`, `FakeClient`
├── host_session.rs       the process-global state, its lock and the one-host-at-a-time session
└── mod.rs                the module lines and the public re-exports
```

## How it works

`host_session.rs` keeps one `FakeTeamSpeak` in a `static` (a function pointer carries no
context, so the fakes reach it globally) behind a re-entrant `parking_lot` mutex over a
`RefCell`. `HostSession::begin` waits on a condition variable until no other session is active,
then replaces the state with a fresh one whose plugin and config paths are new temporary folders
(or the ones given); its drop frees every block still handed out and wakes the next waiter. The
ledger in `allocation_ledger.rs` maps each outstanding block's address to the function that
handed it out; a free of an unknown or already freed address is recorded, never passed to
`free`.

## Boundaries

- Depends on: `temporary_folder`, `host_call`; `parking_lot`, `libc`, `tfar_ts3_host`.
- Used by: `fake_functions` (every fake), `mock_host` (the drivers and the session it holds).
- Rules: at most one session per process; a session starts from fresh state
  (`a_new_session_starts_from_fresh_state_and_frees_what_was_left`); a nested use fails instead of
  deadlocking (`a_nested_use_of_the_fake_state_fails_at_once_instead_of_deadlocking`).
