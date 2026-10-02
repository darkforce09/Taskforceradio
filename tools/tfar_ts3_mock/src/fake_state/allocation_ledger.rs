//! [`AllocationLedger`] and [`LeakReport`]: the memory the fakes hand the plugin with `malloc`,
//! checked off when the plugin passes it back to `freeMemory`.
//!
//! **Role:** gives the mock TeamSpeak's ownership rule for returned strings and lists (the
//! plugin frees each with `freeMemory`, exactly once) and reports every block that breaks it.
//!
//! **Position:** inside the fake state; the fakes in `fake_functions` call [`AllocationLedger::hand_out`]
//! and `free`; [`MockHost::leak_report`](crate::MockHost::leak_report) reads the report.
//!
//! **Signals and state:** the outstanding blocks by address, the frees of unknown addresses, and
//! the plugin's own `infoData` blocks the mock could not hand back; all behind the fake-state
//! mutex.
//!
//! **Invariants:** every block comes from `libc::malloc` and is released with `libc::free` only
//! when the ledger handed it out; an unknown or repeated free is recorded, never passed to `free`.

use std::collections::BTreeMap;
use std::ffi::c_void;

/// The blocks handed to the plugin and not yet freed, and the bad frees seen.
#[derive(Debug, Default)]
pub(crate) struct AllocationLedger {
    /// Outstanding blocks: address to the SDK function that handed it out.
    outstanding: BTreeMap<usize, &'static str>,
    /// Addresses passed to `freeMemory` the ledger never handed out (or already took back).
    unknown_frees: Vec<usize>,
    /// `infoData` answers the plugin allocated and the mock could not free (no `freeMemory`).
    unfreed_plugin_blocks: usize,
}

/// A block handed to the plugin that it never passed to `freeMemory`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnfreedAllocation {
    /// The SDK function whose answer the block holds.
    pub function: &'static str,
    /// The block's address.
    pub address: usize,
}

/// What [`MockHost::leak_report`](crate::MockHost::leak_report) found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LeakReport {
    /// Strings and lists the fakes handed out that the plugin never freed.
    pub unfreed_host_allocations: Vec<UnfreedAllocation>,
    /// Addresses the plugin passed to `freeMemory` that the fakes never handed out, or freed twice.
    pub unknown_frees: Vec<usize>,
    /// `infoData` answers the plugin allocated with no `ts3plugin_freeMemory` to give them back.
    pub unfreed_plugin_blocks: usize,
}

impl LeakReport {
    /// Whether every block went back to its owner exactly once.
    pub fn is_clean(&self) -> bool {
        self.unfreed_host_allocations.is_empty()
            && self.unknown_frees.is_empty()
            && self.unfreed_plugin_blocks == 0
    }
}

impl AllocationLedger {
    /// A ledger with nothing outstanding.
    pub(crate) const fn empty() -> Self {
        Self {
            outstanding: BTreeMap::new(),
            unknown_frees: Vec::new(),
            unfreed_plugin_blocks: 0,
        }
    }

    /// A `malloc`ed copy of `bytes`, recorded as handed out by `function`; null when `malloc`
    /// fails.
    pub(crate) fn hand_out(&mut self, function: &'static str, bytes: &[u8]) -> *mut c_void {
        // SAFETY: `malloc` has no precondition; a null result is returned as is.
        let block = unsafe { libc::malloc(bytes.len().max(1)) };
        if block.is_null() {
            return block;
        }
        // SAFETY: `block` holds at least `bytes.len()` fresh bytes, distinct from `bytes`.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), block.cast::<u8>(), bytes.len()) };
        self.outstanding.insert(block.expose_provenance(), function);
        block
    }

    /// Takes back `block` from `freeMemory`: frees it when the ledger handed it out, else records
    /// the address. A null `block` is a no-op, as for `free`.
    pub(crate) fn free(&mut self, block: *mut c_void) {
        if block.is_null() {
            return;
        }
        if self
            .outstanding
            .remove(&block.expose_provenance())
            .is_some()
        {
            // SAFETY: the ledger handed `block` out from `malloc` and has just forgotten it, so
            // this is its one `free`.
            unsafe { libc::free(block) };
        } else {
            self.unknown_frees.push(block.expose_provenance());
        }
    }

    /// Counts an `infoData` block the mock could not give back to the plugin.
    pub(crate) fn note_unfreed_plugin_block(&mut self) {
        self.unfreed_plugin_blocks = self.unfreed_plugin_blocks.saturating_add(1);
    }

    /// The report of what is outstanding and what went wrong.
    pub(crate) fn report(&self) -> LeakReport {
        LeakReport {
            unfreed_host_allocations: self
                .outstanding
                .iter()
                .map(|(&address, &function)| UnfreedAllocation { function, address })
                .collect(),
            unknown_frees: self.unknown_frees.clone(),
            unfreed_plugin_blocks: self.unfreed_plugin_blocks,
        }
    }

    /// Frees every outstanding block (the session is over and the plugin is gone).
    pub(crate) fn release_all(&mut self) {
        for (address, _) in std::mem::take(&mut self.outstanding) {
            // SAFETY: every outstanding address came from `malloc` in `hand_out` and has not been
            // freed; it is removed from the ledger with this one `free`.
            unsafe { libc::free(std::ptr::with_exposed_provenance_mut::<c_void>(address)) };
        }
    }
}
