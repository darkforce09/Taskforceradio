//! `DelayLine`: the 2400-slot delay every speaker effect runs (`RadioEffect::delay`,
//! `reference/tfar_1_0/ts/src/RadioEffect.hpp:32`).

#[cfg(test)]
#[path = "tests/delay_line.rs"]
mod tests;

/// The slots of the delay line: a twentieth of a second at 48 kHz.
pub(crate) const DELAY_LINE_SLOTS: usize = 48_000 / 20;

/// A ring of [`DELAY_LINE_SLOTS`] samples. Each call writes the input at the current slot and
/// reads the next one, so a sample comes back `DELAY_LINE_SLOTS − 1` = 2399 calls later.
#[derive(Debug, Clone)]
pub(crate) struct DelayLine {
    slots: Box<[f32; DELAY_LINE_SLOTS]>,
    position: usize,
}

impl DelayLine {
    /// A silent delay line. Allocates: call it off the audio thread.
    pub(crate) fn new() -> Self {
        Self {
            slots: Box::new([0.0; DELAY_LINE_SLOTS]),
            position: 0,
        }
    }

    /// Stores `input` and returns the sample stored 2399 calls ago (silence at first).
    pub(crate) fn delay(&mut self, input: f32) -> f32 {
        if let Some(slot) = self.slots.get_mut(self.position) {
            *slot = input;
        }
        let next = (self.position + 1) % DELAY_LINE_SLOTS;
        self.position = next;
        self.slots.get(next).copied().unwrap_or(0.0)
    }
}
