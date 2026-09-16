pub mod backtracker;

use rand::Rng;

use crate::maze::MazeKind;
use crate::maze::grid::{Cell, Grid, Step};

pub fn generate(
    kind: MazeKind,
    corridors_x: usize,
    corridors_y: usize,
    rng: &mut impl Rng,
) -> Vec<Step> {
    match kind {
        MazeKind::Perfect => generate_perfect(corridors_x, corridors_y, rng),
        // Braiding and rooms come in later steps; until then every kind is a perfect maze.
        MazeKind::Imperfect | MazeKind::Dungeon => generate_perfect(corridors_x, corridors_y, rng),
    }
}

/// A perfect maze: carve with the algorithm, then place Start and End.
/// The carving is replayed once, silently, to know the final layout and
/// find the end; Start and End are then emitted first so they are visible
/// during the whole animation.
pub fn generate_perfect(corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Step> {
    let carving = backtracker::carve(corridors_x, corridors_y, rng);

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
