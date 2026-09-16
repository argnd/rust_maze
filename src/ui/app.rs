use eframe::egui;

use super::render::draw_grid;
use super::tiles::Tiles;
use crate::maze::algorithms;
use crate::maze::grid::{Grid, Step};

/// Maze size in corridors, not squares; the grid is 2n+1 squares wide (see Grid::new).
/// Hard-coded until the menu screen lets the user choose.
const CORRIDORS_X: usize = 20;
const CORRIDORS_Y: usize = 15;
/// How many of the most recently applied steps are highlighted during the replay.
const TRAIL_LENGTH: usize = 5;
pub struct MazeApp {
    grid: Grid,
    tiles: Tiles,
    steps: Vec<Step>,
    next_step: usize,
    steps_per_second: f32,
    pending_steps: f32,
}

impl MazeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            grid: Grid::new(CORRIDORS_X, CORRIDORS_Y),
            tiles: Tiles::load(&cc.egui_ctx),
            steps: Vec::new(),
            next_step: 0,
            steps_per_second: 120.0,
            pending_steps: 0.0,
        };
        app.generate();
        app
    }

    fn generate(&mut self) {
        self.grid = Grid::new(CORRIDORS_X, CORRIDORS_Y);
        self.steps = algorithms::generate_perfect(CORRIDORS_X, CORRIDORS_Y, &mut rand::rng());
        self.next_step = 0;
        self.pending_steps = 0.0;
    }

    fn advance(&mut self, dt: f32) {
        self.pending_steps += dt * self.steps_per_second;
        let count = self.pending_steps.floor();
        self.pending_steps -= count;
        let until = (self.next_step + count as usize).min(self.steps.len());
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
                ui.add(
                    egui::Slider::new(&mut self.steps_per_second, 1.0..=5000.0)
                        .logarithmic(true)
                        .text("steps / s"),
                );
                ui.label(format!("{} / {}", self.next_step, self.steps.len()));
            });

            if self.next_step < self.steps.len() {
                let dt = ui.ctx().input(|i| i.stable_dt);
                self.advance(dt);
                ui.ctx().request_repaint();
            }
            let trail_start = self.next_step.saturating_sub(TRAIL_LENGTH);
            draw_grid(
                ui,
                &self.grid,
                &self.tiles,
                &self.steps[trail_start..self.next_step],
            );
            (ui, &self.grid, &self.tiles);
        });
    }
}
