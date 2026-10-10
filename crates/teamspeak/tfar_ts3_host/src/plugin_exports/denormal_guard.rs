//! `DenormalGuard`: flush-to-zero and denormals-are-zero for the length of one voice callback.
//!
//! **Role:** sets the SSE control register's FTZ and DAZ bits on entry to a voice callback and
//! restores the caller's register on exit, as the original plugins do with
//! `_MM_SET_FLUSH_ZERO_MODE` and `_MM_SET_DENORMALS_ZERO_MODE`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2652-2653`). A decaying filter, compressor or HRTF
//! tail otherwise reaches denormal numbers, whose arithmetic is many times slower and can make a
//! callback late.
//!
//! **Position:** made by the audio glue around each voice block; on targets other than x86-64 it
//! does nothing.
//!
//! **Signals and state:** the caller's MXCSR value, held from creation to drop on TeamSpeak's
//! audio thread; nothing is shared.
//!
//! **Invariants:** the register is restored exactly as it was when the guard drops, so TeamSpeak's
//! own code after the callback sees its own floating-point mode.

#[cfg(test)]
#[path = "tests/denormal_guard.rs"]
mod tests;

/// The MXCSR bit that flushes denormal results to zero.
#[cfg(target_arch = "x86_64")]
const FLUSH_TO_ZERO: u32 = 1 << 15;

/// The MXCSR bit that reads denormal inputs as zero.
#[cfg(target_arch = "x86_64")]
const DENORMALS_ARE_ZERO: u32 = 1 << 6;

/// Flush-to-zero and denormals-are-zero until dropped; see the module header.
#[derive(Debug)]
pub(super) struct DenormalGuard {
    #[cfg(target_arch = "x86_64")]
    saved: u32,
}

impl DenormalGuard {
    /// Saves the control register and turns both bits on.
    #[cfg(target_arch = "x86_64")]
    pub(super) fn new() -> Self {
        let saved = read_control_register();
        write_control_register(saved | FLUSH_TO_ZERO | DENORMALS_ARE_ZERO);
        Self { saved }
    }

    /// Does nothing on this target.
    #[cfg(not(target_arch = "x86_64"))]
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl Default for DenormalGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DenormalGuard {
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        write_control_register(self.saved);
    }
}

/// The current MXCSR value.
#[cfg(target_arch = "x86_64")]
fn read_control_register() -> u32 {
    let mut value: u32 = 0;
    // SAFETY: `stmxcsr` stores the 32-bit control register into `value`, a live, aligned local
    // the pointer names for the instruction's duration; it touches no other memory or flag.
    unsafe {
        core::arch::asm!(
            "stmxcsr [{value}]",
            value = in(reg) &raw mut value,
            options(nostack, preserves_flags),
        );
    }
    value
}

/// Loads `value` into MXCSR.
#[cfg(target_arch = "x86_64")]
fn write_control_register(value: u32) {
    // SAFETY: `ldmxcsr` reads 32 bits from `value`, a live, aligned local; every caller passes a
    // value read from the register with only the FTZ and DAZ bits changed, so no reserved bit is
    // set and the instruction cannot fault.
    unsafe {
        core::arch::asm!(
            "ldmxcsr [{value}]",
            value = in(reg) &raw const value,
            options(nostack, preserves_flags, readonly),
        );
    }
}
