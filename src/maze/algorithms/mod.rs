mod backtracker;
mod binary_tree;
mod braid;
mod disjoint_sets;
mod division;
mod dungeon;
mod eller;
mod growing_tree;
mod hunt_and_kill;
mod kruskal;
mod prim;
mod sidewinder;
mod wilson;

use rand::Rng;
use rand::seq::SliceRandom;

use crate::maze::MazeKind;
use crate::maze::grid::{Cell, Grid, Step, neighbours};
use disjoint_sets::DisjointSets;

/// The corridor lattice an algorithm carves: its size in corridors and which
/// corridors are off-limits (inside dungeon rooms). Algorithms work in
/// corridor coordinates only; the step helpers below translate to squares.
pub struct Lattice {
    width: usize,
    height: usize,
    blocked: Vec<bool>,
}

impl Lattice {
    fn open(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            blocked: vec![false; width * height],
        }
    }

    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn block(&mut self, x: usize, y: usize) {
        let i = self.index(x, y);
        self.blocked[i] = true;
    }

    fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.blocked[self.index(x, y)]
    }

    /// Every corridor that is not blocked, row by row.
    fn corridors(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.len());
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_blocked(x, y) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// Orthogonal neighbours that are not blocked.
    fn neighbours(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        neighbours(x, y, self.width, self.height)
            .into_iter()
            .filter(|&(nx, ny)| !self.is_blocked(nx, ny))
            .collect()
    }

    /// Each pair of adjacent open corridors once: right and lower neighbours.
    fn edges(&self) -> Vec<((usize, usize), (usize, usize))> {
        let mut out = Vec::new();
        for (x, y) in self.corridors() {
            for (nx, ny) in self.neighbours(x, y) {
                if nx > x || ny > y {
                    out.push(((x, y), (nx, ny)));
                }
            }
        }
        out
    }

    /// The groups of open corridors that can reach each other; rooms can
    /// split the lattice into several.
    fn pockets(&self) -> Vec<Vec<(usize, usize)>> {
        let mut seen = vec![false; self.len()];
        let mut pockets = Vec::new();
        for (x, y) in self.corridors() {
            if seen[self.index(x, y)] {
                continue;
            }
            seen[self.index(x, y)] = true;
            let mut pocket = Vec::new();
            let mut stack = vec![(x, y)];
            while let Some((cx, cy)) = stack.pop() {
                pocket.push((cx, cy));
                for (nx, ny) in self.neighbours(cx, cy) {
                    if !seen[self.index(nx, ny)] {
                        seen[self.index(nx, ny)] = true;
                        stack.push((nx, ny));
                    }
                }
            }
            pockets.push(pocket);
        }
        pockets
    }
}

/// The corridor carvers the user can choose from. Each one produces a
/// perfect maze on its own; the kind decides what happens around it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    Backtracker,
    Prim,
    Kruskal,
    Wilson,
    HuntAndKill,
    GrowingTree,
    BinaryTree,
    Sidewinder,
    Eller,
    Division,
}

impl Algorithm {
    pub const ALL: [Algorithm; 10] = [
        Algorithm::Backtracker,
        Algorithm::Prim,
        Algorithm::Kruskal,
        Algorithm::Wilson,
        Algorithm::HuntAndKill,
        Algorithm::GrowingTree,
        Algorithm::BinaryTree,
        Algorithm::Sidewinder,
        Algorithm::Eller,
        Algorithm::Division,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Algorithm::Backtracker => "Recursive backtracker",
            Algorithm::Prim => "Prim",
            Algorithm::Kruskal => "Kruskal",
            Algorithm::Wilson => "Wilson",
            Algorithm::HuntAndKill => "Hunt-and-kill",
            Algorithm::GrowingTree => "Growing tree",
            Algorithm::BinaryTree => "Binary tree",
            Algorithm::Sidewinder => "Sidewinder",
            Algorithm::Eller => "Eller",
            Algorithm::Division => "Recursive division",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Algorithm::Backtracker => {
                "Depth-first walk that backs up when stuck. Long, winding corridors."
            }
            Algorithm::Prim => {
                "Grows from a seed by opening a random frontier wall. Short, bushy dead ends."
            }
            Algorithm::Kruskal => "Opens random walls between separate islands until all merge.",
            Algorithm::Wilson => "Loop-erased random walks: every possible maze is equally likely.",
            Algorithm::HuntAndKill => {
                "Random walk; when stuck, hunts for a new start beside the maze."
            }
            Algorithm::GrowingTree => {
                "Half backtracker, half Prim: extends the newest or a random active cell."
            }
            Algorithm::BinaryTree => {
                "Each cell links north or west. Instant, with a strong diagonal bias."
            }
            Algorithm::Sidewinder => {
                "Runs east, then links each run north once. Open top corridor."
            }
            Algorithm::Eller => "Builds one row at a time with merging sets; could run forever.",
            Algorithm::Division => "Starts empty and splits chambers with walls that keep one gap.",
        }
    }

    /// Name of the carving phase, shown while it replays.
    fn phase_name(self) -> &'static str {
        match self {
            Algorithm::Backtracker => "Walking and backtracking",
            Algorithm::Prim => "Growing the frontier",
            Algorithm::Kruskal => "Merging islands",
            Algorithm::Wilson => "Loop-erased random walks",
            Algorithm::HuntAndKill => "Walking, then hunting",
            Algorithm::GrowingTree => "Growing the tree",
            Algorithm::BinaryTree => "Linking north or west",
            Algorithm::Sidewinder => "Running east, linking north",
            Algorithm::Eller => "Sweeping row by row",
            Algorithm::Division => "Dividing chambers",
        }
    }

    /// Wilson's first walks wander for a long time before hitting the tree;
    /// they replay faster so the maze still appears in reasonable time.
    fn pace(self) -> f32 {
        match self {
            Algorithm::Wilson => 3.0,
            _ => 1.0,
        }
    }

    /// Steps that set up the field before carving; only recursive division,
    /// which works by adding walls to an open field, needs any.
    fn prepare(self, lattice: &Lattice) -> Vec<Step> {
        match self {
            Algorithm::Division => division::open_field(lattice),
            _ => Vec::new(),
        }
    }

    fn carve(self, lattice: &Lattice, rng: &mut impl Rng) -> Vec<Step> {
        match self {
            Algorithm::Backtracker => backtracker::carve(lattice, rng),
            Algorithm::Prim => prim::carve(lattice, rng),
            Algorithm::Kruskal => kruskal::carve(lattice, rng),
            Algorithm::Wilson => wilson::carve(lattice, rng),
            Algorithm::HuntAndKill => hunt_and_kill::carve(lattice, rng),
            Algorithm::GrowingTree => growing_tree::carve(lattice, rng),
            Algorithm::BinaryTree => binary_tree::carve(lattice, rng),
            Algorithm::Sidewinder => sidewinder::carve(lattice, rng),
            Algorithm::Eller => eller::carve(lattice, rng),
            Algorithm::Division => division::carve(lattice, rng),
        }
    }
}

/// A named stretch of the step list, from `start` to the next phase. `pace`
/// multiplies the playback speed: bulk stages run faster so the interesting
/// part isn't buried under thousands of routine steps.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Phase {
    pub name: &'static str,
    pub start: usize,
    pub pace: f32,
}

/// Everything the viewer replays: the steps, and the phases they fall into.
pub struct Generation {
    pub steps: Vec<Step>,
    pub phases: Vec<Phase>,
}

impl Generation {
    /// The phase the step at `index` belongs to.
    pub fn phase_at(&self, index: usize) -> Option<&Phase> {
        self.phases.iter().rev().find(|phase| phase.start <= index)
    }
}

/// Collects steps and opens a new phase at each stage of a generation.
#[derive(Default)]
struct Recorder {
    steps: Vec<Step>,
    phases: Vec<Phase>,
}

impl Recorder {
    fn phase(&mut self, name: &'static str, pace: f32) {
        self.phases.push(Phase {
            name,
            start: self.steps.len(),
            pace,
        });
    }

    fn extend(&mut self, steps: impl IntoIterator<Item = Step>) {
        self.steps.extend(steps);
    }

    /// Prepends Start at `start` and End at the farthest walkable square, so
    /// both are visible during the whole animation. The steps are replayed
    /// once, silently, to know the final layout.
    fn finish(self, corridors_x: usize, corridors_y: usize, start: (usize, usize)) -> Generation {
        let finished = grid_after(corridors_x, corridors_y, &self.steps);
        let (end_x, end_y) = finished.farthest_walkable_from(start.0, start.1);

        let mut steps = Vec::with_capacity(self.steps.len() + 2);
        steps.push(Step {
            x: start.0,
            y: start.1,
            cell: Cell::Start,
        });
        steps.push(Step {
            x: end_x,
            y: end_y,
            cell: Cell::End,
        });
        steps.extend(self.steps);

        let mut phases: Vec<Phase> = self
            .phases
            .into_iter()
            .map(|phase| Phase {
                start: phase.start + 2,
                ..phase
            })
            .collect();
        if let Some(first) = phases.first_mut() {
            first.start = 0;
        }
        Generation { steps, phases }
    }
}

/// `braiding` is the share of dead ends turned into loops for imperfect
/// mazes (0 = none, 1 = all); ignored by the other kinds.
pub fn generate(
    kind: MazeKind,
    algorithm: Algorithm,
    corridors_x: usize,
    corridors_y: usize,
    braiding: f32,
    rng: &mut impl Rng,
) -> Generation {
    let mut recorder = Recorder::default();
    let start = match kind {
        MazeKind::Perfect => {
            carve_into(
                &mut recorder,
                algorithm,
                &Lattice::open(corridors_x, corridors_y),
                rng,
            );
            (1, 1)
        }
        MazeKind::Imperfect => {
            carve_into(
                &mut recorder,
                algorithm,
                &Lattice::open(corridors_x, corridors_y),
                rng,
            );
            let mut grid = grid_after(corridors_x, corridors_y, &recorder.steps);
            recorder.phase("Braiding loops", 1.0);
            recorder.extend(braid::braid(&mut grid, braiding, rng));
            (1, 1)
        }
        MazeKind::Dungeon => {
            dungeon::generate(&mut recorder, algorithm, corridors_x, corridors_y, rng)
        }
    };
    recorder.finish(corridors_x, corridors_y, start)
}

/// Runs one algorithm on the lattice, then joins whatever it left in
/// separate pieces inside a pocket. Algorithms that sweep rows (binary
/// tree, sidewinder, Eller, division) can leave pieces when rooms cut the
/// rows; on an open lattice this pass finds nothing to do.
fn carve_into(
    recorder: &mut Recorder,
    algorithm: Algorithm,
    lattice: &Lattice,
    rng: &mut impl Rng,
) {
    let prepared = algorithm.prepare(lattice);
    if !prepared.is_empty() {
        recorder.phase("Clearing the field", 25.0);
        recorder.extend(prepared);
    }
    recorder.phase(algorithm.phase_name(), algorithm.pace());
    recorder.extend(algorithm.carve(lattice, rng));

    let grid = grid_after(lattice.width, lattice.height, &recorder.steps);
    let stitches = join_pieces(lattice, &grid, rng);
    if !stitches.is_empty() {
        recorder.phase("Joining stray corridors", 1.0);
        recorder.extend(stitches);
    }
}

/// Opens random walls between corridors of the same pocket that the carving
/// left unconnected, Kruskal-style, so no loop is created.
fn join_pieces(lattice: &Lattice, grid: &Grid, rng: &mut impl Rng) -> Vec<Step> {
    let mut pieces = DisjointSets::new(lattice.len());
    let mut closed = Vec::new();
    for ((ax, ay), (bx, by)) in lattice.edges() {
        let wall = wall_between(ax, ay, bx, by);
        if grid.walkable(wall.x, wall.y) {
            pieces.merge(lattice.index(ax, ay), lattice.index(bx, by));
        } else {
            closed.push(((ax, ay), (bx, by)));
        }
    }
    closed.shuffle(rng);

    let mut steps = Vec::new();
    for ((ax, ay), (bx, by)) in closed {
        if pieces.merge(lattice.index(ax, ay), lattice.index(bx, by)) {
            steps.push(wall_between(ax, ay, bx, by));
        }
    }
    steps
}

/// The grid as it looks once every step has been applied.
fn grid_after(corridors_x: usize, corridors_y: usize, steps: &[Step]) -> Grid {
    let mut grid = Grid::new(corridors_x, corridors_y);
    for step in steps {
        grid.apply(*step);
    }
    grid
}

/// A corridor square set to `cell`, in square coordinates.
fn corridor(corridor_x: usize, corridor_y: usize, cell: Cell) -> Step {
    Step {
        x: corridor_x * 2 + 1,
        y: corridor_y * 2 + 1,
        cell,
    }
}

/// The wall square between two adjacent corridors, their midpoint, set to `cell`.
fn between(ax: usize, ay: usize, bx: usize, by: usize, cell: Cell) -> Step {
    Step {
        x: ax + bx + 1,
        y: ay + by + 1,
        cell,
    }
}

/// The floor step for one corridor.
fn floor(corridor_x: usize, corridor_y: usize) -> Step {
    corridor(corridor_x, corridor_y, Cell::Floor)
}

/// Opens the wall square between two adjacent corridors.
fn wall_between(ax: usize, ay: usize, bx: usize, by: usize) -> Step {
    between(ax, ay, bx, by, Cell::Floor)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    const SIZES: [(usize, usize); 3] = [(5, 5), (20, 15), (60, 45)];

    fn count_cells(grid: &Grid, wanted: impl Fn(Cell) -> bool) -> usize {
        let mut count = 0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if wanted(grid.get(x, y)) {
                    count += 1;
                }
            }
        }
        count
    }

    fn walkable_count(grid: &Grid) -> usize {
        count_cells(grid, |cell| {
            matches!(cell, Cell::Floor | Cell::Door | Cell::Start | Cell::End)
        })
    }

    fn reachable_from(grid: &Grid, start: (usize, usize)) -> usize {
        let (w, h) = (grid.width(), grid.height());
        let mut seen = vec![false; w * h];
        let mut stack = vec![start];
        seen[start.1 * w + start.0] = true;
        let mut count = 0;
        while let Some((x, y)) = stack.pop() {
            count += 1;
            for (nx, ny) in neighbours(x, y, w, h) {
                if grid.walkable(nx, ny) && !seen[ny * w + nx] {
                    seen[ny * w + nx] = true;
                    stack.push((nx, ny));
                }
            }
        }
        count
    }

    fn finished_grid(
        kind: MazeKind,
        algorithm: Algorithm,
        cx: usize,
        cy: usize,
        braiding: f32,
    ) -> Grid {
        let generation = generate(kind, algorithm, cx, cy, braiding, &mut rand::rng());
        grid_after(cx, cy, &generation.steps)
    }

    #[test]
    fn steps_start_with_start_and_end() {
        for kind in MazeKind::ALL {
            let generation = generate(kind, Algorithm::Backtracker, 20, 15, 0.5, &mut rand::rng());
            assert_eq!(generation.steps[0].cell, Cell::Start);
            assert_eq!(generation.steps[1].cell, Cell::End);
        }
    }

    #[test]
    fn perfect_mazes_are_spanning_trees() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let grid = finished_grid(MazeKind::Perfect, algorithm, cx, cy, 0.0);
                // A tree over n corridors opens exactly n - 1 walls.
                assert_eq!(
                    walkable_count(&grid),
                    2 * cx * cy - 1,
                    "{algorithm:?} {cx}x{cy}"
                );
                assert_eq!(reachable_from(&grid, (1, 1)), walkable_count(&grid));
            }
        }
    }

    #[test]
    fn imperfect_mazes_have_loops_and_stay_connected() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let grid = finished_grid(MazeKind::Imperfect, algorithm, cx, cy, 1.0);
                assert!(
                    walkable_count(&grid) > 2 * cx * cy - 1,
                    "{algorithm:?} {cx}x{cy}"
                );
                assert_eq!(reachable_from(&grid, (1, 1)), walkable_count(&grid));
            }
        }
    }

    #[test]
    fn dungeons_are_connected_through_doors() {
        for algorithm in Algorithm::ALL {
            for (cx, cy) in SIZES {
                let grid = finished_grid(MazeKind::Dungeon, algorithm, cx, cy, 0.0);
                let start = grid.find(Cell::Start).expect("dungeon has a start");
                assert_eq!(
                    reachable_from(&grid, start),
                    walkable_count(&grid),
                    "{algorithm:?} {cx}x{cy}"
                );
                if cx >= 20 {
                    let doors = count_cells(&grid, |cell| cell == Cell::Door);
                    assert!(doors >= 1, "{algorithm:?} {cx}x{cy}");
                }
            }
        }
    }

    #[test]
    fn finished_mazes_hold_no_probes() {
        for kind in MazeKind::ALL {
            for algorithm in Algorithm::ALL {
                let grid = finished_grid(kind, algorithm, 20, 15, 0.5);
                assert_eq!(
                    count_cells(&grid, |cell| cell == Cell::Probe),
                    0,
                    "{kind:?} {algorithm:?}"
                );
            }
        }
    }

    #[test]
    fn same_seed_gives_same_maze() {
        for kind in MazeKind::ALL {
            for algorithm in Algorithm::ALL {
                let first = generate(kind, algorithm, 20, 15, 0.5, &mut StdRng::seed_from_u64(7));
                let second = generate(kind, algorithm, 20, 15, 0.5, &mut StdRng::seed_from_u64(7));
                assert_eq!(first.steps, second.steps, "{kind:?} {algorithm:?}");
            }
        }
    }

    #[test]
    fn phases_start_at_zero_and_are_ordered() {
        for kind in MazeKind::ALL {
            for algorithm in Algorithm::ALL {
                let generation = generate(kind, algorithm, 20, 15, 0.5, &mut rand::rng());
                assert_eq!(generation.phases[0].start, 0, "{kind:?} {algorithm:?}");
                assert!(
                    generation
                        .phases
                        .windows(2)
                        .all(|pair| pair[0].start <= pair[1].start),
                    "{kind:?} {algorithm:?}"
                );
                assert!(generation.phases.iter().all(|phase| phase.pace > 0.0));
            }
        }
    }
}
