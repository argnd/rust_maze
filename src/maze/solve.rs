use std::collections::VecDeque;

use crate::maze::grid::{Cell, Grid, neighbours};

/// A breadth-first flood from the start and the shortest path it yields.
pub struct Solution {
    /// Every reachable square with its distance from the start, in the order
    /// the flood reaches them, so distances never decrease along the list.
    pub flood: Vec<(usize, usize, usize)>,
    pub max_distance: usize,
    /// Shortest path from Start to End, both included.
    pub path: Vec<(usize, usize)>,
}

/// None when the grid has no Start, no End, or no route between them.
pub fn solve(grid: &Grid) -> Option<Solution> {
    let start = grid.find(Cell::Start)?;
    let end = grid.find(Cell::End)?;
    let (width, height) = (grid.width(), grid.height());
    let index = |x: usize, y: usize| y * width + x;

    let mut distance = vec![usize::MAX; width * height];
    let mut came_from = vec![usize::MAX; width * height];
    let mut flood = Vec::new();
    let mut queue = VecDeque::from([start]);
    distance[index(start.0, start.1)] = 0;

    while let Some((x, y)) = queue.pop_front() {
        let d = distance[index(x, y)];
        flood.push((x, y, d));
        for (nx, ny) in neighbours(x, y, width, height) {
            if grid.walkable(nx, ny) && distance[index(nx, ny)] == usize::MAX {
                distance[index(nx, ny)] = d + 1;
                came_from[index(nx, ny)] = index(x, y);
                queue.push_back((nx, ny));
            }
        }
    }

    if distance[index(end.0, end.1)] == usize::MAX {
        return None;
    }
    let mut path = vec![end];
    let mut current = index(end.0, end.1);
    while current != index(start.0, start.1) {
        current = came_from[current];
        path.push((current % width, current / width));
    }
    path.reverse();

    let max_distance = flood.last().map_or(0, |&(_, _, d)| d);
    Some(Solution {
        flood,
        max_distance,
        path,
    })
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;
    use crate::maze::MazeKind;
    use crate::maze::algorithms::{Algorithm, generate};

    fn finished(kind: MazeKind, algorithm: Algorithm, seed: u64) -> Grid {
        let generation = generate(
            kind,
            algorithm,
            20,
            15,
            0.5,
            &mut StdRng::seed_from_u64(seed),
        );
        let mut grid = Grid::new(20, 15);
        for step in generation.steps {
            grid.apply(step);
        }
        grid
    }

    #[test]
    fn path_runs_from_start_to_end_through_adjacent_walkable_squares() {
        for kind in MazeKind::ALL {
            for algorithm in Algorithm::ALL {
                let grid = finished(kind, algorithm, 3);
                let solution = solve(&grid).expect("every generated maze is solvable");
                let path = &solution.path;
                assert_eq!(path.first(), grid.find(Cell::Start).as_ref());
                assert_eq!(path.last(), grid.find(Cell::End).as_ref());
                for pair in path.windows(2) {
                    let ((ax, ay), (bx, by)) = (pair[0], pair[1]);
                    assert_eq!(
                        ax.abs_diff(bx) + ay.abs_diff(by),
                        1,
                        "{kind:?} {algorithm:?}"
                    );
                }
                assert!(path.iter().all(|&(x, y)| grid.walkable(x, y)));
            }
        }
    }

    #[test]
    fn path_is_as_short_as_the_flood_says() {
        let grid = finished(MazeKind::Imperfect, Algorithm::Kruskal, 11);
        let solution = solve(&grid).unwrap();
        let end = grid.find(Cell::End).unwrap();
        let end_distance = solution
            .flood
            .iter()
            .find(|&&(x, y, _)| (x, y) == end)
            .map(|&(_, _, d)| d)
            .unwrap();
        assert_eq!(solution.path.len(), end_distance + 1);
        assert!(solution.flood.windows(2).all(|pair| pair[0].2 <= pair[1].2));
    }

    #[test]
    fn no_route_means_no_solution() {
        let mut grid = Grid::new(3, 1);
        grid.set(1, 1, Cell::Start);
        grid.set(5, 1, Cell::End);
        assert!(solve(&grid).is_none());
    }
}
