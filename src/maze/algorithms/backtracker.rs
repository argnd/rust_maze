use rand::Rng;
use rand::seq::IndexedRandom;

use crate::maze::grid::{neighbours, Cell, Step};

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
        // The wall square between two corridor squares is their midpoint.
        steps.push(Step { x: x + nx + 1, y: y + ny + 1, cell: Cell::Floor });
        steps.push(floor(nx, ny));
        stack.push((nx, ny));
    }
    steps
}

fn floor(corridor_x: usize, corridor_y: usize) -> Step {
    Step { x: corridor_x * 2 + 1, y: corridor_y * 2 + 1, cell: Cell::Floor }
}