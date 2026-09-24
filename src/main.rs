// No console window for the shipped exe; debug builds keep it so panics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod maze;
mod ui;

use eframe::egui;
use ui::MazeApp;

fn main() -> eframe::Result {
    let demo = std::env::args().any(|arg| arg == "--demo");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Maze Generator")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([820.0, 560.0])
            .with_icon(ui::app_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Maze Generator",
        options,
        Box::new(move |cc| Ok(Box::new(MazeApp::new(cc, demo)))),
    )
}
