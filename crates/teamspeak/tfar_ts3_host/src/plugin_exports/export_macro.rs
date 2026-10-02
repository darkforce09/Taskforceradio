//! [`ts3_plugin!`](crate::ts3_plugin): expands to every `ts3plugin_*` export the TFAR plugins need.
//!
//! **Role:** the C symbols TeamSpeak looks up with `dlsym`, each a thin `extern "C"` shell that
//! forwards its raw arguments to `$crate::export_glue`, where the conversion and the
//! [`Ts3Plugin`](crate::Ts3Plugin) call live.
//!
//! **Position:** used once by each plugin app, at its crate root.
//!
//! **Signals and state:** the plugin is a `static` of the app; the shells hold no state. Threads
//! as in [`Ts3Plugin`](crate::Ts3Plugin): audio callbacks on TeamSpeak's audio threads, the rest
//! on its main thread.
//!
//! **Invariants:** the signatures match `src/plugin.h` of the SDK at API 26 argument for argument;
//! no shell panics (the glue turns every failure into a return value), and with `panic = "abort"`
//! nothing can unwind into TeamSpeak.

/// Expands to every `ts3plugin_*` export for the plugin `static` given.
///
/// ```ignore
/// static PLUGIN: MyPlugin = MyPlugin::new();
/// tfar_ts3_host::ts3_plugin!(PLUGIN);
/// ```
///
/// The type of `PLUGIN` implements [`Ts3Plugin`](crate::Ts3Plugin). The crate expanding the macro
/// lifts `unsafe_code`, since the exports are `#[unsafe(no_mangle)]`.
#[macro_export]
macro_rules! ts3_plugin {
    ($plugin:path) => {
        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_name() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(&$plugin, $crate::export_glue::MetadataText::Name)
        }

        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_version() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(&$plugin, $crate::export_glue::MetadataText::Version)
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_apiVersion() -> ::std::ffi::c_int {
            $crate::export_glue::API_VERSION
        }

        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_author() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(&$plugin, $crate::export_glue::MetadataText::Author)
        }

        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_description() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(
                &$plugin,
                $crate::export_glue::MetadataText::Description,
            )
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_setFunctionPointers(functions: $crate::export_glue::Ts3Functions) {
            $crate::export_glue::set_function_pointers(functions);
        }

        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_init() -> ::std::ffi::c_int {
            $crate::export_glue::init(&$plugin)
        }

        #[unsafe(no_mangle)]
        extern "C" fn ts3plugin_shutdown() {
            $crate::export_glue::shutdown(&$plugin);
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_offersConfigure() -> ::std::ffi::c_int {
            $crate::export_glue::OFFERS_CONFIGURE
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_registerPluginID(id: *const ::std::ffi::c_char) {
            // SAFETY: TeamSpeak passes a NUL-terminated ID valid for this call.
            unsafe { $crate::export_glue::register_plugin_id(&$plugin, id) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_commandKeyword() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(
                &$plugin,
                $crate::export_glue::MetadataText::CommandKeyword,
            )
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_processCommand(
            server: u64,
            command: *const ::std::ffi::c_char,
        ) -> ::std::ffi::c_int {
            // SAFETY: TeamSpeak passes a NUL-terminated command valid for this call.
            unsafe { $crate::export_glue::process_command(&$plugin, server, command) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_currentServerConnectionChanged(server: u64) {
            $crate::export_glue::current_server_connection_changed(&$plugin, server);
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_infoTitle() -> *const ::std::ffi::c_char {
            $crate::export_glue::metadata_text(
                &$plugin,
                $crate::export_glue::MetadataText::InfoTitle,
            )
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_infoData(
            server: u64,
            id: u64,
            item_type: ::std::ffi::c_int,
            data: *mut *mut ::std::ffi::c_char,
        ) {
            // SAFETY: TeamSpeak passes a writable `char*` slot for the answer.
            unsafe { $crate::export_glue::info_data(&$plugin, server, id, item_type, data) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_freeMemory(data: *mut ::std::ffi::c_void) {
            // SAFETY: TeamSpeak passes back only blocks `ts3plugin_infoData` handed it, once each.
            unsafe { $crate::export_glue::free_memory(data) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_requestAutoload() -> ::std::ffi::c_int {
            $crate::export_glue::REQUESTS_AUTOLOAD
        }

        $crate::__ts3_plugin_events!($plugin);
        $crate::__ts3_plugin_audio!($plugin);
    };
}
