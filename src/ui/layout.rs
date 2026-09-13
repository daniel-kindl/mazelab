//! The screen layout: the split of the screen between the maze pane and the
//! chrome, and the status row at the top of it.
//!
//! The screen is **variant B, Bands**, and it is **fixed at every terminal
//! size**. No band appears or disappears with the width. Section 7.1.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;

use crate::app::{App, Phase};
use crate::generator::GENERATORS;
use crate::solver::SOLVERS;

/// The rows of the bordered statistics band: five content rows and the
/// border. Section 8.1.
pub const STATS_ROWS: u16 = 7;

/// The width at which the legend row and the help row change form. At this
/// many columns and above, each draws its full form. Below it, each draws its
/// short form. Sections 7.6 and 7.7.
pub const BREAKPOINT: u16 = 100;

/// The five bands of section 7.1, top to bottom.
///
/// **Chrome** is every band but the maze pane, and it costs 0 columns and
/// [`crate::app::CHROME_ROWS`] rows: the status row, the statistics band, the
/// legend row and the help row. The maze pane keeps the full width and the
/// rows that are left.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bands {
    /// One row: the seed, the selection, the phase and the speed.
    pub status: Rect,
    /// The rest of the screen. The maze is centred in it on both axes.
    pub maze: Rect,
    /// [`STATS_ROWS`] rows: this run, the previous run and the maze.
    pub stats: Rect,
    /// One row: every cell state of the current phase.
    pub legend: Rect,
    /// One row: the keys.
    pub help: Rect,
}

/// Splits the screen into the five bands.
#[must_use]
pub fn bands(area: Rect) -> Bands {
    let [status_row, maze, stats, legend, help_row] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(STATS_ROWS),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    Bands {
        status: status_row,
        maze,
        stats,
        legend,
        help: help_row,
    }
}

/// The status row of section 7.2.
///
/// `MazeLab  seed <n>  gen <full name>  solver <full name>  phase <phase>  speed <rung>x`
///
/// The **full** algorithm names are here, because this is where there is
/// width; the statistics band has the abbreviated ones. The names are those of
/// the **selection**, which is what the next `g` or `s` runs. The seed is
/// always on screen.
#[must_use]
pub fn status_line(app: &App) -> Line<'static> {
    Line::from(format!(
        "MazeLab  seed {}  gen {}  solver {}  phase {}  speed {}x",
        app.seed,
        GENERATORS[app.generator_ix].name,
        SOLVERS[app.solver_ix].name,
        phase_name(app.phase),
        // `f64` prints without a fraction when it has none, so the ladder
        // prints as `0.5x` through `1024x`.
        app.budget.rung(),
    ))
}

/// The name of a phase, as the glossary gives it.
const fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Idle => "Idle",
        Phase::Generating => "Generating",
        Phase::Paused(_) => "Paused",
        Phase::Ready => "Ready",
        Phase::Solving => "Solving",
        Phase::Solved => "Solved",
    }
}
