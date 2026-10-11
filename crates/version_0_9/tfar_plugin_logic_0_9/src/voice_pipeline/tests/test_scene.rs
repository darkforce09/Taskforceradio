//! The synthetic scene every scenario test of the crate builds on: a listener, a sender, a
//! loudspeaker owner, the real HRIR sphere, tone and noise frames, and level measurements.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test signals are generated in f64 and truncated to samples as the originals do; \
              sample indices and counts stay far below 2^52"
)]

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::client_state::prelude::{
    Position, RadioSubtype, StereoMode, TangentKind, UsableRadios, VehicleDescriptor,
};
use crate::client_state::{ClientData, ClientId, FrequencySettings, ServerRadioData, SpeakerRadio};
use crate::radio_routing::dsp_requirements;
use tfar_audio::hrtf::HrirLibrary;
use tfar_audio::sample_buffer::StereoBuffer;

use crate::voice_pipeline::voice_context::{VoiceFrameContext, VoiceOutcome};
use crate::voice_pipeline::voice_processor::VoiceProcessor;

/// The local player, `Me`.
pub(crate) const ME: ClientId = ClientId(1);
/// The sender, `Alpha`.
pub(crate) const ALPHA: ClientId = ClientId(2);
/// The owner of the loudspeaker radio, `Bravo`.
pub(crate) const BRAVO: ClientId = ClientId(3);
/// The frequency `Alpha` transmits on and the listener's short-range radio listens to.
pub(crate) const FREQUENCY: &str = "30.1";
/// TeamSpeak's usual voice frame: 10 ms at 48 kHz.
pub(crate) const FRAMES: usize = 480;
/// The data frame every entry of the scene carries.
const DATA_FRAME: i32 = 4;

/// The real sphere the plugins ship, loaded once for every test.
pub(crate) fn real_library() -> Arc<HrirLibrary> {
    static LIBRARY: OnceLock<Arc<HrirLibrary>> = OnceLock::new();
    LIBRARY
        .get_or_init(|| {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../assets/hrir/IRC_1002_C.bin");
            HrirLibrary::load(path).expect("the shipped sphere loads")
        })
        .clone()
}

/// A position from its coordinates.
pub(crate) fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

/// A vehicle descriptor, shared as the client state holds it.
pub(crate) fn vehicle(name: &str, isolation: f64) -> Arc<VehicleDescriptor> {
    Arc::new(VehicleDescriptor {
        vehicle_name: name.to_owned(),
        isolation,
    })
}

/// A listener, a sender and a loudspeaker owner on one server, with the processor and the
/// sphere the scene renders with.
pub(crate) struct Scene {
    pub(crate) state: ServerRadioData,
    pub(crate) processor: VoiceProcessor,
    pub(crate) hrir: Option<Arc<HrirLibrary>>,
    pub(crate) now: Instant,
}

impl Scene {
    /// `Me` alive at the origin looking north, `Alpha` 10 m north with a 20 m voice, `Bravo`
    /// 50 m east; everyone current, on foot, able to use every radio; the listener has a
    /// short-range radio on [`FREQUENCY`] at volume 7. No HRIR sphere: directions use the ILD.
    pub(crate) fn new() -> Self {
        let mut state = ServerRadioData::new("Me".to_owned());
        state.current_data_frame = DATA_FRAME;
        state.is_alive = true;
        state.sw_frequencies.insert(
            FREQUENCY.to_owned(),
            FrequencySettings {
                volume: 7,
                stereo_mode: StereoMode::Both,
            },
        );
        for (nickname, id, position) in [
            ("Me", ME, Position::ORIGIN),
            ("Alpha", ALPHA, at(0.0, 10.0, 0.0)),
            ("Bravo", BRAVO, at(50.0, 0.0, 0.0)),
        ] {
            let mut client = ClientData::new(id);
            client.data_frame = DATA_FRAME;
            client.position = position;
            client.position_time = Some(Instant::now());
            client.voice_volume = 20;
            client.usable_radios = UsableRadios {
                short_range: true,
                long_range: true,
                diver: true,
            };
            client.vehicle = vehicle("no", 0.0);
            state.clients.insert(nickname.to_owned(), client);
        }
        Self {
            state,
            processor: VoiceProcessor::new(4 * FRAMES),
            hrir: None,
            now: Instant::now(),
        }
    }

    /// [`Scene::new`] rendering directions through the real HRIR sphere.
    pub(crate) fn with_hrtf() -> Self {
        Self {
            hrir: Some(real_library()),
            ..Self::new()
        }
    }

    /// The entry of `nickname`, to change.
    pub(crate) fn client(&mut self, nickname: &str) -> &mut ClientData {
        self.state.client_mut(nickname).unwrap()
    }

    /// `nickname` holds the push-to-talk of `kind` on [`FREQUENCY`] with a radio of `subtype`
    /// and `range` metres.
    pub(crate) fn transmit(
        &mut self,
        nickname: &str,
        kind: TangentKind,
        subtype: RadioSubtype,
        range: i32,
    ) {
        let client = self.client(nickname);
        client.tangent = Some(kind);
        client.frequency = FREQUENCY.to_owned();
        client.range = range;
        client.subtype = subtype;
    }

    /// A loudspeaker radio of `Bravo` on [`FREQUENCY`] at `position`, at `volume`.
    pub(crate) fn add_loudspeaker(&mut self, position: Position, volume: i32, wave_z: f64) {
        self.state
            .speakers
            .entry(FREQUENCY.to_owned())
            .or_default()
            .push(SpeakerRadio {
                radio_id: "bravo_speaker".into(),
                owner_nickname: "Bravo".to_owned(),
                position: Some(position),
                volume,
                vehicle: vehicle("no", 0.0),
                wave_z,
            });
    }

    /// Makes the sound state `nickname`'s voice needs now, as the plugin does after a command.
    pub(crate) fn prepare(&mut self, nickname: &str) {
        let requirements = dsp_requirements(&self.state, nickname, ME).unwrap();
        let hrir = self.hrir.clone();
        self.client(nickname)
            .ensure_dsp(&requirements, hrir.as_deref())
            .unwrap();
    }

    /// The context of a frame from `sender`, whose metadata shows the plugin, in the channel
    /// `Lobby`, in game.
    pub(crate) fn context(sender: ClientId) -> VoiceFrameContext<'static> {
        VoiceFrameContext {
            sender,
            listener: ME,
            sender_metadata_shows_plugin: true,
            sender_channel_name: "Lobby",
            is_in_game: true,
        }
    }

    /// Processes one frame of `sender` with `context`.
    pub(crate) fn process_with(
        &mut self,
        context: &VoiceFrameContext<'_>,
        frame: &mut StereoBuffer,
    ) -> VoiceOutcome {
        self.processor
            .process_sender(&mut self.state, context, frame, self.now)
    }

    /// Runs `frames` frames of [`FRAMES`] of `signal` from `sender` through the processor and
    /// returns the output, interleaved stereo.
    pub(crate) fn run(&mut self, sender: ClientId, signal: &Signal, frames: usize) -> Vec<i16> {
        let context = Self::context(sender);
        let mut output = Vec::with_capacity(frames * FRAMES * 2);
        let mut frame = StereoBuffer::with_frames(FRAMES);
        for index in 0..frames {
            signal.fill(&mut frame, index * FRAMES, FRAMES);
            self.process_with(&context, &mut frame);
            output.extend_from_slice(frame.as_slice());
        }
        output
    }
}

/// A test signal, the same in both channels.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Signal {
    /// A sine of this frequency in Hz at this amplitude.
    Tone(f64, f64),
    /// Uniform white noise at this amplitude.
    Noise(f64),
}

impl Signal {
    /// Fills `frame` with `frames` frames of the signal from frame `start` on.
    pub(crate) fn fill(&self, frame: &mut StereoBuffer, start: usize, frames: usize) {
        frame.set_frames(frames);
        for (offset, pair) in frame
            .as_mut_slice()
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .enumerate()
        {
            let sample = self.sample(start + offset) as i16;
            *pair = [sample, sample];
        }
    }

    /// Sample `index` of the signal.
    fn sample(&self, index: usize) -> f64 {
        match *self {
            Self::Tone(hz, amplitude) => {
                amplitude * (2.0 * std::f64::consts::PI * hz * index as f64 / 48_000.0).sin()
            }
            Self::Noise(amplitude) => {
                let mut state = (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5DEE_CE66;
                state ^= state >> 29;
                state = state.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                state ^= state >> 32;
                amplitude * ((state % 20_001) as f64 / 10_000.0 - 1.0)
            }
        }
    }
}

/// The root-mean-square level of the left and the right channel of `output` after its first
/// `skip_frames` frames.
pub(crate) fn channel_rms(output: &[i16], skip_frames: usize) -> (f64, f64) {
    let pairs: Vec<&[i16; 2]> = output.as_chunks::<2>().0.iter().skip(skip_frames).collect();
    let mean_square = |channel: usize| {
        pairs
            .iter()
            .map(|pair| f64::from(pair[channel]).powi(2))
            .sum::<f64>()
            / pairs.len().max(1) as f64
    };
    (mean_square(0).sqrt(), mean_square(1).sqrt())
}

/// The root-mean-square level of both channels of `output` together after `skip_frames` frames.
pub(crate) fn rms(output: &[i16], skip_frames: usize) -> f64 {
    let (left, right) = channel_rms(output, skip_frames);
    f64::midpoint(left * left, right * right).sqrt()
}

/// `numerator` over `denominator` in decibels.
pub(crate) fn decibels(numerator: f64, denominator: f64) -> f64 {
    20.0 * (numerator / denominator).log10()
}
