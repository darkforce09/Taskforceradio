# Local push-to-talk

The local player's push-to-talk: what a press does while a delayed release waits, and the TeamSpeak
side of a press and a release: voice activation, the microphone switch and the broadcast.

## Contents

```text
crates/version_0_9/tfar_plugin_0_9/src/push_to_talk/
├── mod.rs                   the module lines and the re-exports
├── push_to_talk_control.rs  `PushToTalk`: press, release, the game-end cancel, the release thread's loop, stop
├── push_to_talk_state.rs    `PushToTalkState`: the pending release, the saved VAD state, the stop flag
└── tests/                   unit tests of the state machine
```

## How it works

A press with no release waiting saves the `vad` pre-processor state, switches voice activation off
when it was on, switches the microphone on (`CLIENT_INPUT_DEACTIVATED` = `INPUT_ACTIVE`, flushed)
and broadcasts the `TANGENT…PRESSED` command; a press while a release waits cancels that release and
sends nothing, since the others never saw it. A release is scheduled with TeamSpeak's push-to-talk
delay; the `tfar-push-to-talk-release` thread waits on a condvar until it is due, then restores voice
activation, switches the microphone off unless voice activation keeps it open, and broadcasts the
release. The lock is held across those TeamSpeak calls so a press and a release never interleave;
no audio callback and no TeamSpeak event takes it. A game end cancels a waiting release: voice
activation and the microphone are restored at once and nothing is broadcast, so a plugin dormant
again after its game sends no release. On stop, a waiting release is sent at once.

## Boundaries

- Depends on: `parking_lot` (mutex and condvar), the crate's `team_speak/` traits.
- Used by: `crates/version_0_9/tfar_plugin_0_9/src/game_messages/local_tangent_message.rs` (press and release),
  `crates/version_0_9/tfar_plugin_0_9/src/running_plugin.rs` (the thread),
  `crates/version_0_9/tfar_plugin_0_9/src/game_session_transitions.rs` (the game-end cancel).
- Rules: a cancelled release never fires
  (`press_release_press_release_within_the_delay_sends_only_the_last_release`); voice activation is
  restored after the release
  (`voice_activation_is_switched_off_for_the_press_and_back_on_after_the_release`); a game end
  cancels a waiting release
  (`a_game_end_cancels_a_waiting_release_so_a_dormant_plugin_never_broadcasts_it`).
