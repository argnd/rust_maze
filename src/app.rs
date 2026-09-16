use eframe::egui;

pub struct MazeApp;

impl eframe::App for MazeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Maze generator");
        });
    }
}