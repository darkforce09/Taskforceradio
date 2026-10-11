//! Unit tests for `GameLinkState`.

use std::time::Duration;

use super::*;

fn settings() -> PluginSettings {
    PluginSettings::default()
}

#[test]
fn a_message_connects_and_starts_the_game_once() {
    let start = Instant::now();
    let mut link = GameLinkState::new();
    assert_eq!(link.poll(start, &settings()).transition, None);
    link.record_game_message(start);
    let poll = link.poll(start, &settings());
    assert!(poll.is_connected);
    assert_eq!(
        poll.transition,
        Some(GameLinkTransition::Connected {
            is_game_start: true
        })
    );
    assert!(link.is_in_game());
    link.record_game_message(start + Duration::from_secs(1));
    assert_eq!(
        link.poll(start + Duration::from_secs(1), &settings())
            .transition,
        None
    );
}

#[test]
fn silence_longer_than_the_plugin_timeout_disconnects_and_ends_the_game() {
    let start = Instant::now();
    let mut link = GameLinkState::new();
    link.record_game_message(start);
    link.poll(start, &settings());
    let timeout = settings().plugin_timeout();
    assert!(
        link.is_connected(
            (start + timeout)
                .checked_sub(Duration::from_millis(1))
                .unwrap(),
            &settings()
        )
    );
    let poll = link.poll(start + timeout, &settings());
    assert!(!poll.is_connected);
    assert_eq!(
        poll.transition,
        Some(GameLinkTransition::Disconnected { is_game_end: true })
    );
    assert!(!link.is_in_game());
}

#[test]
fn transitions_inside_the_debounce_window_wait_for_it_to_end() {
    let start = Instant::now();
    let mut link = GameLinkState::new();
    link.record_game_message(start);
    link.poll(start, &settings());
    link.forget_game_messages();
    let early = link.poll(start + Duration::from_millis(100), &settings());
    assert!(!early.is_connected);
    assert_eq!(early.transition, None);
    let due = link.poll(start + TRANSITION_DEBOUNCE, &settings());
    assert_eq!(
        due.transition,
        Some(GameLinkTransition::Disconnected { is_game_end: true })
    );
}

#[test]
fn the_plugin_timeout_comes_from_the_settings() {
    let start = Instant::now();
    let mut settings = settings();
    settings
        .set(
            "pluginTimeout",
            "1",
            Some(crate::plugin_settings::TypeTag::Scalar),
        )
        .unwrap();
    let mut link = GameLinkState::new();
    link.record_game_message(start);
    assert!(link.is_connected(start + Duration::from_millis(999), &settings));
    assert!(!link.is_connected(start + Duration::from_secs(1), &settings));
}
