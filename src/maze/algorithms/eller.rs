use std::collections::BTreeMap;

use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use super::disjoint_sets::DisjointSets;
use super::{Lattice, floor, wall_between};
use crate::maze::grid::Step;

/// Eller's algorithm: one row at a time, tracking which corridors of the
/// row are already connected (sets). Randomly join neighbours of different
/// sets, then send every set at least one link down so none is left behind.
/// The last row joins everything that is still apart. It only ever needs
/// the current row, so it could generate an endless maze.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut sets = DisjointSets::new(lattice.len());
    let mut steps = Vec::new();

    for y in 0..lattice.height {
        let last_row = y + 1 == lattice.height;
        for x in 0..lattice.width {
            if !lattice.is_blocked(x, y) {
                steps.push(floor(x, y));
            }
        }

        for x in 0..lattice.width.saturating_sub(1) {
            if lattice.is_blocked(x, y) || lattice.is_blocked(x + 1, y) {
                continue;
            }
            let (a, b) = (lattice.index(x, y), lattice.index(x + 1, y));
            if sets.find(a) != sets.find(b) && (last_row || rng.random_bool(0.5)) {
                sets.merge(a, b);
                steps.push(wall_between(x, y, x + 1, y));
            }
        }
        if last_row {
            break;
        }

        // BTreeMap, not HashMap: its iteration order is fixed, which keeps a
        // seed producing the same maze every run.
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for x in 0..lattice.width {
            if !lattice.is_blocked(x, y) && !lattice.is_blocked(x, y + 1) {
                let root = sets.find(lattice.index(x, y));
                groups.entry(root).or_default().push(x);
            }
        }
        for xs in groups.values() {
            let mut down: Vec<usize> = xs
                .iter()
                .copied()
                .filter(|_| rng.random_bool(0.35))
                .collect();
            if down.is_empty() {
                down.push(*xs.choose(rng).expect("groups are never empty"));
            }
            for x in down {
                sets.merge(lattice.index(x, y), lattice.index(x, y + 1));
                steps.push(wall_between(x, y, x, y + 1));
            }
        }
    }
    steps
}
