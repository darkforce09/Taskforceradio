//! The radio paths: the plan of every reception of a sender, and its audio-path half.

mod plan;
mod render;

pub(crate) use plan::{
    CarriedRadioSound, RoutePlan, SpeakerMuffling, plan_routes, speaker_vehicle_loss,
};
pub(crate) use render::render_route;
