//! The two too-small panels: what the screen shows when the terminal is below
//! a threshold.
//!
//! **There are two thresholds, not one.** They have different causes,
//! different remedies and different wordings, and they must not be collapsed.
//! Section 13.
//!
//! | panel | fires when | replaces | remedy |
//! | --- | --- | --- | --- |
//! | [`maze_fit`] | `cols < 4W+2` or `rows < 2H+11` | the maze pane only | resize, or `f` |
//! | [`layout_floor`] | below 79 x 19 | the whole screen | resize only |
//!
//! Both are one box, 46 x 8, centred, in `LightRed`. "Terminal too small" is
//! wrong for an oversized maze on a roomy terminal: the terminal is fine and
//! the maze is too large. Section 13.5.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Widget};

use crate::app::{CHROME_ROWS, FLOOR_COLS, FLOOR_ROWS};
use crate::maze::Maze;

/// The columns of the box, border and all: the widest line, the maze-fit
/// hint, is 42 columns, with one column of space and the border on each side.
const WIDTH: u16 = 46;

/// The rows of the box, border and all: the six rows of a panel and the
/// border.
const HEIGHT: u16 = 8;

/// The lines of a panel that carry text: the title, `need`, `have` and the
/// hint.
const LINES: usize = 4;

/// One too-small panel: a title, the size that is needed, the size that is
/// there, and the remedy.
///
/// The lines drop from the bottom when the area is too small: the hint first,
/// then `have`, then `need`. The title drops last. Section 13.5.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Panel {
    /// What is wrong.
    title: &'static str,
    /// The size that is needed.
    need: String,
    /// The size that is there.
    have: String,
    /// The remedy.
    hint: &'static str,
}

/// The panel for a maze that does not fit the terminal of `screen`. It
/// replaces the maze pane only.
///
/// A maze needs `4W + 2` columns and `2H + 11` rows: its footprint, and the
/// rows of the chrome.
#[must_use]
pub fn maze_fit(maze: &Maze, screen: Rect) -> Panel {
    let cols = maze.display_width().saturating_mul(2);
    let rows = maze.display_height().saturating_add(CHROME_ROWS);
    Panel {
        title: "Maze too large for this terminal",
        need: format!(
            "maze {} x {} needs {cols} x {rows}",
            maze.width(),
            maze.height()
        ),
        have: format!("terminal is {} x {}", screen.width, screen.height),
        hint: "resize, or press f to refit and regenerate",
    }
}

/// The panel for a terminal below the layout floor. It replaces the whole
/// screen.
///
/// **It has no `press f` line.** Below the floor `f` is inert, so the hint
/// names the one remedy that works. Section 13.3.
#[must_use]
pub fn layout_floor(screen: Rect) -> Panel {
    Panel {
        title: "Terminal too small",
        need: format!("need {FLOOR_COLS} x {FLOOR_ROWS}"),
        have: format!("have {} x {}", screen.width, screen.height),
        hint: "resize to continue",
    }
}

impl Panel {
    /// The rows of the panel that keep the top `kept` of its [`LINES`] lines.
    ///
    /// A blank row separates the title from the two sizes, and the two sizes
    /// from the hint. A blank row drops together with the line below it.
    fn rows(&self, kept: usize) -> Vec<&str> {
        match kept {
            0 => vec![],
            1 => vec![self.title],
            2 => vec![self.title, "", &self.need],
            3 => vec![self.title, "", &self.need, &self.have],
            _ => vec![self.title, "", &self.need, &self.have, "", self.hint],
        }
    }
}

impl Widget for &Panel {
    /// Draws the panel centred in `area`, in the largest form that fits.
    ///
    /// **It degrades by dropping the border first, then dropping lines from
    /// the bottom**: the hint, then `have`, then `need`. **It never clips a
    /// line in the middle**: a line that is too wide for `area` drops whole,
    /// with every line below it. So the title survives to the smallest area
    /// that can hold it, and below that nothing is drawn. Section 13.5.
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let style = Style::new().fg(Color::LightRed);
        if area.width >= WIDTH && area.height >= HEIGHT {
            let boxed = centred(area, WIDTH, HEIGHT);
            buffer.set_style(boxed, style);
            let block = Block::bordered().border_type(BorderType::Rounded);
            let inner = block.inner(boxed);
            block.render(boxed, buffer);
            draw_rows(&self.rows(LINES), inner, buffer);
            return;
        }
        let fits = |rows: &[&str]| {
            rows.len() <= usize::from(area.height)
                && rows
                    .iter()
                    .all(|row| columns(row) <= usize::from(area.width))
        };
        let Some(rows) = (1..=LINES)
            .rev()
            .map(|kept| self.rows(kept))
            .find(|rows| fits(rows))
        else {
            return;
        };
        // `fits` holds both numbers inside the area, so neither conversion
        // saturates.
        let widest = rows.iter().map(|row| columns(row)).max().unwrap_or(0);
        let text_area = centred(
            area,
            u16::try_from(widest).unwrap_or(u16::MAX),
            u16::try_from(rows.len()).unwrap_or(u16::MAX),
        );
        buffer.set_style(text_area, style);
        draw_rows(&rows, text_area, buffer);
    }
}

/// The screen columns that one row of text takes.
fn columns(text: &str) -> usize {
    Line::from(text).width()
}

/// Draws `rows` from the top of `area`, each one centred on its row.
fn draw_rows(rows: &[&str], area: Rect, buffer: &mut Buffer) {
    for (y, text) in (area.y..area.bottom()).zip(rows) {
        Line::from(*text)
            .centered()
            .render(Rect::new(area.x, y, area.width, 1), buffer);
    }
}

/// A rectangle of `width` x `height` centred in `area`, and held inside it.
fn centred(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
    .clamp(area)
}
