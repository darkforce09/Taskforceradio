//! The DSP objects a sender's frames will need, and their creation off the audio thread.
//!
//! **Role:** list, from the same planning the audio path runs, every filter, radio effect and
//! HRTF source a sender's next frames borrow, and create the missing ones through
//! `ClientEffects::ensure_dsp`, so the audio path finds them and allocates nothing.
//!
//! **Position:** a module of `tfar_voice_pipeline_1_0`; called by the plugin app off the audio
//! thread (after `POS`, a tangent, `SPEAKERS`, a settings change, or from the service loop),
//! under the server lock.
//!
//! **Signals and state:** none of its own; it writes the sender's `ClientEffects`.
//!
//! **Invariants:** the keys and the vehicle losses are computed by the functions the audio path
//! uses, so an ensured object is the one the next frame looks up; an HRTF source (about 2 MB) is
//! created only for a sender heard directly now and for speaker radios.

use std::time::Instant;

use tfar_antennas_1_0::AntennaManager;
use tfar_client_state_1_0::prelude::{
    CANT_SPEAK_FILTER_KEY, ClientData, DspContext, DspRequirement, HeardVia, INTERCOM_RADIO_ID,
    SessionState, TeamspeakClientId, VEHICLE_FILTER_KEY, VOICE_HRTF_KEY,
};
use tfar_plugin_settings_1_0::PluginSettings;

use crate::direct_speech::VehicleRelation;
use crate::error::Result;
use crate::frame_plan::{FramePlan, PlanInputs};
use crate::radio_paths::{CarriedRadioSound, RoutePlan, SpeakerMuffling, speaker_vehicle_loss};
use crate::scene::Scene;
use crate::sender_gate::SpectatorView;

/// Creates every DSP object the frames of `sender` need at `now` and does not have yet; returns
/// how many it created. Allocates and designs filters: call it off the audio thread. An unknown
/// sender, or no local player, needs nothing.
///
/// # Errors
///
/// [`crate::Error::Dsp`] with the first refused design; the other objects are still created.
pub fn ensure_sender_dsp(
    session: &mut SessionState,
    settings: &PluginSettings,
    antennas: &AntennaManager,
    sender: TeamspeakClientId,
    dsp: &DspContext<'_>,
    now: Instant,
) -> Result<usize> {
    let mut route_plans = Vec::new();
    let Some(needs) =
        SenderDspNeeds::plan(session, settings, antennas, sender, now, &mut route_plans)
    else {
        return Ok(0);
    };
    let requirements = needs.requirements(&route_plans, dsp.hrir_library.is_some());
    let Some(client) = session.clients.by_id_mut(sender) else {
        return Ok(0);
    };
    Ok(client.effects.ensure_dsp(&requirements, dsp)?)
}

/// [`ensure_sender_dsp`] for every client of the session, the local player included (whose
/// captured voice plays on speakers); returns how many objects it created.
///
/// # Errors
///
/// [`crate::Error::Dsp`] with the first refused design; every client is still visited.
pub fn ensure_every_sender_dsp(
    session: &mut SessionState,
    settings: &PluginSettings,
    antennas: &AntennaManager,
    dsp: &DspContext<'_>,
    now: Instant,
) -> Result<usize> {
    let senders: Vec<TeamspeakClientId> =
        session.clients.iter().map(ClientData::client_id).collect();
    let mut created = 0_usize;
    let mut first_error = None;
    for sender in senders {
        match ensure_sender_dsp(session, settings, antennas, sender, dsp, now) {
            Ok(count) => created = created.saturating_add(count),
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    first_error.map_or(Ok(created), Err)
}

/// What one sender needs, planned as the audio path plans its frames.
struct SenderDspNeeds {
    /// The direct-speech vehicle relation; its loss keys the vehicle low-pass.
    vehicles: VehicleRelation,
    /// The objects between the two now.
    object_count: i32,
    /// The sender is heard directly now (the voice HRTF source is needed).
    is_heard_directly: bool,
    /// The sender's receptions now.
    routes: Vec<HeardVia>,
    /// The vehicle loss of each reception (0 for anything but a speaker).
    route_losses: Vec<f64>,
}

impl SenderDspNeeds {
    /// Plans `sender` in `session` at `now`, filling `route_plans`.
    fn plan(
        session: &SessionState,
        settings: &PluginSettings,
        antennas: &AntennaManager,
        sender: TeamspeakClientId,
        now: Instant,
        route_plans: &mut Vec<RoutePlan>,
    ) -> Option<Self> {
        let scene = Scene::of(session, settings, sender, now)?;
        let inputs = PlanInputs {
            settings,
            antennas,
            sender,
            view: SpectatorView::of(scene.listener, scene.sender, settings),
            is_from_microphone: scene.is_own_voice,
            now,
        };
        let mut routes = Vec::new();
        let plan = FramePlan::build(session, inputs, &mut routes, route_plans)?;
        let route_losses = routes
            .iter()
            .map(|route| speaker_vehicle_loss(&scene, route))
            .collect();
        Some(Self {
            vehicles: VehicleRelation::of(&scene),
            object_count: scene.sender.object_interception,
            is_heard_directly: plan.direct_speech.is_some(),
            routes,
            route_losses,
        })
    }

    /// The requirements of the planned frames, for `route_plans` of the same planning; HRTF
    /// sources only when `has_hrtf`.
    fn requirements<'a>(
        &'a self,
        route_plans: &[RoutePlan],
        has_hrtf: bool,
    ) -> Vec<DspRequirement<'a>> {
        let mut requirements = vec![DspRequirement::CantSpeakFilter(CANT_SPEAK_FILTER_KEY)];
        if has_hrtf && self.is_heard_directly {
            requirements.push(DspRequirement::Hrtf(VOICE_HRTF_KEY));
        }
        if !self.vehicles.is_unmuffled() {
            requirements.push(DspRequirement::VehicleFilter {
                key: VEHICLE_FILTER_KEY,
                vehicle_volume_loss: self.vehicles.loss,
            });
        }
        if self.object_count > 0 {
            requirements.push(DspRequirement::ObjectInterceptionFilter {
                object_count: self.object_count,
            });
        }
        let routes = self.routes.iter().zip(&self.route_losses);
        for ((route, &loss), plan) in routes.zip(route_plans) {
            route_requirements(&route.radio_id, loss, plan, has_hrtf, &mut requirements);
        }
        requirements
    }
}

/// The requirements of one reception with radio id `key`, planned as `plan`.
fn route_requirements<'a>(
    key: &'a str,
    loss: f64,
    plan: &RoutePlan,
    has_hrtf: bool,
    requirements: &mut Vec<DspRequirement<'a>>,
) {
    match plan {
        RoutePlan::Skipped => {}
        RoutePlan::Carried { sound, .. } => match sound {
            // A short-range radio turns into a diver radio as the sender dives: both are kept.
            CarriedRadioSound::Personal | CarriedRadioSound::UnderWater => {
                requirements.push(DspRequirement::PersonalRadio(key));
                requirements.push(DspRequirement::UnderWaterRadio(key));
            }
            CarriedRadioSound::Airborne => requirements.push(DspRequirement::AirborneRadio(key)),
            CarriedRadioSound::LongRange => requirements.push(DspRequirement::LongRangeRadio(key)),
            CarriedRadioSound::Phone => requirements.push(DspRequirement::PhoneFilter(key)),
            CarriedRadioSound::Silence => {}
        },
        RoutePlan::Speaker(speaker) => {
            requirements.push(DspRequirement::SpeakerFilter(key));
            requirements.push(DspRequirement::PersonalRadio(key));
            if matches!(speaker.muffling, SpeakerMuffling::Vehicle { .. }) {
                requirements.push(DspRequirement::VehicleFilter {
                    key,
                    vehicle_volume_loss: loss,
                });
            }
            if speaker.is_under_water {
                requirements.push(DspRequirement::CantSpeakFilter(key));
            }
            if has_hrtf {
                requirements.push(DspRequirement::Hrtf(key));
            }
        }
        RoutePlan::Intercom { .. } => {
            requirements.push(DspRequirement::LongRangeRadio(INTERCOM_RADIO_ID));
        }
    }
}
