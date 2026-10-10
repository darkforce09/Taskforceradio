//! The local push-to-talk: the state machine of press, delayed release and re-press, and its
//! TeamSpeak side with the release thread.

mod push_to_talk_control;
mod push_to_talk_state;

pub use push_to_talk_control::PushToTalk;
pub use push_to_talk_state::{PendingRelease, PressAction, PushToTalkState};
