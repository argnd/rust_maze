use rand::{Rng, RngExt};

use super::{floor, wall_between};
use crate::maze::grid::{Step, neighbours};

/// Randomised Prim: grow the visited region by knocking down a random wall
/// on its frontier.
pub fn carve(corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Step> {
    let mut visited = vec![false; corridors_x * corridors_y];
    let mut steps = Vec::new();
    let mut frontier = Vec::new();

    visited[0] = true;
    steps.push(floor(0, 0));

    for neighbour in neighbours(0, 0, corridors_x, corridors_y) {
        frontier.push(((0, 0), neighbour));
    }

    while !frontier.is_empty() {
        let i = rng.random_range(0..frontier.len());
        let ((from_x, from_y), (to_x, to_y)) = frontier.swap_remove(i);

        if visited[to_y * corridors_x + to_x] {
            continue;
        }
        visited[to_y * corridors_x + to_x] = true;
        steps.push(wall_between(from_x, from_y, to_x, to_y));
        steps.push(floor(to_x, to_y));

        for (next_x, next_y) in neighbours(to_x, to_y, corridors_x, corridors_y) {
            if !visited[next_y * corridors_x + next_x] {
                frontier.push(((to_x, to_y), (next_x, next_y)));
            }
        }
    }

    steps
}
