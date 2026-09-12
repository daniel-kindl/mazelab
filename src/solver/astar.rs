//! The A* solver.

use super::Solver;
use crate::maze::{Cell, Maze};

/// Makes the A* solver, which searches `maze` from `start` for `goal`.
///
/// # Panics
///
/// Always. Section 4.6 holds the algorithm, and it is not written yet.
#[must_use]
pub fn make(_maze: &Maze, _start: Cell, _goal: Cell) -> Box<dyn Solver> {
    todo!("SPEC.md section 4.6 gives A*")
}
