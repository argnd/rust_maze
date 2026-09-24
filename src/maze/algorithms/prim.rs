use rand::{Rng, RngExt};

use super::{Lattice, corridor, floor, wall_between};
use crate::maze::grid::{Cell, Step};

/// Randomised Prim: grow the visited region by knocking down a random wall
/// on its frontier. The frontier holds (from, to) pairs: `from` is visited,
/// `to` is the corridor on the other side of the wall. Frontier corridors
/// are shown as probe squares until they are carved.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut visited = vec![false; lattice.len()];
    let mut queued = vec![false; lattice.len()];
    let mut steps = Vec::new();

    // One growth per unvisited corridor: a single growth on an open lattice,
    // one per pocket when rooms split it apart.
    for (seed_x, seed_y) in lattice.corridors() {
        if visited[lattice.index(seed_x, seed_y)] {
            continue;
        }
        visited[lattice.index(seed_x, seed_y)] = true;
        steps.push(floor(seed_x, seed_y));
        let mut frontier = Vec::new();
        push_frontier(
            lattice,
            &visited,
            &mut queued,
            &mut frontier,
            &mut steps,
            (seed_x, seed_y),
        );

        while !frontier.is_empty() {
            let i = rng.random_range(0..frontier.len());
            let ((from_x, from_y), (to_x, to_y)) = frontier.swap_remove(i);

            // A corridor can sit in the frontier several times, once per visited
            // neighbour; only the first pick carves, the others are dropped here.
            if visited[lattice.index(to_x, to_y)] {
                continue;
            }
            visited[lattice.index(to_x, to_y)] = true;
            steps.push(wall_between(from_x, from_y, to_x, to_y));
            steps.push(floor(to_x, to_y));
            push_frontier(
                lattice,
                &visited,
                &mut queued,
                &mut frontier,
                &mut steps,
                (to_x, to_y),
            );
        }
    }
    steps
}

type Wall = ((usize, usize), (usize, usize));

fn push_frontier(
    lattice: &Lattice,
    visited: &[bool],
    queued: &mut [bool],
    frontier: &mut Vec<Wall>,
    steps: &mut Vec<Step>,
    (x, y): (usize, usize),
) {
    for (next_x, next_y) in lattice.neighbours(x, y) {
        let next = lattice.index(next_x, next_y);
        if visited[next] {
            continue;
        }
        frontier.push(((x, y), (next_x, next_y)));
        if !queued[next] {
            queued[next] = true;
            steps.push(corridor(next_x, next_y, Cell::Probe));
        }
    }
}
