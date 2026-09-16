use rand::{Rng, RngExt};

use super::{floor, wall_between};
use crate::maze::grid::{Step, neighbours};

/// Randomised Prim: grow the visited region by knocking down a random wall
/// on its frontier. The frontier holds (from, to) pairs: `from` is visited,
/// `to` is the corridor on the other side of the wall.
pub fn carve(corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Step> {
    let index = |x: usize, y: usize| y * corridors_x + x;
    let mut visited = vec![false; corridors_x * corridors_y];
    let mut steps = vec![floor(0, 0)];
    let mut frontier = Vec::new();

    visited[index(0, 0)] = true;
    for neighbour in neighbours(0, 0, corridors_x, corridors_y) {
        frontier.push(((0, 0), neighbour));
    }

    while !frontier.is_empty() {
        let i = rng.random_range(0..frontier.len());
        let ((from_x, from_y), (to_x, to_y)) = frontier.swap_remove(i);

        // A corridor can sit in the frontier several times, once per visited
        // neighbour; only the first pick carves, the others are dropped here.
        if visited[index(to_x, to_y)] {
            continue;
        }
        visited[index(to_x, to_y)] = true;
        steps.push(wall_between(from_x, from_y, to_x, to_y));
        steps.push(floor(to_x, to_y));

        for (next_x, next_y) in neighbours(to_x, to_y, corridors_x, corridors_y) {
            if !visited[index(next_x, next_y)] {
                frontier.push(((to_x, to_y), (next_x, next_y)));
            }
        }
    }

    steps
}
