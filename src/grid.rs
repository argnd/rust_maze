use crate::tiles::Tiles;
use eframe::egui;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Wall,
    Floor,
    Door,
    Start,
    End,
}

/// One change to the grid, in square coordinates. Algorithms produce a list
/// of these; the app replays them to animate the generation.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub x: usize,
    pub y: usize,
    pub cell: Cell,
}

pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Takes the number of corridors the user asked for; the square grid
    /// is 2n+1 in each direction so that walls occupy full squares.
    pub fn new(corridors_x: usize, corridors_y: usize) -> Self {
        let width = corridors_x * 2 + 1;
        let height = corridors_y * 2 + 1;
        Self {
            width,
            height,
            cells: vec![Cell::Wall; width * height],
        }
    }

    /// Start and End are placed before the carving is replayed, so carving
    /// steps must not paint over them.
    pub fn apply(&mut self, step: Step) {
        let current = self.get(step.x, step.y);
        if current == Cell::Start || current == Cell::End {
            return;
        }
        self.set(step.x, step.y, step.cell);
    }

    /// Breadth-first search over walkable squares; the last square dequeued
    /// is (one of) the farthest.
    pub fn farthest_walkable_from(&self, start_x: usize, start_y: usize) -> (usize, usize) {
        let mut distance = vec![usize::MAX; self.cells.len()];
        let mut queue = VecDeque::new();
        distance[start_y * self.width + start_x] = 0;
        queue.push_back((start_x, start_y));
        let mut farthest = (start_x, start_y);

        while let Some((x, y)) = queue.pop_front() {
            farthest = (x, y);
            let next_distance = distance[y * self.width + x] + 1;
            for (nx, ny) in neighbours(x, y, self.width, self.height) {
                let index = ny * self.width + nx;
                if self.get(nx, ny) != Cell::Wall && distance[index] == usize::MAX {
                    distance[index] = next_distance;
                    queue.push_back((nx, ny));
                }
            }
        }
        farthest
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * self.width + x] = cell;
    }

    pub fn draw(&self, ui: &mut egui::Ui, tiles: &Tiles) {
        let available = ui.available_size();
        let cell_size = (available.x / self.width as f32)
            .min(available.y / self.height as f32)
            .floor();
        let size = egui::vec2(
            cell_size * self.width as f32,
            cell_size * self.height as f32,
        );
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let origin = response.rect.min;
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get(x, y);
                let min = origin + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                let rect = egui::Rect::from_min_size(min, egui::vec2(cell_size, cell_size));
                if cell != Cell::Wall && cell != Cell::Floor {
                    painter.image(tiles.floor(), rect, uv, egui::Color32::WHITE);
                }
                painter.image(tiles.for_cell(cell), rect, uv, egui::Color32::WHITE);
            }
        }
    }
}

/// The up-to-four orthogonal neighbours of (x, y) inside a width x height grid.
pub fn neighbours(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    if x + 1 < width {
        out.push((x + 1, y));
    }
    if y + 1 < height {
        out.push((x, y + 1));
    }
    out
}


