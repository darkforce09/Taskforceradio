//! Whether the window in front is Arma's, the test behind the 1.0 `FOCUS` frame.

#[cfg(test)]
#[path = "tests/foreground_window.rs"]
mod tests;

/// The class-name buffer length of the original test, in UTF-16 units including the NUL
/// (`reference/tfar_1_0/ts/src/Teamspeak.cpp:210`).
#[cfg(windows)]
const CLASS_NAME_CAPACITY: usize = 32;

/// Whether the foreground window's class name contains `arma`, ignoring case.
///
/// The test of the original (`reference/tfar_1_0/ts/src/Teamspeak.cpp:207`): `GetForegroundWindow`,
/// then `GetClassNameW` into a 32-unit buffer, lower-cased and searched for `arma`. No foreground
/// window means `false`. Off Windows there is no Arma window to find, so it is always `false`.
pub fn foreground_window_is_arma() -> bool {
    #[cfg(windows)]
    {
        foreground_class_name().is_some_and(|class_name| class_name_is_arma(&class_name))
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Whether a window class name names Arma: it contains `arma`, ignoring case.
#[cfg(any(windows, test))]
fn class_name_is_arma(class_name: &str) -> bool {
    class_name.to_lowercase().contains("arma")
}

#[cfg(windows)]
fn foreground_class_name() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow};

    // SAFETY: `GetForegroundWindow` takes no argument and returns a window handle or null.
    let window = unsafe { GetForegroundWindow() };
    if window.is_null() {
        return None;
    }
    let mut buffer = [0_u16; CLASS_NAME_CAPACITY];
    let capacity = i32::try_from(buffer.len()).ok()?;
    // SAFETY: `buffer` is a local array of `capacity` writable units; `GetClassNameW` writes at
    // most that many including the NUL and returns the count written without it, or 0.
    let written = unsafe { GetClassNameW(window, buffer.as_mut_ptr(), capacity) };
    let written = usize::try_from(written).ok()?;
    buffer.get(..written).map(String::from_utf16_lossy)
}
