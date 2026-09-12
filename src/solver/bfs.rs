//! The breadth-first solver.

use super::Solver;
use crate::maze::{Cell, Maze};

/// Makes the BFS solver, which searches `maze` from `start` for `goal`.
///
/// # Panics
///
/// Always. Section 4.5 holds the algorithm, and it is not written yet.
#[must_use]
pub fn make(_maze: &Maze, _start: Cell, _goal: Cell) -> Box<dyn Solver> {
    todo!("SPEC.md section 4.5 gives BFS")
}
