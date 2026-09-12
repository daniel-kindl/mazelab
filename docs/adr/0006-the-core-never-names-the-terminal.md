# The core never names `ratatui` or `crossterm`

MazeLab is a library crate plus a thin binary in one package. Nothing in the
model or the algorithms names `ratatui` or `crossterm`: `maze/`, `braid.rs`,
`rng.rs`, `generator/`, `solver/`, `run.rs`, `app.rs` and `cli.rs` know cells,
walls, frontiers and steps, and know nothing of colours, glyphs or key codes.
Only `ui/` and `main.rs` name the terminal, and nothing in the core depends on
`ui`.

## The palette seam

The rule bites hardest at cell priority, where the naive design returns a
colour. It returns a **cell state** instead: `SolverCellState` and
`GenCellState`, whose variant order is the priority order. `ui/palette.rs` maps
a cell state to a colour and a glyph, and that mapping is a flat table with no
logic in it.

## Consequences

- The maze invariant tests and the solver correctness tests need no terminal.
  This is the reason for the library-plus-binary split rather than a single
  binary crate.
- `--ascii` costs one more glyph table and nothing else. The cell states do not
  change.
- The cell priority tests are ordinary tests over an enum, with no colours in
  them.

A workspace was rejected: at this size it would buy nothing that one package
with a library target does not already give.
