mod app;
mod grid;
mod tiles;
mod algorithms;
use app::MazeApp;
use eframe::egui;

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