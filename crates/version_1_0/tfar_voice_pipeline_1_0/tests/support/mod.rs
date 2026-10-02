//! The world the voice scenarios run in: a session, its settings and towers, a processor, and
//! the signal helpers that measure what comes out.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of the helpers"
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test signals are built in f64 and written as i16 samples"
)]

use std::f64::consts::TAU;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tfar_antennas_1_0::{Antenna, AntennaManager, NetId};
use tfar_client_state_1_0::prelude::{
    ClientData, DataFrame, DspContext, FrequencyEntry, RadioSubtype, SessionState, Speaker,
    StereoMode, TangentRadio, TeamspeakClientId, VehicleDescriptor,
};
use tfar_geometry::{Direction, Position};
use tfar_hrtf::HrirLibrary;
use tfar_plugin_settings_1_0::{PluginSettings, TypeTag};
use tfar_sample_buffer::StereoBuffer;
use tfar_voice_pipeline_1_0::{VoiceFrameContext, VoiceOutcome, VoiceProcessor, ensure_sender_dsp};

pub(crate) const OWN: TeamspeakClientId = TeamspeakClientId(1);
pub(crate) const ALPHA: TeamspeakClientId = TeamspeakClientId(2);
pub(crate) const BRAVO: TeamspeakClientId = TeamspeakClientId(3);

/// TeamSpeak's frame: 10 ms at 48 kHz.
pub(crate) const FRAMES: usize = 480;
/// A short-range frequency the listener carries.
pub(crate) const SW_FREQUENCY: &str = "100";
/// A long-range frequency the listener carries.
pub(crate) const LR_FREQUENCY: &str = "40";
/// A frequency only speakers play.
pub(crate) const SPEAKER_FREQUENCY: &str = "77";
/// The sample rate of every signal.
const SAMPLE_RATE: f64 = 48_000.0;

pub(crate) fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

pub(crate) fn facing(x: f64, y: f64, z: f64) -> Direction {
    Direction::from_array([x, y, z]).unwrap()
}

pub(crate) fn in_vehicle(name: &str, isolation: f64, intercom_slot: i16) -> VehicleDescriptor {
    VehicleDescriptor {
        name: name.to_owned(),
        isolation,
        intercom_slot,
        velocity: [0.0; 3],
    }
}

fn entry(frequency: &str, volume: f64) -> (String, FrequencyEntry) {
    (
        frequency.to_owned(),
        FrequencyEntry {
            frequency: frequency.to_owned(),
            volume,
            stereo_mode: StereoMode::Both,
            radio_classname: Some("tf_radio".to_owned()),
        },
    )
}

/// One listener (`me`), two senders (`alpha`, `bravo`), the settings, the towers and a
/// processor.
pub(crate) struct World {
    pub(crate) session: SessionState,
    pub(crate) settings: PluginSettings,
    pub(crate) antennas: AntennaManager,
    pub(crate) processor: VoiceProcessor,
    pub(crate) hrir: Option<Arc<HrirLibrary>>,
    pub(crate) is_serious_mode: bool,
    pub(crate) now: Instant,
}

impl World {
    /// `me` alive at the origin facing north with a short- and a long-range radio; `alpha` and
    /// `bravo` alive at the origin with TFAR on and a 20 m voice; a game running; no HRTF; the
    /// voice cone off, so a level is the distance curve alone.
    pub(crate) fn new() -> Self {
        let now = Instant::now();
        let mut session = SessionState::new();
        session.clients.set_own_client(OWN, "me");
        session.clients.client_joined(ALPHA, "alpha");
        session.clients.client_joined(BRAVO, "bravo");
        session.start_game();
        session.game.is_alive = true;
        session.game.sw_frequencies = [entry(SW_FREQUENCY, 7.0)].into_iter().collect();
        session.game.lr_frequencies = [entry(LR_FREQUENCY, 5.0)].into_iter().collect();
        for client in session.clients.iter_mut() {
            client.data_frame = DataFrame::FIRST;
            client.last_activity_at = Some(now);
            client.voice_volume = 20;
            client.mark_plugin_enabled(now);
        }
        let own = session.clients.own_mut().unwrap();
        own.can_use_sw_radio = true;
        own.can_use_lr_radio = true;
        own.view_direction = facing(0.0, 1.0, 0.0);
        let mut world = Self {
            session,
            settings: PluginSettings::default(),
            antennas: AntennaManager::new(),
            processor: VoiceProcessor::new(FRAMES),
            hrir: None,
            is_serious_mode: false,
            now,
        };
        world.set_bool("voiceCone", false);
        world
    }

    /// The world with the real HRIR sphere loaded, so HRTF sources are created.
    pub(crate) fn with_hrtf() -> Self {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../assets/hrir/IRC_1002_C.bin");
        let mut world = Self::new();
        world.hrir = Some(HrirLibrary::load(path).unwrap());
        world
    }

    pub(crate) fn client(&mut self, id: TeamspeakClientId) -> &mut ClientData {
        self.session.clients.by_id_mut(id).unwrap()
    }

    pub(crate) fn set_bool(&mut self, key: &str, value: bool) {
        let text = if value { "true" } else { "false" };
        self.settings
            .set(key, text, Some(TypeTag::from_wire("BOOL")))
            .unwrap();
    }

    pub(crate) fn set_number(&mut self, key: &str, value: f64) {
        self.settings
            .set(key, &value.to_string(), Some(TypeTag::from_wire("SCALAR")))
            .unwrap();
    }

    /// `sender` holds the tangent of `radio` on `frequency` with `range` metres and `subtype`.
    pub(crate) fn transmit(
        &mut self,
        sender: TeamspeakClientId,
        radio: TangentRadio,
        frequency: &str,
        range: i32,
        subtype: RadioSubtype,
    ) {
        let client = self.client(sender);
        client.transmitting_over = Some(radio);
        client.transmitting_frequency = Some(frequency.to_owned());
        client.transmitting_subtype = Some(subtype);
        client.range = range;
        match radio {
            TangentRadio::ShortRange => client.can_use_sw_radio = true,
            TangentRadio::LongRange => client.can_use_lr_radio = true,
        }
    }

    /// A speaker radio `radio_id` on `frequency` at `position`, knob `volume`.
    pub(crate) fn add_speaker(
        &mut self,
        radio_id: &str,
        frequency: &str,
        position: Position,
        volume: f64,
        wave_z: f64,
    ) {
        self.session
            .game
            .speakers
            .entry(frequency.to_owned())
            .or_default()
            .push(Speaker {
                radio_id: radio_id.to_owned(),
                owner: None,
                position: Some(position),
                volume,
                vehicle: VehicleDescriptor::not_in_vehicle(),
                wave_z,
            });
    }

    pub(crate) fn add_tower(&mut self, object: i32, position: Position, range: f64) {
        self.antennas
            .add([Antenna::new(NetId::new(2, object), position, range).unwrap()]);
    }

    /// Creates the DSP objects `sender` needs now; how many.
    pub(crate) fn ensure(&mut self, sender: TeamspeakClientId) -> usize {
        let dsp = DspContext {
            hrir_library: self.hrir.as_deref(),
            object_interception_strength: self.settings.object_interception_strength(),
            dropout_seed: 7,
            frames_per_block: FRAMES,
        };
        ensure_sender_dsp(
            &mut self.session,
            &self.settings,
            &self.antennas,
            sender,
            &dsp,
            self.now,
        )
        .unwrap()
    }

    /// Processes a copy of `input` as `sender`'s frame.
    pub(crate) fn process(
        &mut self,
        sender: TeamspeakClientId,
        input: &StereoBuffer,
    ) -> (VoiceOutcome, StereoBuffer) {
        let mut samples = input.clone();
        let context = VoiceFrameContext {
            settings: &self.settings,
            antennas: &self.antennas,
            is_in_game: true,
            is_serious_mode: self.is_serious_mode,
            sender_metadata: None,
            now: self.now,
        };
        let outcome =
            self.processor
                .process_sender(&mut self.session, &context, sender, &mut samples);
        (outcome, samples)
    }

    /// Ensures `sender`'s DSP, then plays `frames` frames of a sine of `frequency_hz` and
    /// `amplitude` through it; every output frame in order.
    pub(crate) fn play_sine(
        &mut self,
        sender: TeamspeakClientId,
        frequency_hz: f64,
        amplitude: f64,
        frames: usize,
    ) -> Vec<StereoBuffer> {
        self.ensure(sender);
        (0..frames)
            .map(|index| {
                self.process(sender, &sine(frequency_hz, amplitude, index))
                    .1
            })
            .collect()
    }
}

/// Frame `index` of a continuous sine of `frequency_hz` at `amplitude`, both channels alike.
pub(crate) fn sine(frequency_hz: f64, amplitude: f64, index: usize) -> StereoBuffer {
    let mut buffer = StereoBuffer::with_frames(FRAMES);
    let start = index * FRAMES;
    for (frame, pair) in buffer
        .as_mut_slice()
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .enumerate()
    {
        let time = (start + frame) as f64 / SAMPLE_RATE;
        let value = (amplitude * (TAU * frequency_hz * time).sin()) as i16;
        pair[0] = value;
        pair[1] = value;
    }
    buffer
}

/// The root mean square of one channel (0 left, 1 right) over `frames`.
pub(crate) fn channel_rms(frames: &[StereoBuffer], channel: usize) -> f64 {
    let (sum, count) = frames
        .iter()
        .flat_map(|buffer| buffer.as_slice().iter().skip(channel).step_by(2))
        .fold((0.0, 0_usize), |(sum, count), &sample| {
            (sum + f64::from(sample).powi(2), count + 1)
        });
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt()
    }
}

/// The root mean square of both channels together over `frames`.
pub(crate) fn rms(frames: &[StereoBuffer]) -> f64 {
    (channel_rms(frames, 0).powi(2) + channel_rms(frames, 1).powi(2)).sqrt() / 2.0_f64.sqrt()
}

/// The frames after the first `settle`, where filters, delay lines and the HRTF have filled.
pub(crate) fn settled(frames: &[StereoBuffer], settle: usize) -> &[StereoBuffer] {
    &frames[settle..]
}

/// Whether every sample of every frame is zero.
pub(crate) fn is_silent(frames: &[StereoBuffer]) -> bool {
    frames
        .iter()
        .all(|buffer| buffer.as_slice().iter().all(|&sample| sample == 0))
}
