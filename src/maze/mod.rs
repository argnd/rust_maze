pub mod algorithms;
pub mod grid;
pub mod solve;
pub mod stats;

/// What the user asks for; the algorithm carves corridors, the kind decides
/// what happens around that carving.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MazeKind {
    Perfect,
    Imperfect,
    Dungeon,
}

impl MazeKind {
    pub const ALL: [MazeKind; 3] = [MazeKind::Perfect, MazeKind::Imperfect, MazeKind::Dungeon];

    pub fn label(self) -> &'static str {
        match self {
            MazeKind::Perfect => "Perfect (no loops)",
            MazeKind::Imperfect => "Imperfect (loops)",
            MazeKind::Dungeon => "Dungeon (rooms and doors)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MazeKind::Perfect => "Exactly one path between any two squares.",
            MazeKind::Imperfect => {
                "Dead ends are knocked through, creating loops and alternative routes."
            }
            MazeKind::Dungeon => {
                "Rooms joined by doors and winding corridors; dead-end corridors are removed."
            }
        }
    }
}
