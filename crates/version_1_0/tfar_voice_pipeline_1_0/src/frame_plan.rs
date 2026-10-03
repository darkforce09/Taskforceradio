//! `FramePlan`: everything decided about one sender's frame before any sample is touched.
//!
//! **Role:** run the planning phase of `processVoiceData` while the session is borrowed
//! immutably: the direct-speech plan, the receptions from `is_over_radio_into` and a plan per
//! reception, and the voice and global gains, so the audio phase can borrow the sender's
//! effects mutably.
//!
//! **Position:** a module of `tfar_voice_pipeline_1_0`; built by the voice processor and the
//! DSP needs.
//!
//! **Signals and state:** none of its own; the receptions and the route plans go into the
//! caller's reusable lists, the receptions sharing their radio ids and vehicles with the
//! session.
//!
//! **Invariants:** a spectating sender is heard on no radio; the route plans match the
//! receptions one to one, in order; lists that hold every reception are refilled without
//! allocating.

use std::time::Instant;

use tfar_antennas_1_0::AntennaManager;
use tfar_client_state_1_0::prelude::{HeardVia, IgnoreTangents, SessionState, TeamspeakClientId};
use tfar_plugin_settings_1_0::PluginSettings;
use tfar_radio_routing_1_0::{RoutingContext, is_over_radio_into};

use crate::direct_speech::{DirectSpeechPlan, plan_direct_speech};
use crate::distance_gain::gain_as_f32;
use crate::radio_paths::{RoutePlan, plan_routes};
use crate::scene::Scene;
use crate::sender_gate::SpectatorView;

/// What the planning phase hands the audio phase.
#[derive(Clone, Debug)]
pub(crate) struct FramePlan {
    /// The direct-speech processing; `None` when the sender is not heard directly.
    pub(crate) direct_speech: Option<DirectSpeechPlan>,
    /// Both spectate: without direct speech the voice stays as it is instead of silence.
    pub(crate) is_pure_spectator: bool,
    /// The sender's voice volume multiplier.
    pub(crate) voice_volume_multiplier: f32,
    /// The local player's global volume.
    pub(crate) global_volume: f32,
}

/// The inputs of the planning phase besides the session.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlanInputs<'a> {
    /// The plugin settings.
    pub(crate) settings: &'a PluginSettings,
    /// The radio towers.
    pub(crate) antennas: &'a AntennaManager,
    /// The sender.
    pub(crate) sender: TeamspeakClientId,
    /// The spectator rules' view of the sender.
    pub(crate) view: SpectatorView,
    /// The captured-voice path, which skips the intercom.
    pub(crate) is_from_microphone: bool,
    /// The time of the frame.
    pub(crate) now: Instant,
}

impl FramePlan {
    /// Plans the frame of `inputs.sender` in `session`, refilling `routes` with every way the
    /// local player hears the sender over radio and `route_plans` with their plans; `None`
    /// when the sender or the local player is unknown, with both lists emptied.
    pub(crate) fn build(
        session: &SessionState,
        inputs: PlanInputs<'_>,
        routes: &mut Vec<HeardVia>,
        route_plans: &mut Vec<RoutePlan>,
    ) -> Option<Self> {
        routes.clear();
        route_plans.clear();
        let scene = Scene::of(session, inputs.settings, inputs.sender, inputs.now)?;
        if !scene.sender.is_spectating {
            let context = RoutingContext {
                session,
                settings: inputs.settings,
                antennas: inputs.antennas,
                now: inputs.now,
            };
            is_over_radio_into(&context, scene.sender, IgnoreTangents::NONE, routes);
        }
        plan_routes(&scene, routes, inputs.is_from_microphone, route_plans);
        Some(Self {
            direct_speech: plan_direct_speech(&scene, inputs.view),
            is_pure_spectator: inputs.view.is_pure_spectator,
            voice_volume_multiplier: gain_as_f32(scene.sender.voice_volume_multiplier),
            global_volume: gain_as_f32(scene.game.global_volume),
        })
    }
}
