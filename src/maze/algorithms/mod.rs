mod backtracker;
mod braid;
mod disjoint_sets;
mod dungeon;
mod kruskal;
mod prim;

use rand::Rng;

use crate::maze::MazeKind;
use crate::maze::grid::{Cell, Grid, Step, neighbours};

/// The corridor lattice an algorithm carves: its size in corridors and which
/// corridors are off-limits (inside dungeon rooms). Algorithms work in
/// corridor coordinates only; `floor` and `wall_between` translate to squares.
pub struct Lattice {
    width: usize,
    height: usize,
    blocked: Vec<bool>,
}

impl Lattice {
    fn open(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            blocked: vec![false; width * height],
        }
    }

    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn block(&mut self, x: usize, y: usize) {
        let i = self.index(x, y);
        self.blocked[i] = true;
    }

    fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.blocked[self.index(x, y)]
    }

    /// Every corridor that is not blocked, row by row.
    fn corridors(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.len());
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_blocked(x, y) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// Orthogonal neighbours that are not blocked.
    fn neighbours(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        neighbours(x, y, self.width, self.height)
            .into_iter()
            .filter(|&(nx, ny)| !self.is_blocked(nx, ny))
            .collect()
    }
}

/// The corridor carvers the user can choose from. Each one produces a
/// perfect maze on its own; the kind decides what happens around it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    Backtracker,
    Prim,
    Kruskal,
}

impl Algorithm {
    pub const ALL: [Algorithm; 3] = [Algorithm::Backtracker, Algorithm::Prim, Algorithm::Kruskal];

    pub fn label(self) -> &'static str {
        match self {
            Algorithm::Backtracker => "Recursive backtracker",
            Algorithm::Prim => "Prim",
            Algorithm::Kruskal => "Kruskal",
        }
    }

    fn carve(self, lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
        match self {
            Algorithm::Backtracker => backtracker::carve(lattice, rng),
            Algorithm::Prim => prim::carve(lattice, rng),
            Algorithm::Kruskal => kruskal::carve(lattice, rng),
        }
    }
}

/// `braiding` is the share of dead ends turned into loops for imperfect
/// mazes (0 = none, 1 = all); ignored by the other kinds.
pub fn generate(
    kind: MazeKind,
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    braiding: f32,
    rng: &mut impl Rng,
) -> Vec<Step> {
    match kind {
        MazeKind::Perfect => {
            let lattice = Lattice::open(corridors_x, corridors_y);
            let carving = algorithm.carve(&lattice, rng);
            with_start_and_end(carving, corridors_x, corridors_y, (1, 1))
        }
        MazeKind::Imperfect => {
            let lattice = Lattice::open(corridors_x, corridors_y);
            let mut carving = algorithm.carve(&lattice, rng);
            let mut grid = grid_after(corridors_x, corridors_y, &carving);
            carving.extend(braid::braid(&mut grid, braiding, rng));
            with_start_and_end(carving, corridors_x, corridors_y, (1, 1))
        }
        MazeKind::Dungeon => dungeon::generate(algorithm, corridors_x, corridors_y, rng),
    }
}

/// The grid as it looks once every step has been applied.
fn grid_after(corridors_x: usize, corridors_y: usize, steps: &[Step]) -> Grid {
    let mut grid = Grid::new(corridors_x, corridors_y);
    for step in steps {
        grid.apply(*step);
    }
    grid
}

/// Prepends Start at `start` and End at the farthest walkable square, so
/// both are visible during the whole animation. The carving is replayed
/// once, silently, to know the final layout.
fn with_start_and_end(
    carving: Vec<Step>,
    corridors_x: usize,
    corridors_y: usize,
    start: (usize, usize),
) -> Vec<Step> {
    let finished = grid_after(corridors_x, corridors_y, &carving);
    let (end_x, end_y) = finished.farthest_walkable_from(start.0, start.1);

    let mut steps = vec![
        Step {
            x: start.0,
            y: start.1,
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

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: [(usize, usize); 3] = [(5, 5), (20, 15), (60, 45)];

    fn count_cells(grid: &Grid, wanted: impl Fn(Cell) -> bool) -> usize {
        let mut count = 0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if wanted(grid.get(x, y)) {
                    count += 1;
                }
            }
        }
        count
    }

    fn walkable_count(grid: &Grid) -> usize {
        count_cells(grid, |cell| cell != Cell::Wall)
    }

    fn reachable_from(grid: &Grid, start: (usize, usize)) -> usize {
        let (w, h) = (grid.width(), grid.height());
        let mut seen = vec![false; w * h];
        let mut stack = vec![start];
        seen[start.1 * w + start.0] = true;
        let mut count = 0;
        while let Some((x, y)) = stack.pop() {
            count += 1;
            for (nx, ny) in neighbours(x, y, w, h) {
                if grid.walkable(nx, ny) && !seen[ny * w + nx] {
                    seen[ny * w + nx] = true;
                    stack.push((nx, ny));
                }
            }
        }
        count
    }

    fn find_cell(grid: &Grid, wanted: Cell) -> (usize, usize) {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get(x, y) == wanted {
                    return (x, y);
                }
            }
        }
        panic!("no {wanted:?} in grid");
    }

    #[test]
    fn steps_start_with_start_and_end() {
        for kind in MazeKind::ALL {
            let steps = generate(kind, Algorithm::Backtracker, 20, 15, 0.5, &mut rand::rng());
            assert_eq!(steps[0].cell, Cell::Start);
            assert_eq!(steps[1].cell, Cell::End);
        }
    }

    #[test]
    fn perfect_mazes_are_spanning_trees() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let steps = generate(MazeKind::Perfect, algorithm, cx, cy, 0.0, &mut rand::rng());
                let grid = grid_after(cx, cy, &steps);
                // A tree over n corridors opens exactly n - 1 walls.
                assert_eq!(
                    walkable_count(&grid),
                    2 * cx * cy - 1,
                    "{algorithm:?} {cx}x{cy}"
                );
                assert_eq!(reachable_from(&grid, (1, 1)), walkable_count(&grid));
            }
        }
    }

    #[test]
    fn imperfect_mazes_have_loops_and_stay_connected() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let steps = generate(
                    MazeKind::Imperfect,
                    algorithm,
                    cx,
                    cy,
                    1.0,
                    &mut rand::rng(),
                );
                let grid = grid_after(cx, cy, &steps);
                assert!(
                    walkable_count(&grid) > 2 * cx * cy - 1,
                    "{algorithm:?} {cx}x{cy}"
                );
                assert_eq!(reachable_from(&grid, (1, 1)), walkable_count(&grid));
            }
        }
    }

    #[test]
    fn dungeons_are_connected_through_doors() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let steps = generate(MazeKind::Dungeon, algorithm, cx, cy, 0.0, &mut rand::rng());
                let grid = grid_after(cx, cy, &steps);
                let start = find_cell(&grid, Cell::Start);
                assert_eq!(
                    reachable_from(&grid, start),
                    walkable_count(&grid),
                    "{algorithm:?} {cx}x{cy}"
                );
                if cx >= 20 {
                    let doors = count_cells(&grid, |cell| cell == Cell::Door);
                    assert!(doors >= 1, "{algorithm:?} {cx}x{cy}");
                }
            }
        }
    }
}
