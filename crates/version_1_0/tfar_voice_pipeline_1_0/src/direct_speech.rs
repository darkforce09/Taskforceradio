//! Direct speech: the plan of how a sender is heard without radio, and its audio-path half.

mod plan;
mod render;

pub(crate) use plan::{DirectSpeechPlan, VehicleRelation, plan_direct_speech};
pub(crate) use render::render_direct_speech;
