//! Unit tests for `VoiceScratch`.

use super::*;

#[test]
fn scratch_is_sized_for_the_frames_it_is_made_for() {
    let capacities = VoiceScratch::with_frames(960).capacities();
    assert_eq!(capacities.stereo_frames, [960; 4]);
    assert!(capacities.mono_frames >= 960);
    assert!(capacities.routes >= ROUTE_CAPACITY);
    assert!(capacities.nickname_bytes >= NICKNAME_CAPACITY);
}

#[test]
fn the_default_scratch_holds_one_hundred_milliseconds() {
    let capacities = VoiceScratch::default().capacities();
    assert_eq!(capacities.stereo_frames, [DEFAULT_MAX_FRAMES; 4]);
}
