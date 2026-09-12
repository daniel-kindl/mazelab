//! The depth-first solver.
//!
//! Section 4.4 of `SPEC.md` holds the algorithm. The **frontier** is the
//! stack, and drawing it is what makes the signature of a depth-first search
//! legible: the run drives down one corridor to its end before it takes up any
//! of the branches it passed.
//!
//! DFS finds a path, and on a maze with loops that path can be far longer than
//! the shortest one. That is the point of showing it beside BFS.

use super::{Search, Solver, reachable};
use crate::StepOutcome;
use crate::maze::{Cell, Maze};

/// The depth-first solver, carrying its stack.
struct Dfs {
    /// The stack, which is the **frontier**. The cell on top is the one the
    /// next step expands.
    stack: Vec<Cell>,
    /// One flag for each cell: the cell is on the stack. Rule 5 of the
    /// contract asks for an O(1) `is_frontier`, and this is what answers it.
    on_stack: Vec<bool>,
    /// The expanded flags, the parents and the path.
    search: Search,
}

/// Makes the DFS solver, which searches `maze` from `start` for `goal`.
///
/// The stack holds the start cell, so the first [`Solver::step`] expands it.
#[must_use]
pub fn make(maze: &Maze, start: Cell, goal: Cell) -> Box<dyn Solver> {
    let search = Search::new(maze, start, goal);
    let mut on_stack = vec![false; search.cell_count()];
    on_stack[search.index(start)] = true;
    Box::new(Dfs {
        stack: vec![start],
        on_stack,
        search,
    })
}

impl Solver for Dfs {
    fn step(&mut self, maze: &Maze) -> StepOutcome {
        // The caller does not step after `Done`, so the stack holds a cell.
        let Some(c) = self.stack.pop() else {
            // Unreachable: every maze is connected, so a solver reaches the
            // goal before its working set runs out. The arm answers `Done`
            // without a path, which rules 2 and 3 of the contract forbid, so a
            // debug build says so here rather than let the caller read it. A
            // release build ends the run instead of panicking at the user.
            debug_assert!(false, "DFS ran out of cells before it found the goal");
            return StepOutcome::Done;
        };
        self.on_stack[self.search.index(c)] = false;

        if self.search.is_expanded(c) {
            // A stale duplicate. Discarding one is a step too, which is rule 1
            // of the contract: this call expanded no cell and the run goes on.
            return StepOutcome::Stepped;
        }
        self.search.expand(c);
        if c == self.search.goal() {
            return self.search.finish();
        }

        // The neighbours through a carved edge, in the fixed N E S W order.
        // The last one pushed is the first one expanded, so the run walks W,
        // then S, then E, then N, and that is the shape on the screen.
        for n in reachable(maze, c) {
            let i = self.search.index(n);
            if !self.search.is_expanded(n) && !self.on_stack[i] {
                self.search.set_parent(n, c);
                self.stack.push(n);
                self.on_stack[i] = true;
            }
        }
        StepOutcome::Stepped
    }

    fn current(&self) -> Option<Cell> {
        self.search.current()
    }

    fn is_frontier(&self, c: Cell) -> bool {
        self.search.offset(c).is_some_and(|i| self.on_stack[i])
    }

    fn frontier_len(&self) -> usize {
        self.stack.len()
    }

    fn is_expanded(&self, c: Cell) -> bool {
        self.search.is_expanded(c)
    }

    fn expanded_count(&self) -> usize {
        self.search.expanded_count()
    }

    fn path(&self) -> Option<&[Cell]> {
        self.search.path()
    }
}
