//! The fake `Ts3Functions` table: one typed fake per function TFAR calls, and a recording stub in
//! every other slot.
//!
//! **Role:** builds the table the mock hands to `ts3plugin_setFunctionPointers`, and lists which
//! SDK functions have a typed fake ([`FAKED_FUNCTIONS`]).
//!
//! **Position:** called by [`MockHost`](crate::MockHost) when it starts a plugin; the fakes live
//! in the sibling files of this folder.
//!
//! **Signals and state:** none of its own; the stubs and fakes record into the fake state.
//!
//! **Invariants:** every one of the table's slots holds a valid `extern "C"` function, so no call
//! the plugin makes lands on garbage; a slot without a typed fake logs
//! [`HostCall::UnexpectedFunction`] with its index and answers `ERROR_not_implemented`;
//! [`FAKED_FUNCTIONS`] and the installed fakes come from one list, so they cannot drift.

use std::ffi::c_uint;
use std::mem::{MaybeUninit, offset_of};

use ts3plugin_sys::ts3functions::Ts3Functions;

use super::action_fakes as action;
use super::connection_query_fakes as query;
use super::fake_answers::ERROR_NOT_IMPLEMENTED;
use super::memory_and_log_fakes as memory_and_log;
use super::variable_fakes as variable;
use crate::HostCall;
use crate::fake_state::host_session::with_fake_teamspeak;

#[cfg(test)]
#[path = "tests/fake_table.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/fake_calls.rs"]
mod fake_call_tests;

/// The size of one table slot: a function pointer.
const SLOT_SIZE: usize = size_of::<usize>();

/// The number of slots in `Ts3Functions` (230 at plugin API 26).
pub const SLOT_COUNT: usize = size_of::<Ts3Functions>() / SLOT_SIZE;

/// The stubs cover 16 rows of 16 slots.
const STUB_ROW_LENGTH: usize = 16;

// Every slot needs its own stub, so the table may grow to the stubs' 256 slots at most.
const _: () = assert!(SLOT_COUNT <= STUB_ROW_LENGTH * STUB_ROW_LENGTH);

/// The type of a stub: it takes nothing it needs and answers `ERROR_not_implemented`.
type UnexpectedSlotStub = extern "C" fn() -> c_uint;

/// The stub of slot `SLOT`: logs the call and answers `ERROR_not_implemented`.
///
/// It reads no argument, so it is safe in a slot of any signature on the C ABI: the arguments
/// the plugin passes are ignored, and a `void` slot ignores the answer.
extern "C" fn unexpected_slot<const SLOT: usize>() -> c_uint {
    with_fake_teamspeak(|fake| fake.record(HostCall::UnexpectedFunction { slot: SLOT }));
    ERROR_NOT_IMPLEMENTED
}

/// The sixteen stubs of row `$row`.
macro_rules! sixteen_unexpected_slots {
    ($row:literal) => {
        [
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 0 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 1 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 2 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 3 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 4 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 5 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 6 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 7 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 8 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 9 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 10 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 11 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 12 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 13 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 14 }>,
            unexpected_slot::<{ $row * STUB_ROW_LENGTH + 15 }>,
        ]
    };
}

/// One stub per slot index, in rows of [`STUB_ROW_LENGTH`].
static UNEXPECTED_SLOT_STUBS: [[UnexpectedSlotStub; STUB_ROW_LENGTH]; STUB_ROW_LENGTH] = [
    sixteen_unexpected_slots!(0),
    sixteen_unexpected_slots!(1),
    sixteen_unexpected_slots!(2),
    sixteen_unexpected_slots!(3),
    sixteen_unexpected_slots!(4),
    sixteen_unexpected_slots!(5),
    sixteen_unexpected_slots!(6),
    sixteen_unexpected_slots!(7),
    sixteen_unexpected_slots!(8),
    sixteen_unexpected_slots!(9),
    sixteen_unexpected_slots!(10),
    sixteen_unexpected_slots!(11),
    sixteen_unexpected_slots!(12),
    sixteen_unexpected_slots!(13),
    sixteen_unexpected_slots!(14),
    sixteen_unexpected_slots!(15),
];

/// The stub of `slot`.
pub(crate) fn unexpected_slot_stub(slot: usize) -> UnexpectedSlotStub {
    UNEXPECTED_SLOT_STUBS
        .as_flattened()
        .get(slot)
        .copied()
        .unwrap_or(unexpected_slot::<{ usize::MAX }>)
}

/// An SDK function the mock implements with a typed fake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FakedFunction {
    /// The SDK name, as in `ts3_functions.h` (`getClientID`).
    pub sdk_name: &'static str,
    /// The slot's index in `Ts3Functions`, in SDK order.
    pub slot: usize,
}

/// Declares [`FAKED_FUNCTIONS`] and `install_faked_functions` from one list.
macro_rules! faked_functions {
    ($($sdk_name:literal => $field:ident = $fake:path,)*) => {
        /// Every SDK function with a typed fake: the 32 functions the two TFAR plugins call.
        pub const FAKED_FUNCTIONS: &[FakedFunction] = &[
            $(FakedFunction {
                sdk_name: $sdk_name,
                slot: offset_of!(Ts3Functions, $field) / SLOT_SIZE,
            },)*
        ];

        /// Puts every typed fake into its slot of `table`.
        fn install_faked_functions(table: &mut Ts3Functions) {
            $(table.$field = $fake;)*
        }
    };
}

faked_functions! {
    "getErrorMessage" => get_error_message = memory_and_log::get_error_message,
    "freeMemory" => free_memory = memory_and_log::free_memory,
    "logMessage" => log_message = memory_and_log::log_message,
    "playWaveFile" => play_wave_file = memory_and_log::play_wave_file,
    "getPreProcessorConfigValue" => get_pre_processor_config_value
        = action::get_pre_processor_config_value,
    "setPreProcessorConfigValue" => set_pre_processor_config_value
        = action::set_pre_processor_config_value,
    "systemset3DListenerAttributes" => systemset3d_listener_attributes
        = action::systemset3d_listener_attributes,
    "systemset3DSettings" => systemset3d_settings = action::systemset3d_settings,
    "channelset3DAttributes" => channelset3d_attributes = action::channelset3d_attributes,
    "requestClientMove" => request_client_move = action::request_client_move,
    "requestMuteClients" => request_mute_clients = action::request_mute_clients,
    "requestUnmuteClients" => requset_unmute_clients = action::request_unmute_clients,
    "getClientID" => get_client_id = query::get_client_id,
    "getClientSelfVariableAsInt" => get_client_self_variable_as_int
        = variable::get_client_self_variable_as_int,
    "setClientSelfVariableAsInt" => set_client_self_variable_as_int
        = variable::set_client_self_variable_as_int,
    "setClientSelfVariableAsString" => set_client_self_variable_as_string
        = variable::set_client_self_variable_as_string,
    "flushClientSelfUpdates" => flush_client_self_updates = variable::flush_client_self_updates,
    "getClientVariableAsInt" => get_client_variable_as_int = variable::get_client_variable_as_int,
    "getClientVariableAsString" => get_client_variable_as_string
        = variable::get_client_variable_as_string,
    "getClientList" => get_client_list = query::get_client_list,
    "getChannelOfClient" => get_channel_of_client = query::get_channel_of_client,
    "getChannelVariableAsString" => get_channel_variable_as_string
        = variable::get_channel_variable_as_string,
    "getChannelList" => get_channel_list = query::get_channel_list,
    "getChannelClientList" => get_channel_client_list = query::get_channel_client_list,
    "getServerConnectionHandlerList" => get_server_connection_handler_list
        = query::get_server_connection_handler_list,
    "getServerVariableAsString" => get_server_variable_as_string
        = variable::get_server_variable_as_string,
    "getConnectionStatus" => get_connection_status = query::get_connection_status,
    "getConfigPath" => get_config_path = memory_and_log::get_config_path,
    "getPluginPath" => get_plugin_path = memory_and_log::get_plugin_path,
    "getCurrentServerConnectionHandlerID" => get_current_server_connection_handler_id
        = query::get_current_server_connection_handler_id,
    "printMessageToCurrentTab" => print_message_to_current_tab
        = memory_and_log::print_message_to_current_tab,
    "sendPluginCommand" => send_plugin_command = action::send_plugin_command,
}

/// The fake table: a stub in every slot, then the typed fakes over theirs.
pub(crate) fn fake_function_table() -> Ts3Functions {
    let mut table = MaybeUninit::<Ts3Functions>::uninit();
    let slots = table.as_mut_ptr().cast::<UnexpectedSlotStub>();
    for slot in 0..SLOT_COUNT {
        // SAFETY: `Ts3Functions` is `#[repr(C)]` and made only of function pointers (the layout
        // test of `tfar_ts3_host` checks its size against the SDK headers), so slot `slot` is in
        // bounds and pointer-aligned.
        unsafe { slots.add(slot).write(unexpected_slot_stub(slot)) };
    }
    // SAFETY: every slot now holds a valid, non-null function pointer. A stub's signature differs
    // from its slot's; a stub reads no argument and returns an `unsigned int` in the return
    // register, which a C caller of any slot of this table may ignore or read.
    let mut table = unsafe { table.assume_init() };
    install_faked_functions(&mut table);
    table
}
