//! The Recursive Backtracker generator.

use super::Generator;
use crate::maze::Maze;
use crate::rng::Rng;

/// Makes the Recursive Backtracker generator over `maze`, drawing from `rng`.
///
/// # Panics
///
/// Always. Section 4.1 holds the algorithm, and it is not written yet.
#[must_use]
pub fn make(_maze: &Maze, _rng: &mut Rng) -> Box<dyn Generator> {
    todo!("SPEC.md section 4.1 gives the Recursive Backtracker")
}
