use rand::Rng;
use rand::seq::IndexedRandom;

use super::{Lattice, floor, wall_between};
use crate::maze::grid::Step;

/// Hunt-and-kill: random walk through unvisited corridors; when stuck, scan
/// row by row for an unvisited corridor next to the maze, join it, and walk
/// again from there. Like the backtracker, but it never retraces its steps.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let corridors = lattice.corridors();
    let mut visited = vec![false; lattice.len()];
    let mut steps = Vec::new();
    let mut current: Option<(usize, usize)> = None;

    loop {
        let Some((x, y)) = current else {
            current = hunt(lattice, &corridors, &mut visited, &mut steps, rng);
            if current.is_none() {
                break;
            }
            continue;
        };

        let candidates: Vec<(usize, usize)> = lattice
            .neighbours(x, y)
            .into_iter()
            .filter(|&(nx, ny)| !visited[lattice.index(nx, ny)])
            .collect();
        current = candidates.choose(rng).copied();
        if let Some((nx, ny)) = current {
            visited[lattice.index(nx, ny)] = true;
            steps.push(wall_between(x, y, nx, ny));
            steps.push(floor(nx, ny));
        }
    }
    steps
}

/// The next place to walk from: the first unvisited corridor that touches the
/// maze, joined to it; or, when none touches it, the first unvisited corridor
/// of a pocket not reached yet. None once everything is visited.
fn hunt(
    lattice: &Lattice,
    corridors: &[(usize, usize)],
    visited: &mut [bool],
    steps: &mut Vec<Step>,
    rng: &mut impl Rng,
) -> Option<(usize, usize)> {
    for &(x, y) in corridors {
        if visited[lattice.index(x, y)] {
            continue;
        }
        let touching: Vec<(usize, usize)> = lattice
            .neighbours(x, y)
            .into_iter()
            .filter(|&(nx, ny)| visited[lattice.index(nx, ny)])
            .collect();
        if let Some(&(vx, vy)) = touching.choose(rng) {
            visited[lattice.index(x, y)] = true;
            steps.push(wall_between(vx, vy, x, y));
            steps.push(floor(x, y));
            return Some((x, y));
        }
    }

    let &(x, y) = corridors
        .iter()
        .find(|&&(x, y)| !visited[lattice.index(x, y)])?;
    visited[lattice.index(x, y)] = true;
    steps.push(floor(x, y));
    Some((x, y))
}
