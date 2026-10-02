//! Unit tests for the eframe backend's options, and the one test that opens a real X11 window.

use std::time::Instant;

use super::*;
use crate::debug_snapshot::{DebugSection, DebugSnapshot};
use crate::window_control::DebugWindow;

#[test]
fn the_window_uses_glow_returns_on_close_and_configures_the_event_loop() {
    let options = native_options("TFAR Debug UI");
    assert_eq!(options.renderer, eframe::Renderer::Glow);
    assert!(options.run_and_return);
    assert!(options.event_loop_builder.is_some());
    assert_eq!(options.viewport.title.as_deref(), Some("TFAR Debug UI"));
    assert_eq!(
        options.viewport.inner_size,
        Some(egui::vec2(INITIAL_SIZE[0], INITIAL_SIZE[1]))
    );
}

/// Waits up to `seconds` for `condition`.
fn wait_for(seconds: u64, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    condition()
}

#[test]
#[ignore = "needs an X11 display"]
fn a_real_window_opens_closes_reopens_and_shuts_down_on_one_event_loop() {
    let handle = DebugWindow::spawn("TFAR Debug UI (test)").expect("DISPLAY is set");
    handle.publish(
        DebugSnapshot::new()
            .with_section(DebugSection::key_values("Game link", [("session", "up")])),
    );
    handle.show();
    assert!(wait_for(2, || handle.is_open()));
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(handle.take_window_error(), None);
    handle.hide();
    assert!(wait_for(2, || !handle.is_open()));
    handle.show();
    std::thread::sleep(Duration::from_millis(500));
    assert!(handle.is_open(), "the second window reuses the event loop");
    assert_eq!(handle.take_window_error(), None);
    let started = Instant::now();
    handle.shutdown().expect("the UI thread ends");
    assert!(started.elapsed() < Duration::from_secs(1));
}
