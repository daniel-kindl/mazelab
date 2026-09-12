//! The generators: the [`Generator`] trait and the generator registry.
//!
//! A **generator** is an algorithm that carves a maze. It is a struct that owns
//! its working set and exposes one step at a time. There is no coroutine, no
//! thread and no replay of a precomputed result. The reason is that the user
//! interface draws the **frontier** on every redraw, so the working set must be
//! inspectable from outside while the algorithm is part-way through its work.
//! See ADR 0002, algorithms are explicit state machines.
//!
//! # The contract
//!
//! 1. **One call to [`Generator::step`] carves at most one edge.** A step that
//!    only moves the working set carves nothing, and it is a step too.
//! 2. **[`StepOutcome::Done`] comes back on the step that completes the run**,
//!    not on a later call. The step that returns `Done` does not have to be a
//!    step that carves.
//! 3. **When `Done` comes back, every cell is carved and the carved edges form
//!    a spanning tree.** The maze is **perfect**. Loops are added by the
//!    braiding pass of [`crate::braid`] alone. See ADR 0004, perfect mazes and
//!    a separate braiding pass.
//! 4. **`make` takes `&Maze`, so a factory cannot change the maze.** It reads
//!    the width and the height to size its flag grids, and it fills the working
//!    set. The first change a generator makes to a maze is in its first `step`.
//! 5. **Inspection is membership, not iteration.** The renderer resolves one
//!    cell at a time, so [`Generator::is_frontier`] must answer in O(1).
//!    Section 4 does that with a grid of `W * H` flags beside the working set.
//!    Nothing ever iterates a frontier.
//! 6. **The draw is the only source of variation.** Section 4 starts both
//!    generators from cell `(0, 0)`, and that start is fixed so that a seed
//!    reproduces a maze. Walk the neighbours of a cell in the fixed
//!    `North, East, South, West` order of [`Dir::ALL`](crate::maze::Dir::ALL).
//!    Read the three rules of [`crate::rng`] before you sample.
//!
//! # Adding a generator
//!
//! Write one file in this module and add one line to [`GENERATORS`]. The table
//! is the single source of truth for the command line, the help row and the
//! in-application chooser, so the names cannot drift apart. Section 14 puts the
//! worked example in `docs/extending.md`, as a tour of `src/generator/prim.rs`.
//!
//! # A minimal generator
//!
//! `Comb` carves a spine along the top row and one tooth down each column. It
//! is not an interesting maze, and it draws no random number, but it keeps
//! every rule of the contract above: it carves one edge for each step, it
//! returns `Done` on the step that finishes, and what it leaves is **perfect**.
//!
//! ```
//! use mazelab::StepOutcome;
//! use mazelab::generator::Generator;
//! use mazelab::maze::{Cell, Dir, Maze};
//! use mazelab::rng::{self, Rng};
//!
//! struct Comb {
//!     /// The cell the next step carves, in row-major order. `None` once the
//!     /// sweep has left the maze.
//!     next: Option<Cell>,
//!     current: Option<Cell>,
//! }
//!
//! /// The factory a `GeneratorEntry` holds. It reads the maze and fills the
//! /// working set. The sweep starts at `(1, 0)`, because `(0, 0)` is the cell
//! /// the first step carves towards.
//! fn make(_maze: &Maze, _rng: &mut Rng) -> Box<dyn Generator> {
//!     Box::new(Comb {
//!         next: Some(Cell { x: 1, y: 0 }),
//!         current: None,
//!     })
//! }
//!
//! impl Generator for Comb {
//!     fn step(&mut self, maze: &mut Maze) -> StepOutcome {
//!         // The caller does not step after `Done`, so `next` holds a cell.
//!         let Some(c) = self.next else {
//!             return StepOutcome::Done;
//!         };
//!         // The top row hangs off the cell to the west, and every other row
//!         // hangs off the cell above. That is the spine and the teeth.
//!         let towards = if c.y == 0 { Dir::West } else { Dir::North };
//!         if let Some(parent) = maze.neighbour(c, towards) {
//!             maze.carve(c, parent);
//!         }
//!         self.current = Some(c);
//!
//!         let advanced = if c.x + 1 == maze.width() {
//!             Cell { x: 0, y: c.y + 1 }
//!         } else {
//!             Cell { x: c.x + 1, y: c.y }
//!         };
//!         if advanced.y == maze.height() {
//!             self.next = None;
//!             StepOutcome::Done
//!         } else {
//!             self.next = Some(advanced);
//!             StepOutcome::Stepped
//!         }
//!     }
//!
//!     fn current(&self) -> Option<Cell> {
//!         self.current
//!     }
//!
//!     fn is_frontier(&self, c: Cell) -> bool {
//!         self.next == Some(c)
//!     }
//!
//!     fn frontier_len(&self) -> usize {
//!         usize::from(self.next.is_some())
//!     }
//! }
//!
//! let mut maze = Maze::new(4, 3);
//! let mut generator = make(&maze, &mut rng::from_seed(1));
//! while generator.step(&mut maze) == StepOutcome::Stepped {}
//!
//! // Rule 3: every cell is carved, and 12 cells joined by 11 edges are a
//! // spanning tree.
//! assert_eq!(maze.carved_count(), 12);
//! assert_eq!(maze.edge_count(), 11);
//! assert_eq!(generator.frontier_len(), 0);
//! ```

use crate::StepOutcome;
use crate::maze::{Cell, Maze};
use crate::rng::Rng;

pub mod backtracker;
pub mod prim;

/// An algorithm that carves a maze, one step at a time.
///
/// The module documentation holds the contract an implementation must keep.
pub trait Generator {
    /// Advances one step. Carves at most one edge.
    ///
    /// Returns [`StepOutcome::Done`] on the step that completes the run.
    fn step(&mut self, maze: &mut Maze) -> StepOutcome;

    /// The single cell the generator is acting on, for the step now shown.
    fn current(&self) -> Option<Cell>;

    /// True when the cell is in the generator's working set.
    fn is_frontier(&self, c: Cell) -> bool;

    /// The live size of the working set.
    fn frontier_len(&self) -> usize;
}

/// One row of the generator registry.
pub struct GeneratorEntry {
    /// The `--generator` value and the key the command line validates against.
    pub key: &'static str,
    /// The full name, for the status row.
    pub name: &'static str,
    /// The abbreviated name, for the statistics band. See section 13.
    pub short: &'static str,
    /// Makes the generator, ready for its first step.
    pub make: fn(&Maze, &mut Rng) -> Box<dyn Generator>,
}

/// Every generator MazeLab can run, in the order the number keys select them:
/// `1` and `2` index this table.
pub static GENERATORS: &[GeneratorEntry] = &[
    GeneratorEntry {
        key: "backtracker",
        name: "Recursive Backtracker",
        short: "RecBack",
        make: backtracker::make,
    },
    GeneratorEntry {
        key: "prim",
        name: "Randomized Prim",
        short: "RandPrim",
        make: prim::make,
    },
];

#[cfg(test)]
mod tests {
    use super::GENERATORS;

    /// The table is the single source of truth for the command line, the help
    /// row and the in-application chooser, so a duplicated key would make one
    /// of the three pick an entry the other two do not.
    #[test]
    fn every_key_is_unique() {
        for (i, entry) in GENERATORS.iter().enumerate() {
            for other in &GENERATORS[i + 1..] {
                assert_ne!(entry.key, other.key, "two generators answer to one key");
            }
        }
    }

    /// Section 7 selects a generator by index: `1` and `2` index this table. The
    /// order is part of the interface, not an accident of the order the
    /// modules were written in.
    #[test]
    fn the_two_entries_of_section_3_4_stand_in_the_order_the_number_keys_index() {
        let keys: Vec<&str> = GENERATORS.iter().map(|e| e.key).collect();
        assert_eq!(keys, ["backtracker", "prim"]);
    }

    /// The statistics band draws the abbreviated name, because a band column
    /// gives 24 columns inside its border at the 79-column floor and
    /// `algorithm  Recursive Backtracker` needs 33. An abbreviation that is
    /// not shorter than the name it abbreviates buys nothing, and an empty one
    /// leaves the row unreadable.
    #[test]
    fn every_entry_holds_a_name_and_an_abbreviation_of_it() {
        for entry in GENERATORS {
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
