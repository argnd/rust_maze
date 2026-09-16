mod backtracker;
mod prim;

use rand::Rng;

use crate::maze::MazeKind;
use crate::maze::grid::{Cell, Grid, Step};

/// The corridor carvers the user can choose from. Each one produces a
/// perfect maze on its own; the kind decides what happens around it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    Backtracker,
    Prim,
}

impl Algorithm {
    pub const ALL: [Algorithm; 2] = [Algorithm::Backtracker, Algorithm::Prim];

    pub fn label(self) -> &'static str {
        match self {
            Algorithm::Backtracker => "Recursive backtracker",
            Algorithm::Prim => "Prim",
        }
    }

    fn carve(self, corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Step> {
        match self {
            Algorithm::Backtracker => backtracker::carve(corridors_x, corridors_y, rng),
            Algorithm::Prim => prim::carve(corridors_x, corridors_y, rng),
        }
    }
}

pub fn generate(
    kind: MazeKind,
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    rng: &mut impl Rng,
) -> Vec<Step> {
    match kind {
        MazeKind::Perfect => generate_perfect(algorithm, corridors_x, corridors_y, rng),
        // Braiding and rooms come in later steps; until then every kind is a perfect maze.
        MazeKind::Imperfect | MazeKind::Dungeon => {
            generate_perfect(algorithm, corridors_x, corridors_y, rng)
        }
    }
}

/// A perfect maze: carve with the algorithm, then place Start and End.
/// The carving is replayed once, silently, to know the final layout and
/// find the end; Start and End are then emitted first so they are visible
/// during the whole animation.
fn generate_perfect(
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    rng: &mut impl Rng,
) -> Vec<Step> {
    let carving = algorithm.carve(corridors_x, corridors_y, rng);

    let mut finished = Grid::new(corridors_x, corridors_y);
    for step in &carving {
        finished.apply(*step);
    }
    let (end_x, end_y) = finished.farthest_walkable_from(1, 1);

    let mut steps = vec![
        Step {
            x: 1,
            y: 1,
            cell: Cell::Start,
        },
        Step {
            x: end_x,
            y: end_y,
            cell: Cell::End,
        },
    ];
    steps.extend(carving);
    steps
}

/// The floor step for one corridor, in square coordinates.
fn floor(corridor_x: usize, corridor_y: usize) -> Step {
    Step {
        x: corridor_x * 2 + 1,
        y: corridor_y * 2 + 1,
        cell: Cell::Floor,
    }
}

/// The wall square between two adjacent corridors is their midpoint.
fn wall_between(ax: usize, ay: usize, bx: usize, by: usize) -> Step {
    Step {
        x: ax + bx + 1,
        y: ay + by + 1,
        cell: Cell::Floor,
    }
}
