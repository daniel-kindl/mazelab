//! The Randomized Prim generator.

use super::Generator;
use crate::maze::Maze;
use crate::rng::Rng;

/// Makes the Randomized Prim generator over `maze`, drawing from `rng`.
///
/// # Panics
///
/// Always. Section 4.2 holds the algorithm, and it is not written yet.
#[must_use]
pub fn make(_maze: &Maze, _rng: &mut Rng) -> Box<dyn Generator> {
    todo!("SPEC.md section 4.2 gives Randomized Prim, the frontier variant")
}
