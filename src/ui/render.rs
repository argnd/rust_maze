use eframe::egui;

use super::tiles::Tiles;
use crate::maze::grid::{Cell, Grid};

/// Paints the grid as scaled tiles, as large as the available space allows.
pub fn draw_grid(ui: &mut egui::Ui, grid: &Grid, tiles: &Tiles) {
    let available = ui.available_size();
    let width = grid.width();
    let height = grid.height();
    let cell_size = (available.x / width as f32)
        .min(available.y / height as f32)
        .floor();
    let size = egui::vec2(cell_size * width as f32, cell_size * height as f32);
    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
    let origin = response.rect.min;
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

    for y in 0..height {
        for x in 0..width {
            let cell = grid.get(x, y);
            let min = origin + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
            let rect = egui::Rect::from_min_size(min, egui::vec2(cell_size, cell_size));
            if cell != Cell::Wall && cell != Cell::Floor {
                painter.image(tiles.floor(), rect, uv, egui::Color32::WHITE);
            }
            painter.image(tiles.for_cell(cell), rect, uv, egui::Color32::WHITE);
        }
    }
}
