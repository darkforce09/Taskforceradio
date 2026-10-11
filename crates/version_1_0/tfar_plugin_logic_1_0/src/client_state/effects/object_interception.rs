//! The object-interception low-passes of one sender, one per object count.
//!
//! **Role:** design and lend the low-pass that muffles direct speech through objects, one per
//! object count 0 to 5, as `getFilterObjectInterception`
//! (`reference/tfar_1_0/ts/src/clientData.cpp:53-74`).
//!
//! **Position:** a module of the crate's `effects`; part of [`crate::client_state::ClientEffects`].
//!
//! **Signals and state:** the designed slots and the strength they were designed for; designed
//! off the audio thread, borrowed on it.
//!
//! **Invariants:** the slots always match the strength they record (a new strength clears them);
//! a cutoff at or below 0 Hz is silence, never a degenerate filter.

use tfar_audio::dsp_filters::{FilterDesign, StereoFilter};

use crate::client_state::effects::filter_designs::{object_interception_cutoff_hz, stereo};
use crate::client_state::error::Result;

/// The most objects that change the cutoff; more count as this many.
const MOST_OBJECTS: usize = 5;

/// The object-interception low-pass for one object count, as the audio path borrows it.
#[derive(Debug)]
pub enum ObjectInterceptionFilter<'a> {
    /// The low-pass to run.
    Filter(&'a mut StereoFilter),
    /// The cutoff is at or below 0 Hz: the objects block the sound entirely.
    Silence,
}

/// One designed slot.
#[derive(Debug)]
enum DesignedSlot {
    /// A low-pass at a positive cutoff.
    Filter(StereoFilter),
    /// A cutoff at or below 0 Hz.
    Silence,
}

/// One low-pass slot per object count 0 to 5, designed for one strength.
#[derive(Debug, Default)]
pub(crate) struct ObjectInterceptionFilters {
    /// The strength the slots were designed with; a different one clears them.
    strength: Option<f32>,
    /// Per object count, `None` until designed.
    slots: [Option<DesignedSlot>; MOST_OBJECTS + 1],
}

impl ObjectInterceptionFilters {
    /// Designs the slot of `object_count` at `strength`, first clearing every slot designed for
    /// another strength. Returns whether a slot was designed.
    pub(crate) fn ensure(&mut self, object_count: i32, strength: f32) -> Result<bool> {
        if self
            .strength
            .is_none_or(|known| known.to_bits() != strength.to_bits())
        {
            self.slots = Default::default();
            self.strength = Some(strength);
        }
        let Some(slot) = self.slots.get_mut(slot_index(object_count)) else {
            return Ok(false);
        };
        if slot.is_some() {
            return Ok(false);
        }
        let cutoff_hz = object_interception_cutoff_hz(object_count, strength);
        *slot = Some(if cutoff_hz > 0.0 {
            DesignedSlot::Filter(stereo(FilterDesign::ButterworthLowPass {
                order: 2,
                cutoff_hz,
            })?)
        } else {
            DesignedSlot::Silence
        });
        Ok(true)
    }

    /// The designed slot of `object_count`; `None` when it is not designed yet.
    pub(crate) fn get_mut(&mut self, object_count: i32) -> Option<ObjectInterceptionFilter<'_>> {
        match self.slots.get_mut(slot_index(object_count))?.as_mut()? {
            DesignedSlot::Filter(filter) => Some(ObjectInterceptionFilter::Filter(filter)),
            DesignedSlot::Silence => Some(ObjectInterceptionFilter::Silence),
        }
    }

    /// The strength the slots are designed for.
    pub(crate) const fn strength(&self) -> Option<f32> {
        self.strength
    }
}

/// The slot of `object_count`: the count clamped to 0..=5.
fn slot_index(object_count: i32) -> usize {
    usize::try_from(object_count.clamp(0, 5)).unwrap_or(0)
}
