use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};

use super::{Lattice, between, corridor, floor, wall_between};
use crate::maze::grid::{Cell, Step};

const NOT_ON_PATH: usize = usize::MAX;

/// Wilson's algorithm: start a tree with one random corridor, then from each
/// corridor outside it, random-walk until the walk hits the tree. Whenever
/// the walk crosses itself, the loop is erased. The surviving path joins the
/// tree. The result is a uniform spanning tree: every maze equally likely.
///
/// The walk is shown as probe squares, and erased loops turn back to wall,
/// so the animation shows the walk wandering and cutting its own loops.
pub fn carve(lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
    let mut in_tree = vec![false; lattice.len()];
    let mut position = vec![NOT_ON_PATH; lattice.len()];
    let mut steps = Vec::new();

    for pocket in lattice.pockets() {
        let &(seed_x, seed_y) = pocket.choose(rng).expect("pockets are never empty");
        in_tree[lattice.index(seed_x, seed_y)] = true;
        steps.push(floor(seed_x, seed_y));

        let mut order = pocket;
        order.shuffle(rng);
        for start in order {
            if !in_tree[lattice.index(start.0, start.1)] {
                walk(lattice, start, &mut in_tree, &mut position, &mut steps, rng);
            }
        }
    }
    steps
}

/// One loop-erased walk from `start` until it touches the tree, then commits
/// the path. `position` maps a corridor to its index in the current path and
/// is left all NOT_ON_PATH again on return.
fn walk(
    lattice: &Lattice,
    start: (usize, usize),
    in_tree: &mut [bool],
    position: &mut [usize],
    steps: &mut Vec<Step>,
    rng: &mut impl Rng,
) {
    let mut path = vec![start];
    position[lattice.index(start.0, start.1)] = 0;
    steps.push(corridor(start.0, start.1, Cell::Probe));

    loop {
        let (x, y) = *path.last().expect("the path always holds its start");
        // Walks only start in pockets of two or more, so there is a neighbour.
        let &(nx, ny) = lattice
            .neighbours(x, y)
            .choose(rng)
            .expect("a corridor in a pocket of two or more has a neighbour");
        let next = lattice.index(nx, ny);

        if in_tree[next] {
            for i in 0..path.len() {
                let (px, py) = path[i];
                let (qx, qy) = path.get(i + 1).copied().unwrap_or((nx, ny));
                in_tree[lattice.index(px, py)] = true;
                position[lattice.index(px, py)] = NOT_ON_PATH;
                steps.push(floor(px, py));
                steps.push(wall_between(px, py, qx, qy));
            }
            return;
        }

        if position[next] != NOT_ON_PATH {
            // The walk crossed itself: erase everything after the crossing.
            let keep = position[next];
            while path.len() > keep + 1 {
                let (px, py) = path.pop().expect("longer than keep + 1");
                position[lattice.index(px, py)] = NOT_ON_PATH;
                steps.push(corridor(px, py, Cell::Wall));
                let &(qx, qy) = path.last().expect("keep is still on the path");
                steps.push(between(qx, qy, px, py, Cell::Wall));
            }
            continue;
        }

        position[next] = path.len();
        path.push((nx, ny));
        steps.push(between(x, y, nx, ny, Cell::Probe));
        steps.push(corridor(nx, ny, Cell::Probe));
    }
}
