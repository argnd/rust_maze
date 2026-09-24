use eframe::egui;

use crate::maze::grid::Cell;

const WALL_PNG: &[u8] = include_bytes!("../../assets/tiles/wall.png");
const FLOOR_PNG: &[u8] = include_bytes!("../../assets/tiles/floor.png");
const DOOR_PNG: &[u8] = include_bytes!("../../assets/tiles/door.png");
const START_PNG: &[u8] = include_bytes!("../../assets/tiles/start.png");
const END_PNG: &[u8] = include_bytes!("../../assets/tiles/end.png");

/// The embedded PNG a cell is drawn with. Probe squares use the floor tile;
/// the renderer tints them.
pub fn png_for(cell: Cell) -> &'static [u8] {
    match cell {
        Cell::Wall => WALL_PNG,
        Cell::Floor | Cell::Probe => FLOOR_PNG,
        Cell::Door => DOOR_PNG,
        Cell::Start => START_PNG,
        Cell::End => END_PNG,
    }
}

pub fn decode(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes)
        .expect("embedded tile PNG must be valid")
        .to_rgba8()
}

/// The start tile doubles as the window icon.
pub fn app_icon() -> egui::IconData {
    let image = decode(START_PNG);
    egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    }
}

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
            wall: load_png(ctx, "wall", WALL_PNG),
            floor: load_png(ctx, "floor", FLOOR_PNG),
            door: load_png(ctx, "door", DOOR_PNG),
            start: load_png(ctx, "start", START_PNG),
            end: load_png(ctx, "end", END_PNG),
        }
    }

    pub fn wall(&self) -> egui::TextureId {
        self.wall.id()
    }

    pub fn floor(&self) -> egui::TextureId {
        self.floor.id()
    }

    pub fn for_cell(&self, cell: Cell) -> egui::TextureId {
        match cell {
            Cell::Wall => self.wall.id(),
            Cell::Floor | Cell::Probe => self.floor.id(),
            Cell::Door => self.door.id(),
            Cell::Start => self.start.id(),
            Cell::End => self.end.id(),
        }
    }
}

fn load_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let decoded = decode(bytes);
    let size = [decoded.width() as usize, decoded.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &decoded.into_raw());
    ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
}
