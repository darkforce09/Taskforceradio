//! The plugin's door to TeamSpeak: the reads, the side effects, and their function-table
//! implementation.

mod team_speak_actions;
mod team_speak_host;
mod team_speak_queries;
mod ts3_team_speak;

pub use team_speak_actions::TeamSpeakActions;
pub use team_speak_host::TeamSpeak;
pub use team_speak_queries::TeamSpeakQueries;
pub use ts3_team_speak::Ts3TeamSpeak;
