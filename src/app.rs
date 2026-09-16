use eframe::egui;

use crate::algorithms;
use crate::grid::{Grid, Step};
use crate::tiles::Tiles;

const CORRIDORS_X: usize = 20;
const CORRIDORS_Y: usize = 15;

pub struct MazeApp {
    grid: Grid,
    tiles: Tiles,
    steps: Vec<Step>,
    next_step: usize,
    steps_per_frame: usize,
}

impl MazeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            grid: Grid::new(CORRIDORS_X, CORRIDORS_Y),
            tiles: Tiles::load(&cc.egui_ctx),
            steps: Vec::new(),
            next_step: 0,
            steps_per_frame: 2,
        };
        app.generate();
        app
    }

    fn generate(&mut self) {
        self.grid = Grid::new(CORRIDORS_X, CORRIDORS_Y);
        self.steps = algorithms::generate_perfect(CORRIDORS_X, CORRIDORS_Y, &mut rand::rng());
        self.next_step = 0;
    }

    fn advance(&mut self) {
        let until = (self.next_step + self.steps_per_frame).min(self.steps.len());
        for step in &self.steps[self.next_step..until] {
            self.grid.apply(*step);
        }
        self.next_step = until;
    }
}

impl eframe::App for MazeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Generate").clicked() {
                    self.generate();
                }
                ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=50).text("steps / frame"));
                ui.label(format!("{} / {}", self.next_step, self.steps.len()));
            });

            if self.next_step < self.steps.len() {
                self.advance();
                ui.ctx().request_repaint();
            }
            self.grid.draw(ui, &self.tiles);
        });
    }
}