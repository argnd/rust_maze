use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind, vec2};

use super::tiles::Tiles;
use crate::maze::grid::{Cell, Grid, Step};
use crate::maze::solve::Solution;

/// How long a square takes to pop in after it changes.
pub const POP_SECONDS: f64 = 0.3;
/// How long the solution path takes to draw once the flood is done.
const PATH_SECONDS: f64 = 1.2;
/// Speed of the marker running along the solved path, in squares per second.
const WALKER_SPEED: f64 = 14.0;

pub const TRAIL_COLOR: Color32 = Color32::from_rgba_unmultiplied_const(60, 120, 255, 140);
pub const PROBE_COLOR: Color32 = Color32::from_rgb(255, 170, 40);
pub const PATH_COLOR: Color32 = Color32::from_rgb(255, 214, 90);

/// Cool to hot: how far a square is from the start, `t` from 0 to 1.
pub fn heat_color(t: f64, alpha: u8) -> Color32 {
    const STOPS: [[f64; 3]; 4] = [
        [40.0, 190.0, 255.0],
        [110.0, 90.0, 255.0],
        [240.0, 70.0, 170.0],
        [255.0, 190.0, 60.0],
    ];
    let scaled = t.clamp(0.0, 1.0) * (STOPS.len() - 1) as f64;
    let i = (scaled.floor() as usize).min(STOPS.len() - 2);
    let local = scaled - i as f64;
    let channel = |c: usize| (STOPS[i][c] + (STOPS[i + 1][c] - STOPS[i][c]) * local).round() as u8;
    Color32::from_rgba_unmultiplied(channel(0), channel(1), channel(2), alpha)
}

/// The solve animation at a given moment: the distance flood spreads from
/// the start, then the path draws itself, then a marker runs along it.
pub struct SolutionOverlay<'a> {
    pub solution: &'a Solution,
    /// Seconds since the user asked for the solution.
    pub elapsed: f64,
    pub show_heat: bool,
}

impl SolutionOverlay<'_> {
    /// Long mazes flood a little longer, within limits.
    fn flood_seconds(&self) -> f64 {
        (self.solution.max_distance as f64 / 70.0).clamp(1.0, 3.0)
    }

    /// The distance from the start the flood has reached.
    fn reach(&self) -> f64 {
        self.elapsed / self.flood_seconds() * self.solution.max_distance as f64
    }

    /// Share of the path drawn, from 0 to 1.
    fn path_progress(&self) -> f64 {
        ((self.elapsed - self.flood_seconds()) / PATH_SECONDS).clamp(0.0, 1.0)
    }

    /// True once the flood and the path are fully drawn.
    pub fn drawn(&self) -> bool {
        self.elapsed >= self.flood_seconds() + PATH_SECONDS
    }

    /// Where the marker is, as a fractional index into the path.
    fn walker(&self) -> Option<f64> {
        let segments = self.solution.path.len().checked_sub(1).filter(|&n| n > 0)?;
        let since = self.elapsed - self.flood_seconds() - PATH_SECONDS;
        (since >= 0.0).then(|| (since * WALKER_SPEED) % segments as f64)
    }
}

/// Everything the maze view shows in one frame.
pub struct Scene<'a> {
    pub grid: &'a Grid,
    pub tiles: &'a Tiles,
    pub trail: &'a [Step],
    /// When each square last changed, in UI time, for the pop-in.
    pub changed_at: &'a [f64],
    pub now: f64,
    pub solution: Option<SolutionOverlay<'a>>,
    /// Banner in the maze's top-left corner, used by the demo.
    pub caption: Option<String>,
}

/// Paints the maze centred in the available space, as large as it fits.
pub fn draw_maze(ui: &mut egui::Ui, scene: &Scene) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
    let mut area = response.rect.shrink(12.0);
    // The caption gets its own strip above the maze: drawn over the maze it
    // would hide the top-left corner, which is where the start usually is.
    let caption = scene.caption.as_ref().map(|text| {
        let galley = painter.layout_no_wrap(
            text.clone(),
            egui::FontId::proportional(18.0),
            Color32::WHITE,
        );
        area.min.y += galley.size().y + 20.0;
        galley
    });
    let (width, height) = (scene.grid.width(), scene.grid.height());
    let cell_size = (area.width() / width as f32)
        .min(area.height() / height as f32)
        .floor()
        .max(1.0);
    let size = vec2(cell_size * width as f32, cell_size * height as f32);
    let maze_rect = Rect::from_min_size((area.center() - size / 2.0).round(), size);
    let cell_rect = |x: usize, y: usize| {
        Rect::from_min_size(
            maze_rect.min + vec2(x as f32 * cell_size, y as f32 * cell_size),
            vec2(cell_size, cell_size),
        )
    };
    let centre = |x: usize, y: usize| cell_rect(x, y).center();

    painter.rect_filled(
        maze_rect.expand(8.0).translate(vec2(0.0, 5.0)),
        8.0,
        Color32::from_black_alpha(110),
    );
    painter.rect_stroke(
        maze_rect.expand(2.0),
        3.0,
        Stroke::new(1.5, Color32::from_gray(110)),
        StrokeKind::Outside,
    );

    for y in 0..height {
        for x in 0..width {
            let cell = scene.grid.get(x, y);
            let rect = cell_rect(x, y);
            let age = scene.now - scene.changed_at[y * width + x];
            if (0.0..POP_SECONDS).contains(&age) {
                let t = (age / POP_SECONDS) as f32;
                let ease = 1.0 - (1.0 - t).powi(3);
                // What the square most likely was before: walls turn to floor
                // while carving, floor turns to wall while pruning or dividing.
                let before = if cell == Cell::Wall {
                    scene.tiles.floor()
                } else {
                    scene.tiles.wall()
                };
                painter.image(before, rect, full_uv(), Color32::WHITE);
                let inner =
                    Rect::from_center_size(rect.center(), rect.size() * (0.35 + 0.65 * ease));
                draw_cell(&painter, scene.tiles, cell, inner, scene.now);
                let flash = ((1.0 - t) * 150.0) as u8;
                painter.rect_filled(inner, 0.0, Color32::from_white_alpha(flash));
            } else {
                draw_cell(&painter, scene.tiles, cell, rect, scene.now);
            }
        }
    }

    for step in scene.trail {
        painter.rect_filled(cell_rect(step.x, step.y), CornerRadius::ZERO, TRAIL_COLOR);
    }

    if let Some(overlay) = &scene.solution {
        draw_solution(&painter, overlay, cell_size, &cell_rect, &centre);
    }

    if let Some(galley) = caption {
        let text_pos = egui::pos2(maze_rect.min.x + 10.0, response.rect.min.y + 12.0);
        let background = Rect::from_min_size(text_pos, galley.size()).expand2(vec2(10.0, 6.0));
        painter.rect_filled(background, 6.0, Color32::from_black_alpha(180));
        painter.galley(text_pos, galley, Color32::WHITE);
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
}

fn draw_cell(painter: &egui::Painter, tiles: &Tiles, cell: Cell, rect: Rect, now: f64) {
    if !matches!(cell, Cell::Wall | Cell::Floor) {
        painter.image(tiles.floor(), rect, full_uv(), Color32::WHITE);
    }
    painter.image(tiles.for_cell(cell), rect, full_uv(), Color32::WHITE);
    if cell == Cell::Probe {
        let pulse = 0.5 + 0.5 * (now * 6.0).sin() as f32;
        let alpha = (110.0 + 70.0 * pulse) as u8;
        painter.rect_filled(rect, 0.0, PROBE_COLOR.gamma_multiply_u8(alpha));
    }
}

fn draw_solution(
    painter: &egui::Painter,
    overlay: &SolutionOverlay,
    cell_size: f32,
    cell_rect: &dyn Fn(usize, usize) -> Rect,
    centre: &dyn Fn(usize, usize) -> Pos2,
) {
    let solution = overlay.solution;
    let reach = overlay.reach();
    let max = solution.max_distance.max(1) as f64;
    let flooding = reach < max;
    for &(x, y, distance) in &solution.flood {
        let distance = distance as f64;
        if distance > reach {
            break;
        }
        let on_front = flooding && reach - distance < 2.5;
        if !(overlay.show_heat || on_front) {
            continue;
        }
        let alpha = if on_front { 230 } else { 130 };
        painter.rect_filled(cell_rect(x, y), 0.0, heat_color(distance / max, alpha));
    }

    let path = &solution.path;
    let progress = overlay.path_progress();
    if progress > 0.0 && path.len() > 1 {
        let along = progress * (path.len() - 1) as f64;
        let mut points: Vec<Pos2> = path[..=along.floor() as usize]
            .iter()
            .map(|&(x, y)| centre(x, y))
            .collect();
        if let Some(&(x, y)) = path.get(along.floor() as usize + 1) {
            let last = *points.last().expect("at least the start");
            points.push(last.lerp(centre(x, y), along.fract() as f32));
        }
        painter.line(
            points.clone(),
            Stroke::new(cell_size * 0.9, PATH_COLOR.gamma_multiply(0.25)),
        );
        painter.line(points, Stroke::new((cell_size * 0.3).max(1.5), PATH_COLOR));
    }

    if let Some(position) = overlay.walker() {
        let i = position.floor() as usize;
        let (ax, ay) = path[i];
        let (bx, by) = path[(i + 1).min(path.len() - 1)];
        let at = centre(ax, ay).lerp(centre(bx, by), position.fract() as f32);
        let pulse = 1.0 + 0.12 * (overlay.elapsed * 8.0).sin() as f32;
        painter.circle_filled(at, cell_size * 0.75 * pulse, PATH_COLOR.gamma_multiply(0.3));
        painter.circle_filled(at, cell_size * 0.34, Color32::WHITE);
        painter.circle_stroke(
            at,
            cell_size * 0.34,
            Stroke::new((cell_size * 0.1).max(1.0), PATH_COLOR),
        );
    }
}
