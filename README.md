# MazeLab

MazeLab is a terminal application for exploring maze-generation and
pathfinding algorithms. It runs them one step at a time and shows the current
cell, the **frontier** and the cells that a solver **expanded**.

![MazeLab generates a maze with the Recursive Backtracker, solves it with BFS, then solves it again with A*](docs/media/demo.gif)

## Features

- Recursive Backtracker and Randomized Prim maze generation
- DFS, BFS and A\* pathfinding
- Pause, single step and adjustable animation speed
- Two solver runs side by side on one maze, for comparison
- Braiding, which adds loops so that more than one path can join two cells
- Seeded mazes that you can reproduce
- Automatic terminal fitting at startup, with `f` to refit after a resize
- An ASCII glyph set, `--ascii`, for terminals that cannot draw block glyphs

## Run it

MazeLab needs Rust 1.88.0 or later.

```sh
cargo run --release --locked
```

The demo maze, with BFS selected:

```sh
cargo run --release --locked -- --seed 7 --width 24 --height 8 --solver bfs
```

`cargo run --release -- --help` lists the flags. Without `--seed`, MazeLab
draws a fresh seed.

Without `--width`, the maze is as wide as the terminal can show. Without
`--height`, it is as tall as the terminal can show. If the terminal is smaller
than 79 columns by 19 rows, a width or height that you do not give is 4 cells.

## Controls

These are the keys in the help row at the bottom of the screen:

| Key | Action |
| --- | --- |
| `g` | Generate, animated |
| `G` | Generate instantly |
| `s` | Solve with the selected solver |
| `Space` | Pause or resume |
| `.` | Single step |
| `+` / `-` | Speed up or slow down |
| `1` `2` | Choose the generator |
| `3` `4` `5` | Choose the solver |
| `?` | Show the full keymap |
| `q` | Quit |

The full keymap also includes `Tab`, the arrow keys, `f`, `n` and `b`.

## Comparing solvers

Choose a solver and press `s`. Then choose another solver and press `s` again.
The statistics band shows the new run beside the previous run.

At braid factor 0 the maze is **perfect**, so one path joins start and goal,
and every solver returns it: equal lengths are expected, not a broken
comparison. Watch **expanded** instead, the cells a solver searched: in the
demo, BFS expands 176 and A\* 122.

## Cross-platform is a report, not a promise

The CI matrix proves that MazeLab compiles, tests and lints clean on Linux,
Windows and macOS. For macOS, the matrix is the only evidence.

The tests render into memory, not on a real terminal, so they prove nothing
about fonts or colour tables. What they do prove is that MazeLab stays legible
when colour is absent: the glyph alone identifies each cell state.

If block glyphs do not draw correctly in your terminal, run MazeLab with
`--ascii`. The legend row shows every glyph and colour of the current phase, so
you can check on screen.

## Promises

MazeLab makes two promises, and no others:

- **Determinism.** The same seed, the same size and the same version of MazeLab
  give the same maze on Linux, Windows and macOS. This is not a promise across
  versions ([ADR 0005](docs/adr/0005-determinism-is-scoped-to-one-version.md)).
  The braid factor has no flag, so the command line reproduces a maze only at
  braid factor 0.
- **Minimum Rust version (MSRV).** MazeLab builds with Rust 1.88.0. A CI job
  builds it with that toolchain on every push to `main` and on every pull
  request into `main`.

## Add an algorithm

[`docs/extending.md`](docs/extending.md) adds a generator by a tour of
`src/generator/prim.rs`. The contract that a generator or a solver must keep is
in the rustdoc of `src/generator/mod.rs` and `src/solver/mod.rs`. MazeLab does
not publish its rustdoc. Build it locally:

```sh
cargo doc --no-deps --open
```

## Design

- [`CONTEXT.md`](CONTEXT.md) is the glossary. The code, the screen and these
  documents use its terms.
- [`docs/adr/`](docs/adr/) holds the decisions that are hard to reverse, each
  with its reason.

## Develop

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all --check
```

To record the demo again, install `vhs` and run `vhs docs/media/demo.tape` from
the repository root.
