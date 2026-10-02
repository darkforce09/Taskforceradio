//! Scenario tests for the captured-voice path, against
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2852-2889`.

use tfar_client_state_0_9::prelude::{RadioSubtype, TangentKind};
use tfar_playback_mixer::PlaybackMixer;

use super::*;
use crate::test_scene::{FRAMES, ME, Scene, Signal, at, rms};

/// The context of the local player's own frame, in game, metadata showing the plugin.
fn own_context() -> OwnVoiceContext<'static> {
    OwnVoiceContext {
        my_client_id: ME,
        my_metadata_shows_plugin: true,
        my_channel_name: "Lobby",
        is_in_game: true,
    }
}

/// The local player holding a short-range push-to-talk on the scene frequency, with `Bravo`'s
/// loudspeaker on it 5 m away when `with_loudspeaker`.
fn transmitting_scene(with_loudspeaker: bool) -> Scene {
    let mut scene = Scene::new();
    scene.transmit("Me", TangentKind::ShortRange, RadioSubtype::Digital, 1000);
    if with_loudspeaker {
        scene.add_loudspeaker(at(0.0, 5.0, 0.0), 10, 1.0);
    }
    scene.prepare("Me");
    scene
}

/// Runs `frames` captured frames of a tone through the own-voice path and returns the blocks
/// laid end to end; `None` when any frame produced none.
fn own_voice_output(
    scene: &mut Scene,
    context: &OwnVoiceContext<'_>,
    frames: usize,
) -> Option<Vec<i16>> {
    let signal = Signal::Tone(2000.0, 8000.0);
    let mut captured = StereoBuffer::with_frames(FRAMES);
    let mut output = Vec::new();
    for index in 0..frames {
        signal.fill(&mut captured, index * FRAMES, FRAMES);
        let untouched = captured.clone();
        let block = scene.processor.process_own_voice_for_speakers(
            &mut scene.state,
            context,
            &captured,
            scene.now,
        )?;
        output.extend_from_slice(block.as_slice());
        assert_eq!(captured, untouched);
    }
    Some(output)
}

#[test]
fn a_loudspeaker_on_the_local_players_frequency_plays_their_own_voice() {
    let mut scene = transmitting_scene(true);
    let output = own_voice_output(&mut scene, &own_context(), 40).unwrap();
    assert!(rms(&output, 4_800) > 300.0);
}

#[test]
fn without_a_loudspeaker_the_own_voice_block_is_silent() {
    let mut scene = transmitting_scene(false);
    let output = own_voice_output(&mut scene, &own_context(), 10).unwrap();
    assert!(output.iter().all(|&sample| sample == 0));
}

#[test]
fn outside_a_game_dead_or_without_current_data_there_is_no_own_voice_block() {
    let mut scene = transmitting_scene(true);
    let outside_game = OwnVoiceContext {
        is_in_game: false,
        ..own_context()
    };
    assert_eq!(own_voice_output(&mut scene, &outside_game, 1), None);
    scene.state.is_alive = false;
    assert_eq!(own_voice_output(&mut scene, &own_context(), 1), None);
    scene.state.is_alive = true;
    scene.client("Me").data_frame = 1;
    assert_eq!(own_voice_output(&mut scene, &own_context(), 1), None);
}

#[test]
fn the_own_voice_block_goes_to_the_my_radio_voice_stream() {
    let mut scene = transmitting_scene(true);
    let mixer = PlaybackMixer::new();
    let mut captured = StereoBuffer::with_frames(FRAMES);
    Signal::Tone(2000.0, 8000.0).fill(&mut captured, 0, FRAMES);
    let block = scene
        .processor
        .process_own_voice_for_speakers(&mut scene.state, &own_context(), &captured, scene.now)
        .unwrap();
    append_own_voice(&mixer, block).unwrap();
    assert_eq!(OWN_VOICE_STREAM_NAME, "my_radio_voice");
    assert_eq!(
        mixer.stream_queued_frames(OWN_VOICE_STREAM_NAME),
        Some(FRAMES)
    );
}
