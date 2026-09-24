use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use super::{Lattice, floor, wall_between};
use crate::maze::grid::Step;

/// Sidewinder: sweep each row west to east, extending a run of linked
/// corridors; at random, close the run and link one of its corridors north.
/// The top row, which has nothing north of it, becomes one long corridor.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut steps = Vec::new();
    for y in 0..lattice.height {
        let mut run: Vec<usize> = Vec::new();
        for x in 0..lattice.width {
            if lattice.is_blocked(x, y) {
                continue;
            }
            steps.push(floor(x, y));
            run.push(x);

            let can_go_east = x + 1 < lattice.width && !lattice.is_blocked(x + 1, y);
            let northable: Vec<usize> = run
                .iter()
                .copied()
                .filter(|&rx| y > 0 && !lattice.is_blocked(rx, y - 1))
                .collect();
            let close_run = !can_go_east || (!northable.is_empty() && rng.random_bool(0.5));

            if close_run {
                if let Some(&rx) = northable.choose(rng) {
                    steps.push(wall_between(rx, y, rx, y - 1));
                }
                run.clear();
            } else {
                steps.push(wall_between(x, y, x + 1, y));
            }
        }
    }
    steps
}
