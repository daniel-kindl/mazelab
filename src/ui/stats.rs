//! The statistics band: it shows the statistics of the run and of the previous
//! run, and the facts of the maze.
//!
//! Three `Fill(1)` columns, each bordered and titled: **this run**, **previous
//! run** and **maze**. Five content rows, so the band is seven rows with its
//! border. Section 8.1.
//!
//! **The algorithm name here is always the abbreviated one**: three columns at
//! the 79-column floor give 26 each, and `algorithm  Recursive Backtracker`
//! needs 33. The full name is on the status row.
//!
//! The band compares two runs of one maze, and the runs are made one after the
//! other. ADR 0009, comparison is sequential.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

use crate::app::{Activity, App};
use crate::run::{RunStats, corners};
use crate::ui::shown_activity;

/// The content rows of one column.
const ROWS: usize = 5;

/// One content row: a label on the left and a value against the right edge.
struct Row {
    /// The name of the number, or a row of text with no number.
    label: &'static str,
    /// The number, or nothing.
    value: String,
}

impl Row {
    /// A row that names a number.
    fn new(label: &'static str, value: impl std::fmt::Display) -> Self {
        Self {
            label,
            value: value.to_string(),
        }
    }

    /// A row of text with no number.
    fn text(label: &'static str) -> Self {
        Self::new(label, "")
    }
}

/// Draws the statistics band of `app` in `area`.
pub fn render(app: &App, area: Rect, buffer: &mut Buffer) {
    let [this, previous, maze] = Layout::horizontal([Constraint::Fill(1); 3]).areas(area);
    column(" this run ", this_run(app), this, buffer);
    column(
        " previous run ",
        app.previous.map(|stats| run_rows(stats, "expanded")),
        previous,
        buffer,
    );
    column(" maze ", Some(maze_rows(app)), maze, buffer);
}

/// The rows of the run on screen, or `None` when there is no run to show.
///
/// **The phase decides the label of the third row**: the **carved count** for a
/// generation run, **expanded** for a solver run. The `Ready` that follows a
/// generate still holds the generation run, so the column is never empty
/// there. The `Ready` that a solver key reaches holds no run, and the column
/// is drawn empty until `s`. Section 8.2.
fn this_run(app: &App) -> Option<[Row; ROWS]> {
    let stats = app.run.as_ref()?.stats(&app.maze);
    let counted = match shown_activity(app.phase) {
        Some(Activity::Solving) => "expanded",
        Some(Activity::Generating) | None => "carved",
    };
    Some(run_rows(stats, counted))
}

/// The rows of one run.
///
/// **previous run** only ever holds a finished solver run, so it is always
/// labelled `expanded`. A path length that does not exist yet, and the path
/// length of a generation run, is `-`.
fn run_rows(stats: RunStats, counted: &'static str) -> [Row; ROWS] {
    [
        Row::new("algorithm", stats.algorithm_short),
        Row::new("step", stats.steps),
        Row::new(counted, stats.expanded_or_carved),
        Row::new("frontier", stats.frontier),
        match stats.path_len {
            Some(len) => Row::new("path len", len),
            None => Row::new("path len", "-"),
        },
    ]
}

/// The rows of the maze column.
///
/// `equal path` shows only while the braid factor is 0. It is the quiet form
/// of the statement that an unbraided maze makes every solver return the
/// identical path, so a tie between two path lengths is expected and is not a
/// broken comparison. Section 8.1.
fn maze_rows(app: &App) -> [Row; ROWS] {
    let maze = &app.maze;
    let (start, _) = corners(maze);
    [
        Row::new("maze", format!("{}x{}", maze.width(), maze.height())),
        Row::new("braid", format!("{:.2}", app.braid_factor)),
        Row::new("carved", maze.carved_count()),
        Row::new("start", format!("{},{}", start.x, start.y)),
        Row::text(if app.braid_factor <= 0.0 {
            "equal path"
        } else {
            ""
        }),
    ]
}

/// Draws one bordered, titled column, and its rows when it has any.
///
/// The rows fill the column inside its border, with no padding: three columns
/// at the 79-column floor give 24 columns inside the border, and that is the
/// width section 8.1 sizes the names against.
fn column(title: &'static str, rows: Option<[Row; ROWS]>, area: Rect, buffer: &mut Buffer) {
    let block = Block::bordered().title(title);
    let inner = block.inner(area);
    block.render(area, buffer);
    let Some(rows) = rows else {
        return;
    };
    let lines = Layout::vertical([Constraint::Length(1); ROWS]).areas::<ROWS>(inner);
    for (row, line) in rows.into_iter().zip(lines) {
        Line::from(row.label).render(line, buffer);
        Line::from(row.value).right_aligned().render(line, buffer);
    }
}
