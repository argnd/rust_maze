use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use super::{Lattice, corridor, floor, wall_between};
use crate::maze::grid::{Cell, Step};

/// Growing tree: keep a list of active corridors; extend either the newest
/// (backtracker behaviour) or a random one (Prim behaviour), half and half.
/// Active corridors are shown as probe squares until they are exhausted.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut visited = vec![false; lattice.len()];
    let mut steps = Vec::new();

    for (seed_x, seed_y) in lattice.corridors() {
        if visited[lattice.index(seed_x, seed_y)] {
            continue;
        }
        visited[lattice.index(seed_x, seed_y)] = true;
        steps.push(corridor(seed_x, seed_y, Cell::Probe));
        let mut active = vec![(seed_x, seed_y)];

        while !active.is_empty() {
            let i = if rng.random_bool(0.5) {
                active.len() - 1
            } else {
                rng.random_range(0..active.len())
            };
            let (x, y) = active[i];
            let candidates: Vec<(usize, usize)> = lattice
                .neighbours(x, y)
                .into_iter()
                .filter(|&(nx, ny)| !visited[lattice.index(nx, ny)])
                .collect();

            match candidates.choose(rng) {
                Some(&(nx, ny)) => {
                    visited[lattice.index(nx, ny)] = true;
                    steps.push(wall_between(x, y, nx, ny));
                    steps.push(corridor(nx, ny, Cell::Probe));
                    active.push((nx, ny));
                }
                None => {
                    // `remove`, not `swap_remove`: the order is what makes
                    // "newest" mean something.
                    active.remove(i);
                    steps.push(floor(x, y));
                }
            }
        }
    }
    steps
}
