# Maze generator

A small desktop app that generates mazes and animates the generation, square
by square. Built in Rust with [egui](https://github.com/emilk/egui); the
release build is a single `maze_generator.exe` with no installation and no
files next to it.

## Using it

- **Maze type**: *Perfect* (one path between any two points, no loops),
  *Imperfect* (dead ends are turned into loops, the *loops* slider sets how
  many), *Dungeon* (rooms joined by corridors and doors, dead-end corridors
  removed).
- **Algorithm**: the corridor carver, *Recursive backtracker* (long winding
  tunnels), *Prim* (spreads from a point), *Kruskal* (patches that merge).
  Applies to all three types; for dungeons it carves between the rooms.
- **Width / height**: size in corridors. The drawn grid is twice that plus
  one, because walls occupy full squares.
- **Generate** opens the viewer: the maze carves itself from an empty grid,
  the last few changes highlighted in blue. *Start* (green) and *End* (red,
  the square farthest from start) are visible from the first frame.
  *steps / s* sets the speed, *Replay* restarts the same maze, *Back* returns
  to the menu.

## Building

Needs the Rust toolchain from <https://rustup.rs> (MSVC on Windows).

```
cargo run              # debug build, opens the window, keeps a console for panics
cargo test             # checks every type x algorithm produces a connected maze
cargo build --release  # target/release/maze_generator.exe, standalone
```

The release build links the C runtime statically (`.cargo/config.toml`) and
hides the console window; debug builds keep it.

## Tiles

Every square is drawn from a 32x32 PNG in `assets/tiles/` (`wall`, `floor`,
`door`, `start`, `end`), embedded into the exe at build time. Repaint them
with any pixel editor and rebuild. `door`, `start` and `end` may use
transparency: the floor tile shows through. The placeholder set was made
with `cargo run --example make_tiles`.

## Layout

```
src/main.rs                 window setup
src/ui/                     screens, rendering, tile textures
src/maze/grid.rs            the square grid: Wall / Floor / Door / Start / End
src/maze/algorithms/        carvers (backtracker, prim, kruskal), braiding,
                            dungeon rooms and doors, tests
assets/tiles/               the five PNG tiles
examples/make_tiles.rs      one-shot generator for placeholder tiles
```

Generation records a list of steps (square, new cell); the viewer replays
them at the chosen speed. Adding an algorithm means one new file with a
`carve(&Lattice, rng) -> Vec<Step>` function and one entry in the
`Algorithm` enum.
