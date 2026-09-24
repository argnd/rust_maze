use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use crate::maze::grid::{Cell, Grid, Step};

/// Turns dead ends of a finished maze into loops: each one, with the given
/// probability, gets one extra wall knocked down. Works on the grid as the
/// steps are applied, so a dead end already fixed by a neighbour's loop is
/// not touched twice.
pub fn braid(grid: &mut Grid, probability: f32, rng: &mut impl Rng) -> Vec<Step> {
    let mut steps = Vec::new();
    for y in (1..grid.height()).step_by(2) {
        for x in (1..grid.width()).step_by(2) {
            if !grid.walkable(x, y) || open_sides(grid, x, y) != 1 {
                continue;
            }
            if !rng.random_bool(probability.clamp(0.0, 1.0) as f64) {
                continue;
            }
            let closed: Vec<(usize, usize)> = inner_sides(grid, x, y)
                .into_iter()
                .filter(|&(wx, wy)| !grid.walkable(wx, wy))
                .collect();
            if let Some(&(wx, wy)) = closed.choose(rng) {
                let step = Step {
                    x: wx,
                    y: wy,
                    cell: Cell::Floor,
                };
                grid.apply(step);
                steps.push(step);
            }
        }
    }
    steps
}

/// The wall squares around a corridor square that have another corridor
/// behind them; walls on the outer border are left out.
fn inner_sides(grid: &Grid, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if x >= 2 {
        out.push((x - 1, y));
    }
    if y >= 2 {
        out.push((x, y - 1));
    }
    if x + 2 < grid.width() {
        out.push((x + 1, y));
    }
    if y + 2 < grid.height() {
        out.push((x, y + 1));
    }
    out
}

fn open_sides(grid: &Grid, x: usize, y: usize) -> usize {
    inner_sides(grid, x, y)
        .into_iter()
        .filter(|&(wx, wy)| grid.walkable(wx, wy))
        .count()
}
