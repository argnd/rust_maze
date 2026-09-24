// No console window for the shipped exe; debug builds keep it so panics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod maze;
mod ui;

use eframe::egui;
use ui::MazeApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Maze generator",
        options,
        Box::new(|cc| Ok(Box::new(MazeApp::new(cc)))),
    )
}
