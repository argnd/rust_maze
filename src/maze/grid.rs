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

    /// Back to the all-wall state, keeping the same size.
    pub fn reset(&mut self) {
        self.cells.fill(Cell::Wall);
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
