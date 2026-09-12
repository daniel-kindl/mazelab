//! The Recursive Backtracker generator.
//!
//! Section 4.1 of `SPEC.md` holds the algorithm. The **frontier** is the
//! stack, and drawing the stack is what makes the signature backtracking
//! legible: the run walks a corridor to its end, then unwinds along the cells
//! it still holds.
//!
//! A run takes `2 * W * H - 1` steps. One step carves each cell and one step
//! pops it again, and the step that pops the last cell is the one that returns
//! [`StepOutcome::Done`].

use rand::RngExt as _;

use super::{Generator, START};
use crate::StepOutcome;
use crate::maze::{Cell, Dir, Maze};
use crate::rng::{self, Rng};

/// The Recursive Backtracker, carrying its stack.
struct Backtracker {
    /// The stack, which is the **frontier**. The cell on top is the one the
    /// next step acts on.
    stack: Vec<Cell>,
    /// One flag for each cell: the cell is on the stack. Rule 5 of the
    /// contract asks for an O(1) `is_frontier`, and this is what answers it.
    on_stack: Vec<bool>,
    /// The width of the maze, for the offset into `on_stack`.
    width: u16,
    /// False until a step attached the cell the run starts from.
    attached: bool,
    /// The stream this run draws from. Rule 7 of the contract holds the reason
    /// a generator owns one.
    rng: Rng,
}

/// Makes the Recursive Backtracker generator over `maze`, drawing from `rng`.
///
/// The maze is untouched until the first [`Generator::step`], which is rule 4
/// of the contract. The start cell is attached at the head of that step.
#[must_use]
pub fn make(maze: &Maze, rng: &mut Rng) -> Box<dyn Generator> {
    let cells = usize::from(maze.width()) * usize::from(maze.height());
    let mut backtracker = Backtracker {
        stack: vec![START],
        on_stack: vec![false; cells],
        width: maze.width(),
        attached: false,
        rng: rng::split(rng),
    };
    let start = backtracker.index(START);
    backtracker.on_stack[start] = true;
    Box::new(backtracker)
}

impl Backtracker {
    /// The offset of a cell in `on_stack`.
    fn index(&self, c: Cell) -> usize {
        usize::from(c.y) * usize::from(self.width) + usize::from(c.x)
    }
}

impl Generator for Backtracker {
    fn step(&mut self, maze: &mut Maze) -> StepOutcome {
        if !self.attached {
            maze.mark_carved(START);
            self.attached = true;
        }
        // The caller does not step after `Done`, so the stack holds a cell.
        let Some(&c) = self.stack.last() else {
            return StepOutcome::Done;
        };

        // The neighbours that are not carved, in the fixed N E S W order, so
        // that the draw below is the only source of variation. The list is
        // filled with `c`, which is never its own neighbour, and only the
        // first `count` entries are ever read.
        let mut candidates = [c; 4];
        let mut count: u32 = 0;
        for d in Dir::ALL {
            if let Some(n) = maze.neighbour(c, d)
                && !maze.is_carved(n)
            {
                candidates[count as usize] = n;
                count += 1;
            }
        }

        if count == 0 {
            self.stack.pop();
            let i = self.index(c);
            self.on_stack[i] = false;
            return if self.stack.is_empty() {
                StepOutcome::Done
            } else {
                StepOutcome::Stepped
            };
        }

        let n = candidates[self.rng.random_range(0u32..count) as usize];
        maze.carve(c, n);
        let i = self.index(n);
        self.stack.push(n);
        self.on_stack[i] = true;
        StepOutcome::Stepped
    }

    fn current(&self) -> Option<Cell> {
        self.stack.last().copied()
    }

    fn is_frontier(&self, c: Cell) -> bool {
        // A cell outside the maze is in no working set. The column has to be
        // checked too: an offset past the end of a row lands on the next one.
        c.x < self.width && self.on_stack.get(self.index(c)) == Some(&true)
    }

    fn frontier_len(&self) -> usize {
        self.stack.len()
    }
}
