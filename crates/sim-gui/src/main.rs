//! `embed-emu-gui` native desktop entry point.

mod app;
mod canvas;
mod clipboard;
mod config;
mod history;
mod library;
mod model;
mod palette;
mod persist;
mod scene;
mod selection;
mod sim;

use app::GuiApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "embed-emu",
        options,
        Box::new(|context| Ok(Box::new(GuiApp::new(context)))),
    )
}
