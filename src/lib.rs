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
