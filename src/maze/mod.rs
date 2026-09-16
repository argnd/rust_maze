pub mod algorithms;
pub mod grid;

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
}
