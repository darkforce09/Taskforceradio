//! The real window: eframe's event loop on X11 with the `glow` renderer, reused for every window
//! the plugin opens.
//!
//! **Role:** implements `WindowBackend` with `eframe::run_native` and `run_and_return`, so each
//! show opens a window on the same thread-local event loop and each close returns to the UI
//! thread's loop.
//!
//! **Position:** inside `egui_window`; started by `DebugWindow::spawn`; draws with
//! `snapshot_view`.
//!
//! **Signals and state:** runs only on the UI thread. eframe keeps the event loop in a
//! thread-local, created on the first show with X11 forced and `with_any_thread(true)` (the
//! thread is not TeamSpeak's main thread). The waker is a clone of the egui context.
//!
//! **Invariants:** the window closes on the first frame after the plugin asks for it
//! (`logic` sends `ViewportCommand::Close`); the window redraws at least every
//! [`REFRESH_INTERVAL`] and at once on publish.

#[cfg(test)]
#[path = "tests/eframe_backend.rs"]
mod tests;

use std::time::Duration;

use eframe::egui;
use winit::platform::x11::EventLoopBuilderExtX11;

use super::snapshot_view::show_snapshot;
use crate::window_control::{Waker, WindowBackend, WindowSession};
use crate::{Error, Result};

/// The longest a shown window goes without redrawing.
pub(crate) const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

/// The window's size when it first opens, in points (the original's window was 800 by 600).
const INITIAL_SIZE: [f32; 2] = [800.0, 600.0];

/// The `WindowBackend` the plugin uses.
#[derive(Debug, Default)]
pub(crate) struct EframeBackend;

impl WindowBackend for EframeBackend {
    fn run_window(&mut self, session: &WindowSession<'_>) -> Result<()> {
        let result = eframe::run_native(
            session.title(),
            native_options(session.title()),
            Box::new(|creation| {
                let context = creation.egui_ctx.clone();
                session.set_waker(Waker::new(move || context.request_repaint()));
                Ok(Box::new(DebugWindowApp { session }))
            }),
        );
        result.map_err(|error| Error::Window {
            reason: error.to_string(),
        })
    }
}

/// X11 forced, any thread allowed, `glow`, and a return from `run_native` when the window closes.
fn native_options(title: &str) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size(INITIAL_SIZE),
        renderer: eframe::Renderer::Glow,
        run_and_return: true,
        event_loop_builder: Some(Box::new(|builder| {
            builder.with_x11().with_any_thread(true);
        })),
        ..eframe::NativeOptions::default()
    }
}

/// The eframe app for one open window.
struct DebugWindowApp<'session> {
    session: &'session WindowSession<'session>,
}

impl eframe::App for DebugWindowApp<'_> {
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if self.session.should_close() {
            context.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let snapshot = self.session.snapshot();
        egui::CentralPanel::default_margins().show(ui, |ui| show_snapshot(ui, &snapshot));
        ui.ctx().request_repaint_after(REFRESH_INTERVAL);
    }
}
