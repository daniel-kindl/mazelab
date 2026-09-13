# Adding a generator

This guide is a tour of a real generator,
[`src/generator/prim.rs`](../src/generator/prim.rs). It copies no code of its
own. Open the file beside this guide and read them together. The file is the
source of truth, and this guide names its items so that you can find them.

Randomized Prim is the example, and not the Recursive Backtracker, because its
**frontier** is an explicit set of cells. That set shows how a generator
answers the questions the renderer asks.

Two documents hold the rest:

- **The contract** is the module rustdoc of
  [`src/generator/mod.rs`](../src/generator/mod.rs). It has seven numbered
  rules, and this guide refers to them as rule 1 to rule 7. Its doctest,
  `Comb`, is the smallest generator that compiles and keeps every rule.
  `cargo test` runs it. Read the contract with `cargo doc --no-deps --open`.
- **The glossary** is [`CONTEXT.md`](../CONTEXT.md). It defines the domain
  terms, such as **frontier**, **step** and **perfect**.

## What you write

1. One file in `src/generator/`, with a `pub mod` line for it in
   `src/generator/mod.rs`.
2. One row in `GENERATORS`, the generator registry in `src/generator/mod.rs`.

A generator is a struct that holds its working set and advances one **step**
for each call. It is not a coroutine or a thread, and it does not replay a
result that it computed before. The screen draws the frontier on every redraw,
so the frontier must be readable while the run is part-way through
([ADR 0002](adr/0002-algorithms-are-explicit-state-machines.md)).

## The tour

### The module comment

The file starts with what the algorithm does and why MazeLab has it. It also
states how many steps a run takes: `W * H - 1`. Each step carves one cell, and
the run attaches the start cell without a step of its own. A statement like
that lets a test count the steps.

### `struct Prim`: the working set

Each field is there for a rule of the contract:

- `frontier` is the list of uncarved cells beside the carved region. A step
  draws from it, so its order is part of what a seed fixes.
- `in_frontier` holds one flag for each cell. `is_frontier` reads it in O(1),
  which is rule 5: the renderer asks about one cell at a time, and nothing
  iterates a frontier. The same flags stop a second push of a cell into
  `frontier`.
- `width` gives the offset of a cell into `in_frontier`.
- `current` is the **current cell**, which the screen draws as `@@`.
- `attached` records whether the start cell is attached yet. Rule 4 explains
  why that happens in `step` and not in `make`.
- `rng` is the stream of this run. Rule 7 says that a generator owns its RNG.

The struct is private. Only `make` is public, and the registry holds `make`.

### `make`: the factory

`make` takes `&Maze`, so it cannot change the maze. That is rule 4. It reads
the width and the height to size `in_frontier`, and it fills the frontier with
the neighbours of the start cell, `START`. It does not carve.

It takes its RNG with `rng::split`, and not with a clone. A clone would draw the
same numbers that the braiding pass draws after the generation run ends
(rule 7).

It walks the neighbours in the order of `Dir::ALL`: North, East, South, West.
Rule 6 fixes that order, because the random draws must be the only source of
variation.

### `push_frontier` and `take_frontier`

`push_frontier` sets the flag and pushes the cell, unless the flag is already
set.

`take_frontier` draws a uniformly random index, and the comment on the draw
gives the reason for its type. It samples a `u32` and not a `usize`. That is
rule 2 of `src/rng.rs`: a 32-bit target and a 64-bit target then draw the same
numbers.

`take_frontier` then calls `swap_remove`, which moves the last cell into the
gap. That changes the order of the frontier, and so it changes later draws. The
change is deterministic for a seed, so the design permits it. The
comment records it because an optimisation could change it by accident.

### `step`: one step, at most one edge

Read `step` from top to bottom:

1. **Attach the start cell on the first call.** `mark_carved` attaches a cell
   without an edge. It is not a carve, so rule 1 still holds. Because `make`
   cannot do this, the maze stays unchanged until the first step (rule 4).
2. **Take a frontier cell.** The caller never steps after `Done`, so the
   frontier holds a cell here.
3. **Collect the carved neighbours**, in the order of `Dir::ALL`, into a fixed
   array of four. There is no `HashSet`. Rule 3 of `src/rng.rs` forbids hash
   iteration in an algorithm, because its order is not deterministic.
4. **Carve to one of them**, drawn at random. This is the one edge that the
   step carves (rule 1). Then set `current`.
5. **Push the uncarved neighbours** of the new cell into the frontier.
6. **Return `Done` on the step that empties the frontier** (rule 2), and not
   on a later call.

Each step attaches one uncarved cell to the carved region with exactly one
edge. So when the run ends, the edges form a spanning tree and the maze is
**perfect** (rule 3). Loops come from the braiding pass alone
([ADR 0004](adr/0004-perfect-mazes-and-a-separate-braiding-pass.md)).

### The three inspection methods

- `current` returns the cell that the last step carved.
- `is_frontier` checks the column before it reads a flag. A cell outside the
  maze is in no working set, and an offset past the end of a row lands on the
  next row.
- `frontier_len` returns the live size of the working set. The statistics band
  draws it.

## Register the generator

Add a row to `GENERATORS` and read the comments on `GeneratorEntry`:

- `key` is the value of `--generator`. The command line checks a key against
  this table, and its error message lists the keys of the table.
- `name` is the full name. The status row draws it.
- `short` is the abbreviation. The statistics band draws it, and it must not be
  longer than `name`. A band column has 24 characters inside its border at the
  79-column layout floor, and the `algorithm` label shares that row.
- `make` is your factory.

The order of the rows is part of the interface. The number keys select a
generator by its index in the table.

## What a third generator also touches

The table reaches the command line, the status row and the statistics band
without other changes. The keys do not:

- `src/main.rs` maps only `1` and `2` to a generator, in `action`.
- `src/ui/help.rs` names `1-2` in the help row and `1 2` in the help overlay.
  `tests/help.rs` and `tests/screen.rs` hold those two strings.
- The unit test in `src/generator/mod.rs` holds the keys of the table in their
  order, so it fails until you add the new key.

## Tests

Some tests in `tests/generator.rs` loop over `GENERATORS`, and they run on the
new row with no change. They check that:

- the maze is connected,
- the maze has `W * H - 1` edges,
- the frontier flags agree with the frontier count at every step,
- the braiding pass keeps its rules on the maze.

`every_generator_terminates_and_carves_every_cell` names each generator. It
also checks the step count that each one states. Add the new generator to it.

Add a stored reference maze for the new generator, like
`prim_at_seed_1_and_4x4_carves_a_known_maze`. Store the picture. Do not compute
it again in the test. Suppose the RNG changes under the crate. A test that
generates twice and compares still passes, and the stored picture fails.

A change to the order of the draws changes every maze that a generator makes
from a stored seed. [ADR 0005](adr/0005-determinism-is-scoped-to-one-version.md)
permits that between versions of MazeLab. The reference
maze still fails, and it must fail.

## Adding a solver

The shape is the same. The contract is the module rustdoc of
[`src/solver/mod.rs`](../src/solver/mod.rs), and the registry is `SOLVERS`.
`Solver::step` takes `&Maze` in place of `&mut Maze`, so a solver cannot change
the maze ([ADR 0003](adr/0003-two-traits-not-one.md)). A solver draws no random
number. `src/main.rs` maps `3`, `4` and `5` to a solver, and `Tab` selects
the next row of the table and wraps.
