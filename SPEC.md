# MazeLab v1 specification

MazeLab is a terminal application that animates maze generation and
pathfinding. It exists to make the behavior of an algorithm visible one step at
a time.

This document is the handoff. It is complete enough to build MazeLab v1 without
re-opening a decision. Where a decision is hard to reverse, this document states
the rule and links the ADR that holds the reason. Where the reason is local, it
is stated here.

## How to read this document

- `CONTEXT.md` holds the glossary. Every term in bold in this document is
  defined there. Use those terms in code, in comments and in the user interface.
  **Expanded** is the term; "visited" is banned.
- `docs/adr/` holds the decisions that are hard to reverse. This document does
  not repeat their reasoning. It states what to build.
- A rule written as **must** is a rule an implementer may not change without an
  ADR. A rule written as *should* is a default with a stated reason.
- Section 11 is the test plan. Each implementation issue names the tests it owns.

Status: frozen at handoff. When the code and this document disagree, the code
wins and this document is not updated. It is the record of what was decided
before the first line was written. One exception holds: an error that would
send an implementer the wrong way is corrected in place, and the correction
carries a note that says what changed and in which commit.

## Contents

1. [Crate and module layout](#1-crate-and-module-layout)
2. [The maze, the walls and the display grid](#2-the-maze-the-walls-and-the-display-grid)
3. [The traits, `StepOutcome` and the registries](#3-the-traits-stepoutcome-and-the-registries)
4. [The algorithms](#4-the-algorithms)
5. [Phases and transitions](#5-phases-and-transitions)
6. [The loop, the tick and the step budget](#6-the-loop-the-tick-and-the-step-budget)
7. [The screen](#7-the-screen)
8. [The statistics band](#8-the-statistics-band)
9. [The command line](#9-the-command-line)
10. [Terminal setup, teardown and panic restore](#10-terminal-setup-teardown-and-panic-restore)
11. [The test plan](#11-the-test-plan)
12. [Continuous integration](#12-continuous-integration)
13. [Size bounds and the two thresholds](#13-size-bounds-and-the-two-thresholds)
14. [The documentation surface](#14-the-documentation-surface)
15. [Known limitations of v1](#15-known-limitations-of-v1)

---

## 1. Crate and module layout

ADR: [0006, the core never names the terminal](docs/adr/0006-the-core-never-names-the-terminal.md).

### 1.1 Package shape

One package that holds a library and a thin binary. Not a workspace.

`src/lib.rs` holds the model, the algorithms, the application state and the
rendering. `src/main.rs` holds terminal setup and teardown, the terminal guard
and the call into the loop. Nothing else.

The reason is the test plan of section 11. Maze invariants and solver
correctness are pure logic and need no terminal, so a library makes them
testable from `tests/`.

### 1.2 Modules

```
src/
  lib.rs
  maze/          Maze, Cell, wall flags, carve, the display-grid derivation
  braid.rs       the braiding pass
  rng.rs         the value-stable RNG and its seeding
  generator/     trait Generator, backtracker.rs, prim.rs, the registry
  solver/        trait Solver, dfs.rs, bfs.rs, astar.rs, the registry
  run.rs         Run: start, goal, statistics, cell-state resolution
  app.rs         App: phases, selection, speed, size: the state machine
  cli.rs         the clap surface
  ui/            layout.rs, maze.rs, stats.rs, legend.rs, help.rs, palette.rs,
                 panel.rs
  main.rs        terminal setup and teardown, the loop
```

**The one-way rule is the rule of this section, and it must hold.** Nothing in
`maze`, `braid`, `rng`, `generator`, `solver`, `run` or `app` names `ratatui` or
`crossterm`. The core knows cells, walls, frontiers and steps. It does not know
colours, glyphs or key codes.

`ui` depends on `app`, `run`, `maze`, `generator` and `solver`. None of them
depends on `ui`. This is what keeps the maze invariant tests free of a terminal,
and it is what makes `--ascii` and the palette pure presentation concerns.

Key codes are translated to a MazeLab action in `main.rs` and passed to `app`
as that action. `app` never sees a `KeyEvent`.

### 1.3 Manifest, MSRV and lints

**MSRV is 1.88.0**, with edition 2024 and `resolver = "3"`. The floor is set by
`ratatui` 0.30.2. The MSRV is a promise, not a discovered fact: the README
states it and a pinned CI job builds it (section 12).

```toml
[package]
name = "mazelab"
version = "0.1.0"
edition = "2024"
rust-version = "1.88.0"
resolver = "3"

[dependencies]
ratatui  = "0.30.2"
rand     = "0.10.2"
rand_pcg = "0.10.2"
clap     = { version = "4.6", features = ["derive"] }

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
```

Notes that must be followed:

- **`crossterm` is not a direct dependency.** Since 0.30 `ratatui` selects its
  crossterm by feature through `ratatui-crossterm`. Import through
  `ratatui::crossterm`. A direct `crossterm` dependency risks two incompatible
  copies of the same types.
- **Commit `Cargo.lock`.** It is correct for a binary crate, it makes CI build
  the versions that were tested, and two other decisions rest on it: the
  determinism promise of ADR 0005, and the bounded virtual-terminal-processing
  risk of ADR 0007.
- `resolver = "3"` makes Cargo prefer versions compatible with `rust-version`,
  so a routine `cargo update` cannot raise the MSRV silently.
- Clippy allows go in a short commented list in `[lints.clippy]`, not as
  `#[allow]` scattered through the source. `unwrap_used` and `expect_used` are
  allowed in tests only.
- `cargo fmt` uses the stock style. No `rustfmt.toml`.

### 1.4 The RNG

`src/rng.rs` is a thin wrapper so that one file names the generator:

```rust
pub type Rng = rand_pcg::Pcg64Mcg;

pub fn from_seed(seed: u64) -> Rng {
    <Rng as rand::SeedableRng>::seed_from_u64(seed)
}
```

Three rules follow, and each one is load-bearing for ADR 0005:

1. **`StdRng` is forbidden.** From `rand` 0.10 the portability policy permits a
   value-breaking change to a non-portable item in **any** release, including a
   patch release. A `cargo update` could then change every maze.
   `rand_pcg::Pcg64Mcg` is a documented portable item: reproducible across
   platforms and across patch releases.
2. **Never sample `usize` or `isize`.** Sample `u32` and cast, so a 32-bit and
   a 64-bit target make the same draws:
   `rng.random_range(0u32..len as u32) as usize`.
3. **No algorithm may iterate a `HashMap` or a `HashSet`.** Iteration order is
   not deterministic. Every working set in this specification is a `Vec`, a
   `VecDeque`, a `BinaryHeap` with an explicit `Ord`, or a grid indexed by cell.

The sampling methods live on `rand::RngExt`, which was named `Rng` before
`rand` 0.10. `rand::Rng` is now the low-level trait and importing it alone does
not give `random_range`. Shuffling is `rand::seq::SliceRandom::shuffle`.

---

## 2. The maze, the walls and the display grid

ADR: [0001, walls are edges](docs/adr/0001-walls-are-edges.md).

### 2.1 `Cell` and `Maze`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell { pub x: u16, pub y: u16 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir { North, East, South, West }

pub struct Maze {
    width: u16,
    height: u16,
    walls: Vec<u8>,     // one byte per cell, four flag bits
    carved: Vec<bool>,  // per cell: attached to the maze
}
```

A **maze** is a `W x H` grid of **cells**, each holding four wall flags. A wall
is an absent edge. The index of a cell is `y as usize * width as usize + x as usize`.

A cell's byte holds a set bit for each wall that is **present**, so a new maze
is `0b1111` everywhere: every wall stands, no edge exists.

`W` and `H` are `u16` and are bounded to `4..=512` by section 13. The largest
allocation is therefore 512 x 512 = 262 144 bytes for `walls`.

### 2.2 The operations

```rust
impl Maze {
    pub fn new(width: u16, height: u16) -> Self;
    pub fn width(&self) -> u16;
    pub fn height(&self) -> u16;
    pub fn contains(&self, c: Cell) -> bool;
    pub fn neighbour(&self, c: Cell, d: Dir) -> Option<Cell>;

    /// True when the wall between `c` and its neighbour in `d` is carved.
    pub fn is_open(&self, c: Cell, d: Dir) -> bool;

    /// Remove the wall between two adjacent cells, which adds an edge.
    /// Marks both cells carved. Panics when the cells are not adjacent.
    pub fn carve(&mut self, a: Cell, b: Cell);

    /// Mark one cell carved without adding an edge. A generator calls this
    /// once, for the cell it starts from.
    pub fn mark_carved(&mut self, c: Cell);

    pub fn is_carved(&self, c: Cell) -> bool;
    pub fn carved_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;

    /// The number of carved edges at `c`. A dead end has exactly one.
    pub fn degree(&self, c: Cell) -> u8;
}
```

**`carve` must clear the flag on both sides.** One wall is one edge and it is
stored twice, so the two copies must never disagree. Every read goes through
`is_open`, and no caller touches `walls` directly.

`carved` is held on the maze rather than mirrored inside each generator,
because the renderer has to answer "is this cell carved" in every phase, and a
finished maze has no generator left to ask.

### 2.3 The display grid

A **display grid** is the `(2W+1) x (2H+1)` grid of wall and open positions that
a maze derives for rendering. It is a pure function of the maze, so it lives in
`maze/`, not in `ui/`.

**The interface is an index function.**

```rust
pub enum DisplayCell {
    /// (dx, dy) both odd: the maze cell at ((dx-1)/2, (dy-1)/2).
    Cell(Cell),
    /// A carved wall position, and the two cells it joins.
    Passage(Cell, Cell),
    /// An intact wall, or a post where two walls meet.
    Wall,
}

impl Maze {
    pub fn display_width(&self) -> u16 { 2 * self.width + 1 }
    pub fn display_height(&self) -> u16 { 2 * self.height + 1 }
    pub fn display_cell(&self, dx: u16, dy: u16) -> DisplayCell;
}
```

A materialised `Vec<DisplayCell>` constructor sits beside it **for the tier-3
snapshot tests only**. It is not used at run time.

The reason is that a materialised grid must be rebuilt every frame, or cached
and invalidated on every carve. An index function allocates nothing and cannot
go stale in the middle of a generation, and the snapshot tests still get a whole
value to compare.

The mapping, for `dx` in `0..2W+1` and `dy` in `0..2H+1`:

| `dx` | `dy` | result |
| --- | --- | --- |
| odd | odd | `Cell(((dx-1)/2, (dy-1)/2))` |
| even | odd | `Passage(left, right)` when the wall is carved, else `Wall` |
| odd | even | `Passage(above, below)` when the wall is carved, else `Wall` |
| even | even | `Wall`, always. This is a post. |

A position on the outer border is always `Wall`: it has only one side.

### 2.4 Footprint

At two screen columns per display-grid position (ADR 0007), a maze occupies
**`4W+2` columns by `2H+1` rows**.

---

## 3. The traits, `StepOutcome` and the registries

ADRs: [0002, algorithms are explicit state machines](docs/adr/0002-algorithms-are-explicit-state-machines.md),
[0003, two traits not one](docs/adr/0003-two-traits-not-one.md).

### 3.1 `StepOutcome`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepOutcome { Stepped, Done }
```

Two variants, one enum, returned by both `Generator::step` and `Solver::step`.
The traits stay separate; only the outcome type is shared.

**There is no `Exhausted` variant.** Every maze is connected and braiding only
adds edges, so a solver always reaches the goal. A variant that cannot occur
still has to be matched at every call site forever.

**`step` returns `Done` on the step that completes the run, not on a later
call.** The caller must not call `step` again after `Done`.

`Stepped` is what sets the dirty flag of section 6. That is the reason `step`
returns a value at all.

### 3.2 `Generator`

```rust
pub trait Generator {
    /// Advance one step. Carves at most one edge.
    fn step(&mut self, maze: &mut Maze) -> StepOutcome;

    /// The single cell the generator is acting on, for the step now shown.
    fn current(&self) -> Option<Cell>;

    /// True when the cell is in the generator's working set.
    fn is_frontier(&self, c: Cell) -> bool;

    /// The live size of the working set.
    fn frontier_len(&self) -> usize;
}
```

### 3.3 `Solver`

```rust
pub trait Solver {
    /// Advance one step. Takes `&Maze`: a solver never changes the maze.
    fn step(&mut self, maze: &Maze) -> StepOutcome;

    fn current(&self) -> Option<Cell>;
    fn is_frontier(&self, c: Cell) -> bool;
    fn frontier_len(&self) -> usize;

    /// True when the solver has removed this cell from its frontier
    /// and processed it.
    fn is_expanded(&self, c: Cell) -> bool;
    fn expanded_count(&self) -> usize;

    /// The path from start to goal, inclusive. `Some` once `step` returned
    /// `Done`, `None` before that.
    fn path(&self) -> Option<&[Cell]>;
}
```

**Inspection lives on the traits, and it is membership, not iteration.** The
renderer resolves one cell at a time, so `is_frontier` and `is_expanded` are
what it needs; the statistics band needs the counts. Nothing ever iterates a
frontier. Every implementation therefore keeps a grid of `W*H` flags beside its
working set, which makes both queries O(1) and keeps the working set free of a
`HashSet`.

The alternative, reporting deltas from `step` and mirroring the frontier inside
`app`, spreads one fact across two owners and makes every new algorithm
responsible for reporting correctly.

`Solver::step` takes `&Maze` and `Generator::step` takes `&mut Maze`. This is
the whole reason the two traits are not unified, and it is what keeps the seam
for a future simultaneous comparison open (ADR 0009).

### 3.4 The registries

A const table of factories, one per module:

```rust
pub struct GeneratorEntry {
    /// The `--generator` value and the key the CLI validates against.
    pub key: &'static str,
    /// The full name, for the status row.
    pub name: &'static str,
    /// The abbreviated name, for the statistics band. See section 13.
    pub short: &'static str,
    pub make: fn(&Maze, &mut Rng) -> Box<dyn Generator>,
}

pub static GENERATORS: &[GeneratorEntry] = &[
    GeneratorEntry { key: "backtracker", name: "Recursive Backtracker",
                     short: "RecBack",  make: backtracker::make },
    GeneratorEntry { key: "prim",        name: "Randomized Prim",
                     short: "RandPrim", make: prim::make },
];

pub struct SolverEntry {
    pub key: &'static str,
    pub name: &'static str,
    pub short: &'static str,
    pub make: fn(&Maze, start: Cell, goal: Cell) -> Box<dyn Solver>,
}

pub static SOLVERS: &[SolverEntry] = &[
    SolverEntry { key: "dfs",   name: "DFS", short: "DFS", make: dfs::make },
    SolverEntry { key: "bfs",   name: "BFS", short: "BFS", make: bfs::make },
    SolverEntry { key: "astar", name: "A*",  short: "A*",  make: astar::make },
];
```

**To add a generator a contributor writes one file and adds one line.** That is
the extension promise of section 14, and the doctest in the rustdoc on this
module is what keeps it honest.

The table is the single source of truth for the CLI, the help row and the
in-application chooser, so the names cannot drift apart. The number keys of
section 7 select by index: `1` and `2` index `GENERATORS`, `3`, `4` and `5`
index `SOLVERS`.

Rejected: a `clap::ValueEnum`, because it couples the registry to the CLI while
the in-application chooser has to walk the list anyway. Rejected: a distributed
registry crate such as `inventory` or `linkme`, because it trades a visible list
for link-time magic in a codebase that doubles as a showcase.

---

## 4. The algorithms

ADRs: [0002, algorithms are explicit state machines](docs/adr/0002-algorithms-are-explicit-state-machines.md),
[0004, perfect mazes and a separate braiding pass](docs/adr/0004-perfect-mazes-and-a-separate-braiding-pass.md).

Every algorithm below is written as a state machine that exposes one step at a
time. No coroutine, no thread, no replay of a precomputed result. The user
interface renders the **frontier** every frame, so the working set must be
inspectable from outside.

Two conventions hold for all six:

- **Neighbour order is fixed at `North, East, South, West`.** Where an algorithm
  makes a random draw it draws over that fixed list, so the draw is the only
  source of variation.
- **A generator uses the RNG. A solver does not.** A solver is deterministic
  without a seed, which is what makes the BFS-equals-A\* length test of
  section 11 meaningful.

Both generators start from cell `(0, 0)`. The choice is arbitrary and fixed so
that a seed reproduces a maze.

### 4.1 Recursive Backtracker

The **frontier** is the stack. Rendering the stack is what makes the signature
backtracking legible.

```
state:
    stack: Vec<Cell>            // the frontier
    on_stack: Vec<bool>         // W*H, for O(1) is_frontier

make(maze, rng):
    maze.mark_carved((0, 0))
    stack = [(0, 0)]; on_stack[(0,0)] = true

step(maze):
    c = *stack.last()                       // stack is never empty here
    candidates = neighbours of c, in N E S W order, that are not carved
    if candidates is empty:
        stack.pop(); on_stack[c] = false
        return if stack.is_empty() { Done } else { Stepped }
    n = candidates[rng.random_range(0u32..candidates.len() as u32) as usize]
    maze.carve(c, n)
    stack.push(n); on_stack[n] = true
    return Stepped

current()      = stack.last()
is_frontier(c) = on_stack[c]
frontier_len() = stack.len()
```

A run is roughly `2 * W * H` steps: one step carves each cell, and one step pops
it again.

### 4.2 Randomized Prim, frontier variant

This is the **frontier** variant, not true weighted-edge Prim: pick a uniformly
random uncarved cell beside the carved region, and carve it to a random carved
neighbour. It was chosen because it has a visible frontier set and because it
contrasts sharply with the Recursive Backtracker.

```
state:
    frontier: Vec<Cell>         // uncarved cells beside the carved region
    in_frontier: Vec<bool>      // W*H
    current: Option<Cell>

make(maze, rng):
    maze.mark_carved((0, 0))
    push every neighbour of (0, 0) into frontier

step(maze):
    i = rng.random_range(0u32..frontier.len() as u32) as usize
    c = frontier.swap_remove(i); in_frontier[c] = false
    carved = neighbours of c, in N E S W order, that are carved
    n = carved[rng.random_range(0u32..carved.len() as u32) as usize]
    maze.carve(c, n)
    current = Some(c)
    for m in neighbours of c that are not carved and not in_frontier:
        frontier.push(m); in_frontier[m] = true
    return if frontier.is_empty() { Done } else { Stepped }

current()      = current
is_frontier(c) = in_frontier[c]
frontier_len() = frontier.len()
```

`swap_remove` is O(1) and it reorders the frontier, which changes later draws.
That is deterministic for a given seed, so it is allowed, and it is recorded
here because it is the kind of detail an optimiser changes by accident.

A run is exactly `W * H - 1` steps: every step carves one cell.

### 4.3 The braiding pass

**Braiding** runs after generation and removes a fraction of **dead ends** by
carving one more edge out of each dead end it takes. The **braid factor** is
that fraction. A braid factor of 0 keeps the maze **perfect**.

It is a free function, not a third trait, and it is instant. It draws from the
same RNG stream as generation, so the seed still reproduces the maze.

```rust
pub fn braid(maze: &mut Maze, factor: f64, rng: &mut Rng);
```

```
braid(maze, factor, rng):
    if factor <= 0.0: return
    ends = every cell with degree == 1, collected in row-major order
    ends.shuffle(rng)
    take = (ends.len() as f64 * factor).round() as usize
    for c in ends.iter().take(take):
        if maze.degree(c) != 1: continue      // an earlier carve opened it
        walled = neighbours of c, in N E S W order, whose wall still stands
        prefer the subset of walled that are themselves dead ends
        n = that subset if it is not empty, else walled
        maze.carve(c, n[rng.random_range(0u32..n.len() as u32) as usize])
```

Collecting in row-major order **before** the shuffle is what makes the pass
reproducible; the order of a collection must never come from a hash.

Preferring a neighbour that is itself a dead end removes two dead ends with one
carve, which is what makes a low braid factor visible.

The `degree(c) != 1` re-check is required: a dead end the pass already joined is
no longer a dead end, and carving it twice would be wrong.

### 4.4 DFS

```
state:
    stack: Vec<Cell>, on_stack: Vec<bool>
    expanded: Vec<bool>, expanded_count: usize
    parent: Vec<Option<Cell>>
    current: Option<Cell>
    path: Option<Vec<Cell>>

make(maze, start, goal):
    stack = [start]; on_stack[start] = true

step(maze):
    c = stack.pop(); on_stack[c] = false
    if expanded[c]: return Stepped          // a stale duplicate; skip it
    expanded[c] = true; expanded_count += 1
    current = Some(c)
    if c == goal:
        path = Some(reconstruct(parent, start, goal))
        return Done
    for n in neighbours of c reachable through a carved edge, in N E S W order:
        if !expanded[n] && !on_stack[n]:
            parent[n] = Some(c); stack.push(n); on_stack[n] = true
    return Stepped

is_frontier(c) = on_stack[c]
frontier_len() = stack.len()
```

### 4.5 BFS

Identical to DFS except that the working set is a `VecDeque` and the step takes
from the front:

```
    c = queue.pop_front()
    ...
    for n in ...: queue.push_back(n); in_queue[n] = true
```

BFS is what makes the **path length** authoritative: on an unbraided maze every
solver returns the same length, and BFS is the one that proves it is the
shortest.

### 4.6 A\*

```
state:
    open: BinaryHeap<Node>
    in_open: Vec<bool>, in_open_count: usize
    g: Vec<u32>                     // u32::MAX where unknown
    expanded: Vec<bool>, expanded_count: usize
    parent: Vec<Option<Cell>>
    next_insertion: u32
    current: Option<Cell>
    path: Option<Vec<Cell>>

h(c, goal) = |c.x - goal.x| + |c.y - goal.y|        // Manhattan distance

make(maze, start, goal):
    g[start] = 0
    push Node { f: h(start, goal), h: h(start, goal), insertion: 0, cell: start }

step(maze):
    node = open.pop(); c = node.cell
    if expanded[c]: return Stepped          // lazy deletion of a stale entry
    in_open[c] = false; in_open_count -= 1
    expanded[c] = true; expanded_count += 1
    current = Some(c)
    if c == goal:
        path = Some(reconstruct(parent, start, goal)); return Done
    for n in neighbours of c reachable through a carved edge, in N E S W order:
        tentative = g[c] + 1
        if tentative < g[n]:
            g[n] = tentative; parent[n] = Some(c)
            push Node { f: tentative + h(n, goal), h: h(n, goal),
                        insertion: next_insertion, cell: n }
            next_insertion += 1
            if !in_open[n] { in_open[n] = true; in_open_count += 1 }
    return Stepped

is_frontier(c) = in_open[c] && !expanded[c]
frontier_len() = in_open_count
```

**The open set is cleared on the pop that expands a cell, and not on a pop that
discards one.** The heap holds one entry for each time a cell was reached by a
shorter path, and `in_open` holds one flag for that cell. The pop that expands
the cell clears the flag and lowers the count. A pop that discards a stale entry
must leave both alone, or the count falls once for every duplicate the heap
still holds and goes below zero. That order is what keeps `is_frontier` and
`frontier_len` above true.

> Corrected after handoff, in commit 77291e2. The handed-off text lowered the
> count above the `expanded[c]` check, and a maze with loops then underflows it.

Manhattan distance is admissible here: a step between two adjacent cells costs
1, and a wall can only make the true path longer than the straight-line count.

**The ordering, and the reason that must be written at the `Ord` implementation.**
The open set is a `BinaryHeap` with an explicit `Ord` over
`(f, h, insertion_index)`. A `BinaryHeap` is a max-heap, so the comparison is
inverted to make the smallest tuple pop first:

```rust
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverted: BinaryHeap is a max-heap and the smallest (f, h, insertion)
        // must pop first.
        //
        // The `h` term is deliberate and it is not the standard tie-break.
        // Among nodes with equal `f`, taking the one closest to the goal first
        // drives the search forward in a visible spike. Breaking on `f` alone
        // lets A* expand a broad ring, which on a maze looks exactly like BFS,
        // and MazeLab exists to show the difference between the two.
        //
        // `insertion` keeps the order stable among nodes with equal `f` and
        // equal `h`, so a run is reproducible. Do not "correct" this to a
        // tie-break on `f` alone without reading section 4.6 of SPEC.md.
        other.f.cmp(&self.f)
            .then_with(|| other.h.cmp(&self.h))
            .then_with(|| other.insertion.cmp(&self.insertion))
    }
}
```

`PartialOrd` delegates to `Ord`. `Eq` and `PartialEq` compare the same three
fields, so the `Ord` contract holds.

### 4.7 Path reconstruction

```
reconstruct(parent, start, goal):
    walk parent from goal back to start, collecting cells
    reverse
```

The result includes both start and goal, which is what **path length** counts.

---

## 5. Phases and transitions

A **phase** is the state that decides what the application does with a tick. A
phase belongs to the application, not to an algorithm: an algorithm holds its
own working set and knows nothing about pausing.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity { Generating, Solving }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Generating,
    Paused(Activity),
    Ready,
    Solving,
    Solved,
}
```

`Paused` carries what it paused, because that is what a resume has to return to.
The glossary names six phases and this is those six.

### 5.1 The transitions

```
Idle --------- g ------------> Generating
Idle --------- G ------------> Ready            (instant, see 6.5)
Generating <-- Space --------> Paused(Generating)
Generating --- Done ---------> Ready            (after the braiding pass)
Ready --------- s -----------> Solving
Solving <----- Space --------> Paused(Solving)
Solving ------ Done ---------> Solved
Solved ------- 3 4 5 Tab ----> Ready            (promotes the run, see 8.2)
any ---------- g G n b <- -> Up Down f --> Generating or Ready
```

### 5.2 The rules that the diagram does not carry

- **A new maze resets everything.** `g`, `G`, `n`, `b`, `f` and the four size
  keys build a new maze, clear the previous-run column, and discard any solver
  run in progress.
- **`Done` from a generator is followed by the braiding pass, then `Ready`.**
  The pass is instant, so `Generating` never leads anywhere else.
- **Choosing a generator does not generate.** `1` and `2` select the generator
  for the next generate. The maze on screen is not touched.
- **Choosing a solver from `Solved` goes to `Ready`** and promotes the finished
  run to the previous-run column. Choosing the solver that is already selected
  does the same, so `s` after it re-runs the same solver against the previous
  run.
- **Choosing a solver from `Solving` or `Paused(Solving)` abandons the run** and
  goes to `Ready`. The abandoned run does **not** promote: it is incomplete, and
  a half-finished expanded count beside a finished one would be read as a
  comparison.
- **Single step from a running phase implies pause.** `.` from `Generating` or
  `Solving` moves to `Paused(..)` and advances exactly one step. Someone
  reaching for single step wants to stop and look, and a key that silently does
  nothing is reported as a bug.
- **Too small auto-pauses and never auto-resumes.** Section 13.4.

### 5.3 What `App` holds

```rust
pub struct App {
    pub phase: Phase,
    pub seed: u64,
    pub maze: Maze,
    pub braid_factor: f64,
    pub generator_ix: usize,        // index into GENERATORS
    pub solver_ix: usize,           // index into SOLVERS
    pub run: Option<Run>,           // this run, and the algorithm it runs
    pub previous: Option<RunStats>, // previous run, section 8
    pub budget: StepBudget,
    pub ascii: bool,
    pub help_open: bool,
    pub too_small: Option<TooSmall>, // section 13
    pub quit: bool,                 // the loop of 6.1 breaks on it
}
```

`App` owns the model and the selection. It holds no `Instant` and no terminal
size: the loop measures time and passes a `Duration` in, and the renderer reads
the size from the frame. This is the one-way rule of ADR 0006 applied to the
state machine.

The two index fields hold the **selection**, which is what the next `g` or `s`
runs. `run` holds what is running, together with the algorithm box, because the
cell-state functions of section 7.3 take the maze and nothing else.

> Corrected after handoff, in commit f23fe2b. The handed-off struct held
> `generator: Option<Box<dyn Generator>>` and `solver: Option<Box<dyn Solver>>`
> beside `run`. Section 7.3 gives `solver_state` and `gen_state` no argument
> that could carry an algorithm, so a `Run` has to own one, and the two fields
> would have been a second owner of one box.

> Corrected after handoff, in commit 2ace438. The handed-off struct did not
> hold `quit`, and the loop of section 6.1 reads `app.quit`. The loop owns the
> terminal, so quitting has to reach it as data.
>
> Two private fields stand beside the public ones, and neither breaks the rule
> above. `capacity: Option<(u16, u16)>` is what the caller computes with the
> capacity function of section 7.1 and passes in: `App` learns capacity, which
> the size rules of section 13.2 need, and never the terminal size. `rng: Rng`
> is the stream a generate started on, which the braiding pass of section 4.3
> draws from when the generation run reports `Done`, one or more ticks later.

---

## 6. The loop, the tick and the step budget

ADR: [0008, the tick loop](docs/adr/0008-the-tick-loop.md).

A **tick** is one pass of the application loop. It applies the input that is
waiting, advances the run by the **step budget**, and redraws if anything
changed. A tick is not a fixed length of time.

### 6.1 The shape

```rust
const FRAME: Duration = Duration::from_nanos(16_666_667);   // 60 Hz
const MAX_ELAPSED: Duration = Duration::from_millis(100);
const DEADLINE_CHECK_EVERY: u32 = 64;

loop {
    let timeout = if app.phase.is_animating() {
        next_render_deadline.saturating_duration_since(Instant::now())
    } else {
        Duration::from_secs(1)
    };

    if event::poll(timeout)? {
        apply(&mut app, event::read()?);            // may set dirty
        while event::poll(Duration::ZERO)? {        // drain
            apply(&mut app, event::read()?);
        }
    }

    if app.phase.is_animating() {
        let elapsed = last_step.elapsed().min(MAX_ELAPSED);
        last_step = Instant::now();
        let want = app.budget.advance(elapsed);
        let deadline = Instant::now() + FRAME / 2;
        for i in 0..want {
            if i % DEADLINE_CHECK_EVERY == 0 && Instant::now() > deadline {
                app.budget.discard();
                break;
            }
            if app.step() == StepOutcome::Done { app.advance_phase(); break; }
            dirty = true;
        }
    }

    if dirty {
        terminal.draw(|f| ui::render(f, &app))?;
        dirty = false;
        next_render_deadline = Instant::now() + FRAME;
    }
    if app.quit { break; }
}
```

`is_animating()` is true for `Generating` and `Solving` and false for
everything else.

### 6.2 Render on change, and the resize arm

**Redraw only when the model changed.** A paused application draws nothing and
blocks in `poll` for a second at a time.

This costs one `Event::Resize` arm:

```rust
Event::Resize(..) => dirty = true,
```

The arm sets the flag and **reads nothing from the event**. Layout still comes
from `Frame::area()` inside the draw callback, which is the documented source of
truth; `Terminal::draw` autoresizes the fullscreen viewport on its own. The arm
exists only because a paused application blocked in `poll` would otherwise never
repaint after a resize.

**Drain the input queue** with `poll(Duration::ZERO)` after the first event and
before advancing the run. Without the drain, a held key produces one loop
iteration per repeat and the animation speeds up while the key is down. The
drain is bounded by what is already queued, so input never starves the tick, and
the timeout is exactly the time to the next render deadline, so the tick never
starves input.

**Filter key events on `KeyEventKind::Press`,** or use
`Event::as_key_press_event()`. On Windows crossterm delivers a press and a
release for one keystroke, and most keys in section 7.5 toggle or step, so
without the filter every key acts twice.

### 6.3 Overrun: a deadline, a discard and a clamp

A budget derived from elapsed time spirals if a tick overruns, because the
overrun enlarges the next tick's elapsed time, which buys a larger budget.

**Stop at a deadline and discard the unspent budget.** The deadline is half a
frame, near 8 ms of 16.7 ms, which leaves room for the draw. Test it every 64
steps, not every step: at tens of thousands of steps per second an
`Instant::now()` per step is itself a cost.

**Clamp measured elapsed time to 100 ms**, about six frames, before converting
it. This covers a suspend, a sleep and a debugger breakpoint.

Carrying the unspent budget instead of discarding it guarantees that the next
tick overruns too, and the debt compounds. Discarding is the honest failure: the
animation runs slower than the requested rung because the machine cannot go
faster, and it recovers as soon as the load drops.

### 6.4 `StepBudget`

The **speed ladder** is one fixed, non-linear list spanning generation and
solving:

```rust
pub const LADDER: [f64; 9] = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 64.0, 256.0, 1024.0];
pub const DEFAULT_RUNG: usize = 4;      // 8 steps per frame
```

A **speed rung** is stated in steps per frame at the nominal 60 Hz redraw rate,
so the 0.5 rung is 30 steps per second.

```rust
pub struct StepBudget { rung: usize, residue: f64 }

impl StepBudget {
    pub fn advance(&mut self, elapsed: Duration) -> u32 {
        self.residue += elapsed.as_secs_f64() * LADDER[self.rung] * 60.0;
        let steps = self.residue.floor();
        self.residue -= steps;
        steps as u32
    }
    pub fn discard(&mut self);   // residue = 0.0
    pub fn reset(&mut self);     // residue = 0.0
    pub fn faster(&mut self);    // rung saturates at LADDER.len() - 1
    pub fn slower(&mut self);    // rung saturates at 0
    pub fn rung(&self) -> f64;
}
```

**There is no `Instant` inside this type.** The loop measures and passes the
`Duration` in. The ladder and residue arithmetic is the part of the loop most
likely to be wrong, and the test plan forbids driving the real event loop, so
this is where the seam goes. The deadline and the discard of 6.3 stay in the
loop, because they need a clock read in the middle of a batch, and they go
untested.

Residue rules:

| event | timestamp | residue |
| --- | --- | --- |
| resume from `Paused` | reset to now | zeroed |
| speed change | untouched | **kept** |
| single step | reset to now | zeroed |

Resetting the timestamp on resume is not a preference: keeping it means
resuming after ten seconds buys ten seconds of budget and the algorithm jumps.
The residue goes with it so a resume starts clean. A speed change keeps the
residue because it is under one step by definition, so dropping it is
unobservable, and a rule with no effect is a rule to get wrong later. Single
step zeroes it because single step is exact and should not leave a fraction that
makes the next automatic step arrive early.

Render on change pays for itself here: at the 0.5 rung the ticks where the
budget does not reach one step change nothing, so they are not dirty and never
draw. The redraw rate follows the speed with no separate rule.

### 6.5 Instant generate

`G`, `n`, `b`, `f` and the four size keys run generation to completion outside
the loop:

```rust
while gen.step(&mut maze) == StepOutcome::Stepped {}
braid(&mut maze, factor, &mut rng);
phase = Phase::Ready;
```

The `Generating` phase is never entered. Braiding is already an instant free
function, so instant generate takes the same shape as the pass that follows it:
two synchronous calls, then `Ready`. Running it inside the loop would put an
infinity into a ladder that is defined as a finite list.

### 6.6 Scale

The default maze at 120 x 30 is 29 x 9, which is 261 cells, so a Recursive
Backtracker run is roughly 520 steps. At the top rung of 1024 steps per frame
the whole default maze finishes inside one frame. The top rungs mean nothing
until the maze is far larger than the default, and the loop is self-limiting, so
neither the maze size nor the ladder needs a performance cap.

---

## 7. The screen

ADR: [0007, how MazeLab draws](docs/adr/0007-how-mazelab-draws.md).

### 7.1 The layout

The screen is **variant B, Bands**. It is **fixed at every terminal size**.
There is no responsive switching between layouts and no panel that appears or
disappears with width. Below the layout floor of section 13 the whole screen
becomes the too-small panel.

```
 MazeLab  seed 445  gen Recursive Backtracker  solver A*  phase Solved  speed 8x
                    <the maze pane, centred on both axes>
┌ this run ────┐┌ previous run ┐┌ maze ────┐
│algorithm  A* ││algorithm BFS ││maze  11x6│
│step       52 ││step       64 ││braid 0.00│
│expanded   52 ││expanded   64 ││carved  65│
│frontier    5 ││frontier    0 ││start  0,0│
│path len   34 ││path len   34 ││equal path│
└──────────────┘└──────────────┘└──────────┘
SS start  GG goal  @@ cur  ▓▓ path  ▒▒ front  ░░ exp     floor  ██ wall
g generate  G instant  s solve  Space pause  . step  +/- speed  1-2 gen  3-5 solver
```

**Chrome** is everything the screen draws that is not the maze pane. It costs
**0 columns and 10 rows**:

| band | rows |
| --- | --- |
| status | 1 |
| statistics band, bordered | 7 |
| legend | 1 |
| help | 1 |

The maze pane keeps the full terminal width and the remaining rows, and the
maze is centred inside it on both axes. Because `2H+1` is odd and the pane may
have an even number of rows, up to one row of the pane can go unused; that slack
is inside the pane and is not part of the chrome.

**Capacity** is the largest maze a terminal of a given size can show:

```
W = (cols - 2) / 4          H = (rows - 11) / 2
```

| terminal | capacity |
| --- | --- |
| 120 x 30 | 29 x 9 |
| 80 x 24 | 19 x 6 |
| 79 x 19 | 19 x 4 |

Variant B was chosen over the roomier variant C because the sequential
comparison of ADR 0009 needs both runs side by side, at the same size, in the
same shape. 19 x 6 at 80 x 24 is the price and it was chosen with that number in
view.

### 7.2 The status row

`MazeLab  seed <n>  gen <full name>  solver <full name>  phase <phase>  speed <rung>x`

The **full** algorithm names live here, because this is where there is width.
The seed is always on screen. The rung prints as `0.5x` through `1024x`.

### 7.3 Cell state

A **cell state** is the single state a cell resolves to when it is drawn, chosen
by an ordered priority. A cell can match more than one condition at once, so the
order decides which state shows.

```rust
// Variant order is the priority order. Earlier wins.
pub enum SolverCellState { Start, Goal, Current, Path, Frontier, Expanded, Open, Wall }
pub enum GenCellState    { Current, Frontier, Carved, Uncarved }
```

**These resolve to a state, not to a colour.** That is why they live in `run.rs`
and not in `ui/`, and it is why `--ascii` costs nothing: the same cell state, a
different glyph table. `ui/palette.rs` maps a state to a colour and a glyph and
holds no logic.

One function per phase:

```rust
impl Run {
    pub fn solver_state(&self, maze: &Maze, dx: u16, dy: u16) -> SolverCellState;
    pub fn gen_state(&self, maze: &Maze, dx: u16, dy: u16) -> GenCellState;
}
```

**Resolving a `DisplayCell::Cell(c)`** walks the priority order: `c == start`,
`c == goal`, `Some(c) == solver.current()`, `c` is on `solver.path()`,
`solver.is_frontier(c)`, `solver.is_expanded(c)`, else `Open`. Generation is the
same walk over its four states, with `Carved` from `maze.is_carved(c)`.

**Resolving a `DisplayCell::Passage(a, b)`: take the weaker of the two ends.**
Each end resolves to its *non-positional* state, which means the walk skips
`Start`, `Goal` and `Current` and continues at `Path`. The passage then takes
whichever of the two results is later in the priority order.

The reason is contiguity without inflation. A solved path has to read as one
connected line, so a passage between two path cells is `Path`. An expanded
region has to read as a region, so a passage between two expanded cells is
`Expanded`. The **frontier** must stay a thin ring, and it does: a frontier
cell's neighbour is almost always expanded, so the passage takes `Expanded`, the
weaker of the two. In generation the non-positional walk begins at `Frontier`,
and what it gives a passage depends on what the generator holds in its
frontier. The frontier of Randomized Prim holds uncarved cells, and a passage
joins two carved cells, so every passage of that run is `Carved`. The frontier
of the Recursive Backtracker is its stack and a stack cell is carved, so the
corridor it is walking reads as one `Frontier` line. That is the same
contiguity the rule is for.

> Corrected after handoff, in commit f23fe2b. The handed-off text said that in
> generation every passage joins two carved cells, so every passage is
> `Carved`. That holds for Randomized Prim alone. Reading it as a rule would
> put a `Carved` gap between every two cells of the Recursive Backtracker's
> stack, and the corridor the run is walking would draw as a dotted line.

`DisplayCell::Wall` resolves to `Wall` and `Uncarved` respectively.

### 7.4 The palette and the two glyph sets

A **glyph set** is the complete mapping from cell state to the characters that
draw it. MazeLab has two, and **a glyph set is selected as a whole and the two
are never mixed.**

Two screen columns per display-grid position, so every glyph below is two
characters wide.

| solver cell state | Unicode | `--ascii` | colour |
| --- | --- | --- | --- |
| start | `SS` | `SS` | LightGreen, bold |
| goal | `GG` | `GG` | LightRed, bold |
| current | `@@` | `@@` | White, bold |
| path | `▓▓` | `**` | Green |
| frontier | `▒▒` | `::` | LightCyan |
| expanded | `░░` | `..` | Blue |
| open floor | two spaces | two spaces | terminal default |
| wall | `██` | `##` | DarkGray |

| generation cell state | Unicode | `--ascii` | colour |
| --- | --- | --- | --- |
| current | `@@` | `@@` | White, bold |
| frontier | `▒▒` | `::` | LightMagenta |
| carved | two spaces | two spaces | terminal default |
| uncarved | `██` | `##` | DarkGray |

Four rules hold this table:

1. **The glyph alone identifies the cell state; colour only reinforces it.** The
   16 named colours carry no contrast guarantee, because the user controls what
   each one renders as, so a theme can collide any two of them.
2. **The shade ramp is monotone and carries the ranking**: `██` wall, `▓▓` path,
   `▒▒` frontier, `░░` expanded, blank floor. The ranking survives a terminal
   that renders two of the colours identically.
3. **Start, goal and current are letters in both glyph sets.** A two-column cell
   is too small to carry a fourth shade that stays distinct from `▓▓`, `▒▒` and
   `░░`, and these are the three states where a reader most wants certainty.
4. **`Yellow` (crossterm's `DarkYellow`) is banished from the maze palette.** It
   is a documented offender: it renders grey in PowerShell. It stays in use for
   the help row's key hints, where a contrast failure costs nothing.

Use the light SGR variants (`LightGreen`, `LightRed`, `LightCyan`) directly.
`\x1b[1m` is not a reliable route to a bright colour, because Windows Terminal
makes `intenseTextStyle` user-settable.

Only `U+2588` is verified against the Windows Terminal default face. `U+2591`,
`U+2592` and `U+2593` sit in the same Block Elements range and are assumed to
follow. `--ascii` is the guarantee, not the assumption.

### 7.5 The keymap

| key | action | note |
| --- | --- | --- |
| `g` | generate | animated, from the current seed |
| `G` | generate instantly | skips the animation |
| `s` | solve | runs the selected solver |
| `Space` | pause / resume | `Generating` and `Solving` |
| `.` | single step | exactly one step, whatever the speed; implies pause |
| `+` `-` | speed up / down | one rung of the ladder |
| `1` `2` | choose generator | selects; does not generate |
| `3` `4` `5` | choose solver | `Solved` becomes `Ready` |
| `Tab` | next solver | shortcut for `3`-`5`, wraps |
| `<-` `->` | maze narrower / wider | regenerates instantly |
| `Up` `Down` | maze taller / shorter | regenerates instantly |
| `f` | refit and regenerate | fits the maze to the terminal now |
| `n` | new seed | a fresh random seed, then regenerates instantly |
| `b` | braid factor | cycles 0.00, 0.25, 0.50; regenerates instantly |
| `?` | help | the full keymap over the maze pane |
| `q` `Esc` | quit | restores the terminal |

Direct selection, not cycling, for the generator and the solver: two generators
and three solvers fit on number keys, and a number key is one press from any
state. `Tab` is the one cycling shortcut, for the case the application is built
for, which is running the same maze through solver after solver.

`+` is accepted with and without shift, so `=` is an alias. `-` needs no alias.

> Corrected after handoff, in commit 2ace438. The handed-off table said that
> `n` generates animated. Section 6.5 lists `n` among the keys that run
> generation to completion outside the loop, and the `any` arm of the diagram
> in section 5.1 sends every key but `g` to `Ready`. The two cannot both hold.
> Section 6.5 is the one that gives the shape of the operation, so `n` is
> instant and `g` is the one key that animates.

**The help overlay.** `?` toggles a bordered overlay over the maze pane holding
the whole table above. While it is open, `Esc` closes the overlay instead of
quitting and `?` toggles it; every other key acts normally and leaves the
overlay open. The overlay changes no phase, so an animation continues behind it.

> Corrected after handoff, in commit 55d8f59. The table above is 87 columns
> wide as an overlay row, and section 7.6 makes 79 columns the widest thing
> MazeLab draws. The overlay therefore shortens the notes of `.`, `n` and `b`
> and names the `b` action "cycle braid factor", so that its box is 76 columns
> wide and the meaning of each row stays the same. The box is 18 rows, and the
> maze pane has 14 rows at 80 x 24 and 9 at the layout floor. Where the pane is
> shorter than the box, the box starts at the top of the pane and covers the
> chrome below it. It never covers the status row, because section 7.2 keeps
> the seed on screen.

### 7.6 The legend row

The legend is **the in-application evidence** that the glyphs and the colours
arrived intact, so it is part of ADR 0007 and not layout furniture. Its
contract:

**It shows every cell state of the current phase, in the live glyph set and the
live colour, and it never drops an entry.**

Two forms, on a **100-column breakpoint**:

| form | width | labels |
| --- | --- | --- |
| full, at 100 columns and above | 94 | `start goal current path frontier expanded floor wall` |
| short, below 100 columns | 79 | `start goal cur path frontier expanded floor wall` |

`frontier` and `expanded` are glossary terms and are **never** shortened. Below
100 columns the separators tighten to one space.

The swatch for a state whose glyph is blank — `open floor`, and `carved` in the
generation legend — is bracketed, because a blank swatch demonstrates nothing.

79 columns is the widest thing MazeLab draws, so the legend is what fixes the
layout floor of section 13.

### 7.7 The help row

Two forms, on the same 100-column breakpoint:

| form | keys |
| --- | --- |
| full, at 100 columns and above | all 12: `g G s Space . +/- 1-2 3-5 Tab arrows f n b ?` |
| short, below 100 columns | 7: `g generate  s solve  Space pause  . step  +/- speed  ? help  q quit` |

**The help row must never clip.** The full form's labels are abbreviated so
that 12 keys fit within 100 columns; the short form is measured at 67 columns.
The renderer selects the widest form that fits the current width, and a test
asserts that each form fits its breakpoint.

---

## 8. The statistics band

ADR: [0009, comparison is sequential](docs/adr/0009-comparison-is-sequential.md).

### 8.1 The three columns

Three `Fill(1)` columns, each bordered and titled: **this run**, **previous
run**, **maze**. Five content rows, so the band is 7 rows including its border.

| row | a solver run | a generation run |
| --- | --- | --- |
| 1 | `algorithm  A*` | `algorithm  RecBack` |
| 2 | `step       52` | `step        45` |
| 3 | `expanded   52` | `carved      29` |
| 4 | `frontier    5` | `frontier    12` |
| 5 | `path len   34` | `path len     -` |

The vocabulary is fixed: a **step** is one call to `step`; **expanded** is a
cell removed from the frontier and processed; **frontier** is the live size of
the working set; **path length** counts the cells from start to goal inclusive;
**carved count** is the generation counterpart of expanded.

The maze column:

```
maze  29x9
braid 0.00
carved  261
start   0,0
equal path
```

`equal path` shows only while the braid factor is 0. It is the quiet form of the
statement that an unbraided maze makes every solver return the identical path,
so a tie between two path lengths is expected and is not a broken comparison.
The README says the same thing at more length, and teaches the reader to watch
**expanded** instead.

**The algorithm name in this band is always the abbreviated one**: `RecBack`,
`RandPrim`, `DFS`, `BFS`, `A*`. Three columns at the 79-column floor give 26
each, 24 inside the border, and `algorithm  Recursive Backtracker` needs 33. The
alternative was to keep the full names and lift the floor to about 103 columns,
which would abandon the 80-column terminal that this layout was chosen for. The
full name is on the status row.

### 8.2 What each column holds, and when it changes

**this run** holds the current run:

- the generation run while `Generating` and `Paused(Generating)`,
- the generation run's final numbers while `Ready`, so the column is never empty,
- the solver run from `Solving` onward.

**previous run** holds at most one **finished solver run**. It is filled when
the phase **leaves** `Solved`, not when it enters it.

Promoting on entering `Solved` would empty the column labelled "this run"
exactly when the user wants to read it. Leaving `Solved` means the user has
chosen another solver, so the two columns then always hold the last two solvers
run on this maze, in the order they were run.

It is cleared by any new maze. The comparison is between solvers on one maze.

An abandoned solver run never promotes. Only a run that reached `Solved` does.

> Corrected after handoff, in commit 2ace438. "so the column is never empty"
> holds for the `Ready` that follows a generate, where the generation run is
> still on screen. It does not hold for the `Ready` a solver key reaches from
> `Solved` or from `Solving`: the run that column held is promoted, or
> abandoned, and the next run starts on `s`. The band draws an empty this-run
> column there.

```rust
pub struct RunStats {
    pub algorithm_short: &'static str,
    pub steps: u64,
    pub expanded_or_carved: usize,
    pub frontier: usize,
    pub path_len: Option<usize>,   // None for a generation run
}
```

A **run** is one execution of a generator or a solver over a maze, together with
its statistics. A maze outlives the runs made on it, and holds at most one
**previous run**.

---

## 9. The command line

A small `clap` surface. There is no in-application text entry in v1.

| flag | type | default | bounds |
| --- | --- | --- | --- |
| `--seed <u64>` | `u64` | a fresh random seed | none |
| `--width <u16>` | `u16` | capacity, else 4 | `4..=512` |
| `--height <u16>` | `u16` | capacity, else 4 | `4..=512` |
| `--ascii` | flag | off | — |
| `--generator <key>` | string | `backtracker` | a key in `GENERATORS` |
| `--solver <key>` | string | `astar` | a key in `SOLVERS` |

**Size is rejected at parse time, never clamped.** `--width 2` and
`--width 600` both exit with an error. Clamping would contradict the rule that
maze size is explicit: the user asked for 2 and would silently get 4.

```
error: invalid value '2' for '--width <WIDTH>': maze width must be between 4 and 512
```

**The size default is a fallback chain**, and the same chain is used when only
one of the pair is given: the flag if given, else **capacity** for that axis,
else the 4-cell minimum when the terminal is below the layout floor and capacity
is undefined.

**`--generator` and `--solver` are validated against the registry tables**, not
against a `clap::ValueEnum`. An unknown key exits with an error that lists the
valid keys, taken from the table, so the two can never drift apart.

The braid factor has no flag. It is reachable only by `b`. See section 15.

---

## 10. Terminal setup, teardown and panic restore

Setup and teardown live in `main.rs` and nowhere else.

```rust
fn main() -> Result<()> {
    let args = cli::Args::parse();
    // The first queued crossterm command is what enables
    // ENABLE_VIRTUAL_TERMINAL_PROCESSING on Windows. It is an undocumented
    // side effect. Read "Before you bump crossterm" in docs/adr/0007 before
    // changing the crossterm version.
    let terminal = ratatui::init();
    let _guard = TerminalGuard;
    let result = run(terminal, args);
    result
}

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) { ratatui::restore(); }
}
```

Three points, and each one is a refinement of a decision rather than a
restatement:

1. **Use `ratatui::init` and `ratatui::restore`.** `init` enables raw mode,
   enters the alternate screen and installs a panic hook that restores the
   terminal. `restore` disables raw mode **first** and leaves the alternate
   screen **second**, which is the only teardown order that any primary document
   states as a procedure.
2. **MazeLab installs no panic hook of its own.** The original intent was a
   chained hook; `ratatui`'s hook already does the whole job, and a chained hook
   that adds nothing is a hook to get wrong. If one is ever added it must be
   installed **before** `ratatui::init`, which documents that requirement.
3. **The `Drop` guard stays, and it earns its place.** It covers what the panic
   hook does not: an early `return` or a `?` out of `run`. A double restore is
   harmless, because `restore` ignores its own errors.

Do **not** use `init_with_options` or `try_init_with_options`: they do not enter
the alternate screen.

---

## 11. The test plan

Three tiers. **Nothing drives the real event loop.**

### 11.1 Tier 1: maze invariants, pure logic, no terminal

1. **Reproducibility as a stored reference vector.** A fixed seed at a fixed
   size renders to a fixed string, committed in the test. A self-comparison
   (generate twice, assert equal) passes even if the RNG changed under the
   crate, which is the failure this tier exists to catch.
2. **A maze is connected.** A flood fill from `(0, 0)` reaches every cell.
3. **An unbraided maze has exactly `W*H - 1` carved edges.** This is the
   spanning-tree property that makes it **perfect**.
4. **Braiding at factor 0 changes nothing**, byte for byte.
5. **Braiding at factor `f` removes about `f` of the dead ends**, and never
   disconnects the maze, and only ever adds edges.
6. **The wall flags agree on both sides** after any sequence of carves.
7. **The display-grid derivation**: every position classifies as the table in
   2.3 says, and the outer border is always `Wall`.
8. **Capacity and the size clamp are pure functions of two integers**, tested at
   the boundaries: 79 x 19, 80 x 24, 120 x 30, and one below each floor.
9. **`StepBudget` arithmetic.** Synthetic durations only. At the 0.5 rung
   16.7 ms yields 0 steps and the next 16.7 ms yields 1. `discard` and `reset`
   zero the residue; a speed change keeps it.
10. **Both generators terminate** and leave every cell carved, at several sizes
    including 4 x 4 and a non-square maze.

### 11.2 Tier 2: solver correctness

1. **The path is walkable**: consecutive cells are adjacent and the wall between
   them is carved.
2. **The path starts at start and ends at goal.**
3. **BFS and A\* agree on path length**, over a set of seeds and sizes. This is
   the test the `(f, h, insertion)` tie-break must not break: lower-`h`-first
   changes the *order of expansion*, never the *length of the result*, because
   Manhattan distance is admissible. A failure here means the heuristic or the
   `Ord` is wrong, not that the tie-break is wrong.
4. **DFS finds a path**, and its length is greater than or equal to BFS's.
5. **Every solver reaches the goal on a braided maze too.**
6. **`expanded_count` never exceeds `W*H`**, and every cell on the path is
   expanded.
7. **A solver never changes the maze.** The type system carries this; one test
   asserts the maze is unchanged after a run, to catch interior mutability.

### 11.3 Tier 3: rendering, with `TestBackend`

`TestBackend` renders into a memory buffer and needs no real terminal, so these
run on all three CI platforms. Use `assert_buffer_lines`, not the deprecated
`assert_buffer_eq!`, and **do not type a test helper `io::Result<()>`**:
`TestBackend::Error` is `Infallible`.

1. **The two cell-priority functions**, one case per state, including a cell
   that matches several conditions at once.
2. **Passage resolution takes the weaker end**, with a path passage, an expanded
   passage and a frontier-beside-expanded passage.
3. **No two cell states in one phase share a glyph, in either glyph set.**
4. **Every glyph in the ASCII glyph set is below `U+0080`.** The escape hatch is
   then provably an escape hatch.
5. **`DarkYellow` does not appear in the maze palette**, so the banishment
   cannot be quietly undone by a later edit.
6. **The legend never drops an entry**, at 100 columns and at 79 columns, in
   both glyph sets and in both phases.
7. **Each help row form fits its breakpoint.**
8. **The two too-small panels**, and the degradation order of 13.5.
9. **A full-screen snapshot** of a small solved maze, as one end-to-end check of
   the layout.

Tests 3, 4 and 5 are what turn the cross-platform rendering claim into something
that is proven rather than argued. They prove a claim worth stating: **MazeLab
stays legible when colour is absent, or when a theme renders two of its colours
alike.**

### 11.4 What is not tested, on purpose

- The real event loop, the `poll` timeout and the input drain.
- The deadline and the discard of 6.3. They need a clock read in the middle of a
  batch, and their failure mode is a slow animation rather than a wrong one.
- Anything that needs a real font, a real colour table or `SetConsoleMode`. No
  test can render a glyph. The legend row of 7.6 is what covers that gap, in
  front of the user.

---

## 12. Continuous integration

The matrix proves **portable compilation and portable logic**. It proves nothing
about rendering: `TestBackend` renders into memory and never touches a terminal,
a font, a colour table or `SetConsoleMode`. Section 11.3 and the legend row
carry that half of the claim. The README says so in those terms.

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2025, macos-26]
    steps:
      - uses: actions/checkout@v7
      - name: Show toolchain
        run: |
          rustc --version
          cargo --version
      - name: Add clippy
        run: rustup component add clippy
      - name: Build
        run: cargo build --locked --all-targets
      - name: Test
        run: cargo test --locked
      - name: Clippy
        run: cargo clippy --locked --all-targets -- -D warnings

  msrv:
    name: MSRV 1.88.0
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v7
      - run: rustup toolchain install 1.88.0 --profile minimal
      - run: cargo +1.88.0 build --locked --all-targets

  fmt:
    name: rustfmt
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v7
      - run: cargo fmt --all --check

  docs:
    name: rustdoc
    runs-on: ubuntu-24.04
    env:
      RUSTDOCFLAGS: -D warnings
    steps:
      - uses: actions/checkout@v7
      - run: cargo doc --locked --no-deps
```

Five points behind this file:

- **Explicit runner labels, not `-latest`.** `windows-latest` and `macos-latest`
  both changed image in 2026, and an explicit label makes that a reviewed
  change. The three labels point at the same images `-latest` does today.
- **`fail-fast: false`**, because a failure on one platform is exactly what this
  matrix exists to find.
- **`-D warnings` on clippy**, because `cargo clippy` otherwise exits 0 on a
  lint and the step proves nothing.
- **`rustup component add clippy` is explicit**, because clippy is not listed in
  the documented tool inventory of the Ubuntu images even though the image build
  installs it. The documented guarantee is what the workflow may rely on.
- **`macos-26` is arm64 only.** Intel macOS would need `macos-26-intel` as a
  fourth leg. It is not in v1.

The MSRV job is what makes the 1.88.0 promise real: `ratatui` shipped an MSRV
bump in a **patch** release, and the rand family states that its MSRV can be
raised in any release, so the floor can rise under a routine `cargo update`.

---

## 13. Size bounds and the two thresholds

ADR: [0007, how MazeLab draws](docs/adr/0007-how-mazelab-draws.md).

**There are two thresholds, not one.** They have different causes, different
remedies and different wordings, and they must not be collapsed.

| | fires when | replaces | remedy |
| --- | --- | --- | --- |
| **maze fit**, dynamic, per maze | `cols < 4W+2` or `rows < 2H+11` | the maze pane only; status, statistics, legend and help keep rendering | resize, or `f` |
| **layout floor**, absolute | below **79 x 19** | the whole screen | resize only |

### 13.1 The numbers

- **Minimum maze: 4 x 4.** Correctness bottoms out far lower, but at three rows
  a maze is a corridor and backtracking has nowhere to happen. 4 x 4 also keeps
  the rows floor at 19, below the 80 x 24 case the layout was chosen for; at
  5 x 5 the floor would become 21 rows and an 80 x 20 terminal would be refused.
- **Layout floor: 79 x 19.** 79 columns from the legend's short form, which is
  the widest thing MazeLab draws and which may not be clipped. 19 rows from 10
  rows of chrome plus the 9 a 4-cell-tall maze needs. Capacity at exactly the
  floor is 19 x 4, so the two floors are consistent.
- **No interactive maximum.** Capacity already bounds anything a terminal can
  show: even 300 x 100 gives 74 x 44, about 3 200 cells. The loop of section 6
  is self-limiting, so smoothness caps neither the maze nor the speed ladder.
- **A hard cap of 512 on each axis, reachable only from the command line**, and
  **rejected at parse time rather than clamped**. It refuses an absurd
  allocation; it is not a usability bound.

### 13.2 Maze size is a model concern; the panel is a render concern

This is ADR 0006's one-way rule applied to this question, and the rest follows
from it.

- **A maze is always generated at startup**, whatever the terminal size. A later
  resize reveals a maze that already exists; it never triggers one.
- **The startup size** is `--width` / `--height` if given, else capacity, else
  the 4 x 4 minimum where the terminal is below the floor and capacity is
  undefined. The default is **not** clamped by the 512 cap, because capacity can
  never reach 512.
- **The size keys clamp to capacity. The command line does not.** Growing into a
  panel by pressing `->` is a dead end with no feedback. An explicit
  `--width 60` on a 40-column terminal is a stated intention, and the
  application does not overrule it.
- **A size key is a generate.** It regenerates **instantly** at the current
  seed, and it discards the previous run like any new maze. Animated
  regeneration would start a run per keypress.
- **At the clamp a size key is silently inert.** The `maze 19x4` field in the
  statistics band is the feedback.

### 13.3 `f`, refit and regenerate

At or above the layout floor, `f` refits the maze to capacity and regenerates
instantly. This includes the case where an oversized `--width` put the user in
the maze-fit panel.

**Below the layout floor `f` is inert**, and the panel drops its `press f` line
there. Clamping to 4 x 4 and regenerating would "work" and change nothing
visible, because the chrome still cannot render. A key that silently does
nothing is acceptable only because the panel is naming the one real remedy.

### 13.4 What a run does when a threshold is crossed

**Entering too small auto-pauses the run. Growing the terminal back does not
auto-resume it.** The maze reappears, still paused, and `Space` continues.

MazeLab exists to make an algorithm visible one step at a time, so stepping
where nobody can see is the one outcome with no value. Auto-resume is worse than
it looks: dragging a window edge would fire a stutter of pause and resume, and a
user who shrank the window deliberately would get their run restarted without
asking. Auto-pause needs no new state, because it reuses `Paused`.

### 13.5 The panel

One box, 46 x 8, centred, in `LightRed`, with two messages. "Terminal too small"
is wrong for a 29 x 9 maze on a 100-column terminal: the terminal is fine and
the maze is oversized.

```
╭────────────────────────────────────────────╮
│       Maze too large for this terminal     │
│                                            │
│         maze 29 x 9 needs 118 x 29         │
│            terminal is 100 x 24            │
│                                            │
│  resize, or press f to refit and regenerate│
╰────────────────────────────────────────────╯

╭────────────────────────────────────────────╮
│             Terminal too small             │
│                                            │
│                need 79 x 19                │
│                have  60 x 18               │
│                                            │
│             resize to continue             │
╰────────────────────────────────────────────╯
```

In the maze-fit case the box always fits, because the layout floor guarantees a
pane of at least 79 x 9. Only the layout-floor case can go below 46 x 8.

**It degrades by dropping the border first, then dropping lines from the
bottom**: the hint, then `have`, then `need`. It never clips a line in the
middle, so the first line survives to the smallest terminal that can hold it.

**The legend is not drawn at all in the layout-floor case.** That is not a
breach of the legend contract, which governs the legend when it renders, not
whether the row exists.

---

## 14. The documentation surface

Four things, and nothing else:

| artifact | job |
| --- | --- |
| `README.md` | what MazeLab is, the demo, how to run it, what is promised |
| `docs/extending.md` | the worked example: how to add a generator |
| `docs/media/` | the demo GIF, the still, and one opportunistic screenshot |
| rustdoc on `generator/mod.rs` and `solver/mod.rs` | the contract |

### 14.1 The extension guide splits by job

The **worked example** goes in `docs/extending.md`, where it is reachable
without running a `cargo` command. The **contract** goes in rustdoc, beside the
type it describes.

`docs/extending.md` **inlines no synthetic generator.** It is a narrated tour of
`src/generator/prim.rs`, chosen over the backtracker because its explicit
frontier also teaches the frontier accessors. A guide that points at a real file
cannot drift silently.

The contract stays honest by a **doctest** holding a minimal `impl Generator`.
That works only because the library-plus-thin-binary split of section 1 makes
`cargo test` run doctests.

**Rustdoc is not published.** There is no docs.rs without crates.io and no Pages
without release engineering. The `cargo doc` job of section 12 checks it at
`-D warnings` instead.

### 14.2 What the README promises

**Only determinism and the MSRV are promises a reader may depend on.**

Cross-platform is a **report, not a promise**: the README claims what the CI
matrix proves, which is that MazeLab compiles, tests and lints clean on Linux,
Windows and macOS. It adds the one sentence that section 11.3 earns: the tests
prove that MazeLab stays legible when colour is absent. It says that macOS is
covered by the matrix alone.

The keymap is written down **once**: the README shows the help row's 12 keys and
defers to `?`. A full 16-row table in markdown would be a third copy that
nothing keeps in sync.

The README also explains the **equal path lengths** in two sentences, because
the demo shows every solver returning the same length and the honest reading of
that is a broken comparison. The same sentences teach the reader to watch
**expanded**.

`CONTEXT.md` and `docs/adr/` are linked as the showcase. `docs/research/` and
`docs/agents/` are not. `SPEC.md` is kept frozen with a superseded-by-the-code
header and is not linked.

### 14.3 The demo

A `vhs` GIF from a committed tape, four beats on one seed: generate, solve with
BFS, swap to A\*, solve again. The two-second hold on `Solved` is cut out as the
PNG still. The tape cannot show the theme inheritance of ADR 0007, and that is
accepted.

`docs/media/` also takes **one** Windows Terminal screenshot, with the host and
the font named in the caption. It is an opportunistic report, not a gate.
Manual verification on real hardware is out of scope as a gate: it cannot be
re-run, it cannot be exhaustive, and it must not block v1 on hardware that may
not exist. There is no macOS screenshot.

---

## 15. Known limitations of v1

Stated so that nobody re-opens them by accident. Each is a deliberate boundary,
not an oversight.

1. **The braid factor has no command-line flag.** It is reachable only by `b`,
   which cycles 0.00, 0.25 and 0.50. A braided maze therefore cannot be
   reproduced from the command line alone: a seed, a size and a version fix a
   maze only at braid factor 0. The CLI surface was fixed at six flags before
   the `b` key existed, and widening it is a v2 change, not a spec correction.
2. **Comparison is sequential.** One maze persists, the solver is swapped and
   re-run, and at most one previous run is kept. Simultaneous side-by-side is
   out of scope; the seam is kept open, and ADR 0009 records what keeps it open.
3. **Braiding is not animated.** It is instant, at a default factor of 0.
4. **Start and goal are fixed at opposite corners**, held as data on the run
   rather than hard-coded in a solver. Moving them needs a cursor, a mode and
   its own keybindings.
5. **Determinism is scoped to one version.** The same seed, the same size and
   the same version give the same maze on all three platforms. It is not a
   cross-version promise. ADR 0005.
6. **Rendering is not proven on hardware and will not be in v1.** Portable
   compilation and portable logic are proven by the CI matrix. Portable display
   rests on a tested invariant, on `--ascii` as a user-set escape hatch, and on
   the legend row as in-application evidence.
7. **No packaging, no release binaries, no crates.io.**
