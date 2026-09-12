//! The solvers: the [`Solver`] trait and the solver registry.
//!
//! A **solver** is an algorithm that searches a finished maze for a path from
//! start to goal. It reads the maze and changes only its own search state.
//! [`Solver::step`] takes `&Maze`, so the compiler holds that rule, and that is
//! the whole reason the two traits are not one. See ADR 0003, two traits not
//! one.
//!
//! Like a generator, a solver is a struct that owns its working set and exposes
//! one step at a time, because the user interface draws the **frontier** on
//! every redraw. See ADR 0002, algorithms are explicit state machines.
//!
//! # The contract
//!
//! 1. **One call to [`Solver::step`] expands at most one cell.** A step that
//!    takes a cell it has already expanded and discards it is a step too.
//!    Sections 4.4 and 4.6 both do that, and it is how a working set holds
//!    stale duplicates without a delete.
//! 2. **[`StepOutcome::Done`] comes back on the step that reaches the goal**,
//!    not on a later call.
//! 3. **[`Solver::path`] is `None` before `Done` and `Some` from `Done`
//!    onward.** The path runs from start to goal and holds both, which is what
//!    **path length** counts.
//! 4. **A cell is expanded once.** [`Solver::is_expanded`] is true for a cell
//!    the solver took out of its frontier and processed.
//!    [`Solver::expanded_count`] counts those cells, so it never exceeds
//!    `W * H`. **Expanded** is the count that shows how much of the maze a
//!    solver searched, and the comparison of section 8 rests on it.
//! 5. **Inspection is membership, not iteration.** The renderer resolves one
//!    cell at a time, so [`Solver::is_frontier`] and [`Solver::is_expanded`]
//!    must answer in O(1). Section 4 does that with a grid of `W * H` flags
//!    beside the working set. Nothing ever iterates a frontier.
//! 6. **A solver draws no random number.** It is deterministic without a seed,
//!    which is what makes the test that BFS and A\* agree on path length
//!    meaningful. Walk the neighbours of a cell in the fixed
//!    `North, East, South, West` order of [`Dir::ALL`](crate::maze::Dir::ALL).
//!
//! A solver never runs out of cells before it reaches the goal, which is why
//! [`StepOutcome`] has no `Exhausted` variant.
//!
//! # Adding a solver
//!
//! Write one file in this module and add one line to [`SOLVERS`]. The table is
//! the single source of truth for the command line, the help row and the
//! in-application chooser, so the names cannot drift apart. The shape of the
//! implementation is the one the [`Generator`](crate::generator::Generator)
//! doctest shows, with `&Maze` in place of `&mut Maze`.

use crate::StepOutcome;
use crate::maze::{Cell, Dir, Maze};

pub mod astar;
pub mod bfs;
pub mod dfs;

/// An algorithm that searches a finished maze for a path, one step at a time.
///
/// The module documentation holds the contract an implementation must keep.
pub trait Solver {
    /// Advances one step. Takes `&Maze`: a solver never changes the maze.
    ///
    /// Returns [`StepOutcome::Done`] on the step that reaches the goal.
    fn step(&mut self, maze: &Maze) -> StepOutcome;

    /// The single cell the solver is acting on, for the step now shown.
    fn current(&self) -> Option<Cell>;

    /// True when the cell is in the solver's working set.
    fn is_frontier(&self, c: Cell) -> bool;

    /// The live size of the working set.
    fn frontier_len(&self) -> usize;

    /// True when the solver has removed this cell from its frontier and
    /// processed it.
    fn is_expanded(&self, c: Cell) -> bool;

    /// The number of cells the solver has expanded.
    fn expanded_count(&self) -> usize;

    /// The path from start to goal, inclusive. `Some` once [`Solver::step`]
    /// returned [`StepOutcome::Done`], `None` before that.
    fn path(&self) -> Option<&[Cell]>;
}

/// The neighbours of `c` a solver can move to: the cells on the far side of a
/// carved edge, in the fixed `North, East, South, West` order of [`Dir::ALL`].
///
/// Rule 6 of the contract asks for that order. A solver draws no random
/// number, so the order of this walk is the whole order of a run.
fn reachable(maze: &Maze, c: Cell) -> impl Iterator<Item = Cell> {
    Dir::ALL
        .into_iter()
        .filter(move |&d| maze.is_open(c, d))
        .filter_map(move |d| maze.neighbour(c, d))
}

/// The search state the three solvers of section 4 share: the cells they have
/// expanded, the parent each cell was reached from, and the path that the
/// parents reconstruct.
///
/// The working set is what the three do not share, and it stays in their own
/// files: a stack, a queue and a heap. Everything else here is the same in all
/// three, down to the four inspection methods that delegate to it.
struct Search {
    /// The width of the maze, for the offset into the flag grids.
    width: u16,
    /// The height of the maze, for the bound on a cell from outside.
    height: u16,
    /// The cell the search starts from, where the path begins.
    start: Cell,
    /// The cell the search looks for, where the path ends.
    goal: Cell,
    /// One flag for each cell: the solver took the cell out of its frontier
    /// and processed it. Rule 5 asks for an O(1) `is_expanded`.
    expanded: Vec<bool>,
    /// The number of set flags in `expanded`, which is what the statistics
    /// band draws.
    expanded_count: usize,
    /// The cell each cell was reached from. `None` for the start, and for a
    /// cell no step has reached.
    parent: Vec<Option<Cell>>,
    /// The single cell the step now shown is acting on.
    current: Option<Cell>,
    /// The path, from the step that reached the goal onward.
    path: Option<Vec<Cell>>,
}

impl Search {
    /// Makes the search state for a run over `maze`, from `start` to `goal`.
    fn new(maze: &Maze, start: Cell, goal: Cell) -> Self {
        let cells = usize::from(maze.width()) * usize::from(maze.height());
        Self {
            width: maze.width(),
            height: maze.height(),
            start,
            goal,
            expanded: vec![false; cells],
            expanded_count: 0,
            parent: vec![None; cells],
            current: None,
            path: None,
        }
    }

    /// The number of cells in the maze, which is the size of a flag grid.
    fn cell_count(&self) -> usize {
        self.expanded.len()
    }

    /// The offset of a cell the caller knows is inside the maze.
    fn index(&self, c: Cell) -> usize {
        usize::from(c.y) * usize::from(self.width) + usize::from(c.x)
    }

    /// The offset of a cell, or `None` when the cell is outside the maze.
    ///
    /// The column has to be checked as well as the row: an offset past the end
    /// of a row lands on the next one. This is what every method that answers
    /// a question about an arbitrary cell goes through.
    fn offset(&self, c: Cell) -> Option<usize> {
        (c.x < self.width && c.y < self.height).then(|| self.index(c))
    }

    /// The cell the search looks for.
    const fn goal(&self) -> Cell {
        self.goal
    }

    /// True when the solver has expanded this cell.
    fn is_expanded(&self, c: Cell) -> bool {
        self.offset(c).is_some_and(|i| self.expanded[i])
    }

    /// The number of cells the solver has expanded.
    const fn expanded_count(&self) -> usize {
        self.expanded_count
    }

    /// Records that this step expanded `c`, and makes it the current cell.
    ///
    /// A cell is expanded once, which is rule 4 of the contract. The caller
    /// discards a stale duplicate before it gets here.
    fn expand(&mut self, c: Cell) {
        let i = self.index(c);
        self.expanded[i] = true;
        self.expanded_count += 1;
        self.current = Some(c);
    }

    /// Records the cell `c` was reached from.
    fn set_parent(&mut self, c: Cell, parent: Cell) {
        let i = self.index(c);
        self.parent[i] = Some(parent);
    }

    /// The single cell the step now shown is acting on.
    const fn current(&self) -> Option<Cell> {
        self.current
    }

    /// The path, `None` until the step that reached the goal.
    fn path(&self) -> Option<&[Cell]> {
        self.path.as_deref()
    }

    /// Records the path and answers [`StepOutcome::Done`]. This is the tail of
    /// the step that reached the goal, in all three solvers.
    fn finish(&mut self) -> StepOutcome {
        self.path = Some(self.reconstruct());
        StepOutcome::Done
    }

    /// The path from start to goal, inclusive, which is section 4.7.
    ///
    /// The walk runs back from the goal along `parent` and the result is
    /// reversed. It ends at the start, because the start is the one cell a
    /// solver reaches without a parent and every other cell on the chain was
    /// reached from one. **Path length** counts both ends, so both are here.
    fn reconstruct(&self) -> Vec<Cell> {
        let mut path = Vec::new();
        let mut next = Some(self.goal);
        while let Some(c) = next {
            path.push(c);
            next = self.parent[self.index(c)];
        }
        path.reverse();
        debug_assert_eq!(
            path.first(),
            Some(&self.start),
            "the parent chain from the goal does not reach the start"
        );
        path
    }
}

/// One row of the solver registry.
pub struct SolverEntry {
    /// The `--solver` value and the key the command line validates against.
    pub key: &'static str,
    /// The full name, for the status row.
    pub name: &'static str,
    /// The abbreviated name, for the statistics band. See section 13.
    pub short: &'static str,
    /// Makes the solver, ready for its first step.
    pub make: fn(&Maze, start: Cell, goal: Cell) -> Box<dyn Solver>,
}

/// Every solver MazeLab can run, in the order the number keys select them:
/// `3`, `4` and `5` index this table.
pub static SOLVERS: &[SolverEntry] = &[
    SolverEntry {
        key: "dfs",
        name: "DFS",
        short: "DFS",
        make: dfs::make,
    },
    SolverEntry {
        key: "bfs",
        name: "BFS",
        short: "BFS",
        make: bfs::make,
    },
    SolverEntry {
        key: "astar",
        name: "A*",
        short: "A*",
        make: astar::make,
    },
];

#[cfg(test)]
mod tests {
    use super::SOLVERS;

    /// The table is the single source of truth for the command line, the help
    /// row and the in-application chooser, so a duplicated key would make one
    /// of the three pick an entry the other two do not.
    #[test]
    fn every_key_is_unique() {
        for (i, entry) in SOLVERS.iter().enumerate() {
            for other in &SOLVERS[i + 1..] {
                assert_ne!(entry.key, other.key, "two solvers answer to one key");
            }
        }
    }

    /// Section 7 selects a solver by index: `3`, `4` and `5` index this table. The
    /// order is part of the interface, not an accident of the order the
    /// modules were written in.
    #[test]
    fn the_three_entries_of_section_3_4_stand_in_the_order_the_number_keys_index() {
        let keys: Vec<&str> = SOLVERS.iter().map(|e| e.key).collect();
        assert_eq!(keys, ["dfs", "bfs", "astar"]);
    }

    /// The statistics band draws the abbreviated name, because a band column
    /// gives 24 columns inside its border at the 79-column floor and
    /// `algorithm  Recursive Backtracker` needs 33. An abbreviation that is
    /// not shorter than the name it abbreviates buys nothing, and an empty one
    /// leaves the row unreadable.
    #[test]
    fn every_entry_holds_a_name_and_an_abbreviation_of_it() {
        for entry in SOLVERS {
            assert!(!entry.name.is_empty(), "{} has no name", entry.key);
            assert!(!entry.short.is_empty(), "{} has no short name", entry.key);
            assert!(
                entry.short.chars().count() <= entry.name.chars().count(),
                "the short name of {} is longer than its name",
                entry.key
            );
        }
    }
}
