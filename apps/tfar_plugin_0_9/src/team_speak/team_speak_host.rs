//! `TeamSpeak`: the reads and the side effects together, the one handle the plugin holds.

use super::team_speak_actions::TeamSpeakActions;
use super::team_speak_queries::TeamSpeakQueries;

/// TeamSpeak as the plugin sees it: [`TeamSpeakQueries`] and [`TeamSpeakActions`], shared across
/// the plugin's threads.
pub trait TeamSpeak: TeamSpeakQueries + TeamSpeakActions + Send + Sync {}

impl<Host: TeamSpeakQueries + TeamSpeakActions + Send + Sync> TeamSpeak for Host {}
