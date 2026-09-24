use crate::maze::grid::{Cell, Grid, neighbours};

/// Shape figures for a finished maze, shown next to the viewer.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub walkable: usize,
    /// Walkable squares with a single way out.
    pub dead_ends: usize,
    /// Walkable squares with three or four ways out.
    pub junctions: usize,
    /// Independent cycles: how many walls could be put back without cutting
    /// the maze in two. Zero for a perfect maze.
    pub loops: usize,
    pub doors: usize,
}

pub fn measure(grid: &Grid) -> Stats {
    let (width, height) = (grid.width(), grid.height());
    let mut stats = Stats::default();
    let mut edges: usize = 0;

    for y in 0..height {
        for x in 0..width {
            if !grid.walkable(x, y) {
                continue;
            }
            stats.walkable += 1;
            if grid.get(x, y) == Cell::Door {
                stats.doors += 1;
            }
            let exits = neighbours(x, y, width, height)
                .into_iter()
                .filter(|&(nx, ny)| grid.walkable(nx, ny))
                .count();
            match exits {
                1 => stats.dead_ends += 1,
                3 | 4 => stats.junctions += 1,
                _ => {}
            }
            // Count each edge once, from its left or upper end.
            if x + 1 < width && grid.walkable(x + 1, y) {
                edges += 1;
            }
            if y + 1 < height && grid.walkable(x, y + 1) {
                edges += 1;
            }
        }
    }

    // Cycle rank of a connected graph: edges - vertices + 1.
    stats.loops = (edges + 1).saturating_sub(stats.walkable);
    stats
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;
    use crate::maze::MazeKind;
    use crate::maze::algorithms::{Algorithm, generate};

    fn measured(kind: MazeKind, braiding: f32) -> Stats {
        let generation = generate(
            kind,
            Algorithm::Backtracker,
            20,
            15,
            braiding,
            &mut StdRng::seed_from_u64(5),
        );
        let mut grid = Grid::new(20, 15);
        for step in generation.steps {
            grid.apply(step);
        }
        measure(&grid)
    }

    #[test]
    fn perfect_mazes_have_no_loops() {
        let stats = measured(MazeKind::Perfect, 0.0);
        assert_eq!(stats.loops, 0);
        assert!(stats.dead_ends > 0);
        assert_eq!(stats.doors, 0);
    }

    #[test]
    fn braiding_every_dead_end_creates_loops() {
        let stats = measured(MazeKind::Imperfect, 1.0);
        assert!(stats.loops > 0);
    }
}
