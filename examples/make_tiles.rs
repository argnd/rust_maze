//! One-shot generator for the placeholder tiles in assets/tiles/.
//! Run with `cargo run --example make_tiles`; repaint the PNGs by hand afterwards.

use image::{Rgba, RgbaImage};

const SIZE: u32 = 32;
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn main() {
    // Wall: dark flat square with a lighter one-pixel edge.
    let mut wall = flat(Rgba([60, 60, 75, 255]));
    border(&mut wall, Rgba([90, 90, 110, 255]));
    save(&wall, "wall");

    // Floor: light flat square with a slightly darker edge.
    let mut floor = flat(Rgba([205, 195, 175, 255]));
    border(&mut floor, Rgba([180, 170, 150, 255]));
    save(&floor, "floor");

    // Door: brown rectangle on a transparent background, drawn over the floor tile.
    let mut door = RgbaImage::from_pixel(SIZE, SIZE, TRANSPARENT);
    rect(&mut door, 6, 2, 26, 30, Rgba([120, 75, 35, 255]));
    rect(&mut door, 8, 4, 24, 28, Rgba([150, 95, 45, 255]));
    save(&door, "door");

    // Start / end: filled circle with a darker outline, transparent around.
    let mut start = RgbaImage::from_pixel(SIZE, SIZE, TRANSPARENT);
    circle(&mut start, 11.0, Rgba([30, 110, 50, 255]));
    circle(&mut start, 9.0, Rgba([70, 190, 90, 255]));
    save(&start, "start");

    let mut end = RgbaImage::from_pixel(SIZE, SIZE, TRANSPARENT);
    circle(&mut end, 11.0, Rgba([120, 30, 30, 255]));
    circle(&mut end, 9.0, Rgba([210, 60, 60, 255]));
    save(&end, "end");
}

fn flat(color: Rgba<u8>) -> RgbaImage {
    RgbaImage::from_pixel(SIZE, SIZE, color)
}

fn border(img: &mut RgbaImage, color: Rgba<u8>) {
    for i in 0..SIZE {
        img.put_pixel(i, 0, color);
        img.put_pixel(i, SIZE - 1, color);
        img.put_pixel(0, i, color);
        img.put_pixel(SIZE - 1, i, color);
    }
}

fn rect(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgba<u8>) {
    for y in y0..y1 {
        for x in x0..x1 {
            img.put_pixel(x, y, color);
        }
    }
}

fn circle(img: &mut RgbaImage, radius: f32, color: Rgba<u8>) {
    let center = SIZE as f32 / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x, y, color);
            }
        }
    }
}

fn save(img: &RgbaImage, name: &str) {
    let path = format!("assets/tiles/{name}.png");
    img.save(&path).expect("write tile PNG");
    println!("wrote {path}");
}
