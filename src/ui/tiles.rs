use eframe::egui;

use crate::maze::grid::Cell;

pub struct Tiles {
    wall: egui::TextureHandle,
    floor: egui::TextureHandle,
    door: egui::TextureHandle,
    start: egui::TextureHandle,
    end: egui::TextureHandle,
}

impl Tiles {
    pub fn load(ctx: &egui::Context) -> Self {
        Self {
            wall: load_png(ctx, "wall", include_bytes!("../../assets/tiles/wall.png")),
            floor: load_png(ctx, "floor", include_bytes!("../../assets/tiles/floor.png")),
            door: load_png(ctx, "door", include_bytes!("../../assets/tiles/door.png")),
            start: load_png(ctx, "start", include_bytes!("../../assets/tiles/start.png")),
            end: load_png(ctx, "end", include_bytes!("../../assets/tiles/end.png")),
        }
    }

    pub fn floor(&self) -> egui::TextureId {
        self.floor.id()
    }

    pub fn for_cell(&self, cell: Cell) -> egui::TextureId {
        match cell {
            Cell::Wall => self.wall.id(),
            Cell::Floor => self.floor.id(),
            Cell::Door => self.door.id(),
            Cell::Start => self.start.id(),
            Cell::End => self.end.id(),
        }
    }
}

fn load_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let decoded = image::load_from_memory(bytes)
        .expect("embedded tile PNG must be valid")
        .to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &decoded.into_raw());
    ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
}