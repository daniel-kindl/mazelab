//! The Randomized Prim generator, the frontier variant.
//!
//! Section 4.2 of `SPEC.md` holds the algorithm. This is the frontier variant
//! and not true weighted-edge Prim: the **frontier** is the set of uncarved
//! cells beside the carved region, and one step takes a uniformly random cell
//! out of it and carves it to a random carved neighbour. It was chosen because
//! it has a frontier that is worth drawing, and because it contrasts sharply
//! with the corridor walk of the Recursive Backtracker.
//!
//! A run takes exactly `W * H - 1` steps, because every step carves one cell
//! and the cell the run starts from is attached without a step of its own.
//!
//! Section 14 makes this file the worked example of `docs/extending.md`, so it
//! is written to be read.

use rand::RngExt as _;

use super::{Generator, START};
use crate::StepOutcome;
use crate::maze::{Cell, Dir, Maze};
use crate::rng::{self, Rng};

/// Randomized Prim, carrying its frontier.
struct Prim {
    /// The uncarved cells beside the carved region. A step draws one of these,
    /// so the order matters: `swap_remove` reorders it, and section 4.2 keeps
    /// that on purpose.
    frontier: Vec<Cell>,
    /// One flag for each cell: the cell is in `frontier`. It keeps
    /// `is_frontier` O(1) for the renderer, and it is also what stops a cell
    /// being pushed twice.
    in_frontier: Vec<bool>,
    /// The width of the maze, for the offset into `in_frontier`.
    width: u16,
    /// The cell the last step carved.
    current: Option<Cell>,
    /// False until a step attached the cell the run starts from.
    attached: bool,
    /// The stream this run draws from. Rule 7 of the contract holds the reason
    /// a generator owns one.
    rng: Rng,
}

/// Makes the Randomized Prim generator over `maze`, drawing from `rng`.
///
/// The maze is untouched until the first [`Generator::step`], which is rule 4
/// of the contract. The start cell is attached at the head of that step, and
/// the frontier it seeds is filled here.
#[must_use]
pub fn make(maze: &Maze, rng: &mut Rng) -> Box<dyn Generator> {
    let cells = usize::from(maze.width()) * usize::from(maze.height());
    let mut prim = Prim {
        frontier: Vec::new(),
        in_frontier: vec![false; cells],
        width: maze.width(),
        current: None,
        attached: false,
        rng: rng::split(rng),
    };
    for d in Dir::ALL {
        if let Some(n) = maze.neighbour(START, d) {
            prim.push_frontier(n);
        }
    }
    Box::new(prim)
}

impl Prim {
    /// The offset of a cell in `in_frontier`.
    fn index(&self, c: Cell) -> usize {
        usize::from(c.y) * usize::from(self.width) + usize::from(c.x)
    }

    /// Puts a cell in the frontier, unless it is already there.
    fn push_frontier(&mut self, c: Cell) {
        let i = self.index(c);
        if !self.in_frontier[i] {
            self.in_frontier[i] = true;
            self.frontier.push(c);
        }
    }

    /// Takes a uniformly random cell out of the frontier.
    ///
    /// `swap_remove` is O(1) and it moves the last cell into the hole, which
    /// reorders the frontier and so changes later draws. That is deterministic
    /// for a given seed, so section 4.2 allows it and records it: it is the
    /// kind of detail an optimiser changes by accident.
    fn take_frontier(&mut self) -> Option<Cell> {
        // The frontier holds at most one entry for each cell, and a cell
        // count always fits a `u32`: a maze is `u16 x u16` and
        // `u16::MAX * u16::MAX` is below `u32::MAX`. Rule 2 of `crate::rng` is
        // why the range is drawn over a `u32` and not over a `usize`.
        let Ok(len) = u32::try_from(self.frontier.len()) else {
            unreachable!(
                "a maze cannot hold a frontier of {} cells",
                self.frontier.len()
            )
        };
        if len == 0 {
            return None;
        }
        let c = self
            .frontier
            .swap_remove(self.rng.random_range(0u32..len) as usize);
        let i = self.index(c);
        self.in_frontier[i] = false;
        Some(c)
    }
}

impl Generator for Prim {
    fn step(&mut self, maze: &mut Maze) -> StepOutcome {
        if !self.attached {
            maze.mark_carved(START);
            self.attached = true;
        }
        // The caller does not step after `Done`, so the frontier holds a cell.
        let Some(c) = self.take_frontier() else {
            return StepOutcome::Done;
        };

        // The carved neighbours, in the fixed N E S W order. A frontier cell
        // is beside the carved region, so there is at least one. The list is
        // filled with `c`, which is never its own neighbour, and only the
        // first `count` entries are ever read.
        let mut carved = [c; 4];
        let mut count: u32 = 0;
        for d in Dir::ALL {
            if let Some(n) = maze.neighbour(c, d)
                && maze.is_carved(n)
            {
                carved[count as usize] = n;
                count += 1;
            }
        }
        assert!(
            count > 0,
            "a frontier cell is beside the carved region, and {c:?} is not"
        );
        let n = carved[self.rng.random_range(0u32..count) as usize];
        maze.carve(c, n);
        self.current = Some(c);

        for d in Dir::ALL {
            if let Some(n) = maze.neighbour(c, d)
                && !maze.is_carved(n)
            {
                self.push_frontier(n);
            }
        }

        if self.frontier.is_empty() {
            StepOutcome::Done
        } else {
            StepOutcome::Stepped
        }
    }

    fn current(&self) -> Option<Cell> {
        self.current
    }

    fn is_frontier(&self, c: Cell) -> bool {
        // A cell outside the maze is in no working set. The column has to be
        // checked too: an offset past the end of a row lands on the next one.
        c.x < self.width && self.in_frontier.get(self.index(c)) == Some(&true)
    }

    fn frontier_len(&self) -> usize {
        self.frontier.len()
    }
}
