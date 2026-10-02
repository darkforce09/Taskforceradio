//! The `RVExtension` and `RVExtensionVersion` exports an Arma extension DLL offers.
//!
//! **Role:** the [`arma_extension!`] macro an extension app expands once at its crate root, and the
//! two functions its exports call: [`answer_extension_call`] and [`answer_extension_version`]. They
//! hold every pointer check, so the generated exports are one line each.
//!
//! **Position:** called by Arma through the exports the app's expansion writes; calls the app's
//! handler and [`write_reply`]. Arma x64 resolves the undecorated names `RVExtension` and
//! `RVExtensionVersion`; on `x86_64` Windows `extern "system"` is the C ABI, so the names stay
//! undecorated. No 32-bit build is supported.
//!
//! **Signals and state:** none; each call runs on Arma's calling thread and keeps no pointer.
//!
//! **Invariants:** a null pointer or a non-positive size writes nothing; the input is read as bytes
//! up to its NUL and decoded lossily, so valid UTF-8 reaches the handler unchanged; the reply never
//! passes `output_size − 1` bytes plus a NUL (the extensions spec, `outputSize` includes the NUL).

use core::ffi::{CStr, c_char, c_int};

use crate::reply_buffer::write_reply;

#[cfg(test)]
#[path = "tests/extension_exports.rs"]
mod tests;

/// Writes the `RVExtension` and `RVExtensionVersion` exports into the crate that expands it.
///
/// `handler` is a `fn(&str) -> String`: it receives the call text (decoded lossily, unchanged where
/// it is valid UTF-8) and returns the reply, which is truncated to Arma's buffer. `version` is a
/// `&'static str` written by `RVExtensionVersion`. Expand it once per DLL, at the crate root of the
/// app, which states `#![allow(unsafe_code)]` for its export glue:
///
/// ```ignore
/// tfar_arma_extension_abi::arma_extension!(handler: answer_game_call, version: "1.0.0");
/// ```
#[macro_export]
macro_rules! arma_extension {
    (handler: $handler:expr, version: $version:expr $(,)?) => {
        /// The `RVExtension` export: Arma calls it for `callExtension "<text>"` and reads the reply
        /// from `output`.
        ///
        /// # Safety
        ///
        /// `output` is null or points to `output_size` writable bytes, and `function` is null or
        /// points to a NUL-terminated string; both are owned by Arma and valid for the call.
        #[allow(non_snake_case, reason = "Arma resolves the export by this exact name")]
        #[unsafe(no_mangle)]
        pub unsafe extern "system" fn RVExtension(
            output: *mut ::core::ffi::c_char,
            output_size: ::core::ffi::c_int,
            function: *const ::core::ffi::c_char,
        ) {
            let handler: fn(&str) -> ::std::string::String = $handler;
            // SAFETY: Arma owns `output` (`output_size` bytes) and `function` (NUL-terminated) for
            // the duration of this call, which is the contract `answer_extension_call` states.
            unsafe { $crate::answer_extension_call(output, output_size, function, handler) }
        }

        /// The `RVExtensionVersion` export: Arma calls it once when it loads the DLL and logs the
        /// version written to `output`.
        ///
        /// # Safety
        ///
        /// `output` is null or points to `output_size` writable bytes owned by Arma for the call.
        #[allow(non_snake_case, reason = "Arma resolves the export by this exact name")]
        #[unsafe(no_mangle)]
        pub unsafe extern "system" fn RVExtensionVersion(
            output: *mut ::core::ffi::c_char,
            output_size: ::core::ffi::c_int,
        ) {
            let version: &'static str = $version;
            // SAFETY: Arma owns `output` (`output_size` bytes) for the duration of this call, which
            // is the contract `answer_extension_version` states.
            unsafe { $crate::answer_extension_version(output, output_size, version) }
        }
    };
}

/// Answers one `RVExtension` call: reads `function`, calls `handler` and writes its reply.
///
/// A null `output` or an `output_size` of zero or less writes nothing and does not call the
/// handler. A null `function` writes an empty reply without calling the handler.
///
/// # Safety
///
/// `output` is null or points to at least `output_size` writable bytes, and `function` is null or
/// points to a NUL-terminated string. Both are owned by the caller (Arma) and stay valid until this
/// function returns; neither is kept afterwards.
pub unsafe fn answer_extension_call(
    output: *mut c_char,
    output_size: c_int,
    function: *const c_char,
    handler: fn(&str) -> String,
) {
    // SAFETY: the caller guarantees `output` is null or points to `output_size` writable bytes
    // valid for this call; `output_buffer` checks the null and the size itself.
    let Some(buffer) = (unsafe { output_buffer(output, output_size) }) else {
        return;
    };
    if function.is_null() {
        write_reply(buffer, "");
        return;
    }
    // SAFETY: `function` is not null and the caller guarantees it is NUL-terminated and owned by
    // Arma for this call; the borrowed bytes are dropped before the function returns.
    let input = unsafe { CStr::from_ptr(function) }.to_bytes();
    let reply = handler(&String::from_utf8_lossy(input));
    write_reply(buffer, &reply);
}

/// Answers one `RVExtensionVersion` call by writing `version` into `output`.
///
/// A null `output` or an `output_size` of zero or less writes nothing.
///
/// # Safety
///
/// `output` is null or points to at least `output_size` writable bytes owned by the caller (Arma)
/// and valid until this function returns.
pub unsafe fn answer_extension_version(output: *mut c_char, output_size: c_int, version: &str) {
    // SAFETY: the caller guarantees `output` is null or points to `output_size` writable bytes
    // valid for this call; `output_buffer` checks the null and the size itself.
    if let Some(buffer) = unsafe { output_buffer(output, output_size) } {
        write_reply(buffer, version);
    }
}

/// Turns Arma's output pointer and size into a byte slice, or `None` for a null pointer or a size
/// of zero or less.
///
/// # Safety
///
/// `output` is null or points to at least `output_size` writable bytes that nothing else accesses
/// for the lifetime `'call`.
unsafe fn output_buffer<'call>(output: *mut c_char, output_size: c_int) -> Option<&'call mut [u8]> {
    if output.is_null() {
        return None;
    }
    let length = usize::try_from(output_size)
        .ok()
        .filter(|length| *length > 0)?;
    // SAFETY: `output` is not null and the caller guarantees `length` (= `output_size`) writable
    // bytes owned by Arma, unaliased for `'call`; `c_char` and `u8` have the same size and layout.
    Some(unsafe { core::slice::from_raw_parts_mut(output.cast::<u8>(), length) })
}
