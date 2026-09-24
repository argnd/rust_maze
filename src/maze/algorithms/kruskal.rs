use rand::Rng;
use rand::seq::SliceRandom;

use super::disjoint_sets::DisjointSets;
use super::{Lattice, floor, wall_between};
use crate::maze::grid::Step;

/// Randomised Kruskal: every corridor starts as its own group; walls are
/// visited in random order and knocked down only when they separate two
/// different groups, which then merge.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut groups = DisjointSets::new(lattice.len());

    // Each wall once: a corridor lists only its right and lower neighbours.
    let mut walls = Vec::new();
    for (x, y) in lattice.corridors() {
        for (nx, ny) in lattice.neighbours(x, y) {
            if nx > x || ny > y {
                walls.push(((x, y), (nx, ny)));
            }
        }
    }
    walls.shuffle(rng);

    let mut steps = Vec::new();
    for ((ax, ay), (bx, by)) in walls {
        if groups.merge(lattice.index(ax, ay), lattice.index(bx, by)) {
            steps.push(floor(ax, ay));
            steps.push(wall_between(ax, ay, bx, by));
            steps.push(floor(bx, by));
        }
    }
    steps
}
