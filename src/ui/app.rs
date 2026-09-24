use std::path::PathBuf;

use eframe::egui::{self, Color32, Key, RichText, vec2};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, RngExt, SeedableRng};

use super::export::export_png;
use super::render::{self, POP_SECONDS, Scene, SolutionOverlay};
use super::tiles::Tiles;
use crate::maze::MazeKind;
use crate::maze::algorithms::{self, Algorithm, Generation};
use crate::maze::grid::{Grid, Step};
use crate::maze::solve::{self, Solution};
use crate::maze::stats::{self, Stats};

/// How many of the most recently applied steps are highlighted during the replay.
const TRAIL_LENGTH: usize = 5;
/// Roughly how long one maze takes to build in demo mode, whatever its size.
const DEMO_BUILD_SECONDS: f32 = 9.0;
/// Pause on a solved maze before the demo moves on.
const DEMO_HOLD_SECONDS: f64 = 2.5;
/// How long a status message, such as an export result, stays visible.
const STATUS_SECONDS: f64 = 6.0;

/// What the user picked on the menu screen. Sizes are in corridors, not
/// squares; the grid is 2n+1 squares wide (see Grid::new).
struct Settings {
    kind: MazeKind,
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    /// Share of dead ends turned into loops, imperfect mazes only.
    braiding: f32,
    /// Empty for a random seed; a number is used as is, other text is hashed.
    seed_text: String,
}

impl Settings {
    fn seed(&self) -> u64 {
        let text = self.seed_text.trim();
        if text.is_empty() {
            rand::rng().random()
        } else {
            text.parse().unwrap_or_else(|_| hash_seed(text))
        }
    }
}

/// FNV-1a. A fixed hash so a word typed as seed always gives the same maze;
/// std's hasher is allowed to change between Rust releases.
fn hash_seed(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Everything that determines a maze. Same recipe, same maze.
#[derive(Clone, Copy)]
struct Recipe {
    kind: MazeKind,
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    braiding: f32,
    seed: u64,
}

impl Recipe {
    fn from_settings(settings: &Settings) -> Self {
        Self {
            kind: settings.kind,
            algorithm: settings.algorithm,
            corridors_x: settings.corridors_x,
            corridors_y: settings.corridors_y,
            braiding: settings.braiding,
            seed: settings.seed(),
        }
    }

    /// A random mix for the demo, sized to read well on screen.
    fn random(rng: &mut impl Rng) -> Self {
        let kinds = [
            MazeKind::Perfect,
            MazeKind::Perfect,
            MazeKind::Imperfect,
            MazeKind::Dungeon,
            MazeKind::Dungeon,
        ];
        let corridors_x = rng.random_range(18..=40);
        Self {
            kind: *kinds.choose(rng).expect("not empty"),
            algorithm: *Algorithm::ALL.choose(rng).expect("not empty"),
            corridors_x,
            corridors_y: (corridors_x * 2 / 3).clamp(12, 28),
            braiding: rng.random_range(0.3..=0.8),
            seed: rng.random(),
        }
    }

    fn with_new_seed(self) -> Self {
        Self {
            seed: rand::rng().random(),
            ..self
        }
    }
}

/// A generation being replayed: the grid as it currently looks, the full
/// list of steps, a cursor into it, and what is known about the result.
struct Replay {
    recipe: Recipe,
    generation: Generation,
    grid: Grid,
    finished_grid: Grid,
    stats: Stats,
    solution: Option<Solution>,
    next_step: usize,
    pending_steps: f32,
    paused: bool,
    /// When each square last changed during playback, in UI time; drives the
    /// pop-in. NEG_INFINITY for squares set by a jump, which don't pop.
    changed_at: Vec<f64>,
    last_change: f64,
    /// UI time the solve animation started, once asked for.
    solving_since: Option<f64>,
}

impl Replay {
    fn new(recipe: Recipe) -> Self {
        let generation = algorithms::generate(
            recipe.kind,
            recipe.algorithm,
            recipe.corridors_x,
            recipe.corridors_y,
            recipe.braiding,
            &mut StdRng::seed_from_u64(recipe.seed),
        );
        let mut finished_grid = Grid::new(recipe.corridors_x, recipe.corridors_y);
        for step in &generation.steps {
            finished_grid.apply(*step);
        }
        let squares = finished_grid.width() * finished_grid.height();
        Self {
            recipe,
            stats: stats::measure(&finished_grid),
            solution: solve::solve(&finished_grid),
            grid: Grid::new(recipe.corridors_x, recipe.corridors_y),
            finished_grid,
            generation,
            next_step: 0,
            pending_steps: 0.0,
            paused: false,
            changed_at: vec![f64::NEG_INFINITY; squares],
            last_change: f64::NEG_INFINITY,
            solving_since: None,
        }
    }

    fn total(&self) -> usize {
        self.generation.steps.len()
    }

    fn finished(&self) -> bool {
        self.next_step >= self.total()
    }

    fn phase_name(&self) -> &'static str {
        if self.finished() {
            return "Done";
        }
        self.generation
            .phase_at(self.next_step)
            .map_or("", |phase| phase.name)
    }

    /// Plays `dt` seconds worth of steps at `speed` steps per second, scaled
    /// by the pace of the current phase.
    fn advance(&mut self, dt: f32, speed: f32, now: f64) {
        if self.paused || self.finished() {
            return;
        }
        let pace = self
            .generation
            .phase_at(self.next_step)
            .map_or(1.0, |phase| phase.pace);
        self.pending_steps += dt * speed * pace;
        let count = self.pending_steps.floor();
        self.pending_steps -= count;
        self.play_until(self.next_step + count as usize, now);
    }

    /// Applies steps up to `until`, stamping each square for the pop-in.
    fn play_until(&mut self, until: usize, now: f64) {
        let until = until.min(self.total());
        let width = self.grid.width();
        for step in &self.generation.steps[self.next_step..until] {
            self.grid.apply(*step);
            self.changed_at[step.y * width + step.x] = now;
            self.last_change = now;
        }
        self.next_step = until;
    }

    fn step_once(&mut self, now: f64) {
        self.paused = true;
        self.play_until(self.next_step + 1, now);
    }

    /// Jumps to `target` without animating: forward by applying the steps,
    /// backward by rebuilding from an empty grid.
    fn seek(&mut self, target: usize) {
        let target = target.min(self.total());
        if target < self.next_step {
            self.grid.reset();
            self.next_step = 0;
        }
        for step in &self.generation.steps[self.next_step..target] {
            self.grid.apply(*step);
        }
        self.next_step = target;
        self.pending_steps = 0.0;
        self.changed_at.fill(f64::NEG_INFINITY);
        if target < self.total() {
            self.solving_since = None;
        }
    }

    fn restart(&mut self) {
        self.seek(0);
        self.paused = false;
    }

    fn skip_to_end(&mut self) {
        self.seek(self.total());
    }

    fn start_solving(&mut self, now: f64) {
        if self.solution.is_some() {
            self.skip_to_end();
            self.solving_since = Some(now);
        }
    }

    fn trail(&self) -> &[Step] {
        if self.finished() {
            return &[];
        }
        let start = self.next_step.saturating_sub(TRAIL_LENGTH);
        &self.generation.steps[start..self.next_step]
    }

    fn overlay(&self, now: f64, show_heat: bool) -> Option<SolutionOverlay<'_>> {
        Some(SolutionOverlay {
            solution: self.solution.as_ref()?,
            elapsed: now - self.solving_since?,
            show_heat,
        })
    }

    /// Whether anything on screen still moves and needs another frame.
    fn animating(&self, now: f64) -> bool {
        (!self.paused && !self.finished())
            || now - self.last_change < POP_SECONDS
            || self.solving_since.is_some()
    }

    /// Writes the finished maze next to the working directory, with the path
    /// if it has been solved on screen.
    fn export(&self) -> Result<PathBuf, String> {
        let file = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(format!("maze-{}.png", self.recipe.seed));
        let path = self
            .solving_since
            .and(self.solution.as_ref())
            .map(|solution| solution.path.as_slice());
        export_png(&self.finished_grid, path, &file).map_err(|e| e.to_string())?;
        Ok(file)
    }
}

/// Viewer state that outlives a single maze.
struct Controls {
    steps_per_second: f32,
    show_heat: bool,
    demo: bool,
    /// When the demo's current maze finished drawing its solution.
    demo_solved_at: Option<f64>,
    status: Option<(String, f64)>,
}

enum Screen {
    Menu,
    Viewing(Box<Replay>),
}

/// What a click or key asked for. Collected while the screen is borrowed,
/// carried out afterwards, when replacing the screen is allowed again.
enum Action {
    Back,
    Generate(Recipe),
}

pub struct MazeApp {
    screen: Screen,
    settings: Settings,
    controls: Controls,
    tiles: Tiles,
}

impl MazeApp {
    /// `demo` skips the menu and starts the demo loop right away (kiosk use).
    pub fn new(cc: &eframe::CreationContext<'_>, demo: bool) -> Self {
        let screen = if demo {
            Screen::Viewing(Box::new(Replay::new(Recipe::random(&mut rand::rng()))))
        } else {
            Screen::Menu
        };
        Self {
            screen,
            settings: Settings {
                kind: MazeKind::Perfect,
                algorithm: Algorithm::Backtracker,
                corridors_x: 24,
                corridors_y: 16,
                braiding: 0.5,
                seed_text: String::new(),
            },
            controls: Controls {
                steps_per_second: 240.0,
                show_heat: true,
                demo,
                demo_solved_at: None,
                status: None,
            },
            tiles: Tiles::load(&cc.egui_ctx),
        }
    }
}

impl eframe::App for MazeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = ui.input(|i| i.time);
        // Clamped so a stalled frame doesn't dump a burst of steps at once.
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        let typing = ui.ctx().egui_wants_keyboard_input();

        let action = match &mut self.screen {
            Screen::Menu => {
                let mut action = None;
                egui::CentralPanel::default().show(ui, |ui| {
                    action = menu(ui, &mut self.settings, &mut self.controls, typing);
                });
                action
            }
            Screen::Viewing(replay) => {
                viewer(ui, replay, &mut self.controls, &self.tiles, now, dt, typing)
            }
        };

        match action {
            Some(Action::Back) => {
                self.controls.demo = false;
                self.screen = Screen::Menu;
            }
            Some(Action::Generate(recipe)) => {
                self.controls.demo_solved_at = None;
                self.screen = Screen::Viewing(Box::new(Replay::new(recipe)));
            }
            None => {}
        }
    }
}

fn menu(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    controls: &mut Controls,
    typing: bool,
) -> Option<Action> {
    let mut action = None;
    ui.vertical_centered(|ui| {
        ui.add_space(28.0);
        ui.label(RichText::new("Maze Generator").size(34.0).strong());
        ui.label(RichText::new("Ten algorithms, three kinds of maze, every step animated.").weak());
        ui.add_space(24.0);
    });

    let column = 560.0_f32.min(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - column) / 2.0).max(0.0));
        ui.vertical(|ui| {
            ui.set_width(column);
            egui::Grid::new("settings")
                .num_columns(2)
                .spacing([18.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Maze type");
                    egui::ComboBox::from_id_salt("kind")
                        .width(260.0)
                        .selected_text(settings.kind.label())
                        .show_ui(ui, |ui| {
                            for kind in MazeKind::ALL {
                                ui.selectable_value(&mut settings.kind, kind, kind.label());
                            }
                        });
                    ui.end_row();
                    ui.label("");
                    ui.label(RichText::new(settings.kind.description()).weak());
                    ui.end_row();

                    ui.label("Algorithm");
                    egui::ComboBox::from_id_salt("algorithm")
                        .width(260.0)
                        .selected_text(settings.algorithm.label())
                        .show_ui(ui, |ui| {
                            for algorithm in Algorithm::ALL {
                                ui.selectable_value(
                                    &mut settings.algorithm,
                                    algorithm,
                                    algorithm.label(),
                                )
                                .on_hover_text(algorithm.description());
                            }
                        });
                    ui.end_row();
                    ui.label("");
                    ui.label(RichText::new(settings.algorithm.description()).weak());
                    ui.end_row();

                    ui.label("Width");
                    ui.add(
                        egui::Slider::new(&mut settings.corridors_x, 5..=60).suffix(" corridors"),
                    );
                    ui.end_row();
                    ui.label("Height");
                    ui.add(
                        egui::Slider::new(&mut settings.corridors_y, 5..=45).suffix(" corridors"),
                    );
                    ui.end_row();

                    if settings.kind == MazeKind::Imperfect {
                        ui.label("Loops");
                        ui.add(
                            egui::Slider::new(&mut settings.braiding, 0.0..=1.0).custom_formatter(
                                |value, _| format!("{:.0}% of dead ends", value * 100.0),
                            ),
                        );
                        ui.end_row();
                    }

                    ui.label("Seed");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut settings.seed_text)
                                .hint_text("random")
                                .desired_width(180.0),
                        );
                        if ui.button("Clear").clicked() {
                            settings.seed_text.clear();
                        }
                    });
                    ui.end_row();
                    ui.label("");
                    ui.label(
                        RichText::new(
                            "Any number or word; the same seed always builds the same maze.",
                        )
                        .weak(),
                    );
                    ui.end_row();
                });

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                let generate = egui::Button::new(RichText::new("Generate").size(18.0))
                    .min_size(vec2(170.0, 38.0));
                if ui.add(generate).clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    action = Some(Action::Generate(Recipe::from_settings(settings)));
                }
                let demo = egui::Button::new(RichText::new("Demo mode").size(18.0))
                    .min_size(vec2(170.0, 38.0));
                if ui
                    .add(demo)
                    .on_hover_text("Random mazes, built and solved, one after another")
                    .clicked()
                    || (!typing && ui.input(|i| i.key_pressed(Key::D)))
                {
                    controls.demo = true;
                    action = Some(Action::Generate(Recipe::random(&mut rand::rng())));
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Enter: generate    D: demo mode").weak());
        });
    });
    action
}

fn viewer(
    ui: &mut egui::Ui,
    replay: &mut Replay,
    controls: &mut Controls,
    tiles: &Tiles,
    now: f64,
    dt: f32,
    typing: bool,
) -> Option<Action> {
    let mut action = None;

    if !typing {
        // Consumed, not just read: otherwise a toolbar button holding keyboard
        // focus would also react to Space, and Pause would toggle twice.
        let pressed = |key: Key| {
            ui.ctx()
                .input_mut(|i| i.consume_key(egui::Modifiers::NONE, key))
        };
        if pressed(Key::Escape) {
            action = Some(Action::Back);
        }
        if pressed(Key::Space) && !replay.finished() {
            replay.paused = !replay.paused;
        }
        if pressed(Key::ArrowRight) {
            replay.step_once(now);
        }
        if pressed(Key::End) {
            replay.skip_to_end();
        }
        if pressed(Key::R) {
            replay.restart();
        }
        if pressed(Key::N) {
            action = Some(Action::Generate(replay.recipe.with_new_seed()));
        }
        if pressed(Key::S) {
            replay.start_solving(now);
        }
        if pressed(Key::H) {
            controls.show_heat = !controls.show_heat;
        }
        if pressed(Key::E) {
            controls.status = Some((export_message(replay), now));
        }
    }

    if controls.demo {
        if replay.finished() && replay.solving_since.is_none() {
            replay.start_solving(now);
        }
        if replay
            .overlay(now, true)
            .is_some_and(|overlay| overlay.drawn())
        {
            let solved_at = *controls.demo_solved_at.get_or_insert(now);
            if now - solved_at > DEMO_HOLD_SECONDS {
                action = Some(Action::Generate(Recipe::random(&mut rand::rng())));
            }
        }
    }

    let speed = if controls.demo {
        (replay.total() as f32 / DEMO_BUILD_SECONDS).max(60.0)
    } else {
        controls.steps_per_second
    };
    replay.advance(dt, speed, now);

    egui::Panel::top("toolbar").show(ui, |ui| {
        ui.add_space(6.0);
        if let Some(clicked) = toolbar(ui, replay, controls, now) {
            action = Some(clicked);
        }
        ui.add_space(4.0);
    });
    egui::Panel::right("details")
        .resizable(false)
        .default_size(250.0)
        .show(ui, |ui| details(ui, replay, controls));

    let caption = controls.demo.then(|| {
        format!(
            "{}  ·  {}",
            replay.recipe.algorithm.label(),
            replay.recipe.kind.label()
        )
    });
    egui::CentralPanel::default().show(ui, |ui| {
        let scene = Scene {
            grid: &replay.grid,
            tiles,
            trail: replay.trail(),
            changed_at: &replay.changed_at,
            now,
            solution: replay.overlay(now, controls.show_heat),
            caption,
        };
        render::draw_maze(ui, &scene);
    });

    let status_visible = controls
        .status
        .as_ref()
        .is_some_and(|(_, at)| now - at < STATUS_SECONDS);
    if replay.animating(now) || controls.demo || status_visible {
        ui.ctx().request_repaint();
    }
    action
}

fn export_message(replay: &Replay) -> String {
    match replay.export() {
        Ok(file) => format!("Saved {}", file.display()),
        Err(error) => format!("Export failed: {error}"),
    }
}

fn toolbar(
    ui: &mut egui::Ui,
    replay: &mut Replay,
    controls: &mut Controls,
    now: f64,
) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button("Back").on_hover_text("Esc").clicked() {
            action = Some(Action::Back);
        }
        if controls.demo {
            ui.label(RichText::new("DEMO").strong().color(render::PATH_COLOR));
            if ui.button("Stop demo").clicked() {
                controls.demo = false;
            }
        }
        ui.separator();

        let play_label = if replay.paused { "Play" } else { "Pause" };
        if ui
            .add_enabled(!replay.finished(), egui::Button::new(play_label))
            .on_hover_text("Space")
            .clicked()
        {
            replay.paused = !replay.paused;
        }
        if ui
            .add_enabled(!replay.finished(), egui::Button::new("Step"))
            .on_hover_text("Right arrow")
            .clicked()
        {
            replay.step_once(now);
        }
        if ui
            .add_enabled(!replay.finished(), egui::Button::new("Skip"))
            .on_hover_text("End")
            .clicked()
        {
            replay.skip_to_end();
        }
        if ui.button("Replay").on_hover_text("R").clicked() {
            replay.restart();
        }
        if ui
            .button("New maze")
            .on_hover_text("N: same settings, new seed")
            .clicked()
        {
            action = Some(Action::Generate(replay.recipe.with_new_seed()));
        }
        ui.separator();

        if ui
            .add_enabled(replay.solution.is_some(), egui::Button::new("Solve"))
            .on_hover_text("S")
            .clicked()
        {
            replay.start_solving(now);
        }
        ui.checkbox(&mut controls.show_heat, "Heat map")
            .on_hover_text("H");
        if ui.button("Export PNG").on_hover_text("E").clicked() {
            controls.status = Some((export_message(replay), now));
        }

        if !controls.demo {
            ui.separator();
            ui.add(
                egui::Slider::new(&mut controls.steps_per_second, 1.0..=20000.0)
                    .logarithmic(true)
                    .max_decimals(0)
                    .text("steps / s"),
            );
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new(replay.phase_name()).strong());
        let counter = format!("{} / {}", replay.next_step, replay.total());
        ui.spacing_mut().slider_width = (ui.available_width() - 130.0).max(80.0);
        let mut cursor = replay.next_step;
        if ui
            .add(egui::Slider::new(&mut cursor, 0..=replay.total()).show_value(false))
            .on_hover_text("Drag to scrub through the generation")
            .changed()
        {
            replay.seek(cursor);
        }
        ui.label(RichText::new(counter).monospace());
    });

    if let Some((message, at)) = &controls.status
        && now - at < STATUS_SECONDS
    {
        ui.label(RichText::new(message).weak());
    }
    action
}

fn details(ui: &mut egui::Ui, replay: &Replay, controls: &Controls) {
    let recipe = &replay.recipe;
    let stats = &replay.stats;
    ui.add_space(8.0);
    ui.heading("This maze");
    egui::Grid::new("recipe")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Type");
            ui.label(recipe.kind.label());
            ui.end_row();
            ui.label("Algorithm");
            ui.label(recipe.algorithm.label());
            ui.end_row();
            ui.label("Size");
            ui.label(format!(
                "{} × {} corridors",
                recipe.corridors_x, recipe.corridors_y
            ));
            ui.end_row();
            ui.label("Seed");
            ui.horizontal(|ui| {
                ui.label(RichText::new(recipe.seed.to_string()).monospace());
                if ui.small_button("Copy").clicked() {
                    ui.ctx().copy_text(recipe.seed.to_string());
                }
            });
            ui.end_row();
            ui.label("Steps");
            ui.label(replay.total().to_string());
            ui.end_row();
        });

    ui.separator();
    ui.label(RichText::new("Finished maze").strong());
    egui::Grid::new("stats")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Open squares");
            ui.label(stats.walkable.to_string());
            ui.end_row();
            ui.label("Dead ends");
            ui.label(stats.dead_ends.to_string());
            ui.end_row();
            // Room floors count as junctions and loops on every square, so
            // for dungeons those figures would be noise; doors say more.
            if recipe.kind == MazeKind::Dungeon {
                ui.label("Doors");
                ui.label(stats.doors.to_string());
                ui.end_row();
            } else {
                ui.label("Junctions");
                ui.label(stats.junctions.to_string());
                ui.end_row();
                ui.label("Loops");
                ui.label(stats.loops.to_string());
                ui.end_row();
            }
            if let Some(solution) = &replay.solution {
                let share = 100.0 * solution.path.len() as f64 / stats.walkable.max(1) as f64;
                ui.label("Solution");
                ui.label(format!("{} squares ({share:.0}%)", solution.path.len()));
                ui.end_row();
            }
        });

    ui.separator();
    ui.label(RichText::new("Legend").strong());
    swatch(ui, render::PROBE_COLOR, "Being worked on");
    swatch(ui, render::TRAIL_COLOR, "Latest steps");
    swatch(ui, render::PATH_COLOR, "Shortest path");
    if controls.show_heat {
        ui.horizontal(|ui| {
            gradient_bar(ui);
            ui.label("near to far");
        });
    }

    ui.separator();
    ui.label(RichText::new("Keys").strong());
    for (key, meaning) in [
        ("Space", "play / pause"),
        ("Right", "one step"),
        ("End", "skip to the end"),
        ("R", "replay"),
        ("N", "new maze"),
        ("S", "solve"),
        ("H", "heat map"),
        ("E", "export PNG"),
        ("Esc", "back to menu"),
    ] {
        ui.horizontal(|ui| {
            ui.label(RichText::new(key).monospace().strong());
            ui.label(RichText::new(meaning).weak());
        });
    }
}

fn swatch(ui: &mut egui::Ui, color: Color32, text: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, color);
        ui.label(text);
    });
}

fn gradient_bar(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(110.0, 12.0), egui::Sense::hover());
    const SLICES: usize = 32;
    let slice = rect.width() / SLICES as f32;
    for i in 0..SLICES {
        let min = rect.min + vec2(i as f32 * slice, 0.0);
        let slice_rect = egui::Rect::from_min_size(min, vec2(slice + 0.5, rect.height()));
        let t = i as f64 / (SLICES - 1) as f64;
        ui.painter()
            .rect_filled(slice_rect, 0.0, render::heat_color(t, 255));
    }
}
