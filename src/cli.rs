//! The command line: the clap surface, and the options it parses into.
//!
//! Six flags, and no in-application text entry in v1. Section 9 fixes two
//! rules that shape this file, and both are about where a decision is made:
//!
//! 1. **A size is rejected at parse time, never clamped.** `--width 2` exits
//!    with an error. Clamping would contradict the rule that maze size is
//!    explicit: the user asked for 2 and would silently get 4. The bounds are
//!    [`MIN_SIZE`] and [`MAX_SIZE`], the same pair the model holds, so the
//!    message states the bound that the model enforces.
//! 2. **A key is validated against the registry table**, and not against a
//!    [`clap::ValueEnum`]. [`GENERATORS`] and [`SOLVERS`] are the single source
//!    of truth for the command line, the help row and the in-application
//!    chooser. A `ValueEnum` would be a second list of the same keys, and two
//!    lists drift apart. The error lists the keys it read from the table, so a
//!    row added to a table reaches the user with nothing else to edit.
//!
//! The lookup answers with the **index** into the table, and that is what
//! [`Args::generator`] and [`Args::solver`] hold. It is the form
//! [`Startup`] takes, and it is the form the number keys of section 7 select
//! by, so the key is resolved once, at the edge, and never again.
//!
//! What this module does **not** decide is the size the application starts at.
//! The fallback chain of section 13.2 is capacity's, and capacity is a
//! property of the terminal: `--width` is `None` here when the flag is absent,
//! and [`crate::app::App`] applies the chain.

use clap::Parser;

use crate::app::{MAX_SIZE, MIN_SIZE, Startup};
use crate::generator::GENERATORS;
use crate::rng;
use crate::solver::SOLVERS;

/// The default `--generator`, as a key in [`GENERATORS`].
///
/// It is a key and not an index, because clap runs a default value through the
/// value parser that a given value goes through. The lookup therefore covers
/// the default too, and a default that is not in the table fails on every run
/// and not only on the run that names it.
const DEFAULT_GENERATOR: &str = "backtracker";

/// The default `--solver`, as a key in [`SOLVERS`].
const DEFAULT_SOLVER: &str = "astar";

/// The six flags of section 9.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Animates maze generation and pathfinding, one step at a time."
)]
pub struct Args {
    /// The number that fixes every random draw. A fresh seed when absent.
    #[arg(long)]
    pub seed: Option<u64>,

    /// The maze width in cells, 4 to 512. Capacity when absent, else 4.
    #[arg(long, value_parser = parse_width)]
    pub width: Option<u16>,

    /// The maze height in cells, 4 to 512. Capacity when absent, else 4.
    #[arg(long, value_parser = parse_height)]
    pub height: Option<u16>,

    /// Draw with the ASCII glyph set, which holds no character above U+007F.
    #[arg(long)]
    pub ascii: bool,

    /// The index into [`GENERATORS`] that `--generator` named.
    ///
    /// The help text is stated here and not taken from this comment, because
    /// the user reads a key and the field holds the row the key resolved to.
    #[arg(
        long,
        default_value = DEFAULT_GENERATOR,
        value_parser = parse_generator,
        help = "The generator to run, as a key in the generator table",
        long_help = None,
    )]
    pub generator: usize,

    /// The index into [`SOLVERS`] that `--solver` named.
    ///
    /// The help text is stated here, for the reason [`Args::generator`] gives.
    #[arg(
        long,
        default_value = DEFAULT_SOLVER,
        value_parser = parse_solver,
        help = "The solver to run, as a key in the solver table",
        long_help = None,
    )]
    pub solver: usize,
}

impl Args {
    /// What the command line settled, in the form [`crate::app::App`] starts
    /// from.
    ///
    /// **It takes `self`, and that is the point.** An absent `--seed` is a
    /// fresh seed, and the draw is here, so a second call would answer with a
    /// different seed and a different maze. A `&self` method reads as an
    /// accessor and would invite that call. This one consumes the arguments,
    /// so the compiler holds the rule that a run has one seed.
    #[must_use]
    pub fn into_startup(self) -> Startup {
        Startup {
            seed: self.seed.unwrap_or_else(rng::fresh_seed),
            width: self.width,
            height: self.height,
            ascii: self.ascii,
            generator_ix: self.generator,
            solver_ix: self.solver,
        }
    }
}

/// Parses `--width`, and rejects a width outside the bounds.
fn parse_width(value: &str) -> Result<u16, String> {
    parse_size(value, "width")
}

/// Parses `--height`, and rejects a height outside the bounds.
fn parse_height(value: &str) -> Result<u16, String> {
    parse_size(value, "height")
}

/// Parses one axis of the size, and rejects a value outside
/// `MIN_SIZE..=MAX_SIZE`.
///
/// A value that does not fit a `u16` is the same error as a value that fits it
/// and is out of bounds: the bound is what the user has to read, and 70000 is
/// outside it for the reason 600 is. clap puts the value and the flag in front
/// of this message, so the message states the rule alone.
fn parse_size(value: &str, axis: &str) -> Result<u16, String> {
    let inside = value
        .parse::<u16>()
        .ok()
        .filter(|n| (MIN_SIZE..=MAX_SIZE).contains(n));
    inside.ok_or_else(|| format!("maze {axis} must be between {MIN_SIZE} and {MAX_SIZE}"))
}

/// Parses `--generator` against [`GENERATORS`].
fn parse_generator(value: &str) -> Result<usize, String> {
    let keys: Vec<&str> = GENERATORS.iter().map(|e| e.key).collect();
    parse_key(value, "generator", &keys)
}

/// Parses `--solver` against [`SOLVERS`].
fn parse_solver(value: &str) -> Result<usize, String> {
    let keys: Vec<&str> = SOLVERS.iter().map(|e| e.key).collect();
    parse_key(value, "solver", &keys)
}

/// The index of the row `value` names, or an error that lists `keys`.
///
/// The list the error states is the list the lookup walked, which is rule 2 of
/// the module documentation: there is one list of keys, and it is the table.
/// A command line is parsed once for each run, so the two callers collect the
/// keys of their table and the walk costs one small allocation.
fn parse_key(value: &str, kind: &str, keys: &[&str]) -> Result<usize, String> {
    keys.iter()
        .position(|&key| key == value)
        .ok_or_else(|| format!("the valid {kind} keys are: {}", keys.join(", ")))
}
