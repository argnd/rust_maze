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
    let mut walls = lattice.edges();
    walls.shuffle(rng);

    // Corridors that no wall ever reaches (a pocket of one) still need a floor.
    let mut steps: Vec<Step> = lattice
        .corridors()
        .into_iter()
        .filter(|&(x, y)| lattice.neighbours(x, y).is_empty())
        .map(|(x, y)| floor(x, y))
        .collect();

    for ((ax, ay), (bx, by)) in walls {
        if groups.merge(lattice.index(ax, ay), lattice.index(bx, by)) {
            steps.push(floor(ax, ay));
            steps.push(wall_between(ax, ay, bx, by));
            steps.push(floor(bx, by));
        }
    }
    steps
}
