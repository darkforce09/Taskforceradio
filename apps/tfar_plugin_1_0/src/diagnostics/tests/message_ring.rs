//! Unit tests for the ring of recent game messages.

use super::*;

#[test]
fn the_ring_keeps_only_the_newest_messages_oldest_first() {
    let mut ring = MessageRing::new(3);
    for message in ["one", "two", "three", "four", "five"] {
        ring.push(message);
    }
    assert_eq!(ring.iter().collect::<Vec<_>>(), ["three", "four", "five"]);
    assert_eq!(ring.len(), 3);
}

#[test]
fn a_ring_of_no_capacity_keeps_nothing() {
    let mut ring = MessageRing::new(0);
    ring.push("POS");
    assert!(ring.is_empty());
}

#[test]
fn a_long_message_is_cut_at_a_character_boundary() {
    let mut ring = MessageRing::new(1);
    let long = "é".repeat(MAX_MESSAGE_BYTES);
    ring.push(&long);
    let kept = ring.iter().next().unwrap();
    assert!(kept.len() <= MAX_MESSAGE_BYTES);
    assert!(kept.chars().all(|character| character == 'é'));
}
