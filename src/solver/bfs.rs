//! The breadth-first solver.
//!
//! Section 4.5 of `SPEC.md` holds the algorithm. It differs from DFS in one
//! thing: the working set is a queue and the step takes from the front, so the
//! search spreads as a ring rather than driving down one corridor.
//!
//! BFS is what makes the **path length** authoritative. Every edge costs one
//! step, so the first time the queue reaches a cell it reached it by a shortest
//! path, and the length BFS returns is the shortest that exists.

use std::collections::VecDeque;

use super::{Search, Solver, reachable};
use crate::StepOutcome;
use crate::maze::{Cell, Maze};

/// The breadth-first solver, carrying its queue.
struct Bfs {
    /// The queue, which is the **frontier**. A step takes from the front and
    /// the walk of a cell's neighbours adds to the back.
    queue: VecDeque<Cell>,
    /// One flag for each cell: the cell is in the queue. Rule 5 of the
    /// contract asks for an O(1) `is_frontier`, and this is what answers it.
    in_queue: Vec<bool>,
    /// The expanded flags, the parents and the path.
    search: Search,
}

/// Makes the BFS solver, which searches `maze` from `start` for `goal`.
///
/// The queue holds the start cell, so the first [`Solver::step`] expands it.
#[must_use]
pub fn make(maze: &Maze, start: Cell, goal: Cell) -> Box<dyn Solver> {
    let search = Search::new(maze, start, goal);
    let mut in_queue = vec![false; search.cell_count()];
    in_queue[search.index(start)] = true;
    Box::new(Bfs {
        queue: VecDeque::from(vec![start]),
        in_queue,
        search,
    })
}

impl Solver for Bfs {
    fn step(&mut self, maze: &Maze) -> StepOutcome {
        // The caller does not step after `Done`, so the queue holds a cell.
        let Some(c) = self.queue.pop_front() else {
            // Unreachable, for the reason the same arm of `dfs` gives: the
            // arm answers `Done` without a path, and a connected maze never
            // takes a solver there.
            debug_assert!(false, "BFS ran out of cells before it found the goal");
            return StepOutcome::Done;
        };
        self.in_queue[self.search.index(c)] = false;

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
        // A cell enters the queue once, and the parent it enters with is the
        // one that reached it first, which is a parent on a shortest path.
        for n in reachable(maze, c) {
            let i = self.search.index(n);
            if !self.search.is_expanded(n) && !self.in_queue[i] {
                self.search.set_parent(n, c);
                self.queue.push_back(n);
                self.in_queue[i] = true;
            }
        }
        StepOutcome::Stepped
    }

    fn current(&self) -> Option<Cell> {
        self.search.current()
    }

    fn is_frontier(&self, c: Cell) -> bool {
        self.search.offset(c).is_some_and(|i| self.in_queue[i])
    }

    fn frontier_len(&self) -> usize {
        self.queue.len()
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
