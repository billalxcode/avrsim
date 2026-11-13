//! `embed-emu-gui` native desktop entry point.

mod app;
mod canvas;
mod clipboard;
mod config;
mod export;
mod history;
mod layout;
mod library;
mod log;
mod model;
mod palette;
mod persist;
mod provider;
mod scene;
mod selection;
mod sim;
mod uart;

use app::GuiApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "embed-emu",
        options,
        Box::new(|context| Ok(Box::new(GuiApp::new(context)))),
    )
}
