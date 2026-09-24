use rand::Rng;
use rand::seq::IndexedRandom;

use super::{Lattice, floor, wall_between};
use crate::maze::grid::Step;

/// Binary tree: every corridor links to its north or west neighbour, chosen
/// at random. No memory at all, and a telltale diagonal texture: the top row
/// and left column are always one straight corridor.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut steps = Vec::new();
    for (x, y) in lattice.corridors() {
        steps.push(floor(x, y));
        let mut options = Vec::with_capacity(2);
        if y > 0 && !lattice.is_blocked(x, y - 1) {
            options.push((x, y - 1));
        }
        if x > 0 && !lattice.is_blocked(x - 1, y) {
            options.push((x - 1, y));
        }
        if let Some(&(nx, ny)) = options.choose(rng) {
            steps.push(wall_between(x, y, nx, ny));
        }
    }
    steps
}
