//! The items the expansion of [`ts3_plugin!`](crate::ts3_plugin) calls, gathered under one path
//! (`$crate::export_glue`). Public because a macro expands in the plugin app's crate; hidden from
//! the docs because only the macro uses it.

pub use super::audio_glue::{
    RawVoiceBlock, captured_voice_data, custom_3d_rolloff_calculation_client,
    mixed_playback_voice_data, post_process_voice_data,
};
pub use super::event_glue::{
    InvokedMoveKind, PlainMoveKind, RawClientMove, RawInvoker, channel_move,
    client_self_variable_update, connect_status_change, delete_channel, invoked_client_move,
    new_channel, new_channel_created, plain_client_move, plugin_command, server_error,
    server_permission_error, update_channel, update_channel_edited, update_client,
};
pub use super::metadata_glue::MetadataText;
pub use super::metadata_glue::{
    API_VERSION, OFFERS_CONFIGURE, REQUESTS_AUTOLOAD, current_server_connection_changed,
    free_memory, info_data, init, metadata_text, process_command, register_plugin_id,
    set_function_pointers, shutdown,
};
pub use ts3plugin_sys::ts3functions::Ts3Functions;
