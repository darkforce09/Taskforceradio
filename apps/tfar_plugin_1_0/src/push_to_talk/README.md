# Push to talk

The microphone forced on while a radio tangent is held, voice activation saved and restored, the
delayed release on its own thread, and the tangent broadcast.

## Contents

```text
apps/tfar_plugin_1_0/src/push_to_talk/
├── mod.rs              `PushToTalk`: press, delayed release, game-end cancel and restore, the release thread
├── tangent_gate.rs     `TangentGate`: whether a press or a release acts
└── tests/              unit tests of the gate and of the release thread over a recording fake
```

## How it works

A press, unless a release is waiting, saves and switches off voice activation, sets
`CLIENT_INPUT_DEACTIVATED` to active and sends the broadcast; with a release waiting it marks that
release skipped. A release marks itself waiting and hands a job to `tfar-tangent-release`, which
waits `push_to_talk_delay + tangentReleaseDelay` (cut short only by shutdown) and then, unless
skipped, restores voice activation and the microphone and sends the broadcast. A game end cancels
every release still waiting (the gate's release generation moves on, so a job from before it does
nothing), and the plugin, dormant again after its game, never broadcasts a release (D25).

## Boundaries

- Depends on: `parking_lot`, `crossbeam-channel`, `tfar_ts3_host`; the crate's `teamspeak_actions`.
- Used by: `game_link::local_tangent`, `runtime` (game end and shutdown).
- Rules: the gate lock is held across TeamSpeak calls, which is allowed because no audio callback
  and no TeamSpeak callback takes it; the release thread is joined at shutdown; a game end cancels
  a waiting release
  (`a_game_end_cancels_a_waiting_release_so_a_dormant_plugin_never_broadcasts_it`, and in the mock
  host `missionend_disconnects_with_the_off_sound_and_needcfg_follows_setcfg`).
