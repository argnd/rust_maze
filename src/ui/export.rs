use std::path::Path;

use image::{Rgba, RgbaImage, imageops};

use super::tiles::{decode, png_for};
use crate::maze::grid::{Cell, Grid};

/// Draws the maze at the tiles' native size and writes it as a PNG file.
/// The solution path, when given, is tinted gold on top.
pub fn export_png(
    grid: &Grid,
    path: Option<&[(usize, usize)]>,
    file: &Path,
) -> image::ImageResult<()> {
    let cells = [Cell::Wall, Cell::Floor, Cell::Door, Cell::Start, Cell::End];
    let tiles: Vec<(Cell, RgbaImage)> = cells
        .iter()
        .map(|&cell| (cell, decode(png_for(cell))))
        .collect();
    let tile_for = |cell: Cell| {
        let wanted = if cell == Cell::Probe {
            Cell::Floor
        } else {
            cell
        };
        &tiles
            .iter()
            .find(|(c, _)| *c == wanted)
            .expect("every cell has a tile")
            .1
    };
    let size = tile_for(Cell::Floor).width();

    let mut image = RgbaImage::new(grid.width() as u32 * size, grid.height() as u32 * size);
    let at = |x: usize, y: usize| (x as i64 * size as i64, y as i64 * size as i64);
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let cell = grid.get(x, y);
            let (px, py) = at(x, y);
            if !matches!(cell, Cell::Wall | Cell::Floor) {
                imageops::overlay(&mut image, tile_for(Cell::Floor), px, py);
            }
            imageops::overlay(&mut image, tile_for(cell), px, py);
        }
    }

    if let Some(path) = path {
        let gold = RgbaImage::from_pixel(size, size, Rgba([255, 214, 90, 150]));
        for &(x, y) in path {
            let (px, py) = at(x, y);
            imageops::overlay(&mut image, &gold, px, py);
        }
    }
    image.save(file)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;
    use crate::maze::MazeKind;
    use crate::maze::algorithms::{Algorithm, generate};
    use crate::maze::solve::solve;

    /// Not a check: renders every kind x algorithm to target/previews/ with
    /// its solution, to eyeball the results without opening the app.
    /// Run with `cargo test previews -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn previews() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/previews");
        std::fs::create_dir_all(&folder).unwrap();
        for kind in MazeKind::ALL {
            for algorithm in Algorithm::ALL {
                let generation =
                    generate(kind, algorithm, 20, 14, 0.6, &mut StdRng::seed_from_u64(1));
                let mut grid = Grid::new(20, 14);
                for step in &generation.steps {
                    grid.apply(*step);
                }
                let solution = solve(&grid).unwrap();
                let file = folder.join(format!("{kind:?}-{algorithm:?}.png"));
                export_png(&grid, Some(&solution.path), &file).unwrap();
                let phases: Vec<&str> = generation.phases.iter().map(|phase| phase.name).collect();
                println!(
                    "{kind:?} {algorithm:?}: {} steps, path {}, phases {phases:?}",
                    generation.steps.len(),
                    solution.path.len()
                );
            }
        }
    }
}
