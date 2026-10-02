//! Whether Arma was started with a given command-line flag, such as `-tfdebug`.

#[cfg(test)]
#[path = "tests/command_line.rs"]
mod tests;

/// Whether the process command line contains `flag` anywhere, as a substring.
///
/// On Windows it reads `GetCommandLineW`, the test the original `isDebugArmaInstance` makes
/// (`reference/tfar_1_0/extensions/task_force_radio_pipe/dllmain.cpp:10`); elsewhere it joins
/// [`std::env::args`] with spaces, so tests on Linux exercise the same substring rule.
pub fn arma_command_line_contains(flag: &str) -> bool {
    process_command_line().contains(flag)
}

#[cfg(windows)]
fn process_command_line() -> String {
    use windows_sys::Win32::System::Environment::GetCommandLineW;

    // SAFETY: `GetCommandLineW` takes no argument and returns a pointer to the process's
    // NUL-terminated command line, owned by the system for the life of the process.
    let start = unsafe { GetCommandLineW() };
    if start.is_null() {
        return String::new();
    }
    let mut length = 0_usize;
    // SAFETY: `start` points to a NUL-terminated UTF-16 string, so every offset up to and including
    // the NUL is inside it; the loop stops at the NUL.
    while unsafe { *start.add(length) } != 0 {
        length = length.saturating_add(1);
    }
    // SAFETY: the `length` units before the NUL are initialised and stay valid for the process.
    let units = unsafe { core::slice::from_raw_parts(start, length) };
    String::from_utf16_lossy(units)
}

#[cfg(not(windows))]
fn process_command_line() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}
