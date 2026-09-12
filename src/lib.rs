//! MazeLab: the model, the algorithms, the application state and the rendering.
//!
//! The library holds everything that a terminal is not needed for. `src/main.rs`
//! holds terminal setup, terminal teardown and the loop.
//!
//! One rule holds over this library, and it is the rule of the layout. Nothing
//! in [`maze`], [`braid`], [`rng`], [`generator`], [`solver`], [`run`],
//! [`app`] or [`cli`] names `ratatui` or `crossterm`. The core knows cells,
//! walls, frontiers and steps. It does not know colours, glyphs or key codes.
//! Only [`ui`] and `src/main.rs` name the terminal.
//!
//! [`ui`] depends on [`app`], [`run`], [`maze`], [`generator`] and [`solver`].
//! None of them depends on [`ui`]. See ADR 0006, the core never names the
//! terminal.

pub mod app;
pub mod braid;
pub mod cli;
pub mod generator;
pub mod maze;
pub mod rng;
pub mod run;
pub mod solver;
pub mod ui;

/// What one call to a `step` method did.
///
/// One enum, returned by both [`generator::Generator::step`] and
/// [`solver::Solver::step`]. The two traits stay apart, because a generator
/// takes the maze by `&mut` and a solver takes it by `&`. Only the outcome is
/// shared, and that is what lets the loop advance either kind the same way.
/// See ADR 0003, two traits not one.
///
/// `Done` is returned **on the step that completes the run**, not on a later
/// call. The caller must not call `step` again after `Done`. This is the one
/// place that rule is written down.
///
/// There is no `Exhausted` variant. Every maze is connected and braiding only
/// adds edges, so a solver always reaches the goal. A variant that cannot occur
/// still has to be matched at every call site forever.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepOutcome {
    /// The algorithm advanced and the run goes on. This is what sets the dirty
    /// flag of section 6, and it is the reason `step` returns a value at all.
    Stepped,
    /// The algorithm finished its work on this call.
    Done,
}
