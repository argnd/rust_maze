# Maze generator

A small desktop app that generates mazes and animates the generation, square
by square, then solves them. Built in Rust with
[egui](https://github.com/emilk/egui); the release build is a single
`maze_generator.exe` with no installation and no files next to it.

## Using it

**Menu**

- **Maze type**: *Perfect* (one path between any two points, no loops),
  *Imperfect* (dead ends are knocked through into loops; *Loops* sets the
  share), *Dungeon* (rooms joined by doors and corridors, dead-end corridors
  removed).
- **Algorithm**: ten corridor carvers, each with its own texture and its own
  animation. Applies to all three types; for dungeons it carves between the
  rooms.

  | Algorithm | Look | What the animation shows |
  |---|---|---|
  | Recursive backtracker | long winding corridors | the stack, in amber, unwinding |
  | Prim | short bushy dead ends | the frontier, in amber |
  | Kruskal | even texture | islands merging |
  | Wilson | unbiased: every maze equally likely | a random walk erasing its own loops |
  | Hunt-and-kill | long corridors | walks, then jumps to a new start |
  | Growing tree | between backtracker and Prim | active cells, in amber |
  | Binary tree | strong diagonal bias | a single sweep |
  | Sidewinder | open top row, vertical bias | row by row |
  | Eller | balanced, row by row | row by row, sets merging |
  | Recursive division | long straight walls | walls added to an empty field |

- **Width / height**: size in corridors. The drawn grid is twice that plus
  one, because walls occupy full squares.
- **Seed**: empty for a random maze. Any number or word gives the same maze
  every time, so a maze can be shared by its seed.
- **Generate** (Enter) opens the viewer; **Demo mode** (D) builds, solves and
  replaces random mazes on its own until stopped. `maze_generator.exe --demo`
  starts straight into the demo, for a screen left running.

**Viewer**

The maze builds itself from an empty grid. Squares pop in as they change;
the last five steps are highlighted in blue; squares an algorithm is still
working on pulse in amber. *Start* (green) and *End* (the square farthest
from start) are visible from the first frame. The current phase ("Placing
rooms", "Opening doors", ...) is named above the timeline; bulk phases play
faster.

| Key | Button | Does |
|---|---|---|
| Space | Play / Pause | |
| Right | Step | one step, pausing |
| End | Skip | jump to the finished maze |
| R | Replay | same maze from the start |
| N | New maze | same settings, new seed |
| S | Solve | distance flood from the start, then the shortest path, then a runner along it |
| H | Heat map | keep the distance colours after the flood |
| E | Export PNG | writes `maze-<seed>.png` to the working folder, with the path if solved |
| Esc | Back | to the menu |

The timeline under the toolbar can be dragged to scrub through the
generation. The panel on the right shows the recipe (seed with a copy
button), dead ends, junctions, loops or doors, and the solution length.

## Building

Needs the Rust toolchain from <https://rustup.rs> (MSVC on Windows).

```
cargo run              # debug build, opens the window, keeps a console for panics
cargo test             # every type x algorithm: connected, solvable, reproducible
cargo build --release  # target/release/maze_generator.exe, standalone
```

`cargo test previews -- --ignored` renders every type x algorithm, solved,
to `target/previews/` for a quick visual check without opening the app.

The release build links the C runtime statically (`.cargo/config.toml`) and
hides the console window; debug builds keep it.

## Tiles

Every square is drawn from a 32x32 PNG in `assets/tiles/` (`wall`, `floor`,
`door`, `start`, `end`), embedded into the exe at build time; the start tile
is also the window icon. Repaint them with any pixel editor and rebuild.
`door`, `start` and `end` may use transparency: the floor tile shows through.
The placeholder set was made with `cargo run --example make_tiles`.

## Layout

```
src/main.rs                 window setup, --demo flag
src/ui/app.rs               menu, viewer, playback, demo mode, stats panel
src/ui/render.rs            tiles, pop-in, trail, heat map, path, runner
src/ui/export.rs            PNG export (and the preview renderer)
src/ui/tiles.rs             embedded tile PNGs
src/maze/grid.rs            the square grid: Wall / Floor / Door / Start / End / Probe
src/maze/algorithms/        the ten carvers, braiding, dungeon rooms and doors,
                            phases, generation tests
src/maze/solve.rs           breadth-first flood and shortest path
src/maze/stats.rs           dead ends, junctions, loops, doors
assets/tiles/               the five PNG tiles
examples/make_tiles.rs      one-shot generator for placeholder tiles
```

Generation records a list of steps (square, new cell) split into named
phases; the viewer replays them at the chosen speed. Adding an algorithm
means one new file with a `carve(&Lattice, rng) -> Vec<Step>` function and
one entry in the `Algorithm` enum. It must emit a floor step for every open
corridor; if it can leave separate pieces when dungeon rooms cut its rows,
the shared joining pass connects them without creating loops.
