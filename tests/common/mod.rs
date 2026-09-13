//! The `TestBackend` helpers that the rendering tests of tier 3 share: the
//! legend of item 6, the help row of item 7, the panels of item 8 and the
//! screen of item 9 of section 11.3.
//!
//! `TestBackend` renders into memory and needs no terminal, so every test that
//! uses these helpers runs on all three CI platforms. No helper here is typed
//! `io::Result<()>`.

#![allow(dead_code, reason = "each test crate uses only some helpers")]

use mazelab::app::{App, Startup, capacity};
use mazelab::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::text::Line;

/// The startup options every rendering test starts from: a fixed seed, the
/// Unicode glyph set, the Recursive Backtracker and A\*, which are the
/// defaults of section 9.
pub fn startup() -> Startup {
    Startup {
        seed: 7,
        width: None,
        height: None,
        ascii: false,
        generator_ix: 0,
        solver_ix: 2,
    }
}

/// The application at startup on a terminal of `cols` x `rows`.
pub fn app_on(cols: u16, rows: u16) -> App {
    App::new(startup(), capacity(cols, rows))
}

/// A terminal of `cols` x `rows` with one frame of `app` drawn on it.
pub fn terminal(app: &App, cols: u16, rows: u16) -> Terminal<TestBackend> {
    // `TestBackend::Error` is `Infallible`, so the two patterns are
    // irrefutable.
    let Ok(mut terminal) = Terminal::new(TestBackend::new(cols, rows));
    let Ok(_) = terminal.draw(|frame| ui::render(frame, app));
    terminal
}

/// Draws one frame of `app` on a terminal of `cols` x `rows`.
pub fn draw(app: &App, cols: u16, rows: u16) -> Buffer {
    terminal(app, cols, rows).backend().buffer().clone()
}

/// The symbols of one screen row, joined.
pub fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

/// Every row of the buffer, joined.
pub fn screen_rows(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height).map(|y| row(buffer, y)).collect()
}

/// Holds one frame of `app` on a terminal of `cols` x `rows` to `expected`,
/// one line for each screen row, in symbols and in style.
///
/// This is the snapshot of a whole screen. It uses `assert_buffer_lines`,
/// because `assert_buffer_eq!` is not reachable through `ratatui` 0.30.2.
pub fn assert_screen<'line, Lines>(app: &App, cols: u16, rows: u16, expected: Lines)
where
    Lines: IntoIterator,
    Lines::Item: Into<Line<'line>>,
{
    terminal(app, cols, rows)
        .backend()
        .assert_buffer_lines(expected);
}

/// The text of a line, with no style.
pub fn text(line: &Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
