//! Unit tests for `ReceivingTransmissions`.

use super::*;
use crate::test_support::{ALPHA, BRAVO, OWN};

#[test]
fn a_sender_counts_once_and_leaves_with_its_own_stop() {
    let mut client = ClientData::new(OWN, "me");
    assert!(client.receiving.start(ALPHA, "100"));
    assert!(!client.receiving.start(ALPHA, "100"));
    assert!(client.receiving.start(BRAVO, "100"));
    assert_eq!(client.receiving_transmission_count(), 2);
    assert_eq!(
        client.receiving_frequencies().collect::<Vec<_>>(),
        ["100", "100"]
    );
    assert!(client.receiving.stop(ALPHA));
    assert!(!client.receiving.stop(ALPHA));
    assert!(client.is_receiving_transmission());
    assert_eq!(client.receiving_senders().collect::<Vec<_>>(), [BRAVO]);
    assert!(client.receiving.stop(BRAVO));
    assert!(!client.is_receiving_transmission());
}
