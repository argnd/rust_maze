use rand::Rng;
use rand::seq::IndexedRandom;

use super::{floor, wall_between};
use crate::maze::grid::{Step, neighbours};

/// Recursive backtracker, iterative form: walk to a random unvisited
/// neighbour, knocking down the wall in between; when stuck, step back.
pub fn carve(corridors_x: usize, corridors_y: usize, rng: &mut impl Rng) -> Vec<Step> {
    let mut visited = vec![false; corridors_x * corridors_y];
    let mut steps = Vec::new();
    let mut stack = vec![(0usize, 0usize)];
    visited[0] = true;
    steps.push(floor(0, 0));

    while let Some(&(x, y)) = stack.last() {
        let candidates: Vec<(usize, usize)> = neighbours(x, y, corridors_x, corridors_y)
            .into_iter()
            .filter(|&(nx, ny)| !visited[ny * corridors_x + nx])
            .collect();

        let Some(&(nx, ny)) = candidates.choose(rng) else {
            stack.pop();
            continue;
        };

        visited[ny * corridors_x + nx] = true;
        steps.push(wall_between(x, y, nx, ny));
        steps.push(floor(nx, ny));
        stack.push((nx, ny));
    }
    steps
}
