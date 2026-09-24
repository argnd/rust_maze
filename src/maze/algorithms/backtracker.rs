use rand::Rng;
use rand::seq::IndexedRandom;

use super::{Lattice, between, corridor, floor, wall_between};
use crate::maze::grid::{Cell, Step};

/// Recursive backtracker, iterative form: walk to a random unvisited
/// neighbour, knocking down the wall in between; when stuck, step back.
/// The stack is shown as probe squares and turns to floor as it unwinds.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut visited = vec![false; lattice.len()];
    let mut steps = Vec::new();

    // One walk per unvisited corridor: a single walk on an open lattice, one
    // per pocket when rooms split it apart.
    for (seed_x, seed_y) in lattice.corridors() {
        if visited[lattice.index(seed_x, seed_y)] {
            continue;
        }
        visited[lattice.index(seed_x, seed_y)] = true;
        steps.push(corridor(seed_x, seed_y, Cell::Probe));
        let mut stack = vec![(seed_x, seed_y)];

        while let Some(&(x, y)) = stack.last() {
            let candidates: Vec<(usize, usize)> = lattice
                .neighbours(x, y)
                .into_iter()
                .filter(|&(nx, ny)| !visited[lattice.index(nx, ny)])
                .collect();

            let Some(&(nx, ny)) = candidates.choose(rng) else {
                stack.pop();
                steps.push(floor(x, y));
                if let Some(&(px, py)) = stack.last() {
                    steps.push(wall_between(px, py, x, y));
                }
                continue;
            };

            visited[lattice.index(nx, ny)] = true;
            steps.push(between(x, y, nx, ny, Cell::Probe));
            steps.push(corridor(nx, ny, Cell::Probe));
            stack.push((nx, ny));
        }
    }
    steps
}
