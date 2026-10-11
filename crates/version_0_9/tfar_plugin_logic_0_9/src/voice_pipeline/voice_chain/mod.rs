//! The stereo function's steps: the plan of a sender's frame and its render over the sender's
//! sound state.

mod direct_voice;
mod frame_render;
mod radio_reception;
mod sender_plan;
mod sound_placement;

pub(crate) use frame_render::render_frame;
pub(crate) use sender_plan::{PlannedRoute, plan_sender};
