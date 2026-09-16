use eframe::egui;

use super::render::draw_grid;
use super::tiles::Tiles;
use crate::maze::MazeKind;
use crate::maze::algorithms;
use crate::maze::grid::{Grid, Step};

/// How many of the most recently applied steps are highlighted during the replay.
const TRAIL_LENGTH: usize = 5;

/// What the user picked on the menu screen. Sizes are in corridors, not
/// squares; the grid is 2n+1 squares wide (see Grid::new).
struct Settings {
    kind: MazeKind,
    corridors_x: usize,
    corridors_y: usize,
}

/// A generation being replayed: the grid as it currently looks, the full
/// list of steps, and a cursor into it.
struct Replay {
    grid: Grid,
    steps: Vec<Step>,
    next_step: usize,
    pending_steps: f32,
}

impl Replay {
    fn new(settings: &Settings) -> Self {
        Self {
            grid: Grid::new(settings.corridors_x, settings.corridors_y),
            steps: algorithms::generate(
                settings.kind,
                settings.corridors_x,
                settings.corridors_y,
                &mut rand::rng(),
            ),
            next_step: 0,
            pending_steps: 0.0,
        }
    }

    fn finished(&self) -> bool {
        self.next_step >= self.steps.len()
    }

    fn advance(&mut self, dt: f32, steps_per_second: f32) {
        self.pending_steps += dt * steps_per_second;
        let count = self.pending_steps.floor();
        self.pending_steps -= count;
        let until = (self.next_step + count as usize).min(self.steps.len());
        for step in &self.steps[self.next_step..until] {
            self.grid.apply(*step);
        }
        self.next_step = until;
    }

    fn trail(&self) -> &[Step] {
        if self.finished() {
            return &[];
        }
        let start = self.next_step.saturating_sub(TRAIL_LENGTH);
        &self.steps[start..self.next_step]
    }

    fn restart(&mut self) {
        self.grid.reset();
        self.next_step = 0;
        self.pending_steps = 0.0;
    }
}

enum Screen {
    Menu,
    Viewing(Replay),
}

pub struct MazeApp {
    screen: Screen,
    settings: Settings,
    steps_per_second: f32,
    tiles: Tiles,
}

impl MazeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            screen: Screen::Menu,
            settings: Settings {
                kind: MazeKind::Perfect,
                corridors_x: 20,
                corridors_y: 15,
            },
            steps_per_second: 120.0,
            tiles: Tiles::load(&cc.egui_ctx),
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.heading("Maze generator");
        ui.add_space(8.0);
        egui::ComboBox::from_label("Maze type")
            .selected_text(self.settings.kind.label())
            .show_ui(ui, |ui| {
                for kind in MazeKind::ALL {
                    ui.selectable_value(&mut self.settings.kind, kind, kind.label());
                }
            });
        ui.add(egui::Slider::new(&mut self.settings.corridors_x, 5..=60).text("width"));
        ui.add(egui::Slider::new(&mut self.settings.corridors_y, 5..=45).text("height"));
        ui.add_space(8.0);
        if ui.button("Generate").clicked() {
            self.screen = Screen::Viewing(Replay::new(&self.settings));
        }
    }
}

impl eframe::App for MazeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let mut back = false;
            match &mut self.screen {
                Screen::Menu => self.menu(ui),
                Screen::Viewing(replay) => {
                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            back = true;
                        }
                        if ui.button("Replay").clicked() {
                            replay.restart();
                        }
                        ui.add(
                            egui::Slider::new(&mut self.steps_per_second, 1.0..=5000.0)
                                .logarithmic(true)
                                .text("steps / s"),
                        );
                        ui.label(format!("{} / {}", replay.next_step, replay.steps.len()));
                    });
                    if !replay.finished() {
                        let dt = ui.ctx().input(|i| i.stable_dt);
                        replay.advance(dt, self.steps_per_second);
                        ui.ctx().request_repaint();
                    }
                    draw_grid(ui, &replay.grid, &self.tiles, replay.trail());
                }
            }
            if back {
                self.screen = Screen::Menu;
            }
        });
    }
}
