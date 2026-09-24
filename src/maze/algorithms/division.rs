use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use super::{Lattice, between, floor, wall_between};
use crate::maze::grid::{Cell, Step};

/// Every open corridor and every wall between two open corridors: the empty
/// field that recursive division then splits up.
pub fn open_field(lattice: &Lattice) -> Vec<Step> {
    let mut steps: Vec<Step> = lattice
        .corridors()
        .into_iter()
        .map(|(x, y)| floor(x, y))
        .collect();
    for ((ax, ay), (bx, by)) in lattice.edges() {
        steps.push(wall_between(ax, ay, bx, by));
    }
    steps
}

/// A rectangle of corridors: x..x+width by y..y+height.
#[derive(Clone, Copy)]
struct Chamber {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

/// Recursive division: split the chamber with a wall that keeps a single
/// gap, then split both halves the same way until chambers are one corridor
/// thin. Needs `open_field` applied first. The only algorithm here that
/// builds walls instead of carving passages.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut steps = Vec::new();
    let mut chambers = vec![Chamber {
        x: 0,
        y: 0,
        width: lattice.width,
        height: lattice.height,
    }];

    while let Some(chamber) = chambers.pop() {
        if chamber.width < 2 || chamber.height < 2 {
            continue;
        }
        let horizontal = match chamber.width.cmp(&chamber.height) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => rng.random_bool(0.5),
        };
        let (first, second) = if horizontal {
            split_horizontally(lattice, chamber, &mut steps, rng)
        } else {
            split_vertically(lattice, chamber, &mut steps, rng)
        };
        chambers.push(second);
        chambers.push(first);
    }
    steps
}

/// A wall between rows `wall_y` and `wall_y + 1`, returning the two halves.
fn split_horizontally(
    lattice: &Lattice,
    chamber: Chamber,
    steps: &mut Vec<Step>,
    rng: &mut impl Rng,
) -> (Chamber, Chamber) {
    let wall_y = rng.random_range(chamber.y..chamber.y + chamber.height - 1);
    let crossings: Vec<usize> = (chamber.x..chamber.x + chamber.width)
        .filter(|&x| !lattice.is_blocked(x, wall_y) && !lattice.is_blocked(x, wall_y + 1))
        .collect();
    let gap = crossings.choose(rng).copied();
    for &x in &crossings {
        if Some(x) != gap {
            steps.push(between(x, wall_y, x, wall_y + 1, Cell::Wall));
        }
    }
    let top_height = wall_y - chamber.y + 1;
    (
        Chamber {
            height: top_height,
            ..chamber
        },
        Chamber {
            y: wall_y + 1,
            height: chamber.height - top_height,
            ..chamber
        },
    )
}

/// A wall between columns `wall_x` and `wall_x + 1`, returning the two halves.
fn split_vertically(
    lattice: &Lattice,
    chamber: Chamber,
    steps: &mut Vec<Step>,
    rng: &mut impl Rng,
) -> (Chamber, Chamber) {
    let wall_x = rng.random_range(chamber.x..chamber.x + chamber.width - 1);
    let crossings: Vec<usize> = (chamber.y..chamber.y + chamber.height)
        .filter(|&y| !lattice.is_blocked(wall_x, y) && !lattice.is_blocked(wall_x + 1, y))
        .collect();
    let gap = crossings.choose(rng).copied();
    for &y in &crossings {
        if Some(y) != gap {
            steps.push(between(wall_x, y, wall_x + 1, y, Cell::Wall));
        }
    }
    let left_width = wall_x - chamber.x + 1;
    (
        Chamber {
            width: left_width,
            ..chamber
        },
        Chamber {
            x: wall_x + 1,
            width: chamber.width - left_width,
            ..chamber
        },
    )
}
